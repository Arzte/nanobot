use rand::{self, Rng};
use ::prelude::*;

pub struct ChooseCommand;

impl Command for ChooseCommand {
    fn execute(&self, _: &mut Context, msg: &Message, args: Args) -> CommandResult {
        let query = args.full();
        let mut choices: Vec<&str> = query.split(", ").collect::<Vec<&str>>();

        if choices.len() < 2 {
            choices = query.split(' ').collect();
        }

        choices.sort();
        choices.dedup();

        if choices.len() < 2 {
            let _ = msg.channel_id.say("Must have at least 2 choices");

            return Ok(());
        }

        let _ = match rand::thread_rng().choose(&choices) {
            Some(choice) => msg.channel_id.say(&choice[..]),
            None => msg.channel_id.say("No choice found"),
        };

        Ok(())
    }
}

pub struct CoinflipCommand;

impl Command for CoinflipCommand {
    fn execute(&self, _: &mut Context, msg: &Message, _: Args) -> CommandResult {
        let num = rand::thread_rng().gen::<u8>();

        let _ = msg.channel_id.say(match num {
            0 ... 126 => "Heads",
            128 ... 255 => "Tails",
            _ => "On its side",
        });

        Ok(())
    }
}

pub struct MagicEightBallCommand;

impl Command for MagicEightBallCommand {
    fn execute(&self, _: &mut Context, msg: &Message, _: Args) -> CommandResult {
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
            Some(answer) => msg.channel_id.say(answer),
            None => msg.channel_id.say("No answer found"),
        };

        Ok(())
    }
}

pub struct RollCommand;

impl Command for RollCommand {
    fn execute(&self, _: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
        if !args.is_empty() && args.len() != 2 {
            let _ = msg.channel_id.say("Either 0 or 2 numbers must be given");

            return Ok(());
        }

        let nums = {
            if args.is_empty() {
                [1, 6]
            } else {
                let (arg1, arg2) = (args.single::<String>()?, args.single::<String>()?);

                let arg1 = match arg1.parse::<isize>() {
                    Ok(arg1) => arg1,
                    Err(_) => {
                        let _ = msg.channel_id.say(&format!("{} is not an integer", arg1));

                        return Ok(());
                    },
                };
                let arg2 = match arg2.parse::<isize>() {
                    Ok(arg2) => arg2,
                    Err(_) => {
                        let _ = msg.channel_id.say(&format!("{} is not an integer", arg2));

                        return Ok(());
                    },
                };

                let mut nums = vec![arg1, arg2];
                nums.sort();

                [nums[0], nums[1]]
            }
        };

        if nums[0] == nums[1] {
            let _ = msg.channel_id.say("The given integers can not be equal");

            return Ok(());
        }

        let number = rand::thread_rng().gen_range(nums[0], nums[1]);

        let _ = msg.channel_id.say(&number.to_string());

        Ok(())
    }
}

pub struct RouletteCommand;

impl Command for RouletteCommand {
    fn execute(&self, _: &mut Context, msg: &Message, _: Args) -> CommandResult {
        let result = if rand::thread_rng().gen_range(0, 6) == 0 {
            format!("BANG! {} was shot", msg.author)
        } else {
            r"\*click\*".to_owned()
        };

        let _ = msg.channel_id.say(&result);

        Ok(())
    }
}
