#[macro_use] extern crate lazy_static;
#[macro_use] extern crate log;
#[macro_use] extern crate serenity;

extern crate chrono;
extern crate dotenv;
extern crate env_logger;
extern crate darksky;
extern crate diesel;
extern crate hummingbird;
extern crate hyper;
extern crate rand;
extern crate regex;
extern crate serde_json;
extern crate serde;
extern crate typemap;
extern crate urbandictionary;

#[macro_use] mod utils;

mod commands;
mod misc;
mod store;

use misc::Uptime;
use serenity::client::{Client, Context};
use serenity::model::Message;
use std::env;
use std::collections::HashMap;
use store::{CommandCounter, ShardUptime};

fn main() {
    env_logger::init().expect("env logger");
    dotenv::dotenv().expect("init dotenv");

    let mut client = Client::login_bot(&env::var("DISCORD_TOKEN").unwrap());

    {
        let mut data = client.data.lock().unwrap();
        data.insert::<store::CommandCounter>(HashMap::default());
        data.insert::<ShardUptime>(HashMap::default());
    }

    client.with_framework(|f| f
        .configure(|c| c
            .allow_whitespace(true)
            .on_mention(true)
            .prefix(";;"))
        .before(|context, _message, command_name| {
            let mut data = context.data.lock().unwrap();
            let counter = data.get_mut::<CommandCounter>().unwrap();
            let entry = counter.entry(command_name.clone()).or_insert(0);
            *entry += 1;
        })
        .on("8ball", commands::random::magic_eight_ball)
        .on("aes", commands::misc::aes)
        .on("aescaps", commands::misc::aescaps)
        .on("aesthetic", commands::misc::aes)
        .on("aestheticcaps", commands::misc::aescaps)
        .on("anime", commands::media::anime)
        .on("avatar", commands::misc::avatar)
        .on("choose", commands::random::choose)
        .on("coinflip", commands::random::coinflip)
        .command("$ commands", |c| c
            .check(owner_check)
            .exec(commands::owner::commands))
        .on("emoji", commands::meta::emoji)
        .command("$ eval", |c| c
            .check(owner_check)
            .exec(commands::owner::eval)
            .use_quotes(false))
        .on("hello", commands::misc::hello)
        .on("invite", commands::meta::invite)
        .on("mfw", commands::misc::mfw)
        .on("pi", commands::misc::pi)
        .on("ping", commands::meta::ping)
        .on("roleinfo", commands::meta::role_info)
        .on("roll", commands::random::roll)
        .on("roulette", commands::random::roulette)
        .command("$ set name", |c| c
            .check(owner_check)
            .exec(commands::owner::set_name))
        .command("$ set status", |c| c
            .check(owner_check)
            .exec(commands::meta::set_status))
        .on("udefine", commands::conversation::udefine)
        .on("uptime", commands::meta::uptime)
        .on("userinfo", commands::meta::user_info));

    client.on_ready(|context, ready| {
        info!("Logged in as: {}", ready.user.name);

        let mut data = context.data.lock().unwrap();
        let uptimes = data.get_mut::<ShardUptime>().unwrap();

        let name = if let Some(shard) = ready.shard {
            let entry = uptimes.entry(shard[0]).or_insert_with(Uptime::default);
            entry.connect();

            format!(";;help or ;;invite [{}/{}]", shard[0] + 1, shard[1])
        } else {
            ";;help or ;;invite".to_owned()
        };

        context.set_game_name(&name);
    });

    let _ = client.start_autosharded();
}

fn owner_check(_context: &Context, message: &Message) -> bool {
    let id = env::var("AUTHOR_ID")
        .map(|x| x.parse::<u64>().unwrap_or(0))
        .unwrap_or(0);

    message.author.id == id
}
