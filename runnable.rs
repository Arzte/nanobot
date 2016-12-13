extern crate dotenv;
extern crate serenity;
extern crate typemap;

use serenity::client::gateway::Shard;
use serenity::client::{Context, LoginType, rest};
use serenity::model::ChannelId;
use std::sync::{Arc, Mutex};
use std::env;
use typemap::ShareMap;

fn main() {
    dotenv::dotenv().expect("Failed to initialize dotenv");

    let token = {
        let var = env::var("DISCORD_TOKEN").expect("discord token in env");

        format!("Bot {}", var)
    };

    rest::set_token(&token);

    let url = rest::get_gateway().unwrap().url;
    let (shard, _, _) = Shard::new(&url, &token, Some([0, 1]), LoginType::Bot)
        .expect("err sharding");

    let context = Context::new(Some(ChannelId({CHANNEL_ID})),
                               Arc::new(Mutex::new(shard)),
                               Arc::new(Mutex::new(ShareMap::custom())),
                               LoginType::Bot);

    println!("{:?}", {
        {CODE}
    });
}
