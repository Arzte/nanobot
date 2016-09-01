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

use urbandictionary;
use ::prelude::*;

pub struct Conversation;

impl Conversation {
    pub fn new() -> Conversation {
        Conversation
    }

    pub fn define(&self, context: Context) {
        let mut definition = match urbandictionary::define(&context.text(0)[..]) {
            Ok(Some(definition)) => definition,
            Ok(None) => {
                let _msg = req!(context.say("No definition found"));

                return;
            },
            Err(why) => {
                warn!("[define] Err retrieving {}: {}", &context.text(0), why);

                let _msg = req!(context.say("Error retrieving definition"));

                return;
            },
        };

        let define = if definition.definition.len() > 1600 {
            format!("{}...", &definition.definition[..1600])
        } else {
            definition.definition.clone()
        };

        definition.example.truncate(1900 - define.len());

        let text = format!(r#"**{}**
{}

Example: _{}_"#, definition.word,
                 define,
                 definition.example);

        let _msg = req!(context.say(text));
    }

    /*
    pub fn q(&mut self, context: Context) {
        let text = context.text(0);

        if text.is_empty() {
            let _msg = req!(context.say("Input required"));

            return Ok(());
        }

        let _msg = req!(context.say(match self.cleverbot.say(&text) {
            Ok(response) => response,
            Err(why) => {
                warn!("[q] error retrieving response: {:?}", why);

                let _msg = req!(context.say("Error generating response"));

                return Ok(());
            },
        }));

        Ok(())
    }
    */
}
