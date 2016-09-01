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

use discord::model::{CurrentUser, ReadyEvent, UserId};
use serde_json::Value;
use std::collections::BTreeMap;
use ::error::{Error, Result};

pub fn make_fake_ready_event() -> ReadyEvent {
    ReadyEvent {
        version: 0,
        user: CurrentUser {
            id: UserId(0),
            username: String::new(),
            discriminator: 0,
            avatar: None,
            email: None,
            verified: false,
            bot: false,
            mfa_enabled: false,
        },
        session_id: String::new(),
        user_settings: None,
        read_state: None,
        private_channels: vec![],
        presences: vec![],
        relationships: vec![],
        servers: vec![],
        user_server_settings: None,
        tutorial: None,
        trace: vec![],
        notes: None,
        shard: None,
    }
}

#[macro_escape]
macro_rules! req {
    ($expr:expr) => {
        match $expr {
            Ok(v) => v,
            Err(_why) => return,
        }
    }
}

#[macro_escape]
macro_rules! arc {
    ($db:expr) => {
        match ::std::sync::Arc::try_unwrap($db.clone()) {
            Ok(db) => db,
            Err(_why) => return,
        }
    }
}

#[macro_escape]
macro_rules! reqf {
    ($opt:expr) => {
        try!($opt.ok_or(Error::Decode))
    }
}

pub fn decode_array<T, F: Fn(Value) -> Result<T>>(value: Value,
                                                  f: F)
                                                  -> Result<Vec<T>> {
    into_array(value).and_then(|x| x.into_iter().map(f).collect())
}

pub fn into_array(value: Value) -> Result<Vec<Value>> {
    match value {
        Value::Array(v) => Ok(v),
        _value => Err(Error::Decode),
    }
}

pub fn into_map(value: Value) -> Result<BTreeMap<String, Value>> {
    match value {
        Value::Object(m) => Ok(m),
        _value => Err(Error::Decode),
    }
}

pub fn into_string(value: Value) -> Result<String> {
    match value {
        Value::String(s) => Ok(s),
        _value => Err(Error::Decode),
    }
}

pub fn opt<T, F>(map: &mut BTreeMap<String, Value>,
                 key: &str, f: F)
                 -> Result<Option<T>>
                 where F: FnOnce(Value) -> Result<T> {
    match map.remove(key) {
        None | Some(Value::Null) => Ok(None),
        Some(val) => f(val).map(Some),
    }
}

pub fn remove(map: &mut BTreeMap<String, Value>, key: &str) -> Result<Value> {
    map.remove(key)
        .ok_or(Error::Decode)
}
