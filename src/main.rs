#[macro_use] extern crate lazy_static;
#[macro_use] extern crate log;

extern crate chrono;
extern crate dotenv;
extern crate env_logger;
extern crate darksky;
extern crate hyper;
extern crate kitsu_io;
extern crate psutil;
extern crate rand;
extern crate regex;
extern crate reqwest;
extern crate serde_json;
extern crate serde;
extern crate serenity;
extern crate typemap;
extern crate urbandictionary;

#[macro_use] mod utils;

mod commands;
mod event;
mod misc;
mod prelude;
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
        .command("udefined", |c| c.cmd(commands::conversation::UdefineCommand))
        .help(help_commands::with_embeds)
        .group("Luck", |g| g
            .command("8ball", |c| c
                .cmd(commands::random::MagicEightBallCommand))
            .command("choose", |c| c
                .cmd(commands::random::ChooseCommand))
            .command("coinflip", |c| c
                .cmd(commands::random::CoinflipCommand))
            .command("roll", |c| c
                .cmd(commands::random::RollCommand))
            .command("roulette", |c| c
                .cmd(commands::random::RouletteCommand)))
        .group("Media", |g| g
            .command("anime", |c| c
                .known_as("animu")
                .cmd(commands::media::AnimeCommand)))
        .group("Meta", |g| g
            .command("avatar", |c| c
                .cmd(commands::meta::AvatarCommand))
            .command("rping", |c| c
                .cmd(commands::meta::RpingCommand)
                .help_available(false)
                .owners_only(true))
            .command("gping", |c| c
                .cmd(commands::meta::GpingCommand)
                .help_available(false)
                .owners_only(true))
            .command("roleinfo", |c| c
                .cmd(commands::meta::RoleInfoCommand))
            .command("uptime", |c| c
                .cmd(commands::meta::UptimeCommand))
            .command("userinfo", |c| c
                .known_as("me")
                .cmd(commands::meta::UserInfoCommand)))
        .group("Misc", |g| g
            .command("aes", |c| c
                .cmd(commands::misc::AesCommand))
            .command("aescaps", |c| c
                .cmd(commands::misc::AesCapsCommand))
            .command("aesthetic", |c| c
                .cmd(commands::misc::AesCommand))
            .command("aestheticcaps", |c| c
                .cmd(commands::misc::AesCapsCommand))
            .command("hello", |c| c
                .cmd(commands::misc::HelloCommand))
            .command("mfw", |c| c
                .cmd(commands::misc::MfwCommand))
            .command("pi", |c| c
                .cmd(commands::misc::PiCommand)))
        .command("modping", |c| c
            .cmd(commands::conversation::ModPingCommand)
            .guild_only(true)
            .help_available(false)
            .known_as("pingmod"))
        .command("commands", |c| c
            .cmd(commands::owner::CommandsCommand)
            .help_available(false)
            .owners_only(true))
        .command("eval", |c| c
            .cmd(commands::owner::EvalCommand)
            .help_available(false)
            .owners_only(true))
        .command("stats", |c| c
            .cmd(commands::owner::StatsCommand)
            .help_available(false)
            .owners_only(true))
        .command("events", |c| c
            .cmd(commands::owner::EventsCommand)
            .help_available(false)
            .owners_only(true))
        .command("set name", |c| c
            .cmd(commands::owner::SetNameCommand)
            .help_available(false)
            .owners_only(true))
        .command("set status", |c| c
            .cmd(commands::owner::SetStatusCommand)
            .help_available(false)
            .owners_only(true)));

    if let Err(why) = client.start_autosharded() {
        error!("Err starting client: {:?}", why);
    }
}
