use chrono::Utc;
use serenity::client::CACHE;
use std::u64;
use ::store::ShardUptime;
use ::prelude::*;

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

pub struct AvatarCommand;

impl Command for AvatarCommand {
    fn execute(&self, _: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
        let url = if let Some(user) = msg.mentions.first() {
            user.avatar_url()
        } else if let Ok(arg) = args.single::<String>() {
            let guild_id = CACHE.read()
                .guild_channel(msg.channel_id)
                .map(|c| c.read().guild_id);

            let guild_id = match guild_id {
                Some(guild_id) => guild_id,
                None => {
                    let _ = msg.channel_id.say("Could not find server data.");

                    return Ok(());
                },
            };

            let avatar_url = CACHE.read()
                .guild(guild_id)
                .map(|g| g.read().member_named(&arg).map(|m| m.user.read().avatar_url()));

            match avatar_url {
                Some(Some(avatar_url)) => avatar_url,
                Some(None) => {
                    let _ = msg.channel_id.say("Could not find avatar");

                    return Ok(());
                },
                None => {
                    let _ = msg.channel_id.say("Could not find user");

                    return Ok(());
                },
            }
        } else {
            msg.author.avatar_url()
        };

        let _ = if let Some(url) = url {
            msg.channel_id.say(&url)
        } else {
            msg.channel_id.say("Could not find avatar")
        };

        Ok(())
    }
}

pub struct RpingCommand;

impl Command for RpingCommand {
    fn execute(&self, _: &mut Context, msg: &Message, _: Args) -> CommandResult {
        let start = Utc::now();
        let mut msg = req!(msg.channel_id.say("Ping!"));

        let end = Utc::now();
        let ms = {
            let end_ms = end.timestamp_subsec_millis() as i64;
            let start_ms = start.timestamp_subsec_millis() as i64;

            end_ms - start_ms
        };
        let diff = ((end.timestamp() - start.timestamp()) * 1000) + ms;

        let _ = msg.edit(|m| m.content(&format!("Pong! `[{}ms]`", diff)));

        Ok(())
    }
}

pub struct GpingCommand;

impl Command for GpingCommand {
    fn execute(&self, _: &mut Context, msg: &Message, _: Args) -> CommandResult {
        let _ = msg.channel_id.say("N/A");

        Ok(())
    }
}

pub struct RoleInfoCommand;

impl Command for RoleInfoCommand {
    fn execute(&self, _: &mut Context, msg: &Message, args: Args) -> CommandResult {
        let cache = CACHE.read();

        let guild_id = match cache.guild_channel(msg.channel_id) {
            Some(channel) => channel.read().guild_id,
            None => {
                let _ = msg.channel_id.say("Error finding channel data");

                return Ok(());
            },
        };

        let guild = match cache.guild(guild_id) {
            Some(guild) => guild,
            None => {
                let _ = msg.channel_id.say("Could not find server data");

                return Ok(());
            },
        };

        // It's a pretty inexpensive operation to clone a role, so it should save
        // keeping the cache unlocked.
        let role = if !msg.mention_roles.is_empty() {
            let id = unsafe { msg.mention_roles.get_unchecked(0) };

            match guild.read().roles.values().find(|r| r.id == *id).cloned() {
                Some(role) => role,
                None => {
                    warn!("Couldn't find r{} for c{}", id, msg.channel_id);

                    let _ = msg.channel_id.say("Mentioned role not found; error logged");

                    return Ok(());
                },
            }
        } else if !args.is_empty() {
            let role_name = args.full();

            match guild.read().roles.values().find(|r| r.name == role_name).cloned() {
                Some(role) => role,
                None => {
                    let id = match role_name.parse::<u64>() {
                        Ok(id) => id,
                        Err(_) => {
                            let _ = msg.channel_id.say("Role not found by name");

                            return Ok(());
                        },
                    };

                    match guild.read().roles.values().find(|r| r.id == id).cloned() {
                        Some(role) => role,
                        None => {
                            warn!("Couldn't find r{} for c{}", id, msg.channel_id);
                            let _ = msg.channel_id.say("Role not found; error logged");

                            return Ok(());
                        },
                    }
                },
            }
        } else {
            let _ = msg.channel_id.say("A role name must be given or mentioned");

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

        let _ = msg.channel_id.send_message(|m| m
            .embed(|e| e
                .title(&format!("Role info for {} ({})", role.name, role.id.0))
                .description(&description)
                .colour(role.colour)
                .field("Hoisted", hoisted, true)
                .field("Position", &role.position.to_string(), true)
                .field("Mentionable", mentionable, true)));

        Ok(())
    }
}

pub struct UptimeCommand;

impl Command for UptimeCommand {
    fn execute(&self, ctx: &mut Context, msg: &Message, _: Args) -> CommandResult {
        let (boot, conn) = {
            let data = ctx.data.lock();
            let uptimes = data.get::<ShardUptime>().unwrap();

            if let Some(entry) = uptimes.get(&ctx.shard_id) {
                let boot = entry.boot.to_rfc3339()[..19].to_owned();
                let conn = entry.connection.to_rfc3339()[..19].to_owned();

                (boot, conn)
            } else {
                ("N/A".to_owned(), "N/A".to_owned())
            }
        };

        let name = CACHE.read().user.name.clone();

        let _ = msg.channel_id.send_message(|m| m
            .embed(|e| e
                .colour(0x8700B2)
                .title(&format!("Uptime for {}", name))
                .field("Since Boot", &boot, true)
                .field("Current Connection", &conn, true)));

        Ok(())
    }
}

pub struct UserInfoCommand;

impl Command for UserInfoCommand {
    fn execute(&self, _: &mut Context, msg: &Message, _: Args) -> CommandResult {
        let member = {
            let guild_id = CACHE.read()
                .guild_channel(msg.channel_id)
                .map(|c| c.read().guild_id);

            if let Some(guild_id) = guild_id {
                // Clone so the lock can be dropped ASAP.
                match CACHE.read().guilds.get(&guild_id) {
                    Some(guild) => guild.read().members.get(&msg.author.id).cloned(),
                    None => None,
                }
            } else {
                None
            }
        };
        let discriminator = msg.author.discriminator.to_string();

        let _ = msg.channel_id.send_message(|m| m
            .embed(|mut e| {
                e = e.title(&format!("User info for {}", msg.author.name))
                    .field("ID", &msg.author.id.to_string(), true)
                    .field("Discriminator", &discriminator, true);

                if let Some(ref member) = member {
                    if let Some(joined_at) = member.joined_at {
                        let formatted = format!("{} UTC", &joined_at.to_rfc3339()[..19]);

                        e = e.field("Joined", &formatted, true);
                    }

                    let nick = member.nick.clone()
                        .map_or_else(|| "\u{200b}".to_owned(), |v| v.clone());

                    e = e.field("Nick", &nick, true);

                    if let Some(colour) = member.colour() {
                        let s = format!("rgb({}, {}, {})", colour.r(), colour.g(), colour.b());

                        e = e.colour(colour).field("Colour", &s, true);
                    }
                }

                e
            }));

        Ok(())
    }
}
