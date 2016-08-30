use bot::event_counter::{EventCounter, EventType};
use bot::event_counter;
use chrono::{NaiveDateTime, UTC};
use discord::model::{ChannelId, ChannelType, GameType, OnlineStatus};
use discord::{ChannelRef, State};
use regex::Regex;
use std::env;
use std::collections::BTreeMap;
use ::prelude::*;

pub struct Meta<'a> {
    help: BTreeMap<&'a str, &'a str>,
}

impl<'a> Meta<'a> {
    pub fn new<'b>() -> Meta<'b> {
        let mut map = BTreeMap::new();
        map.insert("8ball", r#"Answers your question.

Answers your question, optionally given, with either a positive or a negative answer. Sometimes nano isn't sure, and will give a neutral response.

Examples:

Don't ask something, just get an answer:
`;8ball` --> "It is positive."

Ask something and get an answer:
`;8ball will I get the job` --> "Very doubtful.""""#);
        map.insert("about", r#"Gives basic information about nano."#);
        map.insert("aestheticcaps", r#"Partial alias of `aesthetic`, capitalizing and bolding everything."#);
        map.insert("aesthetic", r#"ｄａｎｋ

Produces widened text of the given input, aesthetic-style.

Only widens latin characters.

Example:

`;aesthetic dank meme bro` --> `ｄａｎｋ ｍｅｍｅ ｂｒｏ`"#);
        map.insert("aescaps", r#"Alias of `aestheticcaps`."#);
        map.insert("aes", r#"Alias of `aesthetic`."#);
        map.insert("anime", r#"Searches for an anime by name
If the first result is not a TV show, then the first 3 results will be searched for a TV result. If there is one, that will be used. This is done to prioritize TV over OVAs.

Basic information such as the title, a Hummingbird link, when it aired, the score, the current status of the show, and episode count will be returned.

Example:
`;anime nichijou`"#);
        map.insert("bigemoji", r#"Links to a larger, 112x112 version of an
emoji."#);
        map.insert("channelinfo", r#"Gives information about a channel"#);
        map.insert("choose", r#"Randomly chooses an item in the list of choices.

The list of choices can either be separated by spaces (` `) or by commas (`, `), similar to CSV format.

CSV _must_ be used when at least 1 choice is multiple words long.

At least 2 choices must be given.

Examples:

Giving a list separated by spaces:
`;choose cat dog bird turtle`

Giving a list separated by commas:
`;choose cat, dog, bird, turtle`"#);
        map.insert("coinflip", r#"Flips a coin, heads or tails.

Does precisely what it says.

Example:

`;coinflip`"#);
        map.insert("define", r#"Defines a word or phrase.

Searches urbandictionary for the given word or phrase, giving back the first result.

Results _can_ and _often will_ be NSFW due to the nature of urbandictionary.

Example:

`;define lmgtfy`"#);
        map.insert("delete", r#"Deletes a tag by name."#);
        map.insert("get", r#"Gets a tag by name where using the shortcut will not work.

As command names will shadow tag names, this is sometimes necessary.

Example:

A tag named 'coinflip' can be accessed via:

`;get coinflip`"#);
        map.insert("hello", r#"Says hello to you or someone you mention.

Says hi to you! If you mention someone, nano will say hi to them instead.

Examples:

`;hello` --> `Hey @username!`

`;hello @friend` --> `Selamat pagi, @friend!`"#);
        map.insert("help", r#"oh. ok."#);
        map.insert("info", r#"Lists information about a tag, such as the number of uses and the owner."#);
        map.insert("invite", r#"Gives an invite link to invite nano to your server."#);
        map.insert("join", r#"Joins your voice channel, or one by name if one
is given.

Nano can be in a voice channel in multiple servers at once, but only one voice channel per server at once.

If you do not give the name of a Voice Channel, nano will join yours if you are in one.

Example:

`;join #general`"#);
        map.insert("leave", r#"Leaves the current voice channel."#);
        map.insert("list", r#"Creates a list of all tags on this server.

If the list of tags is over one message long, the list will be private messaged."#);
        map.insert("mfw", r#"Your face right now.

Outputs a random emoji.

Example:

`;mfw` --> `:grin:`"#);
        map.insert("ping", r#"Pong! Checks if nano is working, giving the response time.

Nano will reply with "Pong!" and will edit her message with how long it took to send the message.

Example:

`;ping` --> `Pong!`"#);
        map.insert("pi", r#"Lists pi up to the number of digits given

Outputs pi up to the number of digits given (if given). The default number of digits to list to is 100, while the maximum is 1000.

Example:

`;pi 3` --> `3.141`"#);
        map.insert("play", r#"Adds a song to the queue of songs given a URL.

Most popular video websites should be supported.

Will also join your current voice channel if you are in one and nano is not already in a voice channel.

The queue of songs can be viewed via the `queue` command.

Example:

`;play https://www.youtube.com/watch?v=nGtQY2VpVsM`"#);
        map.insert("purge", r#"Deletes the number of messages given, in descending order."#);
        map.insert("queue", r#"Retrieves a list of queued songs"#);
        map.insert("rename", r#"Renames a tag from one key to another.

Example:

`;rename my pic --> someone else's pic`"#);
        map.insert("roleinfo", r#"Lists info about a role by name.

Example:

`;roleinfo Mod`"#);
        map.insert("roll", r#"Rolls for a number between two numbers.

Rolls between two numbers, if they are given. Otherwise, rolls between 1 and 6.

Either 0 or 2 numbers _must_ be given.

The second number _must_ be greater than the other.

Numbers _must_ be integers (whole numbers).

Examples:

Giving 0 numbers:
`;roll`

Giving 2 numbers:
`;roll 1 42`"#);
        map.insert("roulette", r#"Russian roulette; will you survive?

See You Space Cowboy...

Example:

`;roulette` --> "BANG! @you was shot""#);
        map.insert("say", r#"Makes nano say something

Example:

`;say hi!`"#);
        map.insert("search", r#"Searches for a tag by key name.

Example:

`;search dog`"#);
        map.insert("serverinfo", r#"Displays information about the current server.

Does what it says. Lists the following information:

- server name
- id of server
- owner name and discriminator
- name of the region where the voice server is
- total number of members (including offline)
- number of text and voice channels, respectively
- the date that the server was created
- the url of the server's image
- a list of all server roles

Example:

`!serverinfo`"#);
        map.insert("set", r#"Sets a tag by key-and-value.

Example:

`;set cat: https://i.imgur.com/some_url.jpg`;

And to use the tag:

`;cat`"#);
        map.insert("skip", r#"Votes to skip a song.

Only when the number of votes is reached will the current song be passed."#);
        map.insert("stats", r#"Returns a list of the top 35 peoples' message counts.

If there are more than 35 people, they will not be listed."#);
        map.insert("status", r#"Lists information about the current song playing."#);
        map.insert("teams", r#"Creates a number of teams for the usernames given.

Creates randomized teams in the amount given, containing the players given.

Teams do _not_ have to be equal.

Examples:

`;teams 2 a, b, c, d, e, f`

`;teams 3 a, b, c, d, e, f`"#);
        map.insert("uptime", r#"Lists the amount of time nano has been up."#);
        map.insert("userinfo", r#"Displays information about yourself or another member.

Will display basic information about you or a searched member.
Searching by mentioning them, saying their name, etc. are supported.

This information includes:

- username
- user id
- nickname on the server
- user discriminator
- avatar URL if they have one
- status (online/idle/offline)
- current game being played
- when the account was created
- when the account joined the server
- list of roles the user has on this server

Examples:

`;userinfo`
`;userinfo @someone`
`;userinfo zey#5479`
`;userinfo zey`"#);
        map.insert("weather", r#"Retrieves the current weather for a location

Will retrieve the current weather data for a given location, as well as a basic summary of the next week's weather.

Examples:

Retrieve weather for a location:
`;weather New York City`"#);
        Meta {
            help: map,
        }
    }

    pub fn about(&self, context: Context) {
        let client_id = match env::var("DISCORD_CLIENT_ID") {
            Ok(client_id) => client_id,
            Err(_why) => {
                error!("[env] No Client ID");

                let _msg = req!(context.say("Error getting client id"));

                return;
            }
        };

        let _msg = req!(context.say(format!(r#"
nano v{}

Developed by zey (<@114941315417899012>)
Library: discord-rs

nano is a general-purpose, jack-of-all trades bot that can do just about
anything you need. This ranges from metadata about servers, randomizations, mod
tools, tagging, music, server-specific configuration, and more.

Join the nano & friends server!
https://discord.gg/MFHVwvW


Invite nano to your server:
https://discordapp.com/oauth2/authorize?client_id={}&scope=bot&permissions=8
"#, env!("CARGO_PKG_VERSION"), client_id)));
    }

    pub fn big_emoji(&self, context: Context) {
        let arg_found = context.arg(1);

        let arg = match arg_found.as_str() {
            Ok(arg) => arg,
            Err(_why) => {
                let _msg = req!(context.say("Must provide an emoji"));

                return;
            },
        };
        // A fast way to check this. This will technically have the ability to
        // provide a false error message (such as when someone args "test").
        if !arg.starts_with('<') {
            let _msg = req!(context.say("Can not process regular emojis"));

            return;
        }

        let error = "Error processing emoji";

        let re = match Regex::new(r"<:(.*):([0-9]+)>") {
            Ok(re) => re,
            Err(_why) => {
                let _msg = req!(context.say(error));

                return;
            },
        };
        let caps = match re.captures(arg) {
            Some(re) => re,
            None => {
                let _msg = req!(context.say(error));

                return;
            },
        };

        let id = match caps.at(2) {
            Some(id) => id,
            None => {
                let _msg = req!(context.say(error));

                return;
            },
        };

        let text = format!("https://cdn.discordapp.com/emojis/{}.png", id);

        let _msg = req!(context.say(text));
    }

    pub fn channel_info(&self, context: Context, state: &State) {
        let channel_mentions = context.channel_mentions(state);

        let id = if let Some(channel) = channel_mentions.get(0) {
            channel.id
        } else if context.arg(1).exists() {
            if let Ok(id) = context.arg(1).as_u64() {
                ChannelId(id)
            } else {
                let _msg = req!(context.say("Can't find channel"));

                return;
            }
        } else {
            context.message.channel_id
        };

        let channel = if let Some(find) = state.find_channel(&id) {
            let mcid = context.message.channel_id;
            match find {
                ChannelRef::Public(server, channel) => {
                    let srvid = if let Some(find) = state.find_channel(&mcid) {
                        match find {
                            ChannelRef::Public(srv, _channel) => srv.id,
                            _ => {
                                let text = "This channel is not supported";

                                let _msg = req!(context.say(text));

                                return;
                            },
                        }
                    } else {
                        let _msg = req!(context.say("Can't find server"));

                        return;
                    };

                    if server.id != srvid {
                        let text = "Can't find cross-server channels";

                        let _msg = req!(context.say(text));

                        return;
                    }

                    channel
                },
                _ => {
                    let text = "Private Channels are not supported";

                    let _msg = req!(context.say(text));

                    return;
                },
            }
        } else {
            let _msg = req!(context.say("Could not find channel"));

            return;
        };

        let secs = channel.id.creation_date().sec;
        let created_at = NaiveDateTime::from_timestamp(secs, 0)
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();

        let mut text = format!(r#"```xl
     Name: {}
       ID: {}
    Topic: {}
     Type: {}
  Created: {}"#, channel.name,
                 channel.id,
                 channel.topic.as_ref().unwrap_or(&String::new()),
                 channel.kind.name(),
                 created_at);

        if channel.kind == ChannelType::Voice {
            let bitrate = (channel.bitrate.unwrap_or(0) / 1024) as u64;
            let user_limit = channel.user_limit.unwrap_or(0);

            text.push_str(&format!(r#"
  Bitrate: {}kbps
User limit: {}"#, channel.bitrate.unwrap_or(0) / 1024,
                  channel.user_limit.unwrap_or(0)));
        }

        text.push_str("```");

        let _msg = req!(context.say(text));
    }

    pub fn events(&self, context: Context, counter: &EventCounter) {
        let mut text = String::from("Events seen:\n");

        let arg_found = context.arg(1);

        let event_types = if let Ok(arg) = arg_found.as_str() {
            if arg == "--all" {
                event_counter::event_types().to_vec()
            } else {
                vec![
                    EventType::MessageCreate,
                    EventType::PresenceUpdate,
                    EventType::TypingStart,
                ]
            }
        } else {
            vec![
                EventType::MessageCreate,
                EventType::PresenceUpdate,
                EventType::TypingStart,
            ]
        };

        let count_map = counter.map(event_types);

        let mut total = 0;

        for (amount, names) in count_map.iter().rev() {
            for name in names {
                text.push_str(&format!("
- {}: {}", name, amount)[..]);

                total += *amount;
            }
        }

        text.push_str(&format!("\n\nTotal: {}", total)[..]);

        let _msg = req!(context.say(text));
    }

    pub fn help(&self, context: Context) {
        let command = context.text(0);

        // If no command was given, list the names of all commands
        if command.is_empty() {
            let mut names = "```\n".to_owned();

            for key in self.help.keys() {
                names.push_str("- ");
                names.push_str(key);
                names.push('\n');
            }

            names.push_str("```Use `help <command>` for info about a command");

            let _msg = req!(context.say(names));

            return;
        }

        match self.help.get(&command[..]) {
            Some(help) => {
                let _msg = req!(context.say(*help));
            },
            None => {
                let text = format!("Command `{}` not found", &command);

                let _msg = req!(context.say(text));
            },
        }
    }

    pub fn invite(&self, context: Context) {
        let client_id = match env::var("DISCORD_CLIENT_ID") {
            Ok(client_id) => client_id,
            Err(_why) => {
                error!("[base] No Client ID");
                let _msg = req!(context.say("Error getting client id"));
                return;
            }
        };

        let _msg = context.say(format!(r#"Here's a link to invite me:
https://discordapp.com/oauth2/authorize?client_id={}&scope=bot&permissions=3222534
"#, client_id));
    }

    pub fn ping(&self, context: Context) {
        let start = UTC::now();
        let msg = req!(context.say("Ping!"));
        let end = UTC::now();

        let ms = {
            let end_ms = end.timestamp_subsec_millis() as i64;
            let start_ms = start.timestamp_subsec_millis() as i64;

            end_ms - start_ms
        };
        let secs = (end.timestamp() - start.timestamp()) * 1000;
        let diff = secs + ms;

        let _msg = req!(context.edit(&msg, format!("Ping! `[{}ms]`", diff)));
    }

    pub fn role_info(&self, context: Context, state: &State) {
        let server = match state.find_channel(&context.message.channel_id) {
            Some(ChannelRef::Public(server, _channel)) => server,
            _ => {
                let _msg = req!(context.say("Server not found"));
                return;
            },
        };

        let name = context.text(0);

        let role_opt = if let Some(r) = context.message.mention_roles.first() {
            server.roles
                .iter()
                .find(|role| role.id == *r)
        } else {
            if name.is_empty() {
                let _msg = req!(context.say("A role name must be given"));

                return;
            }

            server.roles
                .iter()
                .find(|role| role.name == name)
        };

        let role = match role_opt {
            Some(role) => role,
            None => {
                let _msg = req!(context.say("Role not found"));

                return;
            },
        };

        let created_at = {
            let secs = role.id.creation_date().sec;

            NaiveDateTime::from_timestamp(secs, 0)
                .format("%Y-%m-%d %H:%M:%S")
                .to_string()
        };

        let info = format!(r#"```xl
       Name: {}
         ID: {}
    Hoisted: {}
Mentionable: {}
    Created: {}```"#, role.name,
                      role.id,
                      role.hoist,
                      role.mentionable,
                      created_at);

        let _msg = req!(context.say(info));
    }

    pub fn server_info(&self, context: Context, state: &State) {
        let server = match state.find_channel(&context.message.channel_id) {
            Some(ChannelRef::Public(server, _channel)) => server,
            _ => {
                let _msg = req!(context.say("Server not found"));

                return;
            },
        };

        let owner_info = {
            let mut owner_info = None;

            'servers: for server_iter in state.servers() {
                if server_iter.id != server.id {
                    continue;
                }

                for member in &server_iter.members {
                    owner_info = Some(format!("{}#{}",
                                              member.user.name,
                                              member.user.discriminator));

                    break 'servers;
                }
            }

            if let Some(owner_info) = owner_info {
                owner_info
            } else {
                String::from("Unknown")
            }
        };

        let mut channels = [0, 0];

        for channel in &server.channels {
            match channel.kind {
                ChannelType::Text => {
                    channels[0] += 1;
                },
                ChannelType::Voice => {
                    channels[1] += 1;
                },
                _ => {},
            }
        }

        let text = format!(r#"```xl
Name: {}
ID: {}
Owner: {}
Region: {}
Members: {}
Channels: {} text/{} voice
Created: {}
Icon: {}```"#, server.name,
               server.id,
               owner_info,
               server.region,
               server.member_count,
               channels[0],
               channels[1],
               server.id.creation_date().sec,
               server.icon_url().unwrap_or("N/A".to_owned()));

        let _msg = req!(context.say(text));
    }

    pub fn set_status(&self, context: Context) {
        let author_var = if let Ok(var) = env::var("AUTHOR_ID") {
            var
        } else {
            let _msg = req!(context.say("Error setting status"));
            error!("[env] AUTHOR_ID env var not set");

            return;
        };

        let author_id = if let Ok(id) = author_var.parse::<u64>() {
            id
        } else {
            let _msg = req!(context.reply("Error setting status"));

            return;
        };

        if context.message.author.id.0 != author_id {
            let _msg = req!(context.reply("Only the bot owner can set status"));

            return;
        }

        let new_status = context.text(0);

        let conn = context.conn.lock().unwrap();
        conn.set_game_name(new_status);
    }

    pub fn user_info(&self, context: Context, state: &State) {
        let arg = context.arg(1);

        let user = if let Some(user) = context.message.mentions.get(0) {
            &user
        } else if let Ok(info) = arg.as_str() {
            let (name, discriminator) = if let Some(pos) = info.find('#') {
                let split = info.split_at(pos);

                println!("{:?}", split);

                let discrim = match split.1.replace("#", "").parse::<u16>() {
                    Ok(discrim) => discrim,
                    Err(_why) => {
                        let text = "Error retrieving user data";
                        let _msg = req!(context.say(text));

                        return;
                    },
                };

                (split.0, Some(discrim))
            } else {
                (info, None)
            };

            let server = match state.find_channel(&context.message.channel_id) {
                Some(ChannelRef::Public(server, _channel)) => server,
                _ => {
                    let text = "Error finding user data";
                    let _msg = req!(context.say(text));

                    return;
                },
            };

            let mut member_found = None;

            for member in &server.members {
                if if let Some(discrim) = discriminator {
                    member.user.discriminator == discrim && member.user.name == name
                } else {
                    member.user.name == name
                } {
                    member_found = Some(member);

                    break;
                }
            }

            if let Some(member) = member_found {
                &member.user
            } else {
                let _msg = req!(context.say("Error finding user"));

                return;
            }
        } else {
            &context.message.author
        };

        let mut text = format!(r#"```xl
     Username: {}
Discriminator: {}
           ID: {}
   Avatar URL: {}"#, user.name,
                     user.discriminator,
                     user.id,
                     user.avatar_url().unwrap_or("N/A".to_owned()));

        for server in state.servers() {
            let channel_found = server.channels.iter().any(|channel| {
                channel.id == context.message.channel_id
            });

            if !channel_found {
                continue;
            }

            let mut found = None;

            for member in &server.members {
                if member.user.id == user.id {
                    found = Some(member);

                    break;
                }
            }

            if let Some(member) = found {
                if let Some(ref nick) = member.nick {
                    text.push_str(&format!(r#"
     Nickname: {}"#, nick));
                }

                let mut presence_found = None;

                for presence in &server.presences {
                    if presence.user_id == member.user.id {
                        presence_found = Some(presence);

                        break;
                    }
                }

                let mut role_names = vec![];

                for role in &server.roles {
                    if member.roles.contains(&role.id) {
                        role_names.push(&role.name[..]);
                    }
                }

                let role_list: String = role_names.join(", ");

                let (s_game, s_name) = if let Some(presence) = presence_found {
                    let status_game = if let Some(ref game) = presence.game {
                        let kind = match game.kind {
                            GameType::Playing => "Playing",
                            GameType::Streaming => "Streaming",
                        };

                        let url = game.url.as_ref()
                            .map(|u| format!("({})", u))
                            .unwrap_or_default();

                        format!("{} {} {}", kind, game.name, url)
                    } else {
                        "N/A".to_owned()
                    };

                    let status_name = match presence.status {
                        OnlineStatus::Idle => "Idle",
                        OnlineStatus::Offline => "Offline",
                        OnlineStatus::Online => "Online",
                    };

                    (status_game, status_name)
                } else {
                    (String::from("N/A"), "")
                };

                let time = user.id.creation_date().sec;

                let created_at = NaiveDateTime::from_timestamp(time, 0)
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string();

                text.push_str(&format!(r#"
       Status: {}
         Game: {}
      Created: {}
       Joined: {}
        Roles: {}```"#, s_name,
                        s_game,
                        created_at,
                        &member.joined_at[..19].replace('T', " "),
                        role_list)[..]);
            }
        }

        let _msg = req!(context.say(text));
    }
}
