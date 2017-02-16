use rand::{self, Rng};

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
        141592653589793238462643383279502884197169399375105820974944592307816406
        286208998628034825342117067982148086513282306647093844609550582231725359
        408128481117450284102701938521105559644622948954930381964428810975665933
        446128475648233786783165271201909145648566923460348610454326648213393607
        260249141273724587006606315588174881520920962829254091715364367892590360
        011330530548820466521384146951941511609433057270365759591953092186117381
        932611793105118548074462379962749567351885752724891227938183011949129833
        673362440656643086021394946395224737190702179860943702770539217176293176
        752384674818467669405132000568127145263560827785771342757789609173637178
        721468440901224953430146549585371050792279689258923542019956112129021960
        864034418159813629774771309960518707211349999998372978049951059731732816
        096318595024459455346908302642522308253344685035261931188171010003137838
        752886587533208381420617177669147303598253490428755468731159562863882353
        7875937519577818577805321712268066130019278766111959092164201989"#
        .replace(' ', "").replace("\n", "");
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

command!(aescaps(ctx, _msg, args) {
    let modifiers = vec![AestheticMode::Bold, AestheticMode::Caps];

    if let Some(content) = aestheticize(args.join(" "), modifiers) {
        let _ = ctx.say(&content);
    }
});

command!(aes(ctx, _msg, args) {
    if let Some(content) = aestheticize(args.join(" "), vec![]) {
        let _ = ctx.say(&content);
    }
});

command!(hello(ctx) {
    static GREETINGS: [&'static str; 3] = [
        "Hey!",
        "Selamat pagi",
        "G'day!",
    ];

    match rand::thread_rng().choose(&GREETINGS) {
        Some(greeting) => drop(ctx.say(greeting)),
        None => error!("No greeting found"),
    }
});

command!(mfw(ctx) {
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
        Some(emoji) => ctx.say(&format!(":{}:", emoji)),
        None => ctx.say("Emoji not found"),
    };
});

command!(pi(ctx, _msg, args) {
    let length = match args.first().map(|x| x.parse::<usize>()) {
        Some(Ok(length)) => {
            if length <= 1000 {
                length + 2
            } else {
                let _ = ctx.say("Must be at most 1000");

                return Ok(());
            }
        },
        Some(Err(_why)) => {
            let _ = ctx.say("Must be a natural number");

            return Ok(());
        },
        None => 102,
    };

    let _ = ctx.say(&PI[..length]);
});
