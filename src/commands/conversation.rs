use serenity::model::Permissions;
use serenity::model::{Guild, Member, Mentionable, OnlineStatus};
use rand::{self, Rng};
use urbandictionary::UrbanClient;

command!(modping(_ctx, msg) {
    let guild = match msg.guild() {
        Some(guild) => guild,
        None => return Ok(()),
    };
    let guild = guild.read();

    if guild.id != 272410239947767808 && guild.id != 244567637332328449 {
        return Ok(());
    }

    let found_mod = find_by_status(&*guild, OnlineStatus::Online)
        .or_else(|| find_by_status(&*guild, OnlineStatus::DoNotDisturb))
        .or_else(|| find_by_status(&*guild, OnlineStatus::Idle));

    let chosen_mod = match found_mod {
        Some(chosen_mod) => chosen_mod,
        None => {
            let _ = msg.channel_id.say("There are no online mods to ping.");

            return Ok(());
        },
    };

    let content = format!("{}, you were pinged for a mod action by **{}**.",
                          chosen_mod.mention(),
                          msg.author.tag());
    let _ = msg.channel_id.say(&content);
});

command!(udefine(_ctx, msg, args) {
    if args.is_empty() {
        let _ = msg.channel_id.say("No word given");

        return Ok(());
    }

    let mut msg = match msg.channel_id.say("Searching for definition...") {
        Ok(msg) => msg,
        Err(_) => return Ok(()),
    };

    let query = args.join(" ");

    let client = UrbanClient::new();

    let mut response = match client.definitions(&query[..]) {
        Ok(response) => response,
        Err(why) => {
            warn!("Err retrieving word '{}': {:?}", query, why);

            let _ = msg.channel_id.say("Error retrieving definition");

            return Ok(());
        },
    };

    let mut definition = match response.definitions.get_mut(0) {
        Some(definition) => definition,
        None => {
            let _ = msg.edit(|m| m.content("No definition found"));

            return Ok(());
        },
    };

    if definition.definition.len() > 2048 {
        definition.definition.truncate(2045);
        definition.definition.push_str("...");
    }

    let url = format!("https://www.urbandictionary.com/author.php?author={}",
                      definition.author);

    let _ = msg.edit(|m| m
        .embed(|e| e
            .title(&format!("Definition for **{}**", definition.word))
            .description(&definition.definition)
            .colour(0x1D2439)
            .author(|a| a
                .name(&definition.author)
                .url(&url.replace(' ', "%20")))
            .field("Permalink", &format!("[#{}]({})", definition.id, definition.permalink), true)
            .field(":+1:", &definition.thumbs_up.to_string(), true)
            .field(":-1:", &definition.thumbs_down.to_string(), true)));
});

fn find_by_status(guild: &Guild, status: OnlineStatus) -> Option<&Member> {
    let required_perms = Permissions::BAN_MEMBERS
        | Permissions::KICK_MEMBERS
        | Permissions::MANAGE_MESSAGES;

    let mut members = guild.members.iter().filter(|&(user_id, member)| {
        if member.user.read().bot {
            return false;
        }

        if let Some(presence) = guild.presences.get(&user_id) {
            if presence.status != status {
                return false;
            }
        } else {
            return false;
        }

        // Check if the member has at least one of the required permissions.
        match member.permissions() {
            Ok(perms) if perms.contains(required_perms) => return true,
            _ => return false,
        }
    })
        .map(|x| x.1)
        .collect::<Vec<_>>();

    rand::thread_rng().shuffle(&mut members[..]);

    if members.is_empty() {
        None
    } else {
        Some(members.remove(0))
    }
}
