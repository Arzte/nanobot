use rand::{self, Rng};
use serenity::client::Context;
use serenity::model::Message;

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

lazy_static! {
    static ref PI: String = r#"3.
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
}

#[derive(PartialEq)]
enum AestheticMode {
    Bold,
    Caps,
}

fn aestheticize(mut content: String, modifiers: Vec<AestheticMode>)
    -> Option<String> {
    if content.is_empty() {
        return None;
    }

    if modifiers.contains(&AestheticMode::Caps) {
        content = content.to_uppercase();
    }

    for chars in AESTHETIC_CHARS.iter() {
        content = content.replace(chars.0, &chars.1[..]);
    }

    content = content.replace(' ', "  ");

    if modifiers.contains(&AestheticMode::Bold) {
        content.insert(0, '*');
        content.insert(0, '*');
        content.push_str("**");
    }

    Some(content)
}

command!(aescaps(context, _message, args) {
    let modifiers = vec![AestheticMode::Bold, AestheticMode::Caps];

    if let Some(content) = aestheticize(args.join(" "), modifiers) {
        let _ = context.say(&content);
    }
});

command!(aes(context, _message, args) {
    if let Some(content) = aestheticize(args.join(" "), vec![]) {
        let _ = context.say(&content);
    }
});

command!(hello(context, _message, _args) {
    static GREETINGS: [&'static str; 3] = [
        "Hey!",
        "Selamat pagi",
        "G'day!",
    ];

    match rand::thread_rng().choose(&GREETINGS) {
        Some(greeting) => drop(context.say(greeting)),
        None => error!("No greeting found"),
    }
});

command!(mfw(context, _message, _args) {
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

    let _ = match rand::thread_rng().choose(&EMOJIS) {
        Some(emoji) => context.say(&format!(":{}:", emoji)),
        None => context.say("Emoji not found"),
    };
});

command!(pi(context, _message, args) {
    let length = match args.first().map(|x| x.parse::<usize>()) {
        Some(Ok(length)) => {
            if length <= 1000 {
                length + 2
            } else {
                let _ = context.say("Must be at most 1000");

                return Ok(());
            }
        },
        Some(Err(_why)) => {
            let _ = context.say("Must be a natural number");

            return Ok(());
        },
        None => 102,
    };

    let _ = context.say(&PI[..length]);
});
