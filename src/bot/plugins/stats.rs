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

use discord::ChannelRef;
use ::prelude::*;

pub struct Stats;

impl Stats {
    pub fn new() -> Stats {
        Stats
    }

    pub fn stats(&self, context: Context) {
        let state = context.state.lock().unwrap();
        let server_id = match state.find_channel(&context.message.channel_id) {
            Some(ChannelRef::Public(server, _channel)) => server.id,
            _ => {
                let _msg = req!(context.say("Could not find server"));

                return;
            },
        };
        drop(state);

        let db: PgConn = context.db.lock().unwrap();

        let search_res: PgRes = db.query(
            "select id, message_count, user_id where server_id = $1 limit 30
             order by message_count desc",
            &[&(server_id.0 as i64)]
        );

        let member_list = match search_res {
            Ok(rows) => rows,
            Err(why) => {
                warn!("[stats] Err getting members for guild {}: {:?}",
                      server_id,
                      why);

                let _msg = req!(context.say("Error generating list"));

                return;
            },
        };

        if member_list.is_empty() {
            let _msg = req!(context.say("No members found /shrug"));

            return;
        }

        let mut list = String::new();
        let mut rank = 1;

        for member in member_list.iter() {
            let user_id: i64 = member.get(2);
            let params: Params = vec![
                &user_id,
            ];
            let user_res: PgRes = db.query("select username from users where
                                            id = $1", &params);

            let user = match user_res {
                Ok(ref rows) if !rows.is_empty() => rows.get(0),
                Ok(_rows) => {
                    let id: i32 = member.get(0);

                    warn!("[stats] No user for member {}", id);

                    continue;
                },
                Err(why) => {
                    let id: i32 = member.get(0);

                    warn!("[stats] Err getting user {}: {:?}", id, why);

                    continue;
                },
            };

            let username: String = user.get(0);
            let message_count: i64 = member.get(1);

            list.push_str(&format!("{}. {}: {}\n",
                                   rank,
                                   username,
                                   message_count));

            rank += 1;
        }

        let _msg = req!(context.say(list));
    }
}
