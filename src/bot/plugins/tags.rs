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

use chrono::{NaiveDateTime, UTC};
use discord::model::{ChannelId, permissions};
use discord::ChannelRef;
use ::prelude::*;

pub fn delete(context: Context) {
    enabled!(Available, context);
    enabled!(TagsAvailable, context);

    let key = context.text(0);

    if key.is_empty() {
        let _msg = req!(context.say("No tag given"));
    }

    let aid = context.message.author.id;
    let cid = ChannelId(context.message.channel_id.0 as u64);

    let state = context.state.lock().unwrap();
    let (server_id, perms) = match state.find_channel(&context.message.channel_id) {
        Some(ChannelRef::Public(server, _channel)) => {
            let perms = server.permissions_for(cid, aid)
                .contains(permissions::MANAGE_MESSAGES);

            (server.id, perms)
        },
        _ => {
            let _msg = req!(context.say("Could not find server"));

            return;
        },
    };
    drop(state);

    let db: PgConn = ::DB.lock().unwrap();

    let (tag_id, owner_id) = {
        let filter: PgRes = db.query(
            "select id, owner_id from tags where server_id = $1 and key = $2",
            &[&(server_id.0 as i64), &key]
        );

        match filter {
            Ok(ref rows) if !rows.is_empty() => {
                let tag = rows.get(0);
                let tag_id: i32 = tag.get(0);
                let owner_id: i64 = tag.get(1);

                (tag_id, owner_id)
            },
            Ok(_rows) => {
                let _msg = req!(context.say("Tag not found"));

                return;
            },
            Err(why) => {
                warn!("[delete] Err getting '{}' for {}: {:?}",
                      server_id,
                      key,
                      why);

                let _msg = req!(context.say("Error finding tag"));

                return;
            },
        }
    };

    // Check if the user does _not_ have permission tod elete this tag
    if !(owner_id == context.message.author.id.0 as i64 || perms) {
        drop(db);

        let _msg = req!(context.say("You do not have permission to delete this tag."));

        return;
    }

    let delete = db.execute("delete from tags where id = $1", &[&tag_id]);

    drop(db);

    let _msg = match delete {
        Ok(1) => req!(context.say("Deleted tag")),
        Ok(0) => req!(context.say("No tag deleted")),
        Ok(amount) => {
            warn!("[delete] Multiple deleted for '{}' in {}: {}",
                  server_id,
                  key,
                  amount);

            req!(context.say("Multiple tags deleted somehow"))
        },
        Err(why) => {
            warn!("[delete] Err deleting tag '{}' in {}: {:?}",
                  server_id,
                  key,
                  why);

            req!(context.say("Error deleting tag"))
        },
    };
}

pub fn get(context: Context) {
    enabled!(Available, context);
    enabled!(TagsAvailable, context);

    let mut name = None;

    {
        let text = context.text(0);
        let arg = context.arg(0);
        let arg0 = req!(arg.as_str());

        if !text.is_empty() && arg0 == "get" {
            name = Some(context.text(0));
        }
    }

    if name.is_none() {
        let arg = context.arg(0);
        let text = context.text(0);
        let mut fmt = String::new();
        fmt.push_str(req!(arg.as_str()));

        if !text.is_empty() {
            fmt.push(' ');
            fmt.push_str(&text[..]);
        }

        name = Some(fmt);
    }

    let key = match name {
        Some(key) => key,
        None => return,
    };

    let state = context.state.lock().unwrap();
    let server_id = match state.find_channel(&context.message.channel_id) {
        Some(ChannelRef::Public(server, _channel)) => server.id,
        _ => {
            let _msg = req!(context.say("Could not find server"));

            return;
        },
    };
    drop(state);

    let db: PgConn = ::DB.lock().unwrap();

    let (uses, value) = {
        let filter: PgRes = db.query(
            "select uses, value from tags where server_id = $1 and key = $2",
            &[&(server_id.0 as i64), &key]
        );

        match filter {
            Ok(ref rows) if !rows.is_empty() => {
                let tag = rows.get(0);
                let uses: i32 = tag.get(0);
                let value: String = tag.get(1);

                (uses, value)
            },
            Ok(_rows) => {
                return;
            },
            Err(why) => {
                warn!("[get] Err getting '{}' on {}: {:?}", server_id, key, why);

                return;
            },
        }
    };

    let update = db.execute(
        "update tags set uses = $1 where server_id = $2 and key = $3",
        &[&(uses + 1), &(server_id.0 as i64), &key]
    );

    drop(db);

    match update {
        Ok(_updated) => {},
        Err(why) => {
            warn!("Error incrementing tag uses: {:?}", why);
        },
    }

    let _msg = req!(context.say(value));
}

pub fn info(context: Context) {
    enabled!(Available, context);
    enabled!(TagsAvailable, context);

    let key = context.text(0);

    if key.is_empty() {
        let _msg = req!(context.say("No tag given"));

        return;
    }

    let state = context.state.lock().unwrap();
    let s = match state.find_channel(&context.message.channel_id) {
        Some(ChannelRef::Public(server, _channel)) => server.clone(),
        _ => {
            let _msg = req!(context.say("Could not find server"));
            return;
        },
    };
    drop(state);

    let db: PgConn = ::DB.lock().unwrap();

    let filter: PgRes = db.query(
        "select created_at, key, owner_id, uses from tags where server_id = $1 and key = $2",
        &[&(s.id.0 as i64), &key]
    );

    let tag = match filter {
        Ok(ref rows) if !rows.is_empty() => rows.get(0),
        Ok(_rows) => {
            let _msg = req!(context.say("Tag not found"));

            return;
        },
        Err(why) => {
            warn!("[info] Err querying for '{}' in {}: {:?}",
                  key,
                  s.id,
                  why);

            let _msg = req!(context.say("Error getting tag"));

            return;
        },
    };

    let mut owner = None;

    for member in &s.members {
        if member.user.id.0 as i64 == tag.get(2) {
            owner = Some(member);
        }
    }

    let owner_info = if let Some(owner) = owner {
        format!("{}#{}", owner.user.name, owner.user.discriminator)
    } else {
        "N/A".to_owned()
    };

    let timestamp = NaiveDateTime::from_timestamp(tag.get(0), 0)
        .format("%Y-%m-%d %H:%M:%S")
        .to_string();

    let key: String = tag.get(1);
    let uses: i32 = tag.get(3);

    let info = format!(r#"```xl
       Key: {}
     Owner: {}
Created at: {}
      Uses: {}```"#, key, owner_info, timestamp, uses);

    let _msg = req!(context.say(info));
}

pub fn list(context: Context) {
    enabled!(Available, context);
    enabled!(TagsAvailable, context);

    let state = context.state.lock().unwrap();
    let server_id = match state.find_channel(&context.message.channel_id) {
        Some(ChannelRef::Public(server, _channel)) => server.id,
        _ => {
            let _msg = req!(context.say("Could not find server"));
            return;
        },
    };
    drop(state);

    let db: PgConn = ::DB.lock().unwrap();

    let filter = db.query("select key from tags where server_id = $1",
                          &[&(server_id.0 as i64)]);

    let tag_list = match filter {
        Ok(rows) => rows,
        Err(why) => {
            warn!("[list] retrieving tag list: {:?}", why);

            let _msg = req!(context.say("Error generating list"));

            return;
        },
    };

    if tag_list.is_empty() {
        let _msg = req!(context.say("No tags"));

        return;
    }

    let mut alphabetized: Vec<String> = tag_list.iter()
        .map(|tag| tag.get(0))
        .collect();

    alphabetized.sort();

    let joiner: &str = ", ";
    let mut page_length = 0;
    let mut pages: Vec<String> = vec![];
    let mut page = 0;

    for name in &alphabetized {
        if page_length + name.len() + joiner.len() > 2000 {
            page += 1;
            page_length = 0;
        }

        page_length += name.len() + joiner.len();

        let mut found = false;

        if let Some(page) = pages.get_mut(page) {
            page.push_str(name);
            page.push_str(joiner);

            found = true;
        }

        if !found {
            pages.push(String::new());

            unsafe {
                pages.get_unchecked_mut(page)
                    .push_str(name);
            }
        }
    }

    let first_three: Vec<&String> = pages.iter()
        .take(3)
        .collect();

    if first_three.len() > 1 {
        let aid = context.message.author.id;

        let discord = ::DISCORD.lock().unwrap();
        let channel = match discord.create_private_channel(&aid) {
            Ok(channel) => channel,
            Err(why) => {
                warn!("[list] creating PM: {:?}", why);

                let _msg = req!(context.say("Error returning list"));

                return;
            },
        };

        drop(discord);

        for page in &first_three {
            let _msg = req!(context.message(channel.id, &page[..]));
        }
    } else {
        for page in &first_three {
            let _msg = req!(context.say(&page[..]));
        }
    }
}

pub fn rename(context: Context) {
    enabled!(Available, context);
    enabled!(TagsAvailable, context);

    let text = context.text(0);
    let pos = match text.find(" --> ") {
        Some(pos) => pos,
        None => {
            let _msg = req!(context.say("Requires an old and new key"));

            return;
        },
    };

    if text.len() <= pos + 5 {
        let _msg = req!(context.say("No new key found"));

        return;
    }

    let state = context.state.lock().unwrap();
    let server = match state.find_channel(&context.message.channel_id) {
        Some(ChannelRef::Public(server, _channel)) => server.clone(),
        _ => {
            let _msg = req!(context.say("Could not find server"));

            return;
        },
    };
    drop(state);

    let (key_old, key_new) = (&text[..pos], &text[pos + 5..]);

    if key_new.len() > 100 {
        let _msg = req!(context.say("Max key length is 100"));

        return;
    }

    let db: PgConn = ::DB.lock().unwrap();

    // Check that the tag currently exists
    let res: PgRes = db.query(
        "select owner_id from tags where server_id = $1 and key = $2",
        &[&(server.id.0 as i64), &key_old]
    );

    let tag_ = match res {
        Ok(ref rows) if !rows.is_empty() => rows.get(0),
        Ok(_) => {
            let _msg = req!(context.say("Tag not found"));

            return;
        },
        Err(_why) => {
            let _msg = req!(context.say("Tag does not exist"));

            return;
        },
    };

    // Check that a tag with the new name does not exist
    let exists: PgRes = db.query(
        "select id from tags where server_id = $1 and key = $2",
        &[&(server.id.0 as i64), &key_new]
    );

    match exists {
        Ok(ref rows) if rows.is_empty() => {},
        Ok(_rows) => {
            let _msg = req!(context.say("A tag exists with the new name"));

            return;
        },
        Err(why) => {
            warn!("[rename] Err retrieving old tag: {:?}", why);

            let _msg = req!(context.say("Error retrieving old tag"));

            return;
        },
    }

    // Check that the user can rename this tag. They can rename it if one of
    // the following is true:
    //
    // - they are the owner of the tag;
    // - they have the "MANAGE_MESSAGES" permission.
    let owner_id: i64 = tag_.get(0);

    let can_edit = if owner_id == context.message.author.id.0 as i64 {
        true
    } else {
        let aid = context.message.author.id;
        let cid = ChannelId(context.message.channel_id.0 as u64);

        server.permissions_for(cid, aid)
            .contains(permissions::MANAGE_MESSAGES)
    };

    if !can_edit {
        let _msg = req!(context.say("You do not have permission to rename this tag"));

        return;
    }

    // It's now safe to update the key
    let update = db.execute(
        "update tags set key = $1 where server_id = $2 and key = $3",
        &[&key_new, &(server.id.0 as i64), &key_old]
    );

    match update {
        Ok(1) => {
            let _msg = req!(context.say("Renamed tag"));
        },
        Ok(0) => {
            warn!("[rename] tried to rename nonexistent tag {:?}", key_old);

            let _msg = req!(context.say("Tag does not exist"));

            return;
        },
        Ok(amount) => {
            warn!("[rename] renamed multiple tags {:?}", amount);

            let _msg = req!(context.say("Renamed multiple tags (??!!@??)"));

            return;
        },
        Err(why) => {
            warn!("[rename] updating: {:?}", why);

            let _msg = req!(context.say("Error renaming tag"));
        },
    }
}

pub fn search(context: Context) {
    enabled!(Available, context);
    enabled!(TagsAvailable, context);

    let query = context.text(0);

    if query.is_empty() {
        let _msg = req!(context.say("No query given"));

        return;
    }

    let state = context.state.lock().unwrap();
    let server_id = match state.find_channel(&context.message.channel_id) {
        Some(ChannelRef::Public(server, _channel)) => server.id,
        _ => {
            let _msg = req!(context.say("Could not find server"));

            return;
        },
    };
    drop(state);

    let db: PgConn = ::DB.lock().unwrap();
    let search_res = db.query(
        "select key from tags where server_id = $1 and
         lower(key) ilike '%' || $2 || '%' limit 15",
        &[&(server_id.0 as i64), &query]
    );
    let tag_list: Rows = match search_res {
        Ok(tag_list) => tag_list,
        Err(why) => {
            warn!("[rename] retrieving tag list: {:?}", why);

            let _msg = req!(context.say("Error generating list"));

            return;
        },
    };

    if tag_list.is_empty() {
        let _msg = req!(context.say("No tags found"));

        return;
    }

    let key_list: Vec<String> = tag_list
        .iter()
        .map(|tag| tag.get(0))
        .collect();
    let listing = key_list.join(", ");

    let _msg = req!(context.say(listing));
}

pub fn set(context: Context) {
    enabled!(Available, context);
    enabled!(TagsAvailable, context);

    let text = context.text(0);
    let pos = match text.find(':') {
        Some(pos) => pos,
        None => {
            let _msg = req!(context.say("Requires both a key and a value"));

            return;
        },
    };

    if text.len() <= pos + 2 {
        let _msg = req!(context.say("Requires both a key and a value"));

        return;
    }

    let state = context.state.lock().unwrap();
    let server_id = match state.find_channel(&context.message.channel_id) {
        Some(ChannelRef::Public(server, _channel)) => server.id,
        _ => {
            let _msg = req!(context.say("Could not find server"));

            return;
        },
    };
    drop(state);

    let (key, value) = (&text[..pos], &text[pos + 2..]);

    if key.len() > 100 {
        let _msg = req!(context.say("Key max length is 100"));

        return;
    }

    let db: PgConn = ::DB.lock().unwrap();

    // Check if the tag exists already; we don't want to override it
    {
        let exists: PgRes = db.query(
            "select id from tags where server_id = $1 and key = $2",
            &[&(server_id.0 as i64), &key]
        );

        match exists {
            Ok(ref rows) if !rows.is_empty() => {
                let _msg = req!(context.say("Tag already exists"));

                return;
            },
            Ok(_) => {},
            Err(why) => {
                warn!("[set] Err connecting to db: {:?}", why);

                let _msg = req!(context.say("Error setting tag"));
            },
        }
    }

    let creation = db.execute(
        "insert into tags (created_at, key, owner_id, server_id, value) values ($1, $2, $3, $4, $5)",
        &[
            &(UTC::now().timestamp()),
            &key,
            &(context.message.author.id.0 as i64),
            &(server_id.0 as i64),
            &value,
        ]
    );
    drop(db);

    match creation {
        Ok(_amount) => {
            let _msg = req!(context.say("Tag created"));
        },
        Err(why) => {
            let text = format!("Error making tag: {:?}", why);
            warn!("Error making tag: {:?}", why);

            let _msg = req!(context.say(text));
        },
    }
}
