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

use discord::model::{ChannelId, PublicChannel, Message};
use discord::{ChannelRef, Connection as DiscordConnection, Discord, State};
use postgres::Connection as PgConnection;
use regex::Regex;
use serde::ser::Serialize;
use std::fmt;
use std::sync::{Arc, Mutex};
use ::bot::event_counter::EventCounter;
use ::bot::plugins::music::MusicState;
use ::bot::Uptime;
use ::prelude::*;

pub struct Context {
    pub conn: Arc<Mutex<DiscordConnection>>,
    pub db: Arc<Mutex<PgConnection>>,
    pub discord: Arc<Mutex<Discord>>,
    pub event_counter: Arc<Mutex<EventCounter>>,
    pub message: Message,
    pub music_state: Arc<Mutex<MusicState>>,
    pub state: Arc<Mutex<State>>,
    pub uptime: Arc<Mutex<Uptime>>,
}

impl Context {
    pub fn new(conn: Arc<Mutex<DiscordConnection>>,
               db_connection: Arc<Mutex<PgConnection>>,
               discord: Arc<Mutex<Discord>>,
               event_counter: Arc<Mutex<EventCounter>>,
               message: Message,
               music_state: Arc<Mutex<MusicState>>,
               state: Arc<Mutex<State>>,
               uptime: Arc<Mutex<Uptime>>)
               -> Context {
        Context {
            conn: conn,
            db: db_connection,
            discord: discord,
            event_counter: event_counter,
            message: message,
            music_state: music_state,
            state: state,
            uptime: uptime,
        }
    }

    fn send(&self, channel_id: ChannelId, content: String) -> Result<Message> {
        let discord = self.discord.lock().unwrap();

        match discord.send_message(&channel_id, &content, "", false) {
            Ok(message) => Ok(message),
            Err(why) => {
                error!("[send] Err sending: {:?}", why);

                Err(Error::Discord(why))
            },
        }
    }

    pub fn arg(&self, number: u8) -> ContextArg {
        let split: (&str, &str) = self.message.content.split_at(';'.len_utf8());
        let mut commands: Vec<&str> = split.1.split_whitespace().collect();
        let len = commands.len();

        if len > 0 && number as usize <= len - 1 {
            ContextArg(Some(commands.remove(number as usize)))
        } else {
            ContextArg(None)
        }
    }

    pub fn channel_mentions(&self) -> Vec<PublicChannel> {
        let re = Regex::new(r"<#([0-9]+)>").unwrap();

        let mut channels = vec![];

        for pos in re.find_iter(&self.message.content) {
            let id_as_str = &self.message.content[pos.0..pos.1]
                .replace("<#", "")
                .replace(">", "");

            let id = match id_as_str.parse::<u64>() {
                Ok(id) => ChannelId(id),
                Err(_why) => continue,
            };

            let state = self.state.lock().unwrap();

            if let Some(ChannelRef::Public(_, ch)) = state.find_channel(&id) {
                channels.push(ch.clone());
            }

            drop(state);
        }

        channels
    }

    pub fn edit<S: Into<String>>(&self,
                                 message: &Message,
                                 new_content: S)
                                 -> Result<Message> {
        let discord = self.discord.lock().unwrap();

        match discord.edit_message(&message.channel_id,
                                   &message.id,
                                   &new_content.into()) {
            Ok(message) => Ok(message),
            Err(why) => {
                warn!("[edit] Err editing: {:?}", why);

                Err(Error::Discord(why))
            },
        }
    }

    pub fn message<S: Into<String>>(&self,
                                    channel_id: ChannelId,
                                    content: S)
                                    -> Result<Message> {
        self.send(channel_id, content.into())
    }

    pub fn pm_author<S: Into<String>>(&self, content: S) -> Result<Message> {
        let discord = self.discord.lock().unwrap();

        let ch = match discord.create_private_channel(&self.message.author.id) {
            Ok(ch) => ch,
            Err(why) => {
                error!("[pm_author] Err opening private channel: {:?}", why);

                return Err(Error::Discord(why));
            },
        };

        drop(discord);

        self.send(ch.id, content.into())
    }

    pub fn reply<S: Into<String>>(&self, content: S) -> Result<Message> {
        let reply = format!("{}: {}", self.message.author.mention(), content.into());

        self.send(self.message.channel_id, reply)
    }

    pub fn say<S: Into<String>>(&self, content: S) -> Result<Message> {
        self.send(self.message.channel_id, content.into())
    }

    pub fn text(&self, from: usize) -> String {
        let split: Vec<&str> = self.message.content.split_whitespace()
            .skip(from + 1)
            .collect();
        split.join(" ")
    }
}

#[derive(Debug, Clone)]
pub struct ContextArg<'a>(Option<&'a str>);

impl<'a> ContextArg<'a> {
    fn check<S: Serialize>(&self, v: Option<S>) -> Result<S> {
        if let Some(v) = v {
            Ok(v)
        } else {
            Err(Error::Decode)
        }
    }

    pub fn as_str(&self) -> Result<&str> {
        self.check(self.0)
    }

    pub fn as_u64(&self) -> Result<u64> {
        if let Some(inner) = self.0 {
            Ok(try!(inner.parse::<u64>()))
        } else {
            Err(Error::Decode)
        }
    }

    pub fn as_isize(&self) -> Result<isize> {
        if let Some(inner) = self.0 {
            Ok(try!(inner.parse::<isize>()))
        } else {
            Err(Error::Decode)
        }
    }

    pub fn exists(&self) -> bool {
        self.0.is_some()
    }
}

impl<'a> fmt::Display for ContextArg<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(inner) = self.0 {
            write!(f, "{}", inner)
        } else {
            write!(f, "{}", "None")
        }
    }
}
