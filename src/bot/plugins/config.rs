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

use discord::model::{ChannelId, permissions};
use discord::ChannelRef;
use serde_json::Value;
use std::env;
use ::bot::config;
use ::prelude::*;

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug, Ord, PartialOrd)]
enum Action {
    Get,
    List,
    Set,
}

pub fn base(context: Context) {
    let can_use = {
        let state = context.state.lock().unwrap();

        match state.find_channel(&context.message.channel_id) {
            Some(ChannelRef::Public(server, _channel)) => {
                server.permissions_for(context.message.channel_id,
                                       context.message.author.id)
                    .intersects(permissions::ADMINISTRATOR | permissions::MANAGE_SERVER)
            },
            _ => false,
        }
    };


    let author_var = if let Ok(var) = env::var("AUTHOR_ID") {
        var
    } else {
        let _msg = req!(context.say("Error getting events"));
        error!("[env] AUTHOR_ID env var not set");

        return;
    };

    let author_id = if let Ok(id) = author_var.parse::<u64>() {
        id
    } else {
        let _msg = req!(context.reply("Error getting events"));

        return;
    };

    let is_bot_owner = context.message.author.id.0 == author_id;

    if !(can_use || is_bot_owner) {
        let _msg = req!(context.say("You do not have permission to manage configuration"));

        return;
    }

    let action = {
        let arg1 = context.arg(1).clone();

        let arg = match arg1.as_str() {
            Ok(arg) => arg,
            Err(_why) => {
                let _msg = req!(context.say("Channel name, config name, or action required"));

                return;
            },
        };

        if arg == "set" {
            Action::Set
        } else if arg == "list" {
            Action::List
        } else {
            Action::Get
        }
    };

    match action {
        Action::Get => get(context),
        Action::List => list(context),
        Action::Set => set(context),
    }
}

fn get_mentioned_channel(context: &Context,
                         action: Action) -> Option<ChannelId> {
    let bump = match action {
        Action::Get => 0,
        Action::List | Action::Set => 1,
    };

    let arg1 = context.arg(1 + bump);
    let arg = match arg1.as_str() {
        Ok(contents) => contents,
        Err(_why) => return None,
    };

    if arg.starts_with("<#") {
        let channel_mentions = context.channel_mentions();

        channel_mentions.get(0).map(|channel| channel.id)
    } else if arg.contains('.') {
        None
    } else {
        let state = context.state.lock().unwrap();

        state.servers()
            .iter()
            .find(|server| {
                server.channels
                    .iter()
                    .any(|channel| {
                        channel.id.0 == context.message.channel_id.0
                    })
            })
            .and_then(|server| {
                server.channels
                    .iter()
                    .find(|channel| &*channel.name == arg)
            })
            .map(|channel| channel.id)
    }
}

fn get_key(context: &Context, action: Action) -> Option<String> {
    let bump = match action {
        Action::Get => 0,
        Action::List | Action::Set => 1,
    };

    let with_channel = get_mentioned_channel(context, action).map_or(0, |_v| 1);

    context.arg(1 + bump + with_channel)
        .as_str()
        .ok()
        .map(|v| v.to_owned())
}

fn get_value(context: &Context, action: Action) -> Option<String> {
    let bump = match action {
        Action::Get => 0,
        Action::List | Action::Set => 1,
    };

    let with_channel = get_mentioned_channel(context, action).map_or(0, |_v| 1);

    let text = context.text(1 + bump + with_channel);

    if !text.is_empty() {
        Some(text)
    } else {
        None
    }
}

fn get(context: Context) {
    let name = match get_key(&context, Action::Get) {
        Some(name) => name,
        None => {
            let _msg = req!(context.say("Config name not found"));

            return;
        },
    };

    let location = match get_location(&context) {
        Ok(location) => location,
        Err(_why) => {
            let _msg = req!(context.say("Server not found"));

            return;
        },
    };

    let channel_id = get_mentioned_channel(&context, Action::Get);

    let location = (location.0, channel_id);

    let config = match get_config(&name, (location.0, channel_id)) {
        Some(config) => config,
        None => {
            let _msg = req!(context.say("Config not found"));

            return;
        },
    };

    let default = match config.kind {
        ConfigType::Availability => match config.default {
            Value::U64(0) => "Disabled".to_owned(),
            Value::U64(1) => "Enabled".to_owned(),
            _ => "N/A".to_owned(),
        },
        ConfigType::Int => match config.default {
            Value::I64(v) => v.to_string(),
            Value::U64(v) => v.to_string(),
            _ => "N/A".to_owned(),
        },
        ConfigType::String => match config.default {
            Value::String(ref v) => v.clone(),
            _ => "N/A".to_owned(),
        },
    };
    let value = match config.kind {
        ConfigType::Availability => match config.value {
            Value::U64(0) => "Disabled".to_owned(),
            Value::U64(1) => "Enabled".to_owned(),
            _ => "N/A".to_owned(),
        },
        ConfigType::Int => match config.value {
            Value::I64(v) => v.to_string(),
            Value::U64(v) => v.to_string(),
            _ => "N/A".to_owned(),
        },
        ConfigType::String => match config.value {
            Value::String(ref v) => v.clone(),
            _ => "N/A".to_owned(),
        },
    };

    let mut text = format!("```xl
         Name: {}
         Type: {}

Default Value: {}
Current Value: {}", config.key,
                    config.kind.name(),
                    default,
                    value);

    let min = config.min_value.map_or(String::from("N/A"), |v| v.to_string());
    let max = config.max_value.map_or(String::from("N/A"), |v| v.to_string());

    if config.int() {
        text.push_str(&format!("
Minimum Value: {}
Maximum Value: {}", min, max)[..]);
    }

    text.push_str("

Description:\n\n");
    text.push_str(config.description);
    text.push_str("```");

    let _msg = req!(context.say(text));
}

fn list(context: Context) {
    let text = format!("```xl\n- {}```", config::CONFIGS.join("\n- "));

    let _msg = req!(context.say(text));
}

fn set(context: Context) {
    let name = match get_key(&context, Action::Set) {
        Some(name) => name,
        None => {
            let _msg = req!(context.say("Config name not given"));

            return;
        },
    };

    let location = match get_location(&context) {
        Ok(location) => location,
        Err(_why) => {
            let _msg = req!(context.say("Server not found"));

            return;
        },
    };

    let channel_id = get_mentioned_channel(&context, Action::Get);

    let location = (location.0, channel_id);

    let config = match get_config(&name, (location.0, channel_id)) {
        Some(config) => config,
        None => {
            let _msg = req!(context.say("Config not found"));

            return;
        },
    };

    let value = match get_value(&context, Action::Set) {
        Some(value) => value,
        None => {
            let _msg = req!(context.say("A config value must be given"));

            return;
        },
    };

    let v = match config.kind {
        ConfigType::Availability => match &*value.to_lowercase() {
            "disabled" => Value::U64(0),
            "enabled" => Value::I64(1),
            _ => {
                let _msg = req!(context.say("Valid options are `enabled` and `disabled`"));

                return;
            },
        },
        ConfigType::Int => match value.parse::<i64>() {
            Ok(v) => Value::I64(v),
            Err(_why) => {
                let _msg = req!(context.say("Must be a valid number"));

                return;
            },
        },
        ConfigType::String => Value::String(value),
    };

    match set_config(&name, (location.0, channel_id), v) {
        Some(Ok(())) => {
            let _msg = req!(context.say("Config updated"));
        },
        Some(Err(Error::SqlExecution)) => {
            let _msg = req!(context.say("Error updating config"));
        },
        Some(Err(_)) => {
            let _msg = req!(context.say("Unknown error"));
        },
        None => {
            let _msg = req!(context.say("Config not found"));
        },
    }
}
