#![cfg_attr(feature = "nightly", feature(custom_attribute, custom_derive, plugin))]
#![cfg_attr(feature = "nightly", plugin(diesel_codegen, dotenv_macros))]
#![allow(unknown_lints)]

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate log;

extern crate chrono;
extern crate discord;
extern crate dotenv;
extern crate env_logger;
extern crate forecast_io;
extern crate hummingbird;
extern crate hyper;
extern crate rand;
extern crate regex;
extern crate serde_json;
extern crate serde;
extern crate urbandictionary;

#[cfg(feature = "nightly")]
include!("main.in.rs");

#[cfg(feature = "with-syntex")]
include!(concat!(env!("OUT_DIR"), "/main.rs"));

#[macro_use]
mod utils;

mod bot;
mod error;
mod ext;
mod prelude;

use bot::Bot;
use bot::plugins::music::{MusicPlaying, MusicState};
use chrono::UTC;
use diesel::prelude::*;
use diesel::pg::PgConnection;
use discord::{
    Connection as DiscordConnection,
    Discord,
    Error as DiscordError,
    State,
    voice,
};
use error::{Error, Result};
use std::sync::mpsc::{self, TryRecvError};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{env, fs, thread};

fn db_connect() -> PgConnection {
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    PgConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}

fn login() -> Result<Discord> {
    let token = env::var("DISCORD_TOKEN")
        .expect("DISCORD_TOKEN required");

    match Discord::from_bot_token(&token) {
        Ok(discord) => Ok(discord),
        Err(why) => Err(Error::Discord(why)),
    }
}

fn handle_connection(bot: &mut Bot,
                     conn: Arc<Mutex<DiscordConnection>>,
                     discord: Arc<Mutex<Discord>>,
                     db: &PgConnection) {
    let conn_copy = conn.clone();

    loop {
        let event = {
            let mut conn_copy = conn_copy.lock().unwrap();
            match conn_copy.recv_event() {
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

        bot.state.update(&event);
        bot.event_counter.increment(&event);

        bot.handle_event(event, conn.clone(), db, &discord);
    }
}

#[allow(or_fun_call)]
fn program_loop(bot: &mut Bot, db: &PgConnection) {
    debug!("[base] Logging in...");
    let discord = match login() {
        Ok(discord) => Arc::new(Mutex::new(discord)),
        Err(_why) => return,
    };

    let discord_copy = discord.clone();

    info!("[base] Logged in");
    debug!("[base] Connecting...");

    let (conn, ready) = {
        let discord_copy = discord_copy.lock().unwrap();

        match discord_copy.connect() {
            Ok((conn, ready)) => (Arc::new(Mutex::new(conn)), ready),
            Err(_why) => {
                warn!("[base] Error making a connection");

                return;
            },
        }
    };

    // So storing the music queue here both creates a problem and solves a
    // problem.
    //
    // The problem it solves, is the timer check on audio. If we lose connection
    // to Discord, it'd not be that ergonomic to bump up the ending times of the
    // playing songs appropriately (as audio will have attempted to continue
    // playing). This also is problematic as I don't have complete control over
    // the audio right now.
    // This is also mostly out of laziness.
    //
    // The problem it creates is that all of the queued music is lost; perhaps
    // this is something to fix in the future.
    let music_state: Arc<Mutex<MusicState>>;
    music_state = Arc::new(Mutex::new(MusicState::new()));
    let conn_copy = conn.clone();
    let state_copy = music_state.clone();

    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        loop {
            debug!("[music-check] Iterating");

            {
                let now = UTC::now().timestamp() as u64;
                let mut state = state_copy.lock().unwrap();

                let mut remove = vec![];
                let mut next = vec![];

                // iter is auto-sorted by key
                for (k, v) in &state.song_completion {
                    if now < *k {
                        break;
                    }

                    next.extend_from_slice(v);
                    remove.push(*k);
                }

                for item in remove {
                    state.song_completion.remove(&item);
                }

                for server_id in next {
                    if !state.queue.contains_key(&server_id) {
                        continue;
                    }

                    // safe to unwrap since we already checked
                    let empty = state.queue.get(&server_id)
                        .unwrap()
                        .is_empty();

                    if empty {
                        state.status.insert(server_id, None);

                        continue;
                    }

                    // again: safe because we already checked
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
                        let mut conn = conn_copy.lock().unwrap();
                        {
                            let voice = conn.voice(Some(server_id));
                            voice.play(stream);
                        }

                        drop(conn);
                    }

                    let requested_in = request.requested_in;

                    let text = format!("Playing song **{}** requested by _{}_ [duration: {}]",
                                       request.response.data.title,
                                       request.requester_name,
                                       request.format_duration());

                    // Now update the song completion to re-check specifically
                    // once this song is over.
                    let check_at = now + request.response.data.duration;

                    {
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

                    let discord_copy = discord_copy.lock().unwrap();
                    let _ = discord_copy.send_message(&requested_in,
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

    bot.music.state = music_state.clone();
    bot.uptime.connection = UTC::now();
    bot.state = State::new(ready);

    handle_connection(bot, conn.clone(), discord.clone(), db);

    // stop the music queue check
    let _ = tx.send(());
}

fn main() {
    env_logger::init().expect("env logger");
    dotenv::dotenv().ok();

    // Create the initial directories needed.
    fs::create_dir_all("./mu/").expect("mu dir");

    debug!("[base] Connecting to database");
    let mut bot = Bot::new();

    debug!("[base] Creating secondary connection to database");
    let db = db_connect();

    debug!("[base] Entering program loop");

    loop {
        program_loop(&mut bot, &db);

        // It can be assumed Discord went down or the token reset for one reason
        // or another, so sleep for an amount of time just in case.
        info!("[base] Sleeping for 900 seconds due to disconnect");
        thread::sleep(Duration::from_secs(900));
    }
}
