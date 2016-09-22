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

use hummingbird::{self, ShowType};
use ::prelude::*;

pub fn anime(context: Context) {
    enabled!(Available, context);
    enabled!(AnimeAvailable, context);

    let text = context.text(0);

    if text.is_empty() {
        let _msg = context.say("A name must be given");

        return;
    }

    let msg = req!(context.say(format!("Searching for '{}'...", text)));

    let animes = match hummingbird::search_anime(&text[..]) {
        Ok(animes) => animes,
        Err(why) => {
            warn!("[anime] Err getting '{}': {:?}", text, why);

            let _msg = req!(context.edit(&msg, "Error retrieving anime"));

            return;
        },
    };

    if animes.is_empty() {
        let _msg = context.edit(&msg, "No result found");

        return;
    }

    let anime = animes.iter()
        .take(3)
        .find(|anime| anime.kind == ShowType::TV)
        .unwrap_or(unsafe { animes.get_unchecked(0) });
    let started = anime.started_airing
        .as_ref()
        .map_or("N/A", |v| &v[..]);
    let finished = anime.finished_airing
        .as_ref()
        .map_or("N/A", |v| &v[..]);
    let rating_str = anime.community_rating.to_string();
    let rating = if rating_str.len() < 3 {
        &rating_str[..]
    } else {
        &rating_str[..3]
    };

    let info = format!(r#"**{}**
Hummingbird: {}
Airing from __{}__ to __{}__
Score: {}/5
{} | {} | Eps: {}"#, anime.title,
                     anime.url,
                     started,
                     finished,
                     rating,
                     anime.kind.name(),
                     anime.status.name(),
                     anime.episode_count);

    let _msg = context.edit(&msg, info);
}
