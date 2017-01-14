use urbandictionary;

command!(udefine(context, _message, args) {
    if args.is_empty() {
        let _ = context.say("No word given");

        return Ok(());
    }

    let mut msg = match context.say("Searching for definition...") {
        Ok(msg) => msg,
        Err(_) => return Ok(()),
    };

    let query = args.join(" ");

    let mut response = match urbandictionary::definitions(&query[..]) {
        Ok(response) => response,
        Err(why) => {
            warn!("Err retrieving word '{}': {:?}", query, why);

            let _ = context.say("Error retrieving definition");

            return Ok(());
        },
    };

    let mut definition = match response.definitions.get_mut(0) {
        Some(definition) => definition,
        None => {
            let _ = msg.edit("No definition found", |e| e);

            return Ok(());
        },
    };

    if definition.definition.len() > 2048 {
        definition.definition.truncate(2045);
        definition.definition.push_str("...");
    }

    let url = format!("https://www.urbandictionary.com/author.php?author={}",
                      definition.author);

    let _ = msg.edit("", |e| e
        .title(&format!("Definition for **{}**", definition.word))
        .description(&definition.definition)
        .colour(0x1D2439)
        .author(|a| a
            .name(&definition.author)
            .url(&url.replace(' ', "%20")))
        .field(|f| f
            .name("Permalink")
            .value(&format!("[#{}]({})", definition.id, definition.permalink)))
        .field(|f| f
            .name(":+1:")
            .value(&definition.thumbs_up.to_string()))
        .field(|f| f
            .name(":-1:")
            .value(&definition.thumbs_down.to_string())));
});
