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

use discord::{ChannelRef, State};
use ::prelude::*;

pub struct Stats;

impl Stats {
    pub fn new() -> Stats {
        Stats
    }

    pub fn stats(&self, context: Context, state: &State) {
        use diesel::prelude::*;
        use models::{Member, User};
        use schema::members::dsl as members_dsl;
        use schema::users::dsl as users_dsl;

        let s = match state.find_channel(&context.message.channel_id) {
            Some(ChannelRef::Public(server, _channel)) => server,
            _ => {
                let _msg = req!(context.say("Could not find server"));

                return;
            },
        };

        let search_res = members_dsl::members
            .filter(members_dsl::server_id.eq(s.id.0 as i64))
            .order(members_dsl::message_count.desc())
            .limit(30)
            .load(context.db);

        let member_list: Vec<Member> = match search_res {
            Ok(member_list) => member_list,
            Err(why) => {
                warn!("[stats] Err getting members for guild {}: {:?}",
                      s.id,
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

        for member in member_list {
            let user_res = users_dsl::users
                .filter(users_dsl::id.eq(member.user_id))
                .first::<User>(context.db);

            let user = match user_res {
                Ok(user) => user,
                Err(why) => {
                    warn!("[stats] Err getting user {}: {:?}", member.id, why);

                    continue;
                },
            };

            list.push_str(&format!("{}. {}: {}\n",
                                   rank,
                                   user.username,
                                   member.message_count));

            rank += 1;
        }

        let _msg = req!(context.say(list));
    }
}
