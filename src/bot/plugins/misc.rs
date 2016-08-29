use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel;
use discord::{ChannelRef, State};
use forecast_io::{self, Icon};
use rand::{Rng, thread_rng};
use std::ascii::AsciiExt;
use std::{char, env, str};
use ::bot::Uptime;
use ::ext::google_maps;
use ::models::*;
use ::prelude::*;
use ::schema::members::dsl::*;

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Aesthetic {
    Bold,
    Caps,
}

pub struct Misc<'a> {
    aesthetic_chars: Vec<(char, &'a str)>,
    emojis: Vec<&'a str>,
}

impl<'a> Misc<'a> {
    pub fn new<'b>() -> Misc<'b> {
        Misc {
            aesthetic_chars: vec![
                ('A', "Ａ"),
                ('B', "Ｂ"),
                ('C', "Ｃ"),
                ('D', "Ｄ"),
                ('E', "Ｅ"),
                ('F', "Ｆ"),
                ('G', "Ｇ"),
                ('H', "Ｈ"),
                ('I', "Ｉ"),
                ('J', "Ｊ"),
                ('K', "Ｋ"),
                ('L', "Ｌ"),
                ('M', "Ｍ"),
                ('N', "Ｎ"),
                ('O', "Ｏ"),
                ('P', "Ｐ"),
                ('Q', "Ｑ"),
                ('R', "Ｒ"),
                ('S', "Ｓ"),
                ('T', "Ｔ"),
                ('U', "Ｕ"),
                ('V', "Ｖ"),
                ('W', "Ｗ"),
                ('X', "Ｘ"),
                ('Y', "Ｙ"),
                ('Z', "Ｚ"),
                ('[', "［"),
                ('\\', "＼"),
                (']', "］"),
                ('^', "＾"),
                ('_', "＿"),
                ('`', "｀"),
                ('a', "ａ"),
                ('b', "ｂ"),
                ('c', "ｃ"),
                ('d', "ｄ"),
                ('e', "ｅ"),
                ('f', "ｆ"),
                ('g', "ｇ"),
                ('h', "ｈ"),
                ('i', "ｉ"),
                ('j', "ｊ"),
                ('k', "ｋ"),
                ('l', "ｌ"),
                ('m', "ｍ"),
                ('n', "ｎ"),
                ('o', "ｏ"),
                ('p', "ｐ"),
                ('q', "ｑ"),
                ('r', "ｒ"),
                ('s', "ｓ"),
                ('t', "ｔ"),
                ('u', "ｕ"),
                ('v', "ｖ"),
                ('w', "ｗ"),
                ('x', "ｘ"),
                ('y', "ｙ"),
                ('z', "ｚ"),
            ],
            emojis: vec![
                "blush",
                "cop",
                "cry",
                "disappointed",
                "dizzy",
                "fearful",
                "flushed",
                "frowning",
                "grimacing",
                "grin",
                "heart_eyes",
                "innocent",
                "kissing",
                "kissing_closed_eyes",
                "laughing",
                "man_with_turban",
                "neutral_face",
                "open_mouth",
                "poop",
                "rage",
                "relaxed",
                "scream",
                "sleeping",
                "smile",
                "smiley",
                "smirk",
                "stuck_out_tongue",
                "stuck_out_tongue_closed_eyes",
                "stuck_out_tongue_winking_eye",
                "weary",
                "wink",
                "yum",
            ],
        }
    }

    pub fn aesthetic(&self, context: Context, modifiers: Vec<Aesthetic>) {
        let mut text = context.text(0);

        if text.is_empty() {
            let _msg = req!(context.say("Nothing to aestheticize"));

            return;
        }

        if modifiers.contains(&Aesthetic::Caps) {
            text.make_ascii_uppercase();
        }

        for chars in &self.aesthetic_chars {
            text = text.replace(chars.0, &chars.1[..]);
        }

        text = text.replace(' ', "  ");

        if modifiers.contains(&Aesthetic::Bold) {
            text.insert(0, '*');
            text.insert(0, '*');
            text.push('*');
            text.push('*');
        }

        let _msg = req!(context.say(text));
    }

    pub fn hello(&self, context: Context) {
        let greetings = vec![
            format!("Hello {}", context.message.author.name),
            format!("Hey {}!", context.message.author.name),
            "Hello fella!".to_owned(),
            "Hey fella!".to_owned(),
            format!("What's up {}?", context.message.author.name),
            format!("Selamat pagi, {}", context.message.author.name),
            format!("G'day {}!", context.message.author.name),
        ];

        let _msg = match thread_rng().choose(&greetings) {
            Some(greeting) => req!(context.say(&greeting[..])),
            None => req!(context.reply("No greeting found")),
        };
    }

    pub fn mfw(&self, context: Context) {
        let _msg = match thread_rng().choose(&self.emojis) {
            Some(emoji) => req!(context.say(&emoji[..])),
            None => req!(context.reply("No emoji found")),
        };
    }

    pub fn pi(&self, context: Context) {
        let mut pi = r#"
             1415926535897932384626433832795028841971693993751058209749445923078
             1640628620899862803482534211706798214808651328230664709384460955058
             2231725359408128481117450284102701938521105559644622948954930381964
             4288109756659334461284756482337867831652712019091456485669234603486
             1045432664821339360726024914127372458700660631558817488152092096282
             9254091715364367892590360011330530548820466521384146951941511609433
             0572703657595919530921861173819326117931051185480744623799627495673
             5188575272489122793818301194912983367336244065664308602139494639522
             4737190702179860943702770539217176293176752384674818467669405132000
             5681271452635608277857713427577896091736371787214684409012249534301
             4654958537105079227968925892354201995611212902196086403441815981362
             9774771309960518707211349999998372978049951059731732816096318595024
             4594553469083026425223082533446850352619311881710100031378387528865
             8753320838142061717766914730359825349042875546873115956286388235378
             75937519577818577805321712268066130019278766111959092164201989
             "#.replace(' ', "").replace("\n", "");

        let length = if let Ok(v) = context.arg(1).as_u64() {
            v as usize
        } else {
            let _msg = req!(context.say("Require a positive whole digit count"));

            return;
        };

        if length > 1000 {
            let _msg = req!(context.say("Maximum 1000 digits"));

            return;
        }

        pi.truncate(length);

        let _msg = req!(context.say(pi));
    }

    pub fn say(&self, context: Context) {
        let text = context.text(0);

        if text.is_empty() {
            return;
        }

        let _msg = req!(context.say(text));
    }

    pub fn uptime(&self, context: Context, uptime: &Uptime) {
        let boot = &uptime.boot.to_rfc3339()[..19];
        let connection = &uptime.boot.to_rfc3339()[..19];
        let text = format!(r#"```xl
            Booted: {} UTC
Current Connection: {} UTC```"#, boot, connection);

        let _msg = req!(context.say(text));
    }

    pub fn weather(&self, context: Context, state: &State) {
        let first_arg = context.arg(1);

        let save = if let Ok(arg) = first_arg.as_str() {
            arg == "save"
        } else {
            false
        };

        let full_text = context.text(0);

        let location_name = if !first_arg.exists() {
            let s = match state.find_channel(&context.message.channel_id) {
                Some(ChannelRef::Public(server, _channel)) => server,
                _ => {
                    let _msg = req!(context.say("Could not find server"));

                    return;
                },
            };

            let member_ = {
                let db = arc!(context.db);

                members.filter(server_id.eq(s.id.0 as i64))
                    .filter(user_id.eq(context.message.author.id.0 as i64))
                    .first::<Member>(&db)
            };

            match member_ {
                Ok(member_) => match member_.weather_location {
                    Some(location) => location,
                    None => {
                        let _msg = req!(context.say("
You do not have a location saved on this server!"));

                        return;
                    },
                },
                Err(diesel::NotFound) => {
                    let _msg = req!(context.say("Member data not found"));

                    return;
                },
                Err(why) => {
                    warn!("[weather] err getting user: {:?}", why);

                    let _msg = req!(context.say("Error getting member data"));

                    return;
                },
            }
        } else if save {
            context.text(1)
        } else {
            full_text
        };

        if location_name.is_empty() {
            let _msg = req!(context.say("No location name given"));
        }

        let location_data = match google_maps::get_address(location_name) {
            Ok(location_data) => location_data,
            Err(_why) => {
                let _msg = req!(context.say("Error retrieving location data"));

                return;
            },
        };

        let (lat, long, name) = match location_data.results.get(0) {
            Some(result) => (
                result.geometry.location.lat,
                result.geometry.location.lng,
                result.address_components.get(0).unwrap().long_name.clone(),
            ),
            None => {
                let _msg = req!(context.say("No results found for location"));

                return;
            },
        };

        let token = match env::var("FORECAST_TOKEN") {
            Ok(token) => token,
            Err(why) => {
                warn!("[weather] FORECAST_TOKEN not set: {:?}", why);

                let _msg = req!(context.say("Forecast data misconfigured"));

                return;
            },
        };

        let forecast = match forecast_io::get_forecast(token, lat, long) {
            Ok(forecast) => forecast,
            Err(why) => {
                warn!("[forecast] err getting forecast: {:?}", why);
                let _msg = req!(context.say("Could not retrieve the forecast"));

                return;
            },
        };

        let icon = match forecast.currently.icon {
            Icon::ClearDay => ":sunny:",
            Icon::ClearNight => ":night_with_stars:",
            Icon::Cloudy => ":cloudy:",
            Icon::Fog => ":foggy:",
            Icon::Hail | Icon::Sleet | Icon::Snow => ":cloud_snow:",
            Icon::PartlyCloudyDay => ":partly_sunny:",
            Icon::PartlyCloudyNight => ":cloud:",
            Icon::Rain => ":cloud_rain:",
            Icon::Thunderstorm => ":thunder_cloud_rain:",
            Icon::Tornado => ":cloud_tornado:",
            Icon::Wind => ":wind_blowing_face:",
        };
        let current_time = {
            let offset = (3600 * forecast.offset as i64) as i64;
            let timestamp = forecast.currently.time as i64 + offset;
            NaiveDateTime::from_timestamp(timestamp, 0)
                .format("%H:%M%p")
        };
        let temp_f = forecast.currently.temperature.floor();
        let temp_c = (((forecast.currently.temperature - 32f64) * 5f64) / 9f64)
            .floor() as i64;

        let text = format!(r#"{} **{}**
:clock1: {}
Currently: {}
{}C ({}F)
Rain: {}%"#, icon,
             name,
             current_time,
             forecast.currently.summary,
             temp_c,
             temp_f,
             forecast.currently.precip_probability);

        let _msg = req!(context.say(text));
    }
}
