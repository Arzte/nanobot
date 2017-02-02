use psutil;
use serenity::client::CACHE;
use std::collections::BTreeMap;
use std::fmt::Write as FmtWrite;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::process::{Command, Stdio};
use std::env;
use ::store::{CommandCounter, EventCounter};

command!(commands(context, _message, _args) {
    let list = {
        let mut s = "Commands used:\n".to_owned();

        let data = context.data.lock().unwrap();
        let counter = data.get::<CommandCounter>().unwrap();

        for (k, v) in counter.iter().collect::<BTreeMap<_, _>>() {
            let _ = write!(s, "- {name}: {amount}\n", name=k, amount=v);
        }

        s
    };

    let _ = context.say(&list);
});

command!(eval(context, message, args) {
    let query = args.join(" ");

    let s = {
        let mut runnable = match File::open("./runnable.rs") {
            Ok(runnable) => runnable,
            Err(_) => {
                let _ = context.say("Err opening runnable");

                return Ok(());
            },
        };

        let mut s = String::new();
        let _ = runnable.read_to_string(&mut s);


        s = s.replace("{CHANNEL_ID}", &message.channel_id.0.to_string())
            .replace("{CODE}", &query);

        s
    };

    let id = message.id.0.to_string();

    {
        let mut f = File::create(&id).expect("err creating runnable");

        let _ = f.write(s.as_bytes());
    }

    let path = format!("./out_{}", id);

    {
        let command = Command::new("rustc")
            .arg(&id)
            .arg("--crate-name")
            .arg("runner")
            .arg("-L")
            .arg("target/release/deps")
            .arg("-o")
            .arg(&path)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .output();

        match command {
            Ok(output) => {
                info!("out {:?}", output);
                if !output.stderr.is_empty() {
                    let mut s = String::from_utf8_lossy(&output.stderr).into_owned();
                    s.truncate(500);

                    let _ = context.say(&format!("Error running rustc:
```
{}
```", s));

                    return Ok(());
                }

                info!("end out");
            },
            Err(why) => {
                let _ = context.say(&format!("Error running rustc: {:?}", why));

                return Ok(());
            },
        };
    }

    info!("c");

    match Command::new(&path).stdout(Stdio::piped()).stderr(Stdio::piped()).output() {
        Ok(output) => {
            let mut out = String::from_utf8_lossy(&output.stdout).into_owned();
            out.truncate(2000 - query.len() - 100);

            let _ = context.say(&format!("
**Exit status**: {}
**In**:
```rs
{}
```
**Out**:
```rs
{}
```", output.status.code().unwrap_or(1), query, out));
        },
        Err(why) => {
            let _ = context.say(&format!("Err running program: {:?}", why));
        },
    }

    let _ = fs::remove_file(id);
    let _ = fs::remove_file(path);
});

command!(events(context) {
    let list = {
        let mut s = "Events received:\n".to_owned();

        let data = context.data.lock().unwrap();
        let counter = data.get::<EventCounter>().unwrap();

        for (k, v) in counter.iter().collect::<BTreeMap<_, _>>() {
            let _ = write!(s, "- {name}: {amount}\n", name=k, amount=v);
        }

        s
    };

    let _ = context.say(&list);
});

command!(set_name(context, message, args) {
    if args.is_empty() {
        let _ = message.reply("No name given");

        return Ok(());
    }

    let name = args.join(" ");

    let _ = match context.edit_profile(|p| p.username(&name)) {
        Ok(_) => message.reply(":ok_hand:"),
        Err(why) => {
            warn!("Err setting name: {:?}", why);

            message.reply(":x: Error editing name")
        },
    };
});

command!(set_status(context, message, args) {
    let author_id = match env::var("AUTHOR_ID").map(|x| x.parse::<u64>()) {
        Ok(Ok(author_id)) => author_id,
        _ => {
            error!("Valid AUTHOR_ID env var not set");

            return Ok(());
        },
    };

    if message.author.id != author_id {
        return Ok(());
    }

    context.set_game_name(&args.join(" "));
});

command!(stats(ctx) {
    let processes = match psutil::process::all() {
        Ok(processes) => processes,
        Err(why) => {
            println!("Err getting processes: {:?}", why);

            let _ = ctx.say("Error getting stats");

            return Ok(());
        },
    };

    let process = match processes.iter().find(|p| p.pid == psutil::getpid()) {
        Some(process) => process,
        None => {
            let _ = ctx.say("Error getting stats");

            return Ok(());
        },
    };

    let memory = match process.memory() {
        Ok(memory) => memory,
        Err(why) => {
            println!("Err getting process memory: {:?}", why);

            let _ = ctx.say("Error getting stats");

            return Ok(());
        },
    };

    const B_TO_MB: u64 = 1024 * 1024;

    let mem_total = memory.size / B_TO_MB;
    let mem_rss = memory.resident / B_TO_MB;
    let memory = format!("{}MB/{}MB (RSS/Total)", mem_rss, mem_total);
    let guilds = CACHE.read().unwrap().guilds.len();

    let _ = ctx.send_message(|m|
        m.embed(|e| e
            .title("Stats")
            .field(|f| f.name("Version").value("0.1.0"))
            .field(|f| f.name("Guilds").value(&guilds.to_string()))
            .field(|f| f.name("Memory Used").value(&memory))));
});
