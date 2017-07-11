use urbandictionary::UrbanClient;

command!(udefine(_ctx, msg, args) {
    if args.is_empty() {
        let _ = msg.channel_id.say("No word given");

        return Ok(());
    }

    let mut msg = match msg.channel_id.say("Searching for definition...") {
        Ok(msg) => msg,
        Err(_) => return Ok(()),
    };

    let query = args.join(" ");

    let client = UrbanClient::new();

    let mut response = match client.definitions(&query[..]) {
        Ok(response) => response,
        Err(why) => {
            warn!("Err retrieving word '{}': {:?}", query, why);

            let _ = msg.channel_id.say("Error retrieving definition");

            return Ok(());
        },
    };

    let mut definition = match response.definitions.get_mut(0) {
        Some(definition) => definition,
        None => {
            let _ = msg.edit(|m| m.content("No definition found"));

            return Ok(());
        },
    };

    if definition.definition.len() > 2048 {
        definition.definition.truncate(2045);
        definition.definition.push_str("...");
    }

    let url = format!("https://www.urbandictionary.com/author.php?author={}",
                      definition.author);

    let _ = msg.edit(|m| m
        .embed(|e| e
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
                .value(&definition.thumbs_down.to_string()))));
});
