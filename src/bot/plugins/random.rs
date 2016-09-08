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

use rand::{Rng, thread_rng};
use ::prelude::*;

pub fn choose(context: Context) {
    if ChooseAvailable::find(req!(get_location(&context))).disabled() {
        return;
    }

    let text = context.text(0);

    let mut choices: Vec<&str> = text.split(", ").collect();

    if choices.len() < 2 {
        choices = text.split(' ').collect();
    }

    let _msg = match thread_rng().choose(&choices) {
        Some(choice) => req!(context.say(&choice[..])),
        None => req!(context.reply("No choice found")),
    };
}

pub fn coinflip(context: Context) {
    if CoinflipAvailable::find(req!(get_location(&context))).disabled() {
        return;
    }

    let num = thread_rng().gen::<u8>();

    let _msg = req!(context.say(if num < 127 {
        "Heads"
    } else if num > 127 {
        "Tails"
    } else {
        "On its side"
    }));
}

pub fn magic_eight_ball(context: Context) {
    if MagicEightBallAvailable::find(req!(get_location(&context))).disabled() {
        return;
    }

    let answers = [
        // positive
        "It is certain",
        "Most likely",
        "Outlook good",
        "Without a doubt",
        "Yes",
        "You may rely on it",
        // neutral
        "Better not tell you now",
        "Reply hazy, try again",
        // negative
        "Absolutely not",
        "Don't count on it",
        "My reply is no",
        "My sources say no",
        "Outlook not so good",
        "Very doubtful",
    ];

    let _msg = match thread_rng().choose(&answers) {
        Some(answer) => req!(context.say(&answer[..])),
        None => req!(context.reply("No answer found")),
    };
}

pub fn roll(context: Context) {
    if RollAvailable::find(req!(get_location(&context))).disabled() {
        return;
    }

    let arg1 = context.arg(1);
    let arg2 = context.arg(2);

    let a1 = match arg1.as_isize() {
        Ok(v) => v,
        Err(_why) => {
            let text = format!("Error converting {} to an int", arg1);
            let _msg = req!(context.say(text));

            return;
        },
    };

    let a2 = match arg2.as_isize() {
        Ok(v) => v,
        Err(why) => {
            let _msg = req!(context.say(format!("{:?}", why)));

            return;
        },
    };

    let nums = vec![a1, a2];
    let min = match nums.iter().min() {
        Some(min) => *min,
        None => {
            let _msg = req!(context.say("Error generating min number"));

            return;
        },
    };
    let max = match nums.iter().max() {
        Some(max) => *max,
        None => {
            let _msg = req!(context.say("Error generating max number"));

            return;
        },
    };

    let random = thread_rng().gen_range(min, max);

    let _msg = req!(context.say(format!("{}", random)));
}

pub fn roulette(context: Context) {
    if RouletteAvailable::find(req!(get_location(&context))).disabled() {
        return;
    }

    let _msg = req!(context.say(if thread_rng().gen_range(0, 6) == 0 {
        format!("BANG! {} was shot", context.message.author.mention())
    } else {
        r#"\*click\*"#.to_owned()
    }));
}

pub fn teams(context: Context) {
    if TeamsAvailable::find(req!(get_location(&context))).disabled() {
        return;
    }

    let team_count = match context.arg(1).as_u64() {
        Ok(team_count) => team_count,
        Err(_why) => {
            let _msg = req!(context.say("Team count must be given"));

            return;
        },
    };

    let text = context.text(1);

    if text.is_empty() {
        let _msg = req!(context.say("No names given"));

        return;
    }

    let mut names: Vec<&str> = text.split(", ").collect();
    names.sort();
    names.dedup();

    let players_per_team = {
        let precise = (names.len() / team_count as usize) as f64;

        precise.floor() as u64
    };

    let mut teams: Vec<Vec<&str>> = vec![];
    let mut player_iter = 0;
    let mut team_iter = 0;

    for name in &names {
        let mut found = false;

        if let Some(team) = teams.get_mut(team_iter) {
            team.push(name);

            found = true;
        }

        if !found {
            teams.push(vec![name]);
        }

        player_iter += 1;

        if player_iter == players_per_team {
            player_iter = 0;
            team_iter += 1;
        }
    }

    let mut out = String::from("Teams:\n\n");

    for (pos, team) in teams.iter().enumerate() {
        out.push_str(&format!("{}. {}\n", pos + 1, team.join(", ")));
    }

    let _msg = req!(context.say(out));
}
