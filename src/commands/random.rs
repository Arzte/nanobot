use rand::{self, Rng};

command!(choose(context, _message, args) {
    let query = args.join(" ");
    let mut choices: Vec<&str> = query.split(", ").collect::<Vec<&str>>();

    if choices.len() < 2 {
        choices = query.split(' ').collect();
    }

    choices.sort();
    choices.dedup();

    if choices.len() < 2 {
        let _ = context.say("Must have at least 2 choices");

        return Ok(());
    }

    let _ = match rand::thread_rng().choose(&choices) {
        Some(choice) => context.say(&choice[..]),
        None => context.say("No choice found"),
    };
});

command!(coinflip(context, _message, _args) {
    let num = rand::thread_rng().gen::<u8>();

    let _ = context.say(match num {
        0 ... 126 => "Heads",
        128 ... 255 => "Tails",
        _ => "On its side",
    });
});

command!(magic_eight_ball(context, _message, _args) {
    let answers: [&'static str; 14] = [
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

    let _ = match rand::thread_rng().choose(&answers) {
        Some(answer) => context.say(&answer),
        None => context.say("No answer found"),
    };
});

command!(roll(context, _message, args) {
    if !args.is_empty() && args.len() != 2 {
        let _ = context.say("Either 0 or 2 numbers must be given");

        return Ok(());
    }

    let nums = {
        if args.is_empty() {
            [1, 6]
        } else {
            let (arg1, arg2) = unsafe {
                (args.get_unchecked(0), args.get_unchecked(1))
            };

            let arg1 = match arg1.parse::<isize>() {
                Ok(arg1) => arg1,
                Err(_) => {
                    let _ = context.say(&format!("{} is not an integer", arg1));

                    return Ok(());
                },
            };
            let arg2 = match arg2.parse::<isize>() {
                Ok(arg2) => arg2,
                Err(_) => {
                    let _ = context.say(&format!("{} is not an integer", arg2));

                    return Ok(());
                },
            };

            let mut nums = vec![arg1, arg2];
            nums.sort();

            [nums[0], nums[1]]
        }
    };

    if nums[0] == nums[1] {
        let _ = context.say("The given integers can not be equal");

        return Ok(());
    }

    let number = rand::thread_rng().gen_range(nums[0], nums[1]);

    let _ = context.say(&number.to_string());
});

command!(roulette(context, message, _args) {
    let result = if rand::thread_rng().gen_range(0, 6) == 0 {
        format!("BANG! {} was shot", message.author)
    } else {
        r"\*click\*".to_owned()
    };

    let _ = context.say(&result);
});