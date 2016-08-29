use discord::model::permissions;
use discord::{ChannelRef, GetMessages, State};
use ::prelude::*;

pub struct Admin;

impl Admin {
    pub fn new() -> Admin {
        Admin
    }

    pub fn purge(&self, context: Context, state: &State) {
        if !context.arg(1).exists() {
            let _ = req!(context.say("Must provide message count to delete"));

            return;
        }

        // Check that the person has the 'MANAGE_MESSAGES' permission
        let server = match state.find_channel(&context.message.channel_id) {
            Some(ChannelRef::Public(server, _channel)) => server,
            _ => {
                let _msg = req!(context.say("Could not find server"));

                return;
            },
        };

        let member_perms = server.permissions_for(context.message.channel_id,
                                                  context.message.author.id);

        if member_perms.contains(permissions::MANAGE_MESSAGES) {
            let _msg = req!(context.say("You must be allowed to manage messages to be able to use this command"));

            return;
        }

        let amount = req!(context.arg(1).as_u64());

        if amount > 100 {
            let _msg = req!(context.say("Can only purge 100 messages"));

            return;
        }

        if amount < 2 {
            let _msg = req!(context.say("Must purge at least 2 messages"));

            return;
        }

        let discord = context.discord.lock().unwrap();
        let messages = match discord.get_messages(
            context.message.channel_id,
            GetMessages::Before(context.message.id),
            Some(amount)
        ) {
            Ok(messages) => messages,
            Err(why) => {
                let text = format!("Error getting messages: {:?}", why);
                let _msg = req!(context.say(text));

                return;
            },
        };

        let mut message_ids = vec![];

        for message in messages {
            message_ids.push(message.id);
        }

        let deletion = discord.delete_messages(context.message.channel_id,
                                               &message_ids);

        if let Err(why) = deletion {
            let text = format!("Error deleting messages: {:?}", why);
            let _msg = req!(context.say(text));
        }
    }
}
