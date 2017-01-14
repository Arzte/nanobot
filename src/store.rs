use serenity::model::UserId;
use std::collections::HashMap;
use typemap::Key;
use ::misc::Uptime;

pub struct CommandCounter;

impl Key for CommandCounter {
    type Value = HashMap<String, u64>;
}

pub struct EventCounter;

impl Key for EventCounter {
    type Value = HashMap<&'static str, u64>;
}

pub struct NanoCache;

impl Key for NanoCache {
    type Value = CustomCache;
}

pub struct ShardUptime;

impl Key for ShardUptime {
    type Value = HashMap<u64, Uptime>;
}

pub struct CustomCache {
    pub owner_id: UserId,
}

impl Default for CustomCache {
    fn default() -> Self {
        CustomCache {
            owner_id: UserId(0),
        }
    }
}
