use hummingbird::{self, ShowType};
use serenity::utils::Colour;

command!(anime(context, _message, args) {
    if args.is_empty() {
        let _ = context.say("A name must be given");

        return Ok(());
    }

    let query = args.join(" ");

    let mut msg = match context.say(&format!("Searching for '{}'...", query)) {
        Ok(msg) => msg,
        Err(_) => return Ok(()),
    };

    let series_list = match hummingbird::search_anime(&query[..]) {
        Ok(series_list) => series_list,
        Err(why) => {
            warn!("Err getting anime series '{}': {:?}", query, why);

            let _ = msg.edit("Error retrieving listing", |e| e);

            return Ok(());
        },
    };

    if series_list.is_empty() {
        let _ = msg.edit("No results found", |e| e);

        return Ok(());
    }

    let series = series_list.iter()
        .take(3)
        .find(|series| series.kind == ShowType::TV)
        .unwrap_or(unsafe { series_list.get_unchecked(0) });
    let started = series.started_airing.as_ref().map_or("N/A", |v| &v[..]);
    let finished = series.finished_airing.as_ref().map_or("N/A", |v| &v[..]);
    let rating_str = series.community_rating.to_string();
    let rating = if rating_str.len() < 3 {
        &rating_str[..]
    } else {
        &rating_str[..3]
    };

    let _ = msg.edit("", |e| e
        .title(&series.title)
        .description(&format!("[Hummingbird link]({})", series.url))
        .thumbnail(&series.cover_image)
        .colour(Colour::fabled_pink())
        .field(|f| f
            .name("Aired")
            .value(&format!("{} - {}", &started, &finished)))
        .field(|f| f
            .name("Rating")
            .value(rating))
        .field(|f| f
            .name("Type")
            .value(series.kind.name()))
        .field(|f| f
            .name("Status")
            .value(series.status.name()))
        .field(|f| f
            .name("Episodes")
            .value(&series.episode_count.to_string())));
});
