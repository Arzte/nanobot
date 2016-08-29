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

        let search_res = {
            let db = arc!(context.db);

            members_dsl::members
                .filter(members_dsl::server_id.eq(s.id.0 as i64))
                .order(members_dsl::message_count.desc())
                .limit(30)
                .load(&db)
        };

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
            let user_res = {
                let db = arc!(context.db);

                users_dsl::users
                    .filter(users_dsl::id.eq(member.user_id))
                    .first::<User>(&db)
            };

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
