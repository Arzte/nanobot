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

use discord::model::User;
use discord::ChannelRef;
use ::ext::commands::context::ContextArg;
use ::prelude::*;

pub fn find_user(context: &Context, text: ContextArg) -> Result<User> {
    if let Some(user) = context.message.mentions.get(0) {
        Ok(user.clone())
    } else if let Ok(info) = text.as_str() {
        let (name, discriminator) = if let Some(pos) = info.find('#') {
            let split = info.split_at(pos);

            let discrim = match split.1.replace("#", "").parse::<u16>() {
                Ok(discrim) => discrim,
                Err(_why) => {
                    return Err(Error::FindingUser);
                },
            };

            (split.0, Some(discrim))
        } else {
            (info, None)
        };

        let state = context.state.lock().unwrap();
        let server = match state.find_channel(&context.message.channel_id) {
            Some(ChannelRef::Public(server, _channel)) => server,
            _ => {
                return Err(Error::FindingUser);
            },
        };

        let mut member_found = None;

        for member in &server.members {
            if if let Some(discrim) = discriminator {
                member.user.discriminator == discrim && member.user.name == name
            } else {
                member.user.name == name
            } {
                member_found = Some(member.clone());

                break;
            }
        }

        if let Some(member) = member_found {
            Ok(member.user.clone())
        } else {
            return Err(Error::FindingUser);
        }
    } else {
        Ok(context.message.author.clone())
    }
}
