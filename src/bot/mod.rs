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

pub mod event_counter;
pub mod plugins;
pub mod uptime;

pub use self::uptime::Uptime;

use chrono::UTC;
use discord::model::{
    Event,
    LiveServer,
    PossibleServer,
    ServerId,
    Server,
    User as DiscordUser
};
use discord::{
    ChannelRef,
    Connection as DiscordConnection,
    Discord,
    Error as DiscordError,
    State,
    voice,
};
use postgres::Connection as PgConnection;
use self::event_counter::EventCounter;
use self::plugins::*;
use self::plugins::misc::Aesthetic;
use self::plugins::music::{MusicPlaying, MusicState};
use std::sync::mpsc::{self, TryRecvError};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use ::prelude::*;

pub struct Bot {
    connection: Arc<Mutex<DiscordConnection>>,
    db: Arc<Mutex<PgConnection>>,
    discord: Arc<Mutex<Discord>>,
    pub event_counter: Arc<Mutex<EventCounter>>,
    music_state: Arc<Mutex<MusicState>>,
    pub state: Arc<Mutex<State>>,
    pub uptime: Arc<Mutex<Uptime>>,
}

impl Bot {
    pub fn new(discord: Discord,
               conn: DiscordConnection,
               db: PgConnection,
               state: State)
               -> Bot {
        Bot {
            connection: Arc::new(Mutex::new(conn)),
            db: Arc::new(Mutex::new(db)),
            discord: Arc::new(Mutex::new(discord)),
            event_counter: Arc::new(Mutex::new(EventCounter::default())),
            music_state: Arc::new(Mutex::new(MusicState::default())),
            state: Arc::new(Mutex::new(state)),
            uptime: Arc::new(Mutex::new(Uptime {
                boot: UTC::now(),
                connection: UTC::now(),
            })),
        }
    }

    #[allow(map_entry, or_fun_call)]
    pub fn start(&mut self) {
        // So storing the music queue here both creates a problem and solves a
        // problem.
        //
        // The problem it solves, is the timer check on audio. If we lose
        // connection to Discord, it'd not be that ergonomic to bump up the
        // ending times of the playing songs appropriately (as audio will have
        // attempted to continue playing). This also is problematic as I don't
        // have complete control over the audio right now.
        // This is also mostly out of laziness.
        //
        // The problem it creates is that all of the queued music is lost;
        // perhaps this is something to fix in the future.
        //let conn = self.connection.clone();
        //let state_copy = music_state.clone();

        let (tx, rx) = mpsc::channel();
        let discord_copy = self.discord.clone();
        let state_copy = self.music_state.clone();
        let conn = self.connection.clone();

        thread::spawn(move || {
            loop {
                debug!("[music-check] Iterating");

                {
                    let now = UTC::now().timestamp() as u64;
                    let mut state = state_copy.lock().unwrap();

                    // A list of timestamps to remove from the `song_completion`
                    // map.
                    let mut timestamps_to_remove = vec![];

                    // A list of ServerId's to attempt to play the next song in
                    // the queue for.
                    let mut next = vec![];

                    // iter is auto-sorted by key
                    for (k, v) in &state.song_completion {
                        if *k >= now {
                            break;
                        }

                        next.extend_from_slice(v);
                        timestamps_to_remove.push(*k);
                    }

                    for timestamp in timestamps_to_remove {
                        state.song_completion.remove(&timestamp);
                    }

                    for server_id in next {
                        // If there is no queue for the server, remove the
                        // server from having a status.
                        if !state.queue.contains_key(&server_id) {
                            state.status.remove(&server_id);

                            continue;
                        }

                        // If there is nothing in the queue, but it exists, then
                        // remove the server from the queue and status.
                        //
                        // Safe to unwrap since we already checked.
                        if state.queue.get(&server_id).unwrap().is_empty() {
                            state.status.remove(&server_id);
                            state.queue.remove(&server_id);

                            continue;
                        }

                        // Again: safe because we already checked.
                        let request = state.queue.get_mut(&server_id)
                            .unwrap()
                            .remove(0);

                        let stream = match voice::open_ffmpeg_stream(&request.response.filepath) {
                            Ok(stream) => stream,
                            Err(why) => {
                                warn!("[music-check] Err streaming {}: {:?}",
                                      request.response.filepath, why);

                                continue;
                            },
                        };

                        {
                            let mut conn = conn.lock().unwrap();
                            let voice = conn.voice(Some(server_id));
                            voice.play(stream);
                        }

                        let requested_in = request.requested_in;
                        let text = format!("Playing song **{}** requested by _{}_ [duration: {}]",
                                           request.response.data.title,
                                           request.requester_name,
                                           request.format_duration());

                        // Now update the song completion to re-check
                        // specifically once this song is over.
                        {
                            let check_at = now + request.response.data.duration;

                            let entry = state.song_completion
                                .entry(check_at)
                                .or_insert(vec![]);
                            entry.push(server_id);
                        }

                        state.status.insert(server_id, Some(MusicPlaying {
                            req: request,
                            skip_votes_required: 2,
                            skip_votes: vec![],
                            started_at: now,
                        }));

                        let discord = discord_copy.lock().unwrap();
                        let _ = discord.send_message(&requested_in,
                                                     &text,
                                                     "",
                                                     false);
                    }

                    drop(state);
                }

                thread::sleep(Duration::from_secs(1));

                match rx.try_recv() {
                    Ok(_) | Err(TryRecvError::Disconnected) => {
                        info!("[music-check] Killing music check");

                        break;
                    },
                    Err(TryRecvError::Empty) => {},
                }
            }
        });

        info!("[base] Connected");

        {
            let mut uptime = self.uptime.lock().unwrap();
            uptime.connection = UTC::now();
        }

        self.handle_connection();

        // Stop the music queue check
        let _ = tx.send(());
    }

    fn handle_connection(&mut self) {
        let conn = self.connection.clone();

        loop {
            let event = {
                let mut conn = conn.lock().unwrap();
                match conn.recv_event() {
                    Ok(event) => event,
                    Err(DiscordError::Closed(code, body)) => {
                        error!("[connection] Connection closed status {:?}: {}",
                               code,
                               body);

                        break;
                    },
                    Err(why) => {
                        error!("[connection] Receive error: {:?}", why);

                        continue;
                    },
                }
            };

            debug!("[connection] Received event: {:?}", event);

            {
                let mut state = self.state.lock().unwrap();
                state.update(&event);
            }

            {
                let mut event_counter = self.event_counter.lock().unwrap();
                event_counter.increment(&event);
            }

            self.handle_event(event);
        }
    }

    pub fn handle_event(&mut self, event: Event) {
        debug!("[event] Handling event");

        match event {
            Event::MessageCreate(message) => {
                debug!("[event] Handling MessageCreate");

                let context = Context::new(self.connection.clone(),
                                           self.db.clone(),
                                           self.discord.clone(),
                                           self.event_counter.clone(),
                                           message,
                                           self.music_state.clone(),
                                           self.state.clone(),
                                           self.uptime.clone());
                self.increment_member_messages(&context);

                thread::spawn(move || {
                    handle_message(context);
                });
            },
            Event::ServerCreate(possible_server) => {
                debug!("[event] Handling ServerCreate");

                match possible_server {
                    PossibleServer::Online(server) => {
                        self.handle_server_create(server);
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

                self.handle_server_delete(server_id);
            },
            Event::ServerUpdate(server) => {
                debug!("[event] Handling ServerUpdate");

                self.handle_server_update(server);
            },
            Event::ServerMemberUpdate { server_id, user, nick, .. } => {
                debug!("[event] ServerMemberUpdate");

                self.handle_server_member_update(server_id, user, nick);
            },
            _ => {},
        };
    }

    fn handle_server_create(&mut self, server: LiveServer) {
        let db = self.db.lock().unwrap();
        let exists: PgRes = db.query("select id from guilds where id = $1",
                                     &[&(server.id.0 as i64)]);

        match exists {
            Ok(ref rows) if !rows.is_empty() => {
                let _update = db.execute(
                    "update guilds set active = $1, name = $2, owner_id = $3
                     where id = $4",
                    &[
                        &true,
                        &server.name,
                        &(server.owner_id.0 as i64),
                        &(server.id.0 as i64)]);
            },
            Ok(_rows) => {
                let creation = db.execute(
                    "insert into guilds (active, id, name, owner_id) values
                     ($1, $2, $3, $4)",
                    &[&true, &(server.id.0 as i64), &server.name, &(server.owner_id.0 as i64)]
                );

                if let Err(why) = creation {
                    warn!("[event:servercreate] Err creating guild: {:?}", why);
                }
            },
            Err(why) => {
                warn!("[event:servercrate] Err filtering guilds: {:?}", why);
            },
        }
    }

    fn handle_server_delete(&mut self, server_id: ServerId) {
        let db = self.db.lock().unwrap();
        let update = db.execute("update guilds set active = $1 where id = $2",
                                &[&false, &(server_id.0 as i64)]);
        drop(db);

        match update {
            Ok(1) | Ok(0) => {},
            Ok(amount) => {
                warn!("[event:serverdelete] Multiple deleted: {:?}", amount);
            },
            Err(why) => {
                warn!("[event:serverdelete] Updating {} {:?}", server_id, why);
            },
        }
    }

    fn handle_server_member_update(&mut self,
                                   server_id: ServerId,
                                   user: DiscordUser,
                                   nick: Option<String>) {
        let db = self.db.lock().unwrap();

        let update = db.execute(
            "update members set nick = $1 where server_id = $2 and user_id = $3",
            &[&nick, &(server_id.0 as i64), &(user.id.0 as i64)]
        );

        match update {
            Ok(1) => {},
            // The member doesn't exist in the database; add it
            Ok(0) => {
                let creation = db.execute(
                    "insert into members
                     (message_count, nickname, server_id, user_id, weather_location)
                     values ($1, $2, $3, $4, $5)",
                    &[
                        &0i64,
                        &None::<String>,
                        &(server_id.0 as i64),
                        &(user.id.0 as i64),
                        &None::<String>,
                    ]
                );

                if let Err(why) = creation {
                    warn!("[event:servermemberupdate] Err making member: {:?}",
                          why);
                }

                check_user(&user, &db);
            },
            Ok(amount) => {
                warn!("[event:servermemberupdate] Many updated: {}", amount);
            },
            Err(why) => {
                warn!("[event:servermemberupdate] Err updating: {:?}", why);
            },
        }
    }

    fn handle_server_update(&mut self, srv: Server) {
        let db = self.db.lock().unwrap();

        let update = db.execute(
            "update guilds set active = $2, name = $3, owner_id = $4 where id = $1",
            &[&(srv.id.0 as i64), &true, &srv.name, &(srv.owner_id.0 as i64)]
        );

        match update {
            Ok(1) => {},
            // The server doesn't exist in the database, so add it
            Ok(0) => {
                let creation = db.execute(
                    "insert into guilds (active, id, name, owner_id) values
                     ($1, $2, $3, $4)",
                    &[&true, &(srv.id.0 as i64), &srv.name, &(srv.owner_id.0 as i64)]
                );

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
        let state = self.state.lock().unwrap();

        let server = match state.find_channel(&context.message.channel_id) {
            Some(ChannelRef::Public(server, _channel)) => server.clone(),
            _ => return,
        };
        drop(state);

        let db = self.db.lock().unwrap();

        let retrieval: PgRes = db.query(
            "select id, message_count from members where server_id = $1 and user_id = $2",
            &[&(server.id.0 as i64), &(context.message.author.id.0 as i64)]
        );

        match retrieval {
            Ok(ref rows) if !rows.is_empty() => {
                let member = rows.get(0);

                let id: i32 = member.get(0);
                let mut message_count: i64 = member.get(1);
                message_count += 1;
                let update = db.execute(
                    "update members set message_count = $1 where id = $2",
                    &[&message_count, &id]
                );

                match update {
                    Ok(1) => {},
                    Ok(0) => {
                        warn!("[increment] Incremented none for {}", id);
                    },
                    Ok(amount) => {
                        warn!("[increment] Incremented many: {}", amount);
                    },
                    Err(why) => {
                        warn!("[increment] Err updating member {}: {:?}", id, why);
                    },
                }

                check_user(&context.message.author, &db);
            },
            Ok(_rows) => {
                let insertion = db.execute(
                    "insert into members
                     (message_count, nickname, server_id, user_id, weather_location)
                     values ($1, $2, $3, $4, $5)",
                    &[
                        &1i64,
                        &None::<String>,
                        &(server.id.0 as i64),
                        &(context.message.author.id.0 as i64),
                        &None::<String>,
                    ]
                );

                if let Err(why) = insertion {
                    warn!("[increment] Err creating member: {:?}", why);
                }

                check_user(&context.message.author, &db);
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
    let retrieval = db.query("select id from users where id = $1",
                             &[&(user.id.0 as i64)]);

    match retrieval {
        Ok(ref rows) if !rows.is_empty() => {},
        Ok(_rows) => {
            let insertion = db.execute("insert into users
                                        (id, bot, discriminator, username)
                                        VALUES ($1, $2, $3, $4)",
                                       &[
                                           &(user.id.0 as i64),
                                           &(user.bot),
                                           &(user.discriminator as i16),
                                           &(user.name)
                                       ]);
            if let Err(why) = insertion {
                warn!("[check-user] Err creating user {}: {:?}", user.id, why);
            }
        },
        Err(why) => {
            warn!("[check-user] Err getting {}: {:?}", user.id, why);
        },
    }
}


fn handle_message(context: Context) {
    if !context.message.content.starts_with(';') {
        debug!("[handle-message] Not a command");

        return;
    }

    // Ignore ourself
    {
        let state = context.state.lock().unwrap();

        if context.message.author.id == state.user().id {
            debug!("[handle-message] Ignoring ourself");

            return;
        }
    }

    // Ignore other bots
    {
        let state = context.state.lock().unwrap();
        let s = state.find_channel(&context.message.channel_id);

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
        "8ball" => random::magic_eight_ball(context),
        "aescaps" => misc::aesthetic(context, vec![
                                     Aesthetic::Bold,
                                     Aesthetic::Caps,
        ]),
        "aestheticcaps" => misc::aesthetic(context, vec![
                                           Aesthetic::Bold,
                                           Aesthetic::Caps,
        ]),
        "aesthetic" => misc::aesthetic(context, vec![]),
        "aes" => misc::aesthetic(context, vec![]),
        "about" => meta::about(context),
        "anime" => tv::anime(context),
        "bigemoji" => meta::big_emoji(context),
        "channelinfo" => meta::channel_info(context),
        "choose" => random::choose(context),
        "coinflip" => random::coinflip(context),
        "define" => conversation::define(context),
        "delete" => tags::delete(context),
        "events" => meta::events(context),
        "hello" => misc::hello(context),
        "help" => meta::help(context),
        "info" => tags::info(context),
        "invite" => meta::invite(context),
        "join" => music::join(context),
        "leave" => music::leave(context),
        "list" => tags::list(context),
        "mfw" => misc::mfw(context),
        "ping" => meta::ping(context),
        "pi" => misc::pi(context),
        "play" => music::play(context),
        "purge" => admin::purge(context),
        "queue" => music::queue(context),
        "rename" => tags::rename(context),
        "roleinfo" => meta::role_info(context),
        "roll" => random::roll(context),
        "roulette" => random::roulette(context),
        "search" => tags::search(context),
        "serverinfo" => meta::server_info(context),
        "setstatus" => meta::set_status(context),
        "set" => tags::set(context),
        "skip" => music::skip(context),
        "stats" => stats::stats(context),
        "status" => music::status(context),
        "teams" => random::teams(context),
        "uptime" => misc::uptime(context),
        "userinfo" => meta::user_info(context),
        "weather" => misc::weather(context),
        "get" | _ => {
            debug!("[handle-message] Invalid command");

            tags::get(context);
        },
    }
}
