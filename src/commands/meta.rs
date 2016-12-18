use chrono::UTC;
use serenity::client::{CACHE, Context};
use serenity::model::{EmojiIdentifier, Message};
use std::{env, u64};
use ::store::ShardUptime;

macro_rules! permissions {
    ($perms:ident; $($f:ident $n:expr,)*) => {
        {
            let mut s = vec![];

            $(
                if $perms.$f() {
                    s.push($n);
                }
            )*

            s
        }
    }
}

command!(avatar(context, message, args) {
    let url = if let Some(user) = message.mentions.first() {
        user.avatar_url()
    } else if let Some(arg) = args.first() {
        let guild_id = match message.guild_id() {
            Some(guild_id) => guild_id,
            None => {
                let _ = context.say("Could not find server data.");

                return Ok(());
            },
        };

        let avatar_url = CACHE.read()
            .unwrap()
            .get_guild(guild_id)
            .map(|g| g.get_member_named(arg).map(|m| m.user.avatar_url()));

        match avatar_url {
            Some(Some(avatar_url)) => avatar_url,
            Some(None) => {
                let _ = context.say("Could not find avatar");

                return Ok(());
            },
            None => {
                let _ = context.say("Could not find user");

                return Ok(());
            },
        }
    } else {
        message.author.avatar_url()
    };

    let _ = if let Some(url) = url {
        context.say(&url)
    } else {
        context.say("Could not find avatar")
    };
});

command!(emoji(context, _message, _args, emoji: EmojiIdentifier) {
    let _ = context.say(&emoji.url());
});

command!(invite(context, _message, _args) {
    let client_id = match env::var("DISCORD_CLIENT_ID") {
        Ok(client_id) => client_id,
        Err(_) => {
            error!("No Client ID");

            let _ = context.say("Error getting client ID");

            return Ok(());
        },
    };

    let _ = context.say(&format!("Here's a link to invite me:
<https://discordapp.com/oauth2/authorize?client_id={}&scope=bot&permissions=3222534>", client_id));
});

command!(ping(context, _message, _args) {
    let start = UTC::now();
    let mut message = req!(context.say("Ping!"));

    let end = UTC::now();
    let ms = {
        let end_ms = end.timestamp_subsec_millis() as i64;
        let start_ms = start.timestamp_subsec_millis() as i64;

        end_ms - start_ms
    };
    let diff = ((end.timestamp() - start.timestamp()) * 1000) + ms;

    let _ = message.edit(&format!("Pong! `[{}ms]`", diff), |e| e);
});

command!(role_info(context, message, args) {
    let cache = CACHE.read().unwrap();

    let guild_id = match cache.get_guild_channel(message.channel_id) {
        Some(channel) => channel.guild_id,
        None => {
            let _ = context.say("Error finding channel data");

            return Ok(());
        },
    };

    let guild = match cache.get_guild(guild_id) {
        Some(guild) => guild,
        None => {
            let _ = context.say("Could not find server data");

            return Ok(());
        },
    };

    // It's a pretty inexpensive operation to clone a role, so it should save
    // keeping the cache unlocked.
    let role = if !message.mention_roles.is_empty() {
        let id = unsafe { message.mention_roles.get_unchecked(0) };

        match guild.roles.values().find(|r| r.id == *id).cloned() {
            Some(role) => role,
            None => {
                warn!("Couldn't find r{} for c{}", id, message.channel_id);

                let _ = context.say("Mentioned role not found; error logged");

                return Ok(());
            },
        }
    } else if !args.is_empty() {
        let role_name = args.join(" ");

        match guild.roles.values().find(|r| r.name == role_name).cloned() {
            Some(role) => role,
            None => {
                let id = match role_name.parse::<u64>() {
                    Ok(id) => id,
                    Err(_) => {
                        let _ = context.say("Role not found by name");

                        return Ok(());
                    },
                };

                match guild.roles.values().find(|r| r.id == id).cloned() {
                    Some(role) => role,
                    None => {
                        warn!("Couldn't find r{} for c{}", id, message.channel_id);
                        let _ = context.say("Role not found; error logged");

                        return Ok(());
                    },
                }
            },
        }
    } else {
        let _ = context.say("A role name must be given or mentioned");

        return Ok(());
    };

    let description = {
        let mut s = "**Permissions**:".to_owned();

        let p = &role.permissions;
        let permissions = permissions! { p;
            add_reactions "Add Reactions",
            administrator "Administrator",
            attach_files "Attach Files",
            ban_members "Ban Members",
            change_nickname "Change Nickname",
            connect "Connect",
            create_invite "Create Invite",
            deafen_members "Deafen Members",
            embed_links "Embed Links",
            external_emojis "External Emojis",
            kick_members "Kick Members",
            manage_channels "Manage Channels",
            manage_emojis "Manage Emojis",
            manage_guild "Manage Guild",
            manage_messages "Manage Messages",
            manage_nicknames "Manage Nicknames",
            manage_roles "Manage Roles",
            manage_webhooks "Manage Webhooks",
            mention_everyone "Mention Everyone",
            move_members "Move Members",
            mute_members "Mute Members",
            read_message_history "Read Message History",
            read_messages "Read Messages",
            send_messages "Send Messages",
            send_tts_messages "Send TTS Messages",
            speak "Speak",
            use_external_emojis "Use External Emojis",
            use_vad "Use VAD",
        };

        s.push_str(&permissions.join(", "));

        s
    };
    let hoisted = if role.hoist { "Yes" } else { "No" };
    let mentionable = if role.mentionable { "Yes" } else { "No" };

    let _ = context.send_message(message.channel_id, |m| m
        .embed(|e| e
            .title(&format!("Role info for {} ({})", role.name, role.id.0))
            .description(&description)
            .colour(role.colour)
            .field(|f| f.name("Hoisted").value(hoisted))
            .field(|f| f.name("Position").value(&role.position.to_string()))
            .field(|f| f.name("Mentionable").value(mentionable))));
});

command!(uptime(context, message, _args) {
    let shard_number = {
        match context.shard.lock().unwrap().shard_info() {
            Some(shard) => shard[0],
            None => {
                let _ = context.say("Error retrieving shard number");
                error!("Error retrieving shard count on shard");

                return Ok(());
            },
        }
    };

    let (boot, conn) = {
        let data = context.data.lock().unwrap();
        let uptimes = data.get::<ShardUptime>().unwrap();

        if let Some(entry) = uptimes.get(&shard_number) {
            let boot = entry.boot.to_rfc3339()[..19].to_owned();
            let conn = entry.connection.to_rfc3339()[..19].to_owned();

            (boot, conn)
        } else {
            ("N/A".to_owned(), "N/A".to_owned())
        }
    };

    let name = CACHE.read().unwrap().user.name.clone();

    let _ = context.send_message(message.channel_id, |m| m
        .embed(|e| e
            .colour(0x8700B2)
            .title(&format!("Uptime for {}", name))
            .field(|f| f
                .name("Since Boot")
                .value(&boot))
            .field(|f| f
                .name("Current Connection")
                .value(&conn))));
});

command!(user_info(context, message, _args) {
    let member = {
        if let Some(guild_id) = message.guild_id() {
            let cache = CACHE.read().unwrap();
            let guild = cache.get_guild(guild_id).unwrap();

            // Clone so that the cache can be dropped ASAP.
            guild.get_member(message.author.id).cloned()
        } else {
            None
        }
    };

    let _ = context.send_message(message.channel_id, |m| m
        .embed(|mut e| {
            e = e.title(&format!("User info for {}", message.author.name))
                .field(|f| f.name("ID").value(&message.author.id.to_string()))
                .field(|f| f
                    .name("Discriminator")
                    .value(&message.author.discriminator.to_string()));

            if let Some(ref member) = member {
                e = e
                    // Pad to create a new row.
                    .field(|f| f
                        .name("\u{200b}")
                        .value("\u{200b}"))
                    .field(|f| f
                        .name("Joined at")
                        .value(&member.joined_at[..19]))
                    .field(|f| f
                        .name("Nick")
                        .value(&member.nick.clone().map_or("\u{200b}".to_owned(), |v| v.clone())));

                if let Some(colour) = member.colour() {
                    let s = format!("rgb({}, {}, {})",
                                    colour.get_r(),
                                    colour.get_g(),
                                    colour.get_b());

                    e = e.colour(colour)
                        .field(|f| f.name("Colour").value(&s));
                }
            }

            e
        }));
});
