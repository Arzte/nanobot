use diesel::types::Timestamp;
use super::schema::*;

#[derive(Clone, Debug, Identifiable, Queryable)]
pub struct Config {
    pub id: i32,
    pub channel_id: i64,
    pub key: String,
    pub kind: i16,
    pub server_id: i64,
    pub value: String,
}

#[insertable_into(configs)]
pub struct NewConfig<'a> {
    pub channel_id: i64,
    pub key: &'a str,
    pub kind: i16,
    pub server_id: i64,
    pub value: &'a str,
}

#[derive(Clone, Debug, Identifiable, Queryable)]
pub struct Guild {
    pub id: i64,
    pub active: bool,
    pub name: String,
    pub owner_id: i64,
}

#[insertable_into(guilds)]
pub struct NewGuild<'a> {
    /// Whether or not the bot is still in the guild
    pub active: bool,
    pub id: i64,
    pub name: &'a str,
    pub owner_id: i64,
}

#[derive(Clone, Debug, Identifiable, Queryable)]
#[belongs_to(User)]
pub struct Member {
    pub id: i32,
    pub message_count: i64,
    pub nickname: Option<String>,
    pub server_id: i64,
    pub user_id: i64,
    pub weather_location: Option<String>,
}

#[insertable_into(members)]
pub struct NewMember<'a> {
    pub message_count: i64,
    pub nickname: Option<&'a str>,
    pub server_id: i64,
    pub user_id: i64,
    pub weather_location: Option<&'a str>,
}

#[derive(Clone, Debug, Identifiable, Queryable)]
pub struct Quote {
    pub id: i32,
    pub content: String,
    pub message_id: i64,
    pub quoted_at: Timestamp,
    pub quoted_by: i64,
    pub server_id: i64,
}

#[insertable_into(quotes)]
pub struct NewQuote<'a> {
    pub content: &'a str,
    pub message_id: i64,
    pub quoted_by: i64,
    pub server_id: i64,
}

#[derive(Clone, Debug, Identifiable, Queryable)]
pub struct Tag {
    pub id: i32,
    pub created_at: i64,
    pub key: String,
    pub owner_id: i64,
    pub server_id: i64,
    pub uses: i32,
    pub value: String,
}

#[insertable_into(tags)]
pub struct NewTag<'a> {
    pub created_at: i64,
    pub key: &'a str,
    pub owner_id: i64,
    pub server_id: i64,
    pub value: &'a str,
}

#[derive(Clone, Debug, Identifiable, Queryable)]
pub struct User {
    pub id: i64,
    pub bot: bool,
    pub discriminator: i16,
    pub username: String,
}

#[insertable_into(users)]
pub struct NewUser<'a> {
    pub id: i64,
    pub bot: bool,
    pub discriminator: i16,
    pub username: &'a str,
}
