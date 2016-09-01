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

use hummingbird::ShowType;
use hummingbird;
use ::prelude::*;

pub struct Tv;

impl Tv {
    pub fn new() -> Tv {
        Tv
    }

    pub fn anime(&self, context: Context) {
        let text = context.text(0);

        if text.is_empty() {
            let _msg = req!(context.say("A name must be given"));

            return;
        }

        let msg = req!(context.say(format!("Searching for '{}'...", text)));

        let animes = match hummingbird::search_anime(&text[..]) {
            Ok(animes) => animes,
            Err(why) => {
                warn!("[anime] err getting {}: {:?}", text, why);

                let _msg = req!(context.edit(&msg, "Error retrieving anime"));

                return;
            },
        };

        if animes.is_empty() {
            let _msg = req!(context.edit(&msg, "No result found"));

            return;
        }

        let found = animes.iter()
            .take(3)
            .find(|anime| anime.kind == ShowType::TV);

        let anime = if let Some(anime) = found {
            anime
        } else {
            // this is actually safe, we've already performed a check above
            unsafe {
                animes.get_unchecked(0)
            }
        };

        let started = match anime.started_airing {
            Some(ref time) => &time[..],
            None => "N/A",
        };
        let finished = match anime.finished_airing {
            Some(ref time) => &time[..],
            None => "N/A",
        };

        let info = format!(r#"**{}**
Hummingbird: {}
Airing from __{}__ to __{}__
Score: {}/5
{} | {} | Eps: {}"#, anime.title,
                     anime.url,
                     started,
                     finished,
                     anime.community_rating.round(),
                     anime.kind.name(),
                     anime.status.name(),
                     anime.episode_count);

        let _msg = req!(context.edit(&msg, info));
    }
}
