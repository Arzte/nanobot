use serde_json::Value;
use serenity::client::{Context, EventHandler};
use serenity::model::event::*;
use serenity::model::*;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use super::misc::Uptime;
use super::store::{EventCounter, ShardUptime};

macro_rules! reg {
    ($ctx:ident $name:expr) => {
        {
            let mut data = $ctx.data.lock().unwrap();
            let counter = data.get_mut::<EventCounter>().unwrap();
            let entry = counter.entry($name).or_insert(0);
            *entry += 1;
        }
    }
}

pub struct Handler;

impl EventHandler for Handler {
    fn on_channel_create(&self, ctx: Context, _: Arc<RwLock<GuildChannel>>) {
        reg!(ctx "ChannelCreate");
    }

    fn on_channel_delete(&self, ctx: Context, _: Arc<RwLock<GuildChannel>>) {
        reg!(ctx "ChannelDelete");
    }

    fn on_channel_pins_update(&self, ctx: Context, _: ChannelPinsUpdateEvent) {
        reg!(ctx "ChannelPinsUpdate");
    }

    fn on_guild_create(&self, ctx: Context, guild: Guild, new: bool) {
        let status = if new { "new" } else { "old" };
        debug!("Received guild: {} ({})", guild.name, status);

        reg!(ctx "GuildCreate");
    }

    fn on_guild_emojis_update(&self, ctx: Context, _: GuildId, _: HashMap<EmojiId, Emoji>) {
        reg!(ctx "GuildEmojisUpdate");
    }

    fn on_guild_integrations_update(&self, ctx: Context, _: GuildId) {
        reg!(ctx "GuildIntegrationsUpdate");
    }

    fn on_guild_member_addition(&self, ctx: Context, _: GuildId, _: Member) {
        reg!(ctx "GuildMemberAdd");
    }

    fn on_guild_member_removal(&self, ctx: Context, _: GuildId, _: User, _: Option<Member>) {
        reg!(ctx "GuildMemberRemoval");
    }

    fn on_guild_members_chunk(&self, ctx: Context, _: GuildId, _: HashMap<UserId, Member>) {
        reg!(ctx "GuildMembersChunk");
    }

    fn on_guild_role_create(&self, ctx: Context, _: GuildId, _: Role) {
        reg!(ctx "GuildRoleCreate");
    }

    fn on_guild_unavailable(&self, ctx: Context, _: GuildId) {
        reg!(ctx "GuildUnavailable");
    }

    fn on_guild_ban_addition(&self, ctx: Context, _: GuildId, _: User) {
        reg!(ctx "GuildBanAddition");
    }

    fn on_guild_ban_removal(&self, ctx: Context, _: GuildId, _: User) {
        reg!(ctx "GuildBanRemoval");
    }

    fn on_message(&self, ctx: Context, _: Message) {
        reg!(ctx "MessageCreate");
    }

    fn on_message_delete(&self, ctx: Context, _: ChannelId, _: MessageId) {
        reg!(ctx "MessageDelete");
    }

    fn on_message_delete_bulk(&self, ctx: Context, _: ChannelId, _: Vec<MessageId>) {
        reg!(ctx "MessageDeleteBulk");
    }

    fn on_message_update(&self, ctx: Context, _: MessageUpdateEvent) {
        reg!(ctx "MessageUpdate");
    }

    fn on_presence_replace(&self, ctx: Context, _: Vec<Presence>) {
        reg!(ctx "PresencesReplace");
    }

    fn on_presence_update(&self, ctx: Context, _: PresenceUpdateEvent) {
        reg!(ctx "PresenceUpdate");
    }

    fn on_reaction_add(&self, ctx: Context, _: Reaction) {
        reg!(ctx "ReactionAdd");
    }

    fn on_reaction_remove(&self, ctx: Context, _: Reaction) {
        reg!(ctx "ReactionRemove");
    }

    fn on_reaction_remove_all(&self, ctx: Context, _: ChannelId, _: MessageId) {
        reg!(ctx "ReactionRemoveAll");
    }

    fn on_ready(&self, ctx: Context, ready: Ready) {
        if let Some(s) = ready.shard {
            info!("Logged in as '{}' on {}/{}",
                  ready.user.name,
                  s[0],
                  s[1]);
        } else {
            info!("Logged in as '{}'", ready.user.name);
        }

        let name = {
            let mut data = ctx.data.lock().unwrap();

            {
                let counter = data.get_mut::<EventCounter>().unwrap();
                let entry = counter.entry("Ready").or_insert(0);
                *entry += 1;
            }

            let uptimes = data.get_mut::<ShardUptime>().unwrap();

            if let Some(shard) = ready.shard {
                let entry = uptimes.entry(shard[0]).or_insert_with(Uptime::default);
                entry.connect();

                format!("nano help [{}/{}]", shard[0] + 1, shard[1])
            } else {
                "nano help".to_owned()
            }
        };

        ctx.set_game_name(&name);
    }

    fn on_resume(&self, ctx: Context, _: ResumedEvent) {
        reg!(ctx "Resume");
    }

    fn on_typing_start(&self, ctx: Context, _: TypingStartEvent) {
        reg!(ctx "TypingStart");
    }

    fn on_unknown(&self, _: Context, name: String, value: Value) {
        warn!("Received unknown event '{}': {:?}", name, value);
    }

    fn on_voice_server_update(&self, ctx: Context, _: VoiceServerUpdateEvent) {
        reg!(ctx "VoiceServerUpdate");
    }

    fn on_voice_state_update(&self, ctx: Context, _: Option<GuildId>, _: VoiceState) {
        reg!(ctx "VoiceStateUpdate");
    }

    fn on_webhook_update(&self, ctx: Context, _: GuildId, _: ChannelId) {
        reg!(ctx "WebhookUpdate");
    }
}
