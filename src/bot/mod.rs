pub mod plugins;

mod event_counter;

use chrono::{DateTime, UTC};
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel;
use discord::model::{
    Event,
    LiveServer,
    PossibleServer,
    ServerId,
    Server,
    User as DiscordUser
};
use discord::{ChannelRef, Connection as DiscordConnection, Discord, State};
use self::event_counter::EventCounter;
use self::plugins::*;
use self::plugins::misc::Aesthetic;
use std::sync::{Arc, Mutex};
use ::models::{NewGuild, NewMember, NewUser};
use ::prelude::*;
use ::utils;

pub struct Uptime {
    /// Unix timestamp of when the program itself started
    pub boot: DateTime<UTC>,
    /// Unix timestamp of when the current connection was made. This should
    /// probably _technically_ be an Option, _but_ a user will never be able to
    /// request the uptime if there is no connection, so it's okay.
    pub connection: DateTime<UTC>,
}

pub struct Bot<'a> {
    pub admin: Admin,
    pub conversation: Conversation,
    pub event_counter: EventCounter,
    pub meta: Meta<'a>,
    pub misc: Misc<'a>,
    pub music: Music,
    pub pic: Pic,
    pub random: Random,
    pub stats: Stats,
    pub state: State,
    pub tags: Tags,
    pub tv: Tv,
    pub uptime: Uptime,
}

impl<'a> Bot<'a> {
    pub fn new<'b>() -> Bot<'b> {
        Bot {
            admin: Admin::new(),
            conversation: Conversation::new(),
            event_counter: EventCounter::new(),
            meta: Meta::new(),
            misc: Misc::new(),
            music: Music::new(),
            pic: Pic::new(),
            random: Random::new(),
            stats: Stats::new(),
            state: State::new(utils::make_fake_ready_event()),
            tags: Tags::new(),
            tv: Tv::new(),
            uptime: Uptime {
                boot: UTC::now(),
                connection: UTC::now(),
            },
        }
    }

    pub fn handle_event(&mut self,
                        event: Event,
                        conn: Arc<Mutex<DiscordConnection>>,
                        db: &PgConnection,
                        discord: &Arc<Mutex<Discord>>) {
        debug!("[event] Handling event");

        match event {
            Event::MessageCreate(message) => {
                debug!("[event] Handling MessageCreate");

                let context = Context::new(conn, db, discord, message);
                self.increment_member_messages(&context);

                self.handle_message(context)
            },
            Event::ServerCreate(possible_server) => {
                debug!("[event] Handling ServerCreate");

                match possible_server {
                    PossibleServer::Online(server) => {
                        self.handle_server_create(db, server);
                    },
                    PossibleServer::Offline(_server_id) => {},
                }
            },
            Event::ServerDelete(possible_server) => {
                debug!("[event] Handling ServerDelete");

                let server_id = match possible_server {
                    PossibleServer::Online(server) => server.id,
                    PossibleServer::Offline(server_id) => server_id,
                };

                self.handle_server_delete(db, server_id);
            },
            Event::ServerUpdate(server) => {
                debug!("[event] Handling ServerUpdate");

                self.handle_server_update(db, server);
            },
            Event::ServerMemberUpdate { server_id, user, nick, .. } => {
                debug!("[event] ServerMemberUpdate");

                self.handle_server_member_update(db, server_id, user, nick);
            },
            _ => {},
        };
    }

    pub fn handle_message(&mut self, context: Context) {
        if !context.message.content.starts_with(';') {
            debug!("[handle-message] Not a command");

            return;
        }

        // Ignore ourself
        if context.message.author.id == self.state.user().id {
            debug!("[handle-message] Ignoring ourself");

            return;
        }

        // Ignore other bots
        {
            let s = self.state.find_channel(&context.message.channel_id);

            if let Some(ChannelRef::Public(server, _channel)) = s {
                let finding = server.members
                    .iter()
                    .find(|mem| mem.user.id == context.message.author.id);

                if let Some(member) = finding {
                    if member.user.bot {
                        debug!("[handle-message] Ignoring a bot's message");

                        return;
                    }
                }
            }
        }

        // Retrieve the first command. If one doesn't exist, see if a tag exists
        // for it by name.
        let cmd_str = String::from(req!(context.arg(0).as_str()));
        let cmd = &cmd_str[..];

        debug!("[handle-message] Processing command '{}'", cmd);

        match cmd {
            "8ball" => self.random.magic_eight_ball(context),
            "aescaps" => self.misc.aesthetic(context, vec![
                Aesthetic::Bold,
                Aesthetic::Caps,
            ]),
            "aestheticcaps" => self.misc.aesthetic(context, vec![
                Aesthetic::Bold,
                Aesthetic::Caps,
            ]),
            "aesthetic" => self.misc.aesthetic(context, vec![]),
            "aes" => self.misc.aesthetic(context, vec![]),
            "about" => self.meta.about(context),
            "anime" => self.tv.anime(context),
            "bigemoji" => self.meta.big_emoji(context),
            "channelinfo" => self.meta.channel_info(context, &self.state),
            "choose" => self.random.choose(context),
            "coinflip" => self.random.coinflip(context),
            "define" => self.conversation.define(context),
            "delete" => self.tags.delete(context, &self.state),
            "events" => self.meta.events(context, &self.event_counter),
            "hello" => self.misc.hello(context),
            "help" => self.meta.help(context),
            "info" => self.tags.info(context, &self.state),
            "invite" => self.meta.invite(context),
            "join" => self.music.join(context, &self.state),
            "leave" => self.music.leave(context, &self.state),
            "list" => self.tags.list(context, &self.state),
            "mfw" => self.misc.mfw(context),
            "ping" => self.meta.ping(context),
            "pi" => self.misc.pi(context),
            "play" => self.music.play(context, &self.state),
            "purge" => self.admin.purge(context, &self.state),
            "queue" => self.music.queue(context, &self.state),
            "rename" => self.tags.rename(context, &self.state),
            "roleinfo" => self.meta.role_info(context, &self.state),
            "roll" => self.random.roll(context),
            "roulette" => self.random.roulette(context),
            "say" => self.misc.say(context),
            "search" => self.tags.search(context, &self.state),
            "serverinfo" => self.meta.server_info(context, &self.state),
            "setstatus" => self.meta.set_status(context),
            "set" => self.tags.set(context, &self.state),
            "skip" => self.music.skip(context, &self.state),
            "stats" => self.stats.stats(context, &self.state),
            "status" => self.music.status(context, &self.state),
            "teams" => self.random.teams(context),
            "uptime" => self.misc.uptime(context, &self.uptime),
            "userinfo" => self.meta.user_info(context, &self.state),
            "weather" => self.misc.weather(context, &self.state),
            "get" | _ => {
                debug!("[handle-message] Invalid command");

                self.tags.get(context, &self.state);
            },
        }
    }

    fn handle_server_create(&mut self,
                            db: &PgConnection,
                            server: LiveServer) {
        use models::Guild;
        use schema::guilds::dsl::*;
        use schema::guilds::table as guilds_table;

        let exists = {
            guilds.filter(id.eq(server.id.0 as i64))
                .first::<Guild>(db)
        };

        match exists {
            Ok(_guild) => {
                let _update = diesel::update(guilds_table
                    .filter(id.eq(server.id.0 as i64)))
                    .set((
                        active.eq(true),
                        name.eq(&server.name),
                        owner_id.eq(server.owner_id.0 as i64)
                    )).execute(db);
            },
            Err(diesel::NotFound) => {
                let new = NewGuild {
                    active: true,
                    id: server.id.0 as i64,
                    name: &server.name,
                    owner_id: server.owner_id.0 as i64,
                };

                let creation = {
                    diesel::insert(&new)
                        .into(guilds_table)
                        .get_result::<Guild>(db)
                };

                if let Err(why) = creation {
                    warn!("[event:servercreate] Err creating guild: {:?}", why);
                }
            },
            Err(why) => {
                warn!("[event:servercrate] Err filtering guilds: {:?}", why);
            },
        }
    }

    fn handle_server_delete(&mut self,
                            db: &PgConnection,
                            server_id: ServerId) {
        use schema::guilds::dsl::*;

        let update = {
            diesel::update(guilds.filter(id.eq(server_id.0 as i64)))
                .set(active.eq(false))
                .execute(db)
        };

        match update {
            Ok(1) | Ok(0) => {},
            Ok(amount) => {
                warn!("[event:serverdelete] Multiple updated: {:?}", amount);
            },
            Err(why) => {
                warn!("[event:serverdelete] Updating {} {:?}", server_id, why);
            },
        }
    }

    fn handle_server_member_update(&mut self,
                                   db: &PgConnection,
                                   server_id: ServerId,
                                   user: DiscordUser,
                                   nick: Option<String>) {
        use models::Member;
        use schema::members::dsl;
        use schema::members::table as members_table;

        let update = {
            diesel::update(dsl::members
                .filter(dsl::server_id.eq(server_id.0 as i64))
                .filter(dsl::user_id.eq(user.id.0 as i64)))
                .set(dsl::nickname.eq(nick))
                .execute(db)
        };

        match update {
            Ok(1) => {},
            // The member doesn't exist in the database; add it
            Ok(0) => {
                let new = NewMember {
                    message_count: 0,
                    nickname: None,
                    server_id: server_id.0 as i64,
                    user_id: user.id.0 as i64,
                    weather_location: None,
                };

                let creation = {
                    diesel::insert(&new)
                        .into(members_table)
                        .get_result::<Member>(db)
                };

                if let Err(why) = creation {
                    warn!("[event:servermemberupdate] Err making member: {:?}",
                          why);
                }

                check_user(&user, db);
            },
            Ok(amount) => {
                warn!("[event:servermemberupdate] Many updated: {}", amount);
            },
            Err(why) => {
                warn!("[event:servermemberupdate] Err updating: {:?}", why);
            },
        }
    }

    fn handle_server_update(&mut self,
                            db: &PgConnection,
                            srv: Server) {
        use models::Guild;
        use schema::guilds::dsl::*;
        use schema::guilds::table as guilds_table;

        let update = {
            diesel::update(guilds.filter(id.eq(srv.id.0 as i64)))
                .set((
                    active.eq(true),
                    name.eq(&srv.name),
                    owner_id.eq(srv.owner_id.0 as i64)))
                .execute(db)
        };

        match update {
            Ok(1) => {},
            // The server doesn't exist in the database, so add it
            Ok(0) => {
                let new = NewGuild {
                    active: true,
                    id: srv.id.0 as i64,
                    name: &srv.name[..],
                    owner_id: srv.owner_id.0 as i64,
                };

                let creation = {
                    diesel::insert(&new)
                        .into(guilds_table)
                        .get_result::<Guild>(db)
                };

                if let Err(why) = creation {
                    warn!("[event:serverupdate] Err creating guild: {:?}", why);
                }
            },
            Ok(amount) => {
                warn!("[event:serverupdate] Updated many guilds: {}", amount);
            },
            Err(why) => {
                warn!("[event:serverupdate] Err updating guild {}: {:?}",
                      srv.id,
                      why);
            },
        }
    }

    fn increment_member_messages(&self, context: &Context) {
        use models::Member;
        use schema::members::dsl::*;
        use schema::members::table as members_table;

        let server = match self.state.find_channel(&context.message.channel_id) {
            Some(ChannelRef::Public(server, _channel)) => server,
            _ => return,
        };

        let retrieval = {
            members
                .filter(server_id.eq(server.id.0 as i64))
                .filter(user_id.eq(context.message.author.id.0 as i64))
                .first::<Member>(context.db)
        };

        match retrieval {
            Ok(member_) => {
                let update = {
                    diesel::update(members.filter(id.eq(member_.id)))
                        .set(message_count.eq(member_.message_count + 1))
                        .execute(context.db)
                };

                match update {
                    Ok(1) => {},
                    Ok(amount) => {
                        warn!("[increment] Incremented many: {}", amount);
                    },
                    Err(why) => {
                        warn!("[increment] Err updating member {}: {:?}",
                              member_.id,
                              why);
                    },
                }

                check_user(&context.message.author, context.db);
            },
            Err(diesel::NotFound) => {
                let new = NewMember {
                    message_count: 1,
                    nickname: None,
                    server_id: server.id.0 as i64,
                    user_id: context.message.author.id.0 as i64,
                    weather_location: None,
                };

                let insertion = diesel::insert(&new)
                    .into(members_table)
                    .get_result::<Member>(context.db);

                if let Err(why) = insertion {
                    warn!("[increment] Err creating member: {:?}", why);
                }

                check_user(&context.message.author, context.db);
            },
            Err(why) => {
                warn!("[increment] Err finding user {} on server {}: {:?}",
                      context.message.author.id,
                      server.id,
                      why);
            },
        }
    }
}

/// Check that a user exists, and if not, make their record
fn check_user(user: &DiscordUser, db: &PgConnection) {
    use models::User;
    use schema::users::dsl::*;
    use schema::users::table as users_table;

    let retrieval = users
        .filter(id.eq(user.id.0 as i64))
        .first::<User>(db);

    match retrieval {
        Ok(_user) => {},
        Err(diesel::NotFound) => {
            let new = NewUser {
                id: user.id.0 as i64,
                bot: user.bot,
                discriminator: user.discriminator as i16,
                username: &user.name,
            };

            let insertion = diesel::insert(&new)
                .into(users_table)
                .get_result::<User>(db);

            if let Err(why) = insertion {
                warn!("[check-user] Err creating user {}: {:?}", user.id, why);
            }
        },
        Err(why) => {
            warn!("[check-user] Err getting {}: {:?}", user.id, why);
        },
    }
}
