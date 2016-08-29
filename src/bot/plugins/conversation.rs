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
