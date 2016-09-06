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
use discord::ChannelRef;
use serde_json::Value;
use std::collections::BTreeMap;
use ::prelude::*;

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
macro_rules! reqf {
    ($opt:expr) => {
        try!($opt.ok_or(Error::Decode))
    }
}

pub fn get_location(context: &Context) -> Result<(ServerId, ChannelId)> {
    let state = context.state.lock().unwrap();

    match state.find_channel(&context.message.channel_id) {
        Some(ChannelRef::Public(server, channel)) => Ok((server.id, channel.id)),
        _ => Err(Error::Decode),
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
    map.remove(key).ok_or(Error::Decode)
}
