use serde_json::Value;
use serenity::client::{Context, EventHandler};
use serenity::model::prelude::*;
use serenity::prelude::RwLock;
use serenity::CACHE;
use std::collections::HashMap;
use std::fmt::Write;
use std::sync::Arc;
use super::misc::Uptime;
use super::store::{EventCounter, ShardUptime};

macro_rules! reg {
    ($ctx:ident $name:expr) => {
        {
           let mut data = $ctx.data.lock();
            let counter = data.get_mut::<EventCounter>().unwrap();
            let entry = counter.entry($name).or_insert(0);
            *entry += 1;
        }
    }
}

pub const GUILD_DABBOT_ID: u64 = 272410239947767808;
pub const GUILD_DABBOT_ROLE_STAFF_ID: RoleId = RoleId(325307197666099200);
#[allow(dead_code)]
pub const GUILD_ROLE_ONLINE_COLOUR: u32 = 0x43_B581;
pub const GUILD_ROLE_ONLINE_ID: RoleId = RoleId(395998395803893761);
#[allow(dead_code)]
pub const GUILD_ROLE_IDLE_COLOUR: u32 = 0xFA_A61A;
pub const GUILD_ROLE_IDLE_ID: RoleId = RoleId(395998774390161411);
#[allow(dead_code)]
pub const GUILD_ROLE_OFFLINE_COLOUR: u32 = 0x74_7F8D;
pub const GUILD_ROLE_OFFLINE_ID: RoleId = RoleId(395998775748984832);
#[allow(dead_code)]
pub const GUILD_ROLE_DND_COLOUR: u32 = 0xF0_4747;
pub const GUILD_ROLE_DND_ID: RoleId = RoleId(395998771776847882);

pub struct Handler;

impl EventHandler for Handler {
    fn channel_create(&self, ctx: Context, _: Arc<RwLock<GuildChannel>>) {
        reg!(ctx "ChannelCreate");
    }

    fn channel_delete(&self, ctx: Context, _: Arc<RwLock<GuildChannel>>) {
        reg!(ctx "ChannelDelete");
    }

    fn channel_pins_update(&self, ctx: Context, _: ChannelPinsUpdateEvent) {
        reg!(ctx "ChannelPinsUpdate");
    }

    fn guild_create(&self, ctx: Context, guild: Guild, new: bool) {
        let status = if new { "new" } else { "old" };
        debug!("Received guild: {} ({})", guild.name, status);

        reg!(ctx "GuildCreate");
    }

    fn guild_emojis_update(&self, ctx: Context, _: GuildId, _: HashMap<EmojiId, Emoji>) {
        reg!(ctx "GuildEmojisUpdate");
    }

    fn guild_integrations_update(&self, ctx: Context, _: GuildId) {
        reg!(ctx "GuildIntegrationsUpdate");
    }

    fn guild_member_addition(&self, ctx: Context, guild_id: GuildId, mut member: Member) {
        reg!(ctx "GuildMemberAdd");

        if guild_id == GUILD_DABBOT_ID {
            let user_id = member.user.read().id;

            let diff = match role_diff(member.guild_id, user_id, Vec::new(), member.roles) {
                Some(diff) => diff,
                None => return,
            };

            let _ = ChannelId(301717945854197760).say(&diff);

            return;
        } else if guild_id == 381880193251409931 {
            let role_id = if member.user.read().bot {
                381891974615269376
            } else {
                381891844067557378
            };

            if let Err(why) = member.add_role(role_id) {
                error!("Error adding role to {:?}: {:?}", member, why);
            }
        }
    }

    fn guild_member_removal(&self, ctx: Context, _: GuildId, _: User, _: Option<Member>) {
        reg!(ctx "GuildMemberRemoval");
    }

    fn guild_member_update(&self, ctx: Context, old: Option<Member>, new: Member) {
        reg!(ctx "GuildMemberUpdate");

        if new.guild_id != GUILD_DABBOT_ID {
            return;
        }

        let user_id = new.user.read().id;
        let old_role_ids = old.map(|old| old.roles).unwrap_or_default();

        let diff = match role_diff(new.guild_id, user_id, old_role_ids, new.roles) {
            Some(diff) => diff,
            None => return,
        };

        let _ = ChannelId(301717945854197760).say(&diff);
    }

    fn guild_members_chunk(&self, ctx: Context, _: GuildId, _: HashMap<UserId, Member>) {
        reg!(ctx "GuildMembersChunk");
    }

    fn guild_role_create(&self, ctx: Context, _: GuildId, _: Role) {
        reg!(ctx "GuildRoleCreate");
    }

    fn guild_unavailable(&self, ctx: Context, _: GuildId) {
        reg!(ctx "GuildUnavailable");
    }

    fn guild_ban_addition(&self, ctx: Context, _: GuildId, _: User) {
        reg!(ctx "GuildBanAddition");
    }

    fn guild_ban_removal(&self, ctx: Context, _: GuildId, _: User) {
        reg!(ctx "GuildBanRemoval");
    }

    fn message(&self, ctx: Context, _: Message) {
        reg!(ctx "MessageCreate");
    }

    fn message_delete(&self, ctx: Context, _: ChannelId, _: MessageId) {
        reg!(ctx "MessageDelete");
    }

    fn message_delete_bulk(&self, ctx: Context, _: ChannelId, _: Vec<MessageId>) {
        reg!(ctx "MessageDeleteBulk");
    }

    fn message_update(&self, ctx: Context, _: MessageUpdateEvent) {
        reg!(ctx "MessageUpdate");
    }

    fn presence_replace(&self, ctx: Context, _: Vec<Presence>) {
        reg!(ctx "PresencesReplace");
    }

    fn presence_update(&self, ctx: Context, event: PresenceUpdateEvent) {
        reg!(ctx "PresenceUpdate");

        let guild_id = match event.guild_id {
            Some(guild_id) if guild_id == GUILD_DABBOT_ID => guild_id,
            _ => return,
        };

        let roles = match event.roles {
            Some(roles) => roles,
            None => return,
        };

        if !roles.contains(&GUILD_DABBOT_ROLE_STAFF_ID) {
            return;
        }

        let mut member = {
            let guild = match guild_id.find() {
                Some(guild) => guild,
                None => return,
            };
            let guild = guild.read();

            match guild.member(event.presence.user_id) {
                Ok(member) => member,
                Err(_) => return,
            }
        };

        if !member.roles.contains(&GUILD_DABBOT_ROLE_STAFF_ID) {
            return;
        }

        let give = match event.presence.status {
            OnlineStatus::DoNotDisturb => GUILD_ROLE_DND_ID,
            OnlineStatus::Idle => GUILD_ROLE_IDLE_ID,
            OnlineStatus::Invisible | OnlineStatus::Offline => GUILD_ROLE_OFFLINE_ID,
            OnlineStatus::Online => GUILD_ROLE_ONLINE_ID,
        };

        let roles = [
            GUILD_ROLE_DND_ID,
            GUILD_ROLE_IDLE_ID,
            GUILD_ROLE_OFFLINE_ID,
            GUILD_ROLE_ONLINE_ID,
        ];

        for role in roles.iter() {
            member.roles.retain(|r| r != role);
        }

        if let Err(why) = member.add_roles(&[give]) {
            warn!("Err updating member roles: {:?}", why);
        }
    }

    fn reaction_add(&self, ctx: Context, _: Reaction) {
        reg!(ctx "ReactionAdd");
    }

    fn reaction_remove(&self, ctx: Context, _: Reaction) {
        reg!(ctx "ReactionRemove");
    }

    fn reaction_remove_all(&self, ctx: Context, _: ChannelId, _: MessageId) {
        reg!(ctx "ReactionRemoveAll");
    }

    fn ready(&self, ctx: Context, ready: Ready) {
        if let Some(s) = ready.shard {
            info!("Logged in as '{}' on {}/{}",
                  ready.user.name,
                  s[0],
                  s[1]);
        } else {
            info!("Logged in as '{}'", ready.user.name);
        }

        let name = {
            let mut data = ctx.data.lock();

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

    fn resume(&self, ctx: Context, _: ResumedEvent) {
        reg!(ctx "Resume");
    }

    fn typing_start(&self, ctx: Context, _: TypingStartEvent) {
        reg!(ctx "TypingStart");
    }

    fn unknown(&self, _: Context, name: String, value: Value) {
        warn!("Received unknown event '{}': {:?}", name, value);
    }

    fn voice_server_update(&self, ctx: Context, _: VoiceServerUpdateEvent) {
        reg!(ctx "VoiceServerUpdate");
    }

    fn voice_state_update(&self, ctx: Context, _: Option<GuildId>, _: VoiceState) {
        reg!(ctx "VoiceStateUpdate");
    }

    fn webhook_update(&self, ctx: Context, _: GuildId, _: ChannelId) {
        reg!(ctx "WebhookUpdate");
    }
}

fn role_diff(guild_id: GuildId, user_id: UserId, old_roles: Vec<RoleId>, new_roles: Vec<RoleId>) -> Option<String> {
    let role_ids = [
        RoleId(285375674443759617),
        RoleId(301828565085716480),
        RoleId(301781206347939841),
        RoleId(301781366155247616),
    ];

    let added_ids = new_roles
        .iter()
        .filter(|id| !old_roles.contains(&id))
        .filter(|id| role_ids.contains(*id))
        .collect::<Vec<&RoleId>>();
    let removed_ids = old_roles
        .iter()
        .filter(|id| !new_roles.contains(&id))
        .filter(|id| role_ids.contains(id))
        .collect::<Vec<&RoleId>>();

    if added_ids.is_empty() && removed_ids.is_empty() {
        return None;
    }

    let cache = CACHE.read();

    let mut content = {
        let found = cache.user(user_id).unwrap();
        let user = found.read();

        format!("<@87164639695110144>\n```diff\n{} ({})\n", user.tag(), user.id)
    };

    {
        let guild = cache.guild(guild_id).unwrap();
        let reader = guild.read();

        for role_id in added_ids {
            let role = reader.roles.get(role_id).unwrap();

            let _ = write!(content, "+ {} ({})\n", role.name, role.id);
        }

        for role_id in removed_ids {
            let role = reader.roles.get(role_id).unwrap();

            let _ = write!(content, "- {} ({})\n", role.name, role.id);
        }
    }

    content.push_str("```");

    Some(content)
}
