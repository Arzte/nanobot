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
extern crate psutil;
extern crate rand;
extern crate regex;
extern crate serde_json;
extern crate serde;
extern crate typemap;
extern crate urbandictionary;

#[macro_use] mod utils;

mod commands;
mod event;
mod misc;
mod store;

use misc::Uptime;
use serenity::client::{Client, rest};
use serenity::ext::framework::help_commands;
use std::env;
use std::collections::{HashMap, HashSet};
use store::{CommandCounter, CustomCache, EventCounter, NanoCache, ShardUptime};

fn main() {
    env_logger::init().expect("env logger");
    dotenv::dotenv().expect("init dotenv");

    let mut client = Client::login_bot(&env::var("DISCORD_TOKEN").unwrap());

    {
        let mut data = client.data.lock().unwrap();
        data.insert::<CommandCounter>(HashMap::default());
        data.insert::<EventCounter>(HashMap::default());
        data.insert::<NanoCache>(CustomCache::default());
        data.insert::<ShardUptime>(HashMap::default());
    }

    let owners = {
        let mut set = HashSet::new();

        let info = match rest::get_current_application_info() {
            Ok(info) => info,
            Err(why) => panic!("Couldn't get application info: {:?}", why),
        };

        {
            let mut data = client.data.lock().unwrap();
            let custom_cache = data.get_mut::<NanoCache>().unwrap();
            custom_cache.owner_id = info.owner.id;
        }

        set.insert(info.owner.id);

        set
    };

    event::register(&mut client);

    client.with_framework(|f| f
        .configure(|c| c
            .allow_whitespace(true)
            .on_mention(true)
            .owners(owners)
            .prefix(";;"))
        .before(|context, message, command_name| {
            info!("{} used command '{}'", message.author.name, command_name);

            let mut data = context.data.lock().unwrap();
            let counter = data.get_mut::<CommandCounter>().unwrap();
            let entry = counter.entry(command_name.clone()).or_insert(0);
            *entry += 1;

            true
        })
        .command("help", |c| c.exec_help(help_commands::with_embeds))
        .on("udefine", commands::conversation::udefine)
        .group("Luck", |g| g
            .command("8ball", |c| c
                .exec(commands::random::magic_eight_ball))
            .command("choose", |c| c
                .exec(commands::random::choose))
            .command("coinflip", |c| c
                .exec(commands::random::coinflip))
            .command("roll", |c| c
                .exec(commands::random::roll))
            .command("roulette", |c| c
                .exec(commands::random::roulette)))
        .group("Media", |g| g
            .command("anime", |c| c
                .exec(commands::media::anime)))
        .group("Meta", |g| g
            .command("avatar", |c| c
                .exec(commands::meta::avatar))
            .command("emoji", |c| c
                .exec(commands::meta::emoji))
            .command("rping", |c| c
                .exec(commands::meta::rping)
                .help_available(false)
                .owners_only(true))
            .command("roleinfo", |c| c
                .exec(commands::meta::role_info))
            .command("uptime", |c| c
                .exec(commands::meta::uptime))
            .command("userinfo", |c| c
                .exec(commands::meta::user_info)))
        .group("Misc", |g| g
            .command("aes", |c| c
                .exec(commands::misc::aes))
            .command("aescaps", |c| c
                .exec(commands::misc::aescaps))
            .command("aesthetic", |c| c
                .exec(commands::misc::aes))
            .command("aestheticcaps", |c| c
                .exec(commands::misc::aescaps))
            .command("hello", |c| c
                .exec(commands::misc::hello))
            .command("mfw", |c| c
                .exec(commands::misc::mfw))
            .command("pi", |c| c
                .exec(commands::misc::pi)))
        .command("commands", |c| c
            .exec(commands::owner::commands)
            .help_available(false)
            .owners_only(true))
        .command("eval", |c| c
            .exec(commands::owner::eval)
            .help_available(false)
            .owners_only(true)
            .use_quotes(false))
        .command("stats", |c| c
            .exec(commands::owner::stats)
            .help_available(false)
            .owners_only(true))
        .command("events", |c| c
            .exec(commands::owner::events)
            .help_available(false)
            .owners_only(true))
        .command("set name", |c| c
            .exec(commands::owner::set_name)
            .help_available(false)
            .owners_only(true))
        .command("set status", |c| c
            .exec(commands::owner::set_status)
            .help_available(false)
            .owners_only(true)));

    client.on_ready(|context, ready| {
        if let Some(s) = ready.shard {
            info!("Logged in as '{}' on {}/{}",
                  ready.user.name,
                  s[0],
                  s[1]);
        } else {
            info!("Logged in as '{}'", ready.user.name);
        }

        let name = {
            let mut data = context.data.lock().unwrap();

            {
                let counter = data.get_mut::<EventCounter>().unwrap();
                let entry = counter.entry("Ready").or_insert(0);
                *entry += 1;
            }

            let uptimes = data.get_mut::<ShardUptime>().unwrap();

            if let Some(shard) = ready.shard {
                let entry = uptimes.entry(shard[0]).or_insert_with(Uptime::default);
                entry.connect();

                format!(";;help [{}/{}]", shard[0] + 1, shard[1])
            } else {
                ";;help".to_owned()
            }
        };

        context.set_game_name(&name);
    });

    let _ = client.start_shards(2);
}
