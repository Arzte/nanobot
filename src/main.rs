#[macro_use] extern crate lazy_static;
#[macro_use] extern crate log;
#[macro_use] extern crate serenity;

extern crate chrono;
extern crate dotenv;
extern crate env_logger;
extern crate darksky;
extern crate hyper;
extern crate kitsu_io;
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

use serenity::client::{Client, rest};
use serenity::framework::standard::{StandardFramework, help_commands};
use std::env;
use std::collections::{HashMap, HashSet};
use store::{CommandCounter, CustomCache, EventCounter, NanoCache, ShardUptime};

fn main() {
    dotenv::dotenv().expect("init dotenv");
    env_logger::init().expect("env logger");

    let mut client = Client::new(
        &env::var("DISCORD_TOKEN").expect("no token present"),
        event::Handler
    ).expect("error creating client");

    {
        let mut data = client.data.lock();
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
            let mut data = client.data.lock();
            let custom_cache = data.get_mut::<NanoCache>().unwrap();
            custom_cache.owner_id = info.owner.id;
        }

        set.insert(info.owner.id);

        set
    };

    client.with_framework(StandardFramework::new()
        .configure(|c| c
            .allow_whitespace(true)
            .on_mention(true)
            .owners(owners)
            .prefixes(vec!["nano"]))
        .before(|context, message, command_name| {
            info!("{} used command '{}'", message.author.name, command_name);

            let mut data = context.data.lock();
            let counter = data.get_mut::<CommandCounter>().unwrap();
            let entry = counter.entry(command_name.to_owned()).or_insert(0);
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
                .known_as("animu")
                .exec(commands::media::anime)))
        .group("Meta", |g| g
            .command("avatar", |c| c
                .exec(commands::meta::avatar))
            .command("rping", |c| c
                .exec(commands::meta::rping)
                .help_available(false)
                .owners_only(true))
            .command("gping", |c| c
                .exec(commands::meta::gping)
                .help_available(false)
                .owners_only(true))
            .command("roleinfo", |c| c
                .exec(commands::meta::role_info))
            .command("uptime", |c| c
                .exec(commands::meta::uptime))
            .command("userinfo", |c| c
                .known_as("me")
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
        .command("modping", |c| c
            .exec(commands::conversation::modping)
            .guild_only(true)
            .help_available(false)
            .known_as("pingmod"))
        .command("commands", |c| c
            .exec(commands::owner::commands)
            .help_available(false)
            .owners_only(true))
        .command("eval", |c| c
            .exec(commands::owner::eval)
            .help_available(false)
            .owners_only(true))
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

    if let Err(why) = client.start_autosharded() {
        error!("Err starting client: {:?}", why);
    }
}
