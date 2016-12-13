use std::collections::HashMap;
use typemap::Key;
use ::misc::Uptime;

pub struct CommandCounter;

impl Key for CommandCounter {
    type Value = HashMap<String, u64>;
}

pub struct ShardUptime;

impl Key for ShardUptime {
    type Value = HashMap<u8, Uptime>;
}
