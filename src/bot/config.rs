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

use discord::model::{ChannelId, ServerId};
use serde_json::Value;
use std::default::Default;
use std::i64;
use ::prelude::*;

pub static CONFIGS: [&'static str; 48] = [
    "aesthetic.available",
    "aestheticcaps.available",
    "aes.available",
    "aescaps.available",
    "anime.available",
    "channelinfo.available",
    "choose.available",
    "coinflip.available",
    "coinflip.side",
    "conversation.available",
    "define.available",
    "define.example",
    "emoji.available",
    "hello.available",
    "lmgtfy.available",
    "lmgtfy.results",
    "8ball.available",
    "manga.available",
    "mfw.available",
    "pi.available",
    "pi.precision.default",
    "pi.precision.maximum",
    "ping.available",
    "pixiv.automatic",
    "pixiv.available",
    "pixiv.info",
    "purge.available",
    "purge.default",
    "purge.maximum",
    "purge.minimum",
    "remindme.available",
    "roleinfo.available",
    "roll.available",
    "roll.custom",
    "roll.maximum",
    "roll.minimum",
    "roulette.available",
    "serverinfo.available",
    "skip.available",
    "skip.required",
    "stats.available",
    "tags.available",
    "teams.available",
    "userinfo.available",
    "weather.available",
    "weather.saving",
    "wolfram.available",
    "xkcd.available",
];

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug, Ord, PartialOrd)]
enum Availability {
    /// No one can use the command
    Disabled,
    /// Everyone can use the command
    Enabled,
}

impl Availability {
    #[allow(dead_code)]
    pub fn from_num(num: u8) -> Option<Availability> {
        match num {
            0 => Some(Availability::Disabled),
            1 => Some(Availability::Enabled),
            _ => None,
        }
    }

    #[allow(dead_code)]
    pub fn from_str(name: &str) -> Option<Availability> {
        match name {
            "disabled" | "0" => Some(Availability::Disabled),
            "enabled" | "1" => Some(Availability::Enabled),
            _ => None,
        }
    }

    pub fn num(&self) -> u64 {
        match *self {
            Availability::Disabled => 0,
            Availability::Enabled => 1,
        }
    }
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug, Ord, PartialOrd)]
pub enum ConfigType {
    Availability,
    Int,
    String,
}

impl ConfigType {
    pub fn from_num(num: u8) -> Option<ConfigType> {
        match num {
            0 => Some(ConfigType::Availability),
            1 => Some(ConfigType::Int),
            2 => Some(ConfigType::String),
            _ => None,
        }
    }

    pub fn name(&self) -> &str {
        match *self {
            ConfigType::Availability => "Enable Switch",
            ConfigType::Int => "Integer",
            ConfigType::String => "String",
        }
    }

    pub fn to_num(kind: ConfigType) -> i16 {
        match kind {
            ConfigType::Availability => 0,
            ConfigType::Int => 1,
            ConfigType::String => 2,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ConfigItem<'a> {
    pub default: Value,
    pub description: &'a str,
    pub key: &'a str,
    pub kind: ConfigType,
    pub max_value: Option<i64>,
    pub min_value: Option<i64>,
    pub value: Value,
}

impl<'a> ConfigItem<'a> {
    pub fn as_i64(&self) -> Result<i64> {
        self.value
            .as_i64()
            .or_else(|| self.value.as_u64().map(|v| v as i64))
            .ok_or(Error::Decode)
    }

    pub fn as_isize(&self) -> Result<isize> {
        self.value
            .as_i64()
            .or_else(|| self.value.as_u64().map(|v| v as i64))
            .ok_or(Error::Decode)
            .map(|v| v as isize)
    }

    pub fn as_u64(&self) -> Result<u64> {
        self.value
            .as_u64()
            .or_else(|| self.value.as_i64().map(|v| v as u64))
            .ok_or(Error::Decode)
    }

    pub fn disabled(&self) -> bool {
        !self.enabled()
    }

    pub fn enabled(&self) -> bool {
        match self.kind {
            ConfigType::Availability => {
                match self.value {
                    Value::U64(1) => true,
                    _ => false,
                }
            },
            _ => false,
        }
    }

    pub fn int(&self) -> bool {
        self.kind == ConfigType::Int
    }
}

macro_rules! config {
    ($name:ident, $key:expr, $kind:path, $default:expr, $desc:expr) => {
        config_impl!($name, $key, $kind, $default, None, None, $desc);
    };

    ($name:ident, $key:expr, $kind:path, $default:expr, $min:expr, $max:expr, $desc:expr) => {
        config_impl!($name, $key, $kind, $default, Some($min), Some($max), $desc);
    };
}

macro_rules! config_impl {
    ($name:ident, $key:expr, $kind:path, $default:expr, $min:expr, $max:expr, $desc:expr) => {
        /// $desc
        #[derive(Clone, Debug)]
        pub struct $name {
            default: Value,
            description: String,
            key: String,
            kind: ConfigType,
            max_value: Option<i64>,
            min_value: Option<i64>,
        }

        impl Default for $name {
            fn default() -> $name {
                $name {
                    default: $default,
                    description: String::from($desc),
                    key: String::from($key),
                    max_value: $max,
                    min_value: $min,
                    kind: $kind,
                }
            }
        }

        impl $name {
            fn handle<'a>(res: PgRes,
                          location: (ServerId, Option<ChannelId>))
                          -> ConfigItem<'a> {
                let server = location.0;
                let channel = location.1;

                match res {
                    Ok(ref rows) if rows.len() > 0 => {
                        let row = rows.get(0);

                        let kind_from_db: i16 = row.get(3);
                        let value: String = row.get(5);

                        let kind = ConfigType::from_num(kind_from_db as u8)
                            .unwrap();

                        let v = match kind {
                            ConfigType::Availability => {
                                Value::U64(value.parse::<u64>().unwrap())
                            },
                            ConfigType::Int => {
                                Value::I64(value.parse::<i64>().unwrap())
                            },
                            ConfigType::String => {
                                Value::String(value)
                            },
                        };

                        ConfigItem {
                            default: $default,
                            description: $desc,
                            key: $key,
                            kind: $kind,
                            max_value: $max,
                            min_value: $min,
                            value: v,
                        }
                    },
                    Ok(_rows) => {
                        let default = $name::default();

                        ConfigItem {
                            default: $default,
                            description: $desc,
                            key: $key,
                            kind: $kind,
                            max_value: $max,
                            min_value: $min,
                            value: default.default,
                        }
                    },
                    Err(why) => {
                        warn!("[get] Err getting config for '{}/{:?}': {:?}",
                              server,
                              channel,
                              why);

                        let default = $name::default();

                        ConfigItem {
                            default: $default,
                            description: $desc,
                            key: $key,
                            kind: $kind,
                            max_value: $max,
                            min_value: $min,
                            value: default.default,
                        }
                    },
                }
            }

            #[allow(dead_code)]
            pub fn find<'a>(location: (ServerId, Option<ChannelId>)) -> ConfigItem<'a> {
                let server = location.0;
                let db = ::DB.lock().unwrap();

                let res: PgRes = if let Some(channel) = location.1 {
                    db.query(
                        "select id, channel_id, key, kind, server_id, value
                         from configs where (channel_id = $1 and server_id = $2
                         and key = $3) or (channel_id is null and server_id = $2
                         and key = $3) order by channel_id desc",
                        &[&(channel.0 as i64), &(server.0 as i64), &$key]
                    )
                } else {
                    db.query(
                        "select id, channel_id, key, kind, server_id, value
                         from configs where channel_id is null and server_id = $1
                         and key = $2",
                        &[&(server.0 as i64), &$key])
                };

                Self::handle(res, location)
            }

            #[allow(dead_code)]
            pub fn get<'a>(location: (ServerId, Option<ChannelId>)) -> ConfigItem<'a> {
                let server = location.0;
                let db = ::DB.lock().unwrap();

                let res: PgRes = if let Some(channel) = location.1 {
                    db.query(
                        "select id, channel_id, key, kind, server_id, value
                         from configs where channel_id = $1 and server_id = $2
                         and key = $3",
                        &[&(channel.0 as i64), &(server.0 as i64), &$key]
                    )
                } else {
                    db.query(
                        "select id, channel_id, key, kind, server_id, value
                         from configs where channel_id is null and server_id = $1
                         and key = $2",
                        &[&(server.0 as i64), &$key])
                };

                Self::handle(res, location)
            }

            #[allow(dead_code)]
            pub fn set(location: (ServerId, Option<ChannelId>),
                       value: Value)
                       -> Result<()> {
                let sql_value: String = match value {
                    Value::I64(v) => v.to_string(),
                    Value::U64(v) => v.to_string(),
                    Value::String(v) => v,
                    _ => return Err(Error::Decode),
                };

                let db = ::DB.lock().unwrap();

                let updated = if let Some(channel) = location.1 {
                    db.execute(
                        r#"update configs set value = $1 where key = $2
                         and server_id = $3 and channel_id = $4"#, &[
                            &sql_value,
                            &$key,
                            &((location.0).0 as i64),
                            &(channel.0 as i64),
                        ])
                } else {
                    db.execute(
                        r#"update configs set value = $1 where key = $2
                           and server_id = $3 and channel_id is null"#, &[
                            &sql_value,
                            &$key,
                            &((location.0).0 as i64),
                        ])
                };

                match updated {
                    Ok(1) => Ok(()),
                    Ok(0) => {
                        let insert = if let Some(channel) = location.1 {
                            db.execute(
                                r#"insert into configs
                                 (channel_id, key, kind, server_id, value)
                                 values
                                 ($1, $2, $3, $4, $5)"#,
                                &[
                                    &(channel.0 as i64),
                                    &$key,
                                    &(ConfigType::to_num($kind)),
                                    &((location.0).0 as i64),
                                    &sql_value,
                                ])
                        } else {
                            db.execute(
                                r#"insert into configs
                                 (key, kind, server_id, value)
                                 values
                                 ($1, $2, $3, $4)"#,
                                &[
                                    &$key,
                                    &(ConfigType::to_num($kind)),
                                    &((location.0).0 as i64),
                                    &sql_value,
                                ])
                        };

                        match insert {
                            Ok(_) => Ok(()),
                            Err(why) => {
                                warn!("[save] Err saving new config: [{}/{:?}/{:?}]: {:?}",
                                      location.0,
                                      location.1,
                                      sql_value,
                                      why);

                                Err(Error::SqlExecution)
                            },
                        }
                    },
                    Ok(amount) => {
                        warn!("[save] Updated many configs: [{}/{:?}/{:?}]: {}",
                              location.0,
                              location.1,
                              sql_value,
                              amount);

                        Ok(())
                    },
                    Err(why) => {
                        warn!("[save] Err updating config: [{}/{:?}/{:?}]: {:?}",
                              location.0,
                              location.1,
                              sql_value,
                              why);

                        Err(Error::SqlExecution)
                    },
                }
            }
        }
    }
}

pub fn get_config<'a>(name: &str,
                      location: (ServerId, Option<ChannelId>))
                      -> Option<ConfigItem<'a>> {
    match name {
        "aesthetic.available" => Some(AestheticAvailable::get(location)),
        "aestheticcaps.available" => Some(AestheticCapsAvailable::get(location)),
        "aes.available" => Some(AesAvailable::get(location)),
        "aescaps.available" => Some(AesCapsAvailable::get(location)),
        "anime.available" => Some(AnimeAvailable::get(location)),
        "channelinfo.available" => Some(ChannelInfoAvailable::get(location)),
        "choose.available" => Some(ChooseAvailable::get(location)),
        "coinflip.available" => Some(CoinflipAvailable::get(location)),
        "coinflip.side" => Some(CoinflipSide::get(location)),
        "conversation.available" => Some(ConversationAvailable::get(location)),
        "define.available" => Some(DefineAvailable::get(location)),
        "define.example" => Some(DefineExample::get(location)),
        "emoji.available" => Some(EmojiAvailable::get(location)),
        "hello.available" => Some(HelloAvailable::get(location)),
        "lmgtfy.available" => Some(LmgtfyAvailable::get(location)),
        "lmgtfy.results" => Some(LmgtfyResults::get(location)),
        "8ball.available" => Some(MagicEightBallAvailable::get(location)),
        "manga.available" => Some(MangaAvailable::get(location)),
        "mfw.available" => Some(MfwAvailable::get(location)),
        "pi.available" => Some(PiAvailable::get(location)),
        "pi.precision.default" => Some(PiPrecisionDefault::get(location)),
        "pi.precision.maximum" => Some(PiPrecisionMaximum::get(location)),
        "ping.available" => Some(PingAvailable::get(location)),
        "pixiv.automatic" => Some(PixivAutomatic::get(location)),
        "pixiv.available" => Some(PixivAvailable::get(location)),
        "pixiv.info" => Some(PixivInfo::get(location)),
        "purge.available" => Some(PurgeAvailable::get(location)),
        "purge.default" => Some(PurgeDefault::get(location)),
        "purge.maximum" => Some(PurgeMaximum::get(location)),
        "purge.minimum" => Some(PurgeMinimum::get(location)),
        "remindme.available" => Some(RemindMeAvailable::get(location)),
        "roleinfo.available" => Some(RoleInfoAvailable::get(location)),
        "roll.available" => Some(RollAvailable::get(location)),
        "roll.custom" => Some(RollCustom::get(location)),
        "roll.maximum" => Some(RollMaximum::get(location)),
        "roll.minimum" => Some(RollMinimum::get(location)),
        "roulette.available" => Some(RouletteAvailable::get(location)),
        "serverinfo.available" => Some(ServerInfoAvailable::get(location)),
        "skip.available" => Some(SkipAvailable::get(location)),
        "skip.required" => Some(SkipRequired::get(location)),
        "stats.available" => Some(StatsAvailable::get(location)),
        "tags.available" => Some(TagsAvailable::get(location)),
        "teams.available" => Some(TeamsAvailable::get(location)),
        "userinfo.available" => Some(UserInfoAvailable::get(location)),
        "weather.available" => Some(WeatherAvailable::get(location)),
        "weather.saving" => Some(WeatherSaving::get(location)),
        "wolfram.available" => Some(WolframAvailable::get(location)),
        "xkcd.available" => Some(XkcdAvailable::get(location)),
        _ => None,
    }
}

pub fn set_config(name: &str,
                  location: (ServerId, Option<ChannelId>),
                  value: Value)
                  -> Option<Result<()>> {
    match name {
        "aesthetic.available" => Some(AestheticAvailable::set(location, value)),
        "aestheticcaps.available" => Some(AestheticCapsAvailable::set(location, value)),
        "aes.available" => Some(AesAvailable::set(location, value)),
        "aescaps.available" => Some(AesCapsAvailable::set(location, value)),
        "anime.available" => Some(AnimeAvailable::set(location, value)),
        "channelinfo.available" => Some(ChannelInfoAvailable::set(location, value)),
        "choose.available" => Some(ChooseAvailable::set(location, value)),
        "coinflip.available" => Some(CoinflipAvailable::set(location, value)),
        "coinflip.side" => Some(CoinflipSide::set(location, value)),
        "conversation.available" => Some(ConversationAvailable::set(location, value)),
        "define.available" => Some(DefineAvailable::set(location, value)),
        "define.example" => Some(DefineExample::set(location, value)),
        "emoji.available" => Some(EmojiAvailable::set(location, value)),
        "hello.available" => Some(HelloAvailable::set(location, value)),
        "lmgtfy.available" => Some(LmgtfyAvailable::set(location, value)),
        "lmgtfy.results" => Some(LmgtfyResults::set(location, value)),
        "8ball.available" => Some(MagicEightBallAvailable::set(location, value)),
        "manga.available" => Some(MangaAvailable::set(location, value)),
        "mfw.available" => Some(MfwAvailable::set(location, value)),
        "pi.available" => Some(PiAvailable::set(location, value)),
        "pi.precision.default" => Some(PiPrecisionDefault::set(location, value)),
        "pi.precision.maximum" => Some(PiPrecisionMaximum::set(location, value)),
        "ping.available" => Some(PingAvailable::set(location, value)),
        "pixiv.automatic" => Some(PixivAutomatic::set(location, value)),
        "pixiv.available" => Some(PixivAvailable::set(location, value)),
        "pixiv.info" => Some(PixivInfo::set(location, value)),
        "purge.available" => Some(PurgeAvailable::set(location, value)),
        "purge.default" => Some(PurgeDefault::set(location, value)),
        "purge.maximum" => Some(PurgeMaximum::set(location, value)),
        "purge.minimum" => Some(PurgeMinimum::set(location, value)),
        "remindme.available" => Some(RemindMeAvailable::set(location, value)),
        "roleinfo.available" => Some(RoleInfoAvailable::set(location, value)),
        "roll.available" => Some(RollAvailable::set(location, value)),
        "roll.custom" => Some(RollCustom::set(location, value)),
        "roll.maximum" => Some(RollMaximum::set(location, value)),
        "roll.minimum" => Some(RollMinimum::set(location, value)),
        "roulette.available" => Some(RouletteAvailable::set(location, value)),
        "serverinfo.available" => Some(ServerInfoAvailable::set(location, value)),
        "skip.available" => Some(SkipAvailable::set(location, value)),
        "skip.required" => Some(SkipRequired::set(location, value)),
        "stats.available" => Some(StatsAvailable::set(location, value)),
        "tags.available" => Some(TagsAvailable::set(location, value)),
        "teams.available" => Some(TeamsAvailable::set(location, value)),
        "userinfo.available" => Some(UserInfoAvailable::set(location, value)),
        "weather.available" => Some(WeatherAvailable::set(location, value)),
        "weather.saving" => Some(WeatherSaving::set(location, value)),
        "wolfram.available" => Some(WolframAvailable::set(location, value)),
        "xkcd.available" => Some(XkcdAvailable::set(location, value)),
        _ => None,
    }
}

config! {
    AestheticAvailable,
    "aesthetic.available",
    ConfigType::Availability,
    Value::U64(Availability::Enabled.num()),
    "Whether the ability to use `aesthetic` is available."
}

config! {
    AestheticCapsAvailable,
    "aestheticcaps.available",
    ConfigType::Availability,
    Value::U64(Availability::Enabled.num()),
    "Whether the ability to use `aesthetic` is available."
}

config! {
    AesAvailable,
    "aes.available",
    ConfigType::Availability,
    Value::U64(Availability::Enabled.num()),
    "Whether the ability to use `aes` is available."
}

config! {
    AesCapsAvailable,
    "aescaps.available",
    ConfigType::Availability,
    Value::U64(Availability::Enabled.num()),
    "Whether the ability to use `aescaps` is available."
}

config! {
    AnimeAvailable,
    "anime.available",
    ConfigType::Availability,
    Value::U64(Availability::Enabled.num()),
    "Whether the ability to use `anime` is available."
}

config! {
    ChannelInfoAvailable,
    "channelinfo.available",
    ConfigType::Availability,
    Value::U64(Availability::Enabled.num()),
    "Whether the ability to use `channelinfo` is available."
}

config! {
    ChooseAvailable,
    "choose.available",
    ConfigType::Availability,
    Value::U64(Availability::Enabled.num()),
    "Whether the ability to use `choose` is available."
}

config! {
    CoinflipAvailable,
    "coinflip.available",
    ConfigType::Availability,
    Value::U64(Availability::Enabled.num()),
    "Whether the ability to use `coinflip` is available."
}

config! {
    CoinflipSide,
    "coinflip.side",
    ConfigType::Availability,
    Value::U64(Availability::Enabled.num()),
    "Whether the coin can land on its side."
}

config! {
    ConversationAvailable,
    "conversation.available",
    ConfigType::Availability,
    Value::U64(Availability::Disabled.num()),
    "Whether the ability to converse with nano is available.

    The command `q` is a command to converse with an AI. This allows users to
    ask questions, discover what an electron is, determine whether steel
    memes melt stale memes, and just about basically everything else."
}

config! {
    DefineAvailable,
    "define.available",
    ConfigType::Availability,
    Value::U64(Availability::Enabled.num()),
    "Whether the ability to use `define` is available."
}

config! {
    DefineExample,
    "define.example",
    ConfigType::Availability,
    Value::U64(Availability::Enabled.num()),
    "Whether to display an example with the definition."
}

config! {
    EmojiAvailable,
    "emoji.available",
    ConfigType::Availability,
    Value::U64(Availability::Enabled.num()),
    "Whether the ability to use `emoji` is available."
}

config! {
    HelloAvailable,
    "hello.available",
    ConfigType::Availability,
    Value::U64(Availability::Enabled.num()),
    "Whether the ability to use `hello` is available."
}

config! {
    LmgtfyAvailable,
    "lmgtfy.available",
    ConfigType::Availability,
    Value::U64(Availability::Enabled.num()),
    "Whether the ability to use `lmgtfy` is available."
}

config! {
    LmgtfyResults,
    "lmgtfy.results",
    ConfigType::Int,
    Value::I64(1),
    1,
    5,
    "The number of results to return from a search."
}

config! {
    MagicEightBallAvailable,
    "8ball.available",
    ConfigType::Availability,
    Value::U64(Availability::Enabled.num()),
    "Whether the ability to use `8ball` is available."
}

config! {
    MangaAvailable,
    "manga.available",
    ConfigType::Availability,
    Value::U64(Availability::Enabled.num()),
    "Whether the ability to use `manga` is available."
}

config! {
    MfwAvailable,
    "mfw.available",
    ConfigType::Availability,
    Value::U64(Availability::Enabled.num()),
    "Whether the ability to use `mfw` is available."
}

config! {
    PiAvailable,
    "pi.available",
    ConfigType::Availability,
    Value::U64(Availability::Enabled.num()),
    "Whether the ability to use `pi` is available."
}

config! {
    PiPrecisionDefault,
    "pi.precision.default",
    ConfigType::Int,
    Value::I64(100),
    0,
    1000,
    "The number of digits of pi to return by default."
}

config! {
    PiPrecisionMaximum,
    "pi.precision.maximum",
    ConfigType::Int,
    Value::I64(100),
    1000,
    0,
    "The number of digits of pi that can be returned at a maximum amount.

    This is useful to prevent a serve rmessage flood (1000 is a lot of text)."
}

config! {
    PingAvailable,
    "ping.available",
    ConfigType::Availability,
    Value::U64(Availability::Enabled.num()),
    "Whether the ability to use `ping` is available."
}

config! {
    PixivAutomatic,
    "pixiv.automatic",
    ConfigType::Availability,
    Value::U64(Availability::Disabled.num()),
    "Whether to automatically embed an image when a pixiv link is seen.

    This will automatically retrieve the pixiv image whenever a pixiv link is
    seen _anywhere_ in _any_ message, regardless if the `pixiv` command is
    used."
}

config! {
    PixivAvailable,
    "pixiv.available",
    ConfigType::Availability,
    Value::U64(Availability::Enabled.num()),
    "Whether the ability to use `pixiv` is available."
}

config! {
    PixivInfo,
    "pixiv.info",
    ConfigType::Availability,
    Value::U64(Availability::Enabled.num()),
    "Whether to embed author information at the bottom of the picture.

    Enabling this will give a white bar at the bottom of the image with a URL to
    the illustration and the author."
}

config! {
    PurgeAvailable,
    "purge.available",
    ConfigType::Availability,
    Value::U64(Availability::Enabled.num()),
    "Whether the ability to use `purge` is available.

    This is only available to those with the 'Administrator' and/or
    'Manage Server' permissions."
}

config! {
    PurgeDefault,
    "purge.default",
    ConfigType::Int,
    Value::I64(50),
    2,
    100,
    "The maximum number of messages that can be purged at once."
}

config! {
    PurgeMaximum,
    "purge.maximum",
    ConfigType::Int,
    Value::I64(100),
    2,
    100,
    "The maximum number of messages that can be purged at once."
}

config! {
    PurgeMinimum,
    "purge.minimum",
    ConfigType::Int,
    Value::I64(2),
    2,
    100,
    "The maximum number of messages that can be purged at once."
}

config! {
    RemindMeAvailable,
    "remindme.available",
    ConfigType::Availability,
    Value::U64(Availability::Enabled.num()),
    "Whether the ability to use `remindme` and `remind` are available."
}

config! {
    RoleInfoAvailable,
    "roleinfo.available",
    ConfigType::Availability,
    Value::U64(Availability::Enabled.num()),
    "Whether the ability to use `roleinfo` is available."
}

config! {
    RollAvailable,
    "roll.available",
    ConfigType::Availability,
    Value::U64(Availability::Enabled.num()),
    "Whether the ability to use `roll` is available."
}

config! {
    RollCustom,
    "roll.custom",
    ConfigType::Availability,
    Value::U64(Availability::Enabled.num()),
    "Whether users can give custom numbers to role between.

    Otherwise, a predetermined set of numbers will be rolled."
}

config! {
    RollMaximum,
    "roll.maximum",
    ConfigType::Int,
    Value::I64(i64::MAX),
    i64::MIN + 1,
    i64::MAX,
    "The maximum value that can be rolled."
}

config! {
    RollMinimum,
    "roll.minimum",
    ConfigType::Int,
    Value::I64(i64::MIN),
    i64::MIN,
    i64::MAX - 1,
    "The minimum value that can be rolled."
}

config! {
    RouletteAvailable,
    "roulette.available",
    ConfigType::Availability,
    Value::U64(Availability::Enabled.num()),
    "Whether the ability to use `roulette` is available."
}

config! {
    ServerInfoAvailable,
    "serverinfo.available",
    ConfigType::Availability,
    Value::U64(Availability::Enabled.num()),
    "Whether the ability to use `serverinfo` is available."
}

config! {
    SkipAvailable,
    "skip.available",
    ConfigType::Availability,
    Value::U64(Availability::Enabled.num()),
    "Whether the ability to use `skip` is available."
}

config! {
    SkipRequired,
    "skip.required",
    ConfigType::Int,
    Value::I64(2),
    1,
    50,
    "The number of skip votes required for a song to be skipped."
}

config! {
    StatsAvailable,
    "stats.available",
    ConfigType::Availability,
    Value::U64(Availability::Enabled.num()),
    "Whether the ability to use `stats` is available."
}

config! {
    TagsAvailable,
    "tags.available",
    ConfigType::Availability,
    Value::U64(Availability::Enabled.num()),
    "Whether the ability to use the tagging system is available."
}

config! {
    TeamsAvailable,
    "teams.available",
    ConfigType::Availability,
    Value::U64(Availability::Enabled.num()),
    "Whether the ability to use `teams` is available."
}

config! {
    UserInfoAvailable,
    "userinfo.available",
    ConfigType::Availability,
    Value::U64(Availability::Enabled.num()),
    "Whether the ability to use `userinfo` is available."
}

config! {
    WeatherAvailable,
    "weather.available",
    ConfigType::Availability,
    Value::U64(Availability::Enabled.num()),
    "Whether the ability to use `weather` is available."
}

config! {
    WeatherSaving,
    "weather.saving",
    ConfigType::Availability,
    Value::U64(Availability::Enabled.num()),
    "Whether users can save their location to this server.

    This will allow users to easily retrieve their weather via just `weather`."
}

config! {
    WolframAvailable,
    "wolfram.available",
    ConfigType::Availability,
    Value::U64(Availability::Enabled.num()),
    "Whether the ability to use `wolfram` is available."
}

config! {
    WolframRows,
    "wolfram.rows",
    ConfigType::Int,
    Value::I64(2),
    1,
    10,
    "The number of rows of data to output from Wolfram."
}

config! {
    XkcdAvailable,
    "xkcd.available",
    ConfigType::Availability,
    Value::U64(Availability::Enabled.num()),
    "Whether the ability to use `xkcd` is available."
}
