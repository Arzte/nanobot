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

use discord::model::{MessageId, permissions};
use discord::{ChannelRef, GetMessages};
use ::prelude::*;

pub fn purge(context: Context) {
    // Check that the person has the 'MANAGE_MESSAGES' permission
    let state = context.state.lock().unwrap();
    let member_perms = match state.find_channel(&context.message.channel_id) {
        Some(ChannelRef::Public(server, _channel)) => {
            server.permissions_for(context.message.channel_id,
                                   context.message.author.id)
        },
        _ => {
            let _msg = req!(context.say("Could not find server"));

            return;
        },
    };
    drop(state);

    if !member_perms.contains(permissions::MANAGE_MESSAGES) {
        let _msg = req!(context.say("You must be allowed to manage messages to be able to use this command"));

        return;
    }

    let location = req!(get_location(&context));

    let amount = context.arg(1)
        .as_u64()
        .ok()
        .or_else(|| PurgeDefault::find(location)
            .as_u64()
            .ok());

    let amount = match amount {
        Some(amount) => amount,
        None => {
            let _msg = context.say("Error: No amount given");

            return;
        },
    };

    if PurgeAvailable::find(location).disabled() {
        return;
    }

    let max = req!(PurgeMaximum::find(location).as_u64());

    if amount > max {
        let _msg = context.say(format!("Error: Can only purge {} messages",
                                       max));

        return;
    }

    let min = req!(PurgeMinimum::find(location).as_u64());

    if amount < min {
        let _msg = context.say(format!("Error: Must purge at least {} messages",
                                       min));

        return;
    }

    let discord = ::DISCORD.lock().unwrap();

    let messages = match discord.get_messages(
        context.message.channel_id,
        GetMessages::Before(context.message.id),
        Some(amount)
    ) {
        Ok(messages) => messages,
        Err(why) => {
            warn!("[purge] Error getting messages for {}: {:?}",
                  context.message.channel_id,
                  why);

            let _msg = context.say("Error: Error retrieving messages to purge");

            return;
        },
    };

    let message_ids: Vec<MessageId> = messages.iter()
        .map(|message| message.id)
        .collect();

    let deletion = discord.delete_messages(context.message.channel_id,
                                           &message_ids);

    drop(discord);

    if let Err(why) = deletion {
        warn!("[purge] Err deleting messages: {:?}", why);

        let _msg = context.say("Error: Error while deleting messages");
    }
}
