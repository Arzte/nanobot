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
use discord::{ChannelRef, Connection as DiscordConnection, State};
use regex::Regex;
use serde::ser::Serialize;
use std::fmt;
use std::sync::{Arc, Mutex};
use ::bot::plugins::music::MusicState;
use ::prelude::*;

pub struct Context {
    pub conn: Arc<Mutex<DiscordConnection>>,
    pub message: Message,
    pub music_state: Arc<Mutex<MusicState>>,
    pub state: Arc<Mutex<State>>,
}

impl Context {
    pub fn new(conn: Arc<Mutex<DiscordConnection>>,
               message: Message,
               music_state: Arc<Mutex<MusicState>>,
               state: Arc<Mutex<State>>)
               -> Context {
        Context {
            conn: conn,
            message: message,
            music_state: music_state,
            state: state,
        }
    }

    fn send(&self, channel_id: ChannelId, content: String) -> Result<Message> {
        let discord = ::DISCORD.lock().unwrap();

        match discord.send_message(&channel_id, &content, "", false) {
            Ok(message) => Ok(message),
            Err(why) => {
                error!("[send] Err sending to channel {}: {:?}",
                       channel_id,
                       why);

                Err(Error::Discord(why))
            },
        }
    }

    pub fn arg(&self, number: usize) -> ContextArg {
        let split: (&str, &str) = self.message.content.split_at('`'.len_utf8());

        ContextArg(split.1.split_whitespace().nth(number))
    }

    pub fn channel_mentions(&self) -> Vec<PublicChannel> {
        let re = Regex::new(r"<#([0-9]+)>").unwrap();

        let mut channels = vec![];

        let state = self.state.lock().unwrap();

        for pos in re.find_iter(&self.message.content) {
            let id_as_str = &self.message.content[pos.0..pos.1]
                .replace("<#", "")
                .replace(">", "");

            let id = match id_as_str.parse::<u64>() {
                Ok(id) => ChannelId(id),
                Err(_why) => continue,
            };

            if let Some(ChannelRef::Public(_, ch)) = state.find_channel(&id) {
                channels.push(ch.clone());
            }
        }

        channels
    }

    pub fn edit<S: Into<String>>(&self,
                                 message: &Message,
                                 new_content: S)
                                 -> Result<Message> {
        let discord = ::DISCORD.lock().unwrap();

        match discord.edit_message(&message.channel_id,
                                   &message.id,
                                   &new_content.into()) {
            Ok(message) => Ok(message),
            Err(why) => {
                warn!("[edit] Err editing {}: {:?}", message.id, why);

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
        let discord = ::DISCORD.lock().unwrap();

        let ch = match discord.create_private_channel(&self.message.author.id) {
            Ok(ch) => ch,
            Err(why) => {
                error!("[pm_author] Err opening private channel for {}: {:?}",
                       self.message.author.id,
                       why);

                return Err(Error::Discord(why));
            },
        };

        drop(discord);

        self.send(ch.id, content.into())
    }

    pub fn reply<S: Into<String>>(&self, content: S) -> Result<Message> {
        let reply = format!("{}: {}",
                            self.message.author.mention(),
                            content.into());

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
        v.ok_or(Error::Decode)
    }

    pub fn as_str(&self) -> Result<&str> {
        self.check(self.0)
    }

    #[allow(dead_code)]
    pub fn as_i64(&self) -> Result<i64> {
        self.0
            .ok_or(Error::Decode)
            .and_then(|v| Ok(try!(v.parse::<i64>())))
    }

    pub fn as_u64(&self) -> Result<u64> {
        self.0
            .ok_or(Error::Decode)
            .and_then(|v| Ok(try!(v.parse::<u64>())))
    }

    pub fn as_isize(&self) -> Result<isize> {
        self.0
            .ok_or(Error::Decode)
            .and_then(|v| Ok(try!(v.parse::<isize>())))
    }

    pub fn exists(&self) -> bool {
        self.0.is_some()
    }
}

impl<'a> fmt::Display for ContextArg<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0.unwrap_or("None"))
    }
}
