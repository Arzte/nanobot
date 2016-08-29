use diesel::pg::PgConnection;
use discord::model::{ChannelId, PublicChannel, Message};
use discord::{ChannelRef, Connection as DiscordConnection, Discord, State};
use regex::Regex;
use serde::ser::Serialize;
use std::fmt;
use std::sync::{Arc, Mutex};
use ::prelude::*;

pub struct Context<'a> {
    pub conn: Arc<Mutex<DiscordConnection>>,
    pub db: Arc<PgConnection>,
    pub discord: &'a Arc<Mutex<Discord>>,
    pub message: Message,
}

impl<'a> Context<'a> {
    #[allow(needless_lifetimes)]
    pub fn new<'b>(conn: Arc<Mutex<DiscordConnection>>,
                   db_connection: Arc<PgConnection>,
                   discord: &'b Arc<Mutex<Discord>>,
                   message: Message)
                   -> Context<'b> {
        Context {
            conn: conn,
            db: db_connection,
            discord: discord,
            message: message,
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

    pub fn channel_mentions<'b>(&'b self,
                                state: &'b State)
                                -> Vec<&PublicChannel> {
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

            if let Some(ChannelRef::Public(_, ch)) = state.find_channel(&id) {
                channels.push(ch);
            }
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
            Err(why) => Err(Error::Discord(why)),
        }
    }

    pub fn message<S: Into<String>>(&self,
                                    channel_id: ChannelId,
                                    content: S)
                                    -> Result<Message> {
        let into = content.into();
        let discord = self.discord.lock().unwrap();

        match discord.send_message(&channel_id, &into, "", false) {
            Ok(message) => Ok(message),
            Err(why) => Err(Error::Discord(why)),
        }
    }

    pub fn reply<S: Into<String>>(&self, content: S) -> Result<Message> {
        let reply = format!("{}: {}", self.message.author.mention(), content.into());

        self.say(reply)
    }

    pub fn say<S: Into<String>>(&self, content: S) -> Result<Message> {
        let into = content.into();
        let discord = self.discord.lock().unwrap();

        match discord.send_message(&self.message.channel_id, &into, "", false) {
            Ok(message) => Ok(message),
            Err(why) => Err(Error::Discord(why)),
        }
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
