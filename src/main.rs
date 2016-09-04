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

#![cfg_attr(feature = "nightly", feature(custom_attribute, custom_derive, plugin))]
#![cfg_attr(feature = "nightly", plugin(dotenv_macros))]
#![allow(unknown_lints)]

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;

extern crate chrono;
extern crate discord;
extern crate dotenv;
extern crate env_logger;
extern crate forecast_io;
extern crate hummingbird;
extern crate hyper;
extern crate postgres;
extern crate rand;
extern crate regex;
extern crate serde_json;
extern crate serde;
extern crate urbandictionary;

#[macro_use]
mod utils;

mod bot;
mod error;
mod ext;
mod prelude;

use bot::Bot;
use discord::{
    Discord,
    State,
};
use error::{Error, Result};
use postgres::{Connection as PostgresConnection, SslMode};
use std::time::Duration;
use std::{env, fs, thread};

pub fn db_connect() -> PostgresConnection {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL");

    PostgresConnection::connect(&database_url[..], SslMode::None)
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

fn main() {
    env_logger::init().expect("env logger");
    dotenv::dotenv().ok();

    // Create the initial directories needed.
    fs::create_dir_all("./mu/").expect("mu dir");

    info!("[main] Starting loop");

    loop {
        debug!("[main] Logging in...");
        let discord = match login() {
            Ok(discord) => discord,
            Err(_why) => return,
        };

        info!("[main] Logged in");
        debug!("[main] Connecting...");

        let (conn, state) = {
            match discord.connect() {
                Ok((conn, ready)) => (conn, State::new(ready)),
                Err(_why) => {
                    warn!("[main] Error making a connection");

                    return;
                },
            }
        };

        info!("[main] Initializing bot");
        let mut bot = Bot::new(discord, conn, db_connect(), state);
        bot.start();

        // It can be assumed Discord went down or the token reset for one reason
        // or another, so sleep for an amount of time just in case.
        info!("[base] Sleeping for 900 seconds due to disconnect");
        thread::sleep(Duration::from_secs(900));
    }
}
