use kitsu_io::model::AnimeType;
use kitsu_io::KitsuReqwestRequester;
use reqwest::Client;
use serenity::utils::Colour;
use ::prelude::*;

pub struct AnimeCommand;

impl Command for AnimeCommand {
    fn execute(&self, _: &mut Context, msg: &Message, args: Args) -> CommandResult {
        if args.is_empty() {
            let _ = msg.channel_id.say("A name must be given");

            return Ok(());
        }

        let query = args.full();

        let mut msg = match msg.channel_id.say(&format!("Searching for '{}'...", query)) {
            Ok(msg) => msg,
            Err(_) => return Ok(()),
        };

        let client = Client::new();

        let mut series_list = match client.search_anime(|f| f.filter("text", &query[..])) {
            Ok(series_list) => series_list.data,
            Err(why) => {
                warn!("Err getting anime series '{}': {:?}", query, why);

                let _ = msg.edit(|m| m.content("Error retrieving listing"));

                return Ok(());
            },
        };

        if series_list.is_empty() {
            let _ = msg.edit(|m| m.content("No results found"));

            return Ok(());
        }

        let series = {
            let first = series_list.remove(0);

            if first.attributes.kind == AnimeType::TV {
                first.attributes
            } else {
                let series = series_list.into_iter()
                    .take(3)
                    .find(|series| series.attributes.kind == AnimeType::TV);

                match series {
                    Some(series) => series.attributes,
                    None => first.attributes,
                }
            }
        };

        let rating_str = series.average_rating.map_or_else(|| "N/A".to_owned(),
                                                        |x| x.to_string());
        let rating = if rating_str.len() < 3 {
            &rating_str[..]
        } else {
            &rating_str[..3]
        };

        let description = format!("[Kitsu link](https://kitsu.io/anime/{})", series.slug);
        let title = series.titles.en_jp.unwrap_or(series.titles.ja_jp.unwrap());
        let thumbnail = series.poster_image.original;
        let aired = &format!("{} - {}", series.start_date.unwrap_or_else(|| "N/A".to_owned()), &series.end_date.as_ref().map_or("N/A", |v| &v[..]));
        let episodes = series.episode_count.map_or_else(|| "N/A".to_owned(), |x| x.to_string());
        let series_type = match series.kind {
            AnimeType::Movie => "Movie",
            AnimeType::Music => "Music",
            AnimeType::ONA => "ONA",
            AnimeType::OVA => "OVA",
            AnimeType::Special => "Special",
            AnimeType::TV => "TV",
        };

        let _ = msg.edit(|m| m
            .embed(move |mut e| {
                e = e.title(&title)
                    .description(&description)
                    .colour(Colour::fabled_pink())
                    .field("Aired", aired, true)
                    .field("Rating", rating, true)
                    .field("Type", series_type, true)
                    .field("Episodes", &episodes, true);

                if let Some(ref thumbnail) = thumbnail {
                    e = e.thumbnail(thumbnail);
                }

                e
            }));

        Ok(())
    }
}
