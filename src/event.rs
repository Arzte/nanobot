use serenity::Client;
use ::store::EventCounter;

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

pub fn register(client: &mut Client) {
    client.on_call_create(|ctx, _| {
        reg!(ctx "CallCreate");
    });
    client.on_call_delete(|ctx, _, _| {
        reg!(ctx "CallDelete");
    });
    client.on_call_update(|ctx, _, _| {
        reg!(ctx "CallUpdate");
    });
    client.on_channel_create(|ctx, _| {
        reg!(ctx "ChannelCreate");
    });
    client.on_channel_delete(|ctx, _| {
        reg!(ctx "ChannelDelete");
    });
    client.on_channel_pins_ack(|ctx, _| {
        reg!(ctx "ChannelPinsAck");
    });
    client.on_channel_pins_update(|ctx, _| {
        reg!(ctx "ChannelPinsUpdate");
    });
    client.on_friend_suggestion_create(|ctx, _, _| {
        reg!(ctx "FriendSuggestionCreate");
    });
    client.on_friend_suggestion_delete(|ctx, _| {
        reg!(ctx "FriendSuggestionDelete");
    });
    client.on_guild_create(|ctx, guild| {
        debug!("Received guild: {}", guild.name);

        reg!(ctx "GuildCreate");
    });
    client.on_guild_emojis_update(|ctx, _, _| {
        reg!(ctx "GuildEmojisUpdate");
    });
    client.on_guild_integrations_update(|ctx, _| {
        reg!(ctx "GuildIntegrationsUpdate");
    });
    client.on_guild_member_add(|ctx, _, _| {
        reg!(ctx "GuildMemberAdd");
    });
    client.on_guild_members_chunk(|ctx, _, _| {
        reg!(ctx "GuildMembersChunk");
    });
    client.on_guild_role_create(|ctx, _, _| {
        reg!(ctx "GuildRoleCreate");
    });
    client.on_guild_sync(|ctx, _| {
        reg!(ctx "GuildRoleSync");
    });
    client.on_guild_unavailable(|ctx, _| {
        reg!(ctx "GuildUnavailable");
    });
    client.on_member_ban(|ctx, _, _| {
        reg!(ctx "MemberBan");
    });
    client.on_member_unban(|ctx, _, _| {
        reg!(ctx "MemberUnban");
    });
    client.on_message(|ctx, _| {
        reg!(ctx "MessageCreate");
    });
    client.on_message_ack(|ctx, _, _| {
        reg!(ctx "MessageAck");
    });
    client.on_message_delete(|ctx, _, _| {
        reg!(ctx "MessageDelete");
    });
    client.on_message_delete_bulk(|ctx, _, _| {
        reg!(ctx "MessageDeleteBulk");
    });
    client.on_message_update(|ctx, _| {
        reg!(ctx "MessageUpdate");
    });
    client.on_presence_replace(|ctx, _| {
        reg!(ctx "PresencesReplace");
    });
    client.on_presence_update(|ctx, _| {
        reg!(ctx "PresenceUpdate");
    });
    client.on_reaction_add(|ctx, _| {
        reg!(ctx "ReactionAdd");
    });
    client.on_reaction_remove(|ctx, _| {
        reg!(ctx "ReactionRemove");
    });
    client.on_reaction_remove_all(|ctx, _, _| {
        reg!(ctx "ReactionRemoveAll");
    });
    client.on_ready(|ctx, _| {
        reg!(ctx "Ready");
    });
    client.on_recipient_add(|ctx, _, _| {
        reg!(ctx "ChannelRecipientAdd");
    });
    client.on_recipient_remove(|ctx, _, _| {
        reg!(ctx "ChannelRecipientRemove");
    });
    client.on_relationship_add(|ctx, _| {
        reg!(ctx "RelationshipAdd");
    });
    client.on_relationship_remove(|ctx, _, _| {
        reg!(ctx "RelationshipRemove");
    });
    client.on_resume(|ctx, _| {
        reg!(ctx "Resume");
    });
    client.on_typing_start(|ctx, _| {
        reg!(ctx "TypingStart");
    });
    client.on_unknown(|_, name, value| {
        warn!("Received unknown event '{}': {:?}", name, value);
    });
    client.on_voice_server_update(|ctx, _| {
        reg!(ctx "VoiceServerUpdate");
    });
    client.on_voice_state_update(|ctx, _, _| {
        reg!(ctx "VoiceStateUpdate");
    });
    client.on_webhook_update(|ctx, _, _| {
        reg!(ctx "WebhookUpdate");
    });
}
