use std::{borrow::Cow, iter};

use deadpool_redis::redis::AsyncCommands;
use hashbrown::HashMap;
use serde::Serialize;
use serde_cbor::Error as CborError;
use twilight_model::{
    application::interaction::Interaction,
    channel::Channel,
    gateway::event::Event,
    guild::{Member, Role},
    id::GuildId,
};

use crate::{
    constants::{CHANNEL_KEYS, GUILD_KEYS, MEMBER_KEYS, ROLE_KEYS},
    model::{
        BasicGuildChannel, CurrentUserWrapper, GuildWrapper, MemberUpdateWrapper, MemberWrapper,
        PartialGuildWrapper, PartialMemberWrapper, RedisKey, RoleWrapper, SessionInfo,
    },
    CacheResult,
};

use super::Cache;

impl Cache {
    #[inline]
    pub async fn cache_channel(&self, channel: &Channel) -> CacheResult<()> {
        if let Channel::Guild(channel) = channel {
            if let Some(c) = BasicGuildChannel::from(channel) {
                self.set(RedisKey::from(&c), c).await?;
            }
        }

        Ok(())
    }

    #[inline]
    pub async fn cache_member(&self, member: &Member) -> CacheResult<()> {
        let wrapper = MemberWrapper::from(member);

        if let Some(ttl) = self.config.member_ttl {
            self.set_with_expire(RedisKey::from(member), wrapper, ttl)
                .await
        } else {
            self.set(member.into(), wrapper).await
        }
    }

    #[inline]
    pub async fn cache_role(&self, role: &Role, guild: GuildId) -> CacheResult<()> {
        self.set(RedisKey::from((guild, role.id)), RoleWrapper::from(role))
            .await
    }

    #[inline]
    pub async fn cache_shards(&self, shards: u64) -> CacheResult<()> {
        self.set(RedisKey::Shards, shards).await
    }

    #[inline]
    pub async fn cache_sessions(&self, sessions: &HashMap<u64, SessionInfo>) -> CacheResult<()> {
        self.set_with_expire(RedisKey::Sessions, sessions, 300)
            .await
    }

    pub async fn update(&self, event: &Event) -> CacheResult<()> {
        match event {
            Event::ChannelCreate(e) => self.cache_channel(e).await?,
            Event::ChannelDelete(e) => {
                if let Channel::Guild(channel) = &e.0 {
                    if let Some(c) = BasicGuildChannel::from(channel) {
                        self.del(RedisKey::from(&c)).await?;
                    }
                }
            }
            Event::ChannelUpdate(e) => self.cache_channel(e).await?,
            Event::GuildCreate(e) => {
                self.clear_guild(e.id).await?;

                // Cache channels
                if !e.channels.is_empty() {
                    let channels = e
                        .channels
                        .iter()
                        .filter_map(BasicGuildChannel::from)
                        .map(|channel| (RedisKey::from(&channel), channel));

                    self.set_all(channels).await?;
                }

                // Cache roles
                if !e.roles.is_empty() {
                    let roles = e
                        .roles
                        .iter()
                        .map(|role| (RedisKey::from((e.id, role.id)), RoleWrapper::from(role)));

                    self.set_all(roles).await?;
                }

                // Cache members
                if !e.members.is_empty() {
                    let members = e
                        .members
                        .iter()
                        .map(MemberWrapper::from)
                        .map(|member| (RedisKey::from(&member), member));

                    if let Some(ttl) = self.config.member_ttl {
                        let keys = members
                            .map(|(key, member)| Ok((key, serde_cbor::to_vec(&member)?)))
                            .collect::<Result<Vec<_>, CborError>>()?;

                        self.set_all_with_expire(&keys, ttl).await?;
                    } else {
                        self.set_all(members).await?;
                    }
                }

                // Cache the guild itself
                self.set(e.id.into(), GuildWrapper::from(&e.0)).await?;
            }
            Event::GuildDelete(e) => self.clear_guild(e.id).await?,
            Event::GuildUpdate(e) => {
                self.set(e.id.into(), PartialGuildWrapper::from(&e.0))
                    .await?
            }
            Event::InteractionCreate(e) => {
                let (guild, member) = match &e.0 {
                    Interaction::ApplicationCommand(data) => (data.guild_id, &data.member),
                    Interaction::MessageComponent(data) => (data.guild_id, &data.member),
                    _ => return Ok(()),
                };

                if let (Some(member), Some(guild)) = (member, guild) {
                    if let Some(user) = &member.user {
                        let key = RedisKey::from((guild, user.id));
                        let member = PartialMemberWrapper::from((member, guild, user));

                        if let Some(ttl) = self.config.member_ttl {
                            self.set_with_expire(key, member, ttl).await?;
                        } else {
                            self.set(key, member).await?;
                        }
                    }
                }
            }
            Event::MemberAdd(e) => self.cache_member(e).await?,
            Event::MemberRemove(e) => self.del(RedisKey::from((e.guild_id, e.user.id))).await?,
            Event::MemberUpdate(e) => {
                let key = RedisKey::from((e.guild_id, e.user.id));
                let member = MemberUpdateWrapper::from(e.as_ref());

                if let Some(ttl) = self.config.member_ttl {
                    self.set_with_expire(key, member, ttl).await?;
                } else {
                    self.set(key, member).await?;
                }
            }
            Event::MemberChunk(e) => {
                let keys = e
                    .members
                    .iter()
                    .map(MemberWrapper::from)
                    .map(|member| (RedisKey::from(&member), member));

                if let Some(ttl) = self.config.member_ttl {
                    let keys = keys
                        .map(|(key, member)| Ok((key, serde_cbor::to_vec(&member)?)))
                        .collect::<Result<Vec<_>, CborError>>()?;

                    self.set_all_with_expire(&keys, ttl).await?;
                } else {
                    self.set_all(keys).await?;
                }
            }
            Event::MessageCreate(e) => {
                if let (Some(member), Some(guild)) = (&e.member, e.guild_id) {
                    let key = RedisKey::from((guild, e.author.id));
                    let member = PartialMemberWrapper::from((member, guild, &e.author));

                    if let Some(ttl) = self.config.member_ttl {
                        self.set_with_expire(key, member, ttl).await?;
                    } else {
                        self.set(key, member).await?;
                    }
                }
            }
            Event::ReactionAdd(e) => {
                if let Some(member) = &e.member {
                    self.cache_member(member).await?;
                }
            }
            Event::ReactionRemove(e) => {
                if let Some(member) = &e.member {
                    self.cache_member(member).await?;
                }
            }
            Event::Ready(e) => {
                self.set(RedisKey::BotUser, CurrentUserWrapper::from(&e.user))
                    .await?;
            }
            Event::RoleCreate(e) => self.cache_role(&e.role, e.guild_id).await?,
            Event::RoleDelete(e) => self.del(RedisKey::from((e.guild_id, e.role_id))).await?,
            Event::RoleUpdate(e) => self.cache_role(&e.role, e.guild_id).await?,
            Event::ThreadCreate(e) => self.cache_channel(e).await?,
            Event::ThreadDelete(e) => {
                if let Channel::Guild(channel) = &e.0 {
                    if let Some(c) = BasicGuildChannel::from(channel) {
                        self.del(RedisKey::from(&c)).await?;
                    }
                }
            }
            Event::ThreadListSync(e) => {
                // Cache members
                if !e.members.is_empty() {
                    let keys = e
                        .members
                        .iter()
                        .filter_map(|member| member.member.as_ref())
                        .map(MemberWrapper::from)
                        .map(|member| (RedisKey::from(&member), member));

                    if let Some(ttl) = self.config.member_ttl {
                        let keys = keys
                            .map(|(key, member)| Ok((key, serde_cbor::to_vec(&member)?)))
                            .collect::<Result<Vec<_>, CborError>>()?;

                        self.set_all_with_expire(&keys, ttl).await?;
                    } else {
                        self.set_all(keys).await?;
                    }
                }

                // Cache channels
                if !e.threads.is_empty() {
                    let keys = e
                        .threads
                        .iter()
                        .filter_map(|c| {
                            if let Channel::Guild(channel) = c {
                                BasicGuildChannel::from(channel)
                            } else {
                                None
                            }
                        })
                        .map(|channel| (RedisKey::from(&channel), channel));

                    self.set_all(keys).await?;
                }
            }
            Event::ThreadMemberUpdate(e) => {
                if let Some(member) = &e.member {
                    self.cache_member(member).await?;
                }
            }
            Event::ThreadMembersUpdate(e) => {
                if !e.added_members.is_empty() {
                    let keys = e
                        .added_members
                        .iter()
                        .filter_map(|member| member.member.as_ref())
                        .map(MemberWrapper::from)
                        .map(|member| (RedisKey::from(&member), member));

                    if let Some(ttl) = self.config.member_ttl {
                        let keys = keys
                            .map(|(key, member)| Ok((key, serde_cbor::to_vec(&member)?)))
                            .collect::<Result<Vec<_>, CborError>>()?;

                        self.set_all_with_expire(&keys, ttl).await?;
                    } else {
                        self.set_all(keys).await?;
                    }
                }
            }
            Event::ThreadUpdate(e) => self.cache_channel(e).await?,
            Event::UserUpdate(e) => {
                self.set(RedisKey::BotUser, CurrentUserWrapper::from(&e.0))
                    .await?
            }
            _ => {}
        }

        Ok(())
    }

    async fn set<T>(&self, key: RedisKey, value: T) -> CacheResult<()>
    where
        T: Serialize,
    {
        self.set_all(iter::once((key, value))).await?;

        Ok(())
    }

    async fn set_all<I, T>(&self, keys: I) -> CacheResult<()>
    where
        I: IntoIterator<Item = (RedisKey, T)>,
        T: Serialize,
    {
        let mut members = HashMap::new();

        let keys = keys
            .into_iter()
            .inspect(|(key, _)| populate_members(key, &mut members))
            .map(|(key, value)| serde_cbor::to_vec(&value).map(|value| (key, value)))
            .collect::<Result<Vec<(RedisKey, Vec<u8>)>, CborError>>()?;

        if keys.is_empty() {
            return Ok(());
        }

        let mut conn = self.redis.get().await?;
        conn.set_multiple(&keys).await?;

        for (key, value) in members {
            conn.sadd(key.as_ref(), value).await?;
        }

        Ok(())
    }

    async fn set_with_expire<T>(&self, key: RedisKey, value: T, seconds: usize) -> CacheResult<()>
    where
        T: Serialize,
    {
        let bytes = serde_cbor::to_vec(&value)?;
        let mut conn = self.redis.get().await?;
        conn.set_ex(key, bytes, seconds).await?;

        Ok(())
    }

    async fn set_all_with_expire(
        &self,
        keys: &[(RedisKey, Vec<u8>)],
        seconds: usize,
    ) -> CacheResult<()> {
        if keys.is_empty() {
            return Ok(());
        }

        let mut conn = self.redis.get().await?;
        conn.set_multiple(keys).await?;

        for (key, _) in keys {
            conn.expire(key, seconds).await?;
        }

        Ok(())
    }

    async fn del(&self, key: RedisKey) -> CacheResult<()> {
        let mut members = HashMap::new();
        populate_members(&key, &mut members);

        let mut conn = self.redis.get().await?;
        conn.del(key).await?;

        for (key, value) in members {
            conn.srem(key.as_ref(), value).await?;
        }

        Ok(())
    }

    async fn del_all<I>(&self, keys: I) -> CacheResult<()>
    where
        I: IntoIterator<Item = RedisKey>,
    {
        let mut members = HashMap::new();

        let keys = keys
            .into_iter()
            .inspect(|key| populate_members(key, &mut members))
            .collect::<Vec<RedisKey>>();

        if keys.is_empty() {
            return Ok(());
        }

        let mut conn = self.redis.get().await?;
        conn.del(keys).await?;

        for (key, value) in members {
            conn.srem(key.as_ref(), value).await?;
        }

        Ok(())
    }

    async fn clear_guild(&self, guild: GuildId) -> CacheResult<()> {
        let members = self
            .get_members::<RedisKey>(format!("{}:{}", GUILD_KEYS, guild))
            .await?;

        self.del_all(members).await?;
        self.del(RedisKey::Guild { guild }).await?;

        Ok(())
    }
}

type RedisMembers = HashMap<Cow<'static, str>, Vec<RedisKey>>;

fn populate_members(key: &RedisKey, members: &mut RedisMembers) {
    match key {
        RedisKey::Channel { guild, .. } => {
            populate_member(CHANNEL_KEYS, *key, members);

            if let Some(guild) = guild {
                populate_member(format!("{}:{}", GUILD_KEYS, guild), *key, members);
            }
        }
        RedisKey::Guild { .. } => populate_member(GUILD_KEYS, *key, members),
        RedisKey::Member { guild, .. } => {
            populate_member(MEMBER_KEYS, *key, members);
            populate_member(format!("{}:{}", GUILD_KEYS, guild), *key, members);
        }
        RedisKey::Role { guild, .. } => {
            populate_member(ROLE_KEYS, *key, members);

            if let Some(guild) = guild {
                populate_member(format!("{}:{}", GUILD_KEYS, guild), *key, members);
            }
        }
        _ => {}
    }
}

fn populate_member(key: impl Into<Cow<'static, str>>, value: RedisKey, members: &mut RedisMembers) {
    members
        .entry(key.into())
        .or_insert_with(Vec::new)
        .push(value)
}
