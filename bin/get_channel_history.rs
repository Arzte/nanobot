// ISC License (ISC)
//
// Copyright (c) 2016, Austin Hellyer <hello@austinhellyer.me>
//
// Permission to use, copy, modify, and/or distribute this software for any
// purpose with or without fee is hereby granted, provided that the above
// copyright notice and this permission notice appear in all copies.
//
// THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH
// REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY
// AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY SPECIAL, DIRECT,
// INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM
// LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR
// OTHER TORTIOUS ACTION, ARISING OUT OF OR IN CONNECTION WITH THE USE OR
// PERFORMANCE OF THIS SOFTWARE.

extern crate discord;
extern crate dotenv;
extern crate rusqlite;

use discord::model::{ChannelId, Channel, MessageId};
use discord::{Discord, GetMessages};
use rusqlite::Connection;
use std::collections::HashMap;
use std::env;
use std::path::Path;

struct DataMember {
    message_count: i64,
    nickname: Option<String>,
    server_id: String,
    user_id: String,
}

struct DataMessage {
    channel_id: String,
    content: String,
    member_id: String,
    timestamp: String,
    uid: String,
}

struct DataUser {
    discriminator: String,
    name: String,
    uid: String,
}

#[allow(let_and_return)]
fn main() {
    dotenv::from_path(Path::new("./.env")).ok();

    let discord = Discord::from_bot_token(
        &env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN"),
    ).expect("login failed");

    let channel_id_as_string = env::args()
        .nth(1)
        .expect("channel id as argument required");

    let cid = ChannelId(match channel_id_as_string.clone().parse::<u64>() {
        Ok(cid) => cid,
        Err(why) => panic!("Error parsing {:?} to u64: {:?}",
                           channel_id_as_string,
                           why),
    });

    let conn = Connection::open(Path::new("./database.db"))
        .expect("invalid db path");

    let mut statement = conn.prepare(&format!("SELECT uid FROM messages
                                    WHERE channel_id='{}'
                                    ORDER BY uid DESC LIMIT 1",
                                   channel_id_as_string))
        .unwrap();
    let mut uid_iter = statement.query_map(&[], |row| {
        let t: String = row.get(0);

        t
    }).unwrap();
    let uid = match uid_iter.nth(0) {
        Some(uid) => uid.unwrap(),
        None => String::from("0"),
    };

    let mut after = MessageId(uid.parse::<u64>().unwrap());

    let mut members: HashMap<u64, DataMember> = HashMap::new();
    let mut msgs = vec![];
    let mut users = HashMap::new();

    let sid = match discord.get_channel(cid) {
        Ok(result) => match result {
            Channel::Public(channel) => channel.server_id.0 as i64,
            _ => panic!("Can not retrieve history for a group or PM"),
        },
        Err(why) => panic!("Error retrieving channel {}: {:?}", cid.0, why),
    };
    let sid_as_string = format!("{}", sid);

    loop {
        let mut messages = match discord.get_messages(
            cid,
            GetMessages::After(after),
            Some(100)
        ) {
            Ok(messages) => messages,
            Err(why) => panic!("{:?}", why),
        };

        if messages.is_empty() {
            break;
        }

        messages.reverse();

        for message in messages {
            msgs.push(DataMessage {
                channel_id: format!("{}", message.channel_id.0),
                content: message.content.clone(),
                member_id: format!("{}", message.author.id.0),
                timestamp: format!("{}", message.author.id.0),
                uid: format!("{}", message.id.0),
            });

            let mut found = false;

            if let Some(member) = members.get_mut(&message.author.id.0) {
                member.message_count += 1i64;

                found = true;
            }

            if !found {
                members.insert(message.author.id.0, DataMember {
                    message_count: 1,
                    nickname: None,
                    server_id: sid_as_string.clone(),
                    user_id: format!("{}", message.author.id.0),
                });
            }

            if let None = users.get(&message.author.id.0) {
                users.insert(message.author.id.0, DataUser {
                    discriminator: message.author.discriminator.to_string(),
                    name: message.author.name,
                    uid: format!("{}", message.author.id.0),
                });
            }

            after = MessageId(message.id.0);
        }

        println!("Messages loaded: {}\nUsers loaded: {}",
                 msgs.len(),
                 users.len());
    }

    if msgs.is_empty() {
        println!("No messages to record.");

        return;
    }

    for member in members.values() {
        let mut statement = conn.prepare(
            &format!("SELECT message_count FROM members WHERE user_id='{}'
                      AND server_id='{}'",
                     member.user_id,
                     sid_as_string)
        ).unwrap();

        let mut iter = statement.query_map(&[], |row| {
            let t: i64 = row.get(0);

            t
        }).unwrap();

        match iter.nth(0) {
            Some(count) => {
                let new_count = count.unwrap() + member.message_count;
                conn.execute("UPDATE members SET message_count=$1
                              WHERE member_id=$2 AND server_id=$3",
                             &[
                                 &new_count,
                                 &member.user_id,
                                 &sid
                             ]).ok();
            },
            None => {
                conn.execute("INSERT INTO members
                              (message_count, nickname, server_id, user_id)
                              VALUES ($1, $2, $3, $4)",
                             &[
                                 &member.message_count,
                                 &member.nickname,
                                 &member.server_id,
                                 &member.user_id,
                             ]).ok();
            },
        }
    }

    for user in users.values() {
        let mut statement = conn.prepare(
            &format!("SELECT uid FROM users WHERE uid='{}'", user.uid)
        ).unwrap();
        let mut iter = statement.query_map(&[], |row| {
            let t: String = row.get(0);

            t
        }).unwrap();

        match iter.nth(0) {
            Some(_uid) => {},
            None => {
                conn.execute("INSERT INTO users (discriminator, name, uid)
                              VALUES ($1, $2, $3)",
                             &[
                                 &user.discriminator,
                                 &user.name,
                                 &user.uid,
                             ]).ok();
            },
        }
    }

    for message in msgs {
        conn.execute("INSERT INTO messages
                      (channel_id, content, member_id, timestamp, uid)
                      VALUES ($1, $2, $3, $4, $5)",
                     &[
                         &message.channel_id,
                         &message.content,
                         &message.member_id,
                         &message.timestamp,
                         &message.uid,
                     ]).ok();
    }
}
