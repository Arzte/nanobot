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

use chrono::NaiveDateTime;
use discord::ChannelRef;
use forecast_io::{self, Icon, Unit};
use rand::{Rng, thread_rng};
use std::ascii::AsciiExt;
use std::{char, env, str};
use ::ext::google_maps;
use ::prelude::*;

static EMOJIS: [&'static str; 32] = [
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
];

static AESTHETIC_CHARS: [(char, &'static str); 58] = [
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
];

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
enum Aesthetic {
    Bold,
    Caps,
}

fn aestheticize(context: Context, modifiers: Vec<Aesthetic>) {
    let mut text = context.text(0);

    if text.is_empty() {
        let _msg = req!(context.say("Nothing to aestheticize"));

        return;
    }

    if modifiers.contains(&Aesthetic::Caps) {
        text.make_ascii_uppercase();
    }

    for chars in AESTHETIC_CHARS.iter() {
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

pub fn aescaps(context: Context) {
    if AesCapsAvailable::find(req!(get_location(&context))).disabled() {
        return;
    }

    aestheticize(context, vec![Aesthetic::Bold, Aesthetic::Caps])
}

pub fn aestheticcaps(context: Context) {
    if AestheticCapsAvailable::find(req!(get_location(&context))).disabled() {
        return;
    }

    aestheticize(context, vec![Aesthetic::Bold, Aesthetic::Caps])
}

pub fn aesthetic(context: Context) {
    if AestheticAvailable::find(req!(get_location(&context))).disabled() {
        return;
    }

    aestheticize(context, vec![])
}

pub fn aes(context: Context) {
    if AesAvailable::find(req!(get_location(&context))).disabled() {
        return;
    }

    aestheticize(context, vec![])
}

pub fn hello(context: Context) {
    if HelloAvailable::find(req!(get_location(&context))).disabled() {
        return;
    }

    let user = if let Some(ref mention) = context.message.mentions.get(0) {
        &mention.name
    } else {
        &context.message.author.name
    };

    let greetings = vec![
        format!("Hello {}", user),
        format!("Hey {}!", user),
        "Hello fella!".to_owned(),
        "Hey fella!".to_owned(),
        format!("What's up {}?", user),
        format!("Selamat pagi, {}", user),
        format!("G'day {}!", user),
    ];

    let _msg = match thread_rng().choose(&greetings) {
        Some(greeting) => req!(context.say(&greeting[..])),
        None => req!(context.reply("No greeting found")),
    };
}

pub fn mfw(context: Context) {
    if MfwAvailable::find(req!(get_location(&context))).disabled() {
        return;
    }

    let _msg = match thread_rng().choose(&EMOJIS) {
        Some(emoji) => req!(context.say(&format!(":{}:", emoji)[..])),
        None => req!(context.reply("No emoji found")),
    };
}

pub fn pi(context: Context) {
    let location = req!(get_location(&context));

    if PiAvailable::find(location).disabled() {
        return;
    }

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

    let length = {
        let arg = context.arg(1);

        if let Ok(v) = arg.as_u64() {
            v as usize
        } else if arg.exists() {

            let _msg = req!(context.say("Requires a positive whole number"));

            return;
        } else {
            let default = PiPrecisionDefault::get(location);

            req!(default.as_i64()) as usize
        }
    };

    let max = PiPrecisionMaximum::get(location);
    let max = req!(max.as_i64()) as usize;

    if length > max {
        let _msg = req!(context.say(format!("Maximum {} digits", max)));

        return;
    }

    pi.truncate(length);

    pi.insert(0, '.');
    pi.insert(0, '3');

    let _msg = req!(context.say(pi));
}

pub fn uptime(context: Context) {
    let text = {
        let uptime = ::UPTIME.lock().unwrap();
        let boot = &uptime.boot.to_rfc3339()[..19];
        let connection = &uptime.boot.to_rfc3339()[..19];

        format!(r#"```xl
            Booted: {} UTC
Current Connection: {} UTC```"#, boot, connection)
    };

    let _msg = req!(context.say(text));
}

pub fn weather(context: Context) {
    if WeatherAvailable::find(req!(get_location(&context))).disabled() {
        return;
    }

    let first_arg = context.arg(1);

    let save = if let Ok(arg) = first_arg.as_str() {
        arg == "save"
    } else {
        false
    };

    let full_text = context.text(0);

    let location_name = if !first_arg.exists() {
        let state = context.state.lock().unwrap();
        let server_id = match state.find_channel(&context.message.channel_id) {
            Some(ChannelRef::Public(server, _channel)) => server.id,
            _ => {
                let _msg = req!(context.say("Could not find server"));

                return;
            },
        };
        drop(state);

        let db = ::DB.lock().unwrap();
        let retrieval = db.query(
            "select weather_location from members where server_id = $1 and user_id = $2",
            &[&(server_id.0 as i64), &(context.message.author.id.0 as i64)]
        );

        match retrieval {
            Ok(ref rows) if !rows.is_empty() => {
                let member = rows.get(0);

                match member.get(0) {
                    Some(location) => {
                        let location: String = location;
                        location.clone()
                    },
                    None => {
                        let _msg = req!(context.say("You do not have a location saved on this server!"));

                        return;
                    },
                }
            },
            Ok(_rows) => {
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

    let msg = req!(context.say("Retrieving the forecast..."));

    let location_data = match google_maps::get_address(location_name) {
        Ok(location_data) => location_data,
        Err(_why) => {
            let _msg = req!(context.edit(&msg, "Error retrieving location data"));

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
            let _msg = req!(context.edit(&msg, "No results found for location"));

            return;
        },
    };

    let token = match env::var("FORECAST_TOKEN") {
        Ok(token) => token,
        Err(why) => {
            warn!("[weather] FORECAST_TOKEN not set: {:?}", why);

            let _msg = req!(context.edit(&msg, "Forecast data misconfigured"));

            return;
        },
    };

    let res = forecast_io::get_forecast_with_options(token, lat, long, |o| {
        o.unit(Unit::Si)
    });

    let forecast = match res {
        Ok(forecast) => forecast,
        Err(why) => {
            warn!("[forecast] Err getting forecast: {:?}", why);
            let _msg = req!(context.edit(&msg, "Could not retrieve the forecast"));

            return;
        },
    };

    let currently = match forecast.currently {
        Some(currently) => currently,
        None => {
            let _msg = req!(context.edit(&msg, "Could not retrieve the forecast"));

            return;
        },
    };

    let icon = match currently.icon {
        Some(icon) => match icon {
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
        },
        None => "N/A",
    };
    let current_time = {
        if let Some(offset) = forecast.offset {
            println!("{:?} {:?}", currently.time, offset);
            let timestamp = currently.time as i64 + (offset as i64 * 3600);

            NaiveDateTime::from_timestamp(timestamp, 0)
                .format("%I:%M%p")
                .to_string()
        } else {
            "N/A".to_owned()
        }
    };
    let temp = {
        if let Some(temp_c) = currently.temperature {
            let temp_f = (((temp_c * 9f64) / 5f64) + 32f64) as i16;

            format!("{}C ({}F)", temp_c as i16, temp_f)
        } else {
            "N/A".to_owned()
        }
    };
    let probability = currently.precip_probability
        .map_or(0u8, |v| v as u8);

    let text = format!(r#"{} **{}**
:clock1: {}
Currently: {}
{}
Rain: {}%"#, icon,
             name,
             current_time,
             currently.summary.unwrap_or("No summary available".to_owned()),
             temp,
             probability);

    let _msg = req!(context.edit(&msg, text));
}
