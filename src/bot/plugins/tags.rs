use chrono::{NaiveDateTime, UTC};
use diesel::prelude::*;
use diesel;
use discord::model::{ChannelId, permissions};
use discord::{ChannelRef, State};
use ::models::*;
use ::prelude::*;
use ::schema::tags::dsl::*;

pub struct Tags;

impl Tags {
    pub fn new() -> Tags {
        Tags
    }

    pub fn delete(&self, context: Context, state: &State) {
        let key_ = context.text(0);
        let s = match state.find_channel(&context.message.channel_id) {
            Some(ChannelRef::Public(server, _channel)) => server,
            _ => {
                let _msg = req!(context.say("Could not find server"));
                return;
            },
        };

        let tag: Tag = {
            let db = arc!(context.db);

            let filter = tags.filter(server_id.eq(s.id.0 as i64))
                .filter(key.eq(key_))
                .first(&db);

            match filter {
                Ok(tag_) => tag_,
                Err(_why) => {
                    let _msg = req!(context.say("Tag not found"));

                    return;
                },
            }
        };

        let can_delete = if tag.owner_id == context.message.author.id.0 as i64 {
            true
        } else {
            let aid = context.message.author.id;
            let cid = ChannelId(context.message.channel_id.0 as u64);

            s.permissions_for(cid, aid).contains(permissions::MANAGE_MESSAGES)
        };

        if !can_delete {
            let _msg = req!(context.say("You do not have permission to delete a tag."));

            return;
        }

        let db = arc!(context.db);

        match diesel::delete(tags.filter(id.eq(tag.id))).execute(&db) {
            Ok(_rows_deleted) => {
                let _msg = req!(context.say("Deleted tag"));
            },
            Err(_why) => {
                let _msg = req!(context.say("Error deleting tag"));
            },
        }
    }

    pub fn get(&self, context: Context, state: &State) {
        let arg = context.arg(0);
        let arg2 = context.arg(1);
        let mut name = None;

        if let Ok(arg) = arg.as_str() {
            if arg != "get" {
                name = Some(arg);
            }
        }

        if name.is_none() {
            if let Ok(arg) = arg2.as_str() {
                name = Some(arg);
            }
        }

        let user_key = match name {
            Some(user_key) => user_key,
            None => {
                let _msg = req!(context.say("No tag name given"));

                return;
            },
        };

        let s = match state.find_channel(&context.message.channel_id) {
            Some(ChannelRef::Public(server, _channel)) => server,
            _ => {
                let _msg = req!(context.say("Could not find server"));

                return;
            },
        };

        let tag: Tag = {
            let db = arc!(context.db);

            let filter = tags.filter(server_id.eq(s.id.0 as i64))
                .filter(key.eq(user_key))
                .first(&db);

            match filter {
                Ok(tag_) => tag_,
                Err(_why) => {
                    return;
                },
            }
        };

        let update = {
            let db = arc!(context.db);

            diesel::update(tags.filter(server_id.eq(s.id.0 as i64))
                .filter(key.eq(user_key)))
                .set(uses.eq(tag.uses + 1))
                .execute(&db)
        };

        match update {
            Ok(_updated) => {},
            Err(why) => {
                warn!("Error incrementing tag uses: {:?}", why);
            },
        }

        let _msg = req!(context.say(tag.value));
    }

    pub fn info(&self, context: Context, state: &State) {
        let key_ = context.text(0);
        let s = match state.find_channel(&context.message.channel_id) {
            Some(ChannelRef::Public(server, _channel)) => server,
            _ => {
                let _msg = req!(context.say("Could not find server"));
                return;
            },
        };

        let tag: Tag = {
            let db = arc!(context.db);

            let filter = tags.filter(server_id.eq(s.id.0 as i64))
                .filter(key.eq(key_))
                .first(&db);

            match filter {
                Ok(tag_) => tag_,
                Err(_why) => {
                    let _msg = req!(context.say("Tag not found"));

                    return;
                },
            }
        };

        let mut owner = None;

        for member in &s.members {
            if member.user.id.0 as i64 == tag.owner_id {
                owner = Some(member);
            }
        }

        let owner_info = if let Some(owner) = owner {
            format!("{}#{}", owner.user.name, owner.user.discriminator)
        } else {
            "N/A".to_owned()
        };

        let timestamp = NaiveDateTime::from_timestamp(tag.created_at, 0)
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();

        let info = format!(r#"```xl
       Key: {}
     Owner: {}
Created at: {}
      Uses: {}```"#, tag.key,
         owner_info,
         timestamp,
         tag.uses);

        let _msg = req!(context.say(info));
    }

    pub fn list(&self, context: Context, state: &State) {
        let s = match state.find_channel(&context.message.channel_id) {
            Some(ChannelRef::Public(server, _channel)) => server,
            _ => {
                let _msg = req!(context.say("Could not find server"));
                return;
            },
        };

        let tag_list: Vec<Tag> = {
            let db = arc!(context.db);

            let filter = tags.filter(server_id.eq(s.id.0 as i64)).load(&db);

            match filter {
                Ok(tag_list) => tag_list,
                Err(why) => {
                    warn!("[list] retrieving tag list: {:?}", why);

                    let _msg = req!(context.say("Error generating list"));

                    return;
                },
            }
        };

        if tag_list.is_empty() {
            let _msg = req!(context.say("No tags"));

            return;
        }

        let mut alphabetized: Vec<&str> = tag_list.iter()
            .map(|tag| &tag.key[..])
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

            let discord = context.discord.lock().unwrap();
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

    pub fn rename(&self, context: Context, state: &State) {
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

        let server = match state.find_channel(&context.message.channel_id) {
            Some(ChannelRef::Public(server, _channel)) => server,
            _ => {
                let _msg = req!(context.say("Could not find server"));

                return;
            },
        };

        let (key_old, key_new) = (&text[..pos], &text[pos + 5..]);

        if key_new.len() > 100 {
            let _msg = req!(context.say("Max key length is 100"));

            return;
        }

        // Check that the tag currently exists
        {
            let db = arc!(context.db);

            if let Err(_why) = tags.filter(server_id.eq(server.id.0 as i64))
                .filter(key.eq(key_old))
                .first::<Tag>(&db) {
                let _msg = req!(context.say("Tag does not exist"));

                return;
            }
        }

        // Check that a tag with the new name does not exist
        let exists = {
            let db = arc!(context.db);

            tags.filter(server_id.eq(server.id.0 as i64))
                .filter(key.eq(key_new))
                .first::<Tag>(&db)
        };

        match exists {
            Err(diesel::NotFound) => {},
            Ok(_exists) => {
                let _msg = req!(context.say("A tag exists with the new name"));

                return;
            },
            Err(why) => {
                warn!("[rename] retrieving old tag: {:?}", why);

                let _msg = req!(context.say("Error retrieving old tag"));

                return;
            },
        }

        // It's now safe to update the key
        let update = {
            let db = arc!(context.db);

            diesel::update(tags.filter(server_id.eq(server.id.0 as i64))
                .filter(key.eq(key_old)))
                .set(key.eq(key_new))
                .execute(&db)
        };

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

    pub fn search(&self, context: Context, state: &State) {
        let query = context.text(0);

        if query.is_empty() {
            let _msg = req!(context.say("No query given"));

            return;
        }

        let s = match state.find_channel(&context.message.channel_id) {
            Some(ChannelRef::Public(server, _channel)) => server,
            _ => {
                let _msg = req!(context.say("Could not find server"));

                return;
            },
        };

        let search_res = {
            let db = arc!(context.db);

            tags.filter(server_id.eq(s.id.0 as i64))
                .filter(key.like(format!("%{}%", query)))
                .limit(15)
                .load(&db)
        };

        let tag_list: Vec<Tag> = match search_res {
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
            .map(|tag_| tag_.key.clone())
            .collect();
        let listing = key_list.join(", ");

        let _msg = req!(context.say(listing));
    }

    pub fn set(&self, context: Context, state: &State) {
        use ::schema::tags;
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

        let s = match state.find_channel(&context.message.channel_id) {
            Some(ChannelRef::Public(server, _channel)) => server,
            _ => {
                let _msg = req!(context.say("Could not find server"));

                return;
            },
        };

        let (key_, value_) = (&text[..pos], &text[pos + 2..]);

        if key_.len() > 100 {
            let _msg = req!(context.say("Key max length is 100"));

            return;
        }

        // Check if the tag exists already; we don't want to override it
        {
            let creation = {
                let db = arc!(context.db);

                tags.filter(server_id.eq(s.id.0 as i64))
                    .filter(key.eq(key_))
                    .first::<Tag>(&db)
            };

            if let Ok(_tag) = creation {
                let _msg = req!(context.say("Tag already exists"));

                return;
            }
        }

        let new = NewTag {
            created_at: UTC::now().timestamp(),
            key: key_,
            owner_id: context.message.author.id.0 as i64,
            server_id: s.id.0 as i64,
            value: value_,
        };

        let creation = {
            let db = arc!(context.db);

            diesel::insert(&new)
                .into(tags::table)
                .get_result::<Tag>(&db)
        };

        match creation {
            Ok(_tag) => {
                let _msg = req!(context.say("Tag created"));
            },
            Err(why) => {
                let text = format!("Error making tag: {:?}", why);
                warn!("Error making tag: {:?}", why);

                let _msg = req!(context.say(text));
            },
        }
    }
}
