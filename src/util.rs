use std::borrow::Cow;

use deadpool_redis::redis::AsyncCommands;
use twilight_model::{
    channel::permission_overwrite::PermissionOverwriteType, guild::Permissions, id::UserId,
};

use crate::{
    constants::{CHANNEL_KEYS, GUILD_KEYS, MEMBER_KEYS, OWNER_USER_ID, ROLE_KEYS},
    model::{
        CacheStats, CachedChannel, CachedMember, CachedTextChannel, ChannelOrId, GuildOrId,
        MemberLookup, RedisKey,
    },
    CacheError, CacheResult,
};

use super::Cache;

impl Cache {
    #[inline]
    pub async fn is_guild_owner(&self, guild: &GuildOrId, user: UserId) -> CacheResult<bool> {
        match guild {
            GuildOrId::Guild(guild) => Ok(guild.owner_id == user),
            GuildOrId::Id(id) => {
                let guild = self.guild(*id).await?.ok_or(CacheError::MissingGuild)?;

                Ok(guild.owner_id == user)
            }
        }
    }

    #[inline]
    pub async fn contains(&self, key: impl Into<RedisKey>) -> CacheResult<bool> {
        Ok(self.redis.get().await?.exists(key.into()).await?)
    }

    pub async fn stats(&self) -> CacheResult<CacheStats> {
        let mut conn = self.redis.get().await?;

        let stats = CacheStats {
            channels: conn.scard(CHANNEL_KEYS).await?,
            guilds: conn.scard(GUILD_KEYS).await?,
            members: conn.scard(MEMBER_KEYS).await?,
            roles: conn.scard(ROLE_KEYS).await?,
        };

        Ok(stats)
    }

    pub async fn get_guild_permissions(
        &self,
        user: UserId,
        guild: &GuildOrId,
    ) -> CacheResult<(Permissions, MemberLookup)> {
        if user.get() == OWNER_USER_ID {
            return Ok((Permissions::all(), MemberLookup::NotChecked));
        }

        match self.is_guild_owner(guild, user).await {
            Ok(true) => return Ok((Permissions::all(), MemberLookup::NotChecked)),
            Ok(false) => {}
            Err(CacheError::MissingGuild) => {
                return Ok((Permissions::empty(), MemberLookup::NotChecked))
            }
            Err(err) => return Err(err),
        }

        let member = match self.member(guild.id(), user).await? {
            Some(member) => member,
            None => return Ok((Permissions::empty(), MemberLookup::NotFound)),
        };

        let mut permissions = Permissions::empty();

        for &role_id in &member.roles {
            if let Some(role) = self.role(role_id).await? {
                if role.permissions.contains(Permissions::ADMINISTRATOR) {
                    return Ok((Permissions::all(), MemberLookup::Found(member)));
                }

                permissions |= role.permissions;
            }
        }

        Ok((permissions, MemberLookup::Found(member)))
    }

    pub async fn get_channel_permissions(
        &self,
        user: UserId,
        channel: &ChannelOrId,
        guild: Option<&GuildOrId>,
    ) -> CacheResult<Permissions> {
        let guild = if let Some(guild) = guild {
            guild
        } else {
            // Private channel
            let permissions = Permissions::SEND_MESSAGES
                | Permissions::EMBED_LINKS
                | Permissions::ATTACH_FILES
                | Permissions::USE_EXTERNAL_EMOJIS
                | Permissions::ADD_REACTIONS
                | Permissions::READ_MESSAGE_HISTORY;

            return Ok(permissions);
        };

        let (mut permissions, member) = self.get_guild_permissions(user, guild).await?;

        if permissions.contains(Permissions::ADMINISTRATOR) {
            return Ok(Permissions::all());
        }

        let channel = self.extract_channel(channel).await?;

        if let Some(channel) = channel {
            let member = match member {
                MemberLookup::Found(member) => Some(member),
                MemberLookup::NotChecked => self.member(guild.id(), user).await?,
                MemberLookup::NotFound => None,
            };

            if let Some(member) = member {
                Self::text_channel_permissions(&mut permissions, user, guild, channel, member)
            }
        }

        Ok(permissions)
    }

    #[allow(clippy::needless_lifetimes)]
    async fn extract_channel<'c>(
        &self,
        channel: &'c ChannelOrId,
    ) -> CacheResult<Option<Cow<'c, CachedTextChannel>>> {
        let id = match channel {
            ChannelOrId::Channel(CachedChannel::Text(channel)) => {
                return Ok(Some(Cow::Borrowed(channel)))
            }
            ChannelOrId::Channel(
                CachedChannel::PrivateThread(channel) | CachedChannel::PublicThread(channel),
            ) => channel.id,
            ChannelOrId::Id(id) => *id,
        };

        match self.channel(id).await? {
            Some(CachedChannel::Text(c)) => return Ok(Some(Cow::Owned(c))),
            Some(CachedChannel::PrivateThread(thread))
            | Some(CachedChannel::PublicThread(thread)) => {
                if let Some(parent) = thread.parent_id {
                    if let Some(CachedChannel::Text(channel)) = self.channel(parent).await? {
                        return Ok(Some(Cow::Owned(channel)));
                    }
                }
            }
            _ => {}
        }

        Ok(None)
    }

    fn text_channel_permissions(
        permissions: &mut Permissions,
        user: UserId,
        guild: &GuildOrId,
        channel: Cow<'_, CachedTextChannel>,
        member: CachedMember,
    ) {
        let mut everyone_allowed = Permissions::empty();
        let mut everyone_denied = Permissions::empty();
        let mut user_allowed = Permissions::empty();
        let mut user_denied = Permissions::empty();
        let mut role_allowed = Permissions::empty();
        let mut role_denied = Permissions::empty();

        for overwrite in &channel.permission_overwrites {
            match overwrite.kind {
                PermissionOverwriteType::Member(member) => {
                    if member == user {
                        user_allowed |= overwrite.allow;
                        user_denied |= overwrite.deny;
                    }
                }
                PermissionOverwriteType::Role(role) => {
                    if role.0 == guild.id().0 {
                        everyone_allowed |= overwrite.allow;
                        everyone_denied |= overwrite.deny
                    } else if member.roles.contains(&role) {
                        role_allowed |= overwrite.allow;
                        role_denied |= overwrite.deny;
                    }
                }
            }
        }

        *permissions &= !everyone_denied;
        *permissions |= everyone_allowed;

        *permissions &= !role_denied;
        *permissions |= role_allowed;

        *permissions &= !user_denied;
        *permissions |= user_allowed;
    }
}
