use discord::model::{CurrentUser, ReadyEvent, UserId};

pub fn make_fake_ready_event() -> ReadyEvent {
    ReadyEvent {
        version: 0,
        user: CurrentUser {
            id: UserId(0),
            username: String::new(),
            discriminator: 0,
            avatar: None,
            email: None,
            verified: false,
            bot: false,
            mfa_enabled: false,
        },
        session_id: String::new(),
        user_settings: None,
        read_state: None,
        private_channels: vec![],
        presences: vec![],
        relationships: vec![],
        servers: vec![],
        user_server_settings: None,
        tutorial: None,
        trace: vec![],
        notes: None,
        shard: None,
    }
}

#[macro_escape]
macro_rules! req {
    ($expr:expr) => {
        match $expr {
            Ok(v) => v,
            Err(_why) => return,
        }
    }
}

#[macro_escape]
macro_rules! arc {
    ($db:expr) => {
        match ::std::sync::Arc::try_unwrap($db.clone()) {
            Ok(db) => db,
            Err(_why) => return,
        }
    }
}
