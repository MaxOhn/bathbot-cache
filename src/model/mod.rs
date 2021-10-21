mod redis_key;
mod wrapper;

use std::{iter::FilterMap, vec::IntoIter};

pub use redis_key::RedisKey;
pub(crate) use wrapper::*;

use serde::{Deserialize, Serialize};
use twilight_model::{
    channel::permission_overwrite::PermissionOverwrite,
    guild::Permissions,
    id::{ChannelId, GuildId, RoleId, UserId},
};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum CachedChannel {
    #[serde(rename = "a")]
    PrivateThread(CachedThread),
    #[serde(rename = "b")]
    PublicThread(CachedThread),
    #[serde(rename = "c")]
    Text(CachedTextChannel),
}

impl CachedChannel {
    #[inline]
    pub const fn guild_id(&self) -> Option<GuildId> {
        match self {
            Self::PrivateThread(c) => c.guild_id,
            Self::PublicThread(c) => c.guild_id,
            Self::Text(c) => c.guild_id,
        }
    }

    #[inline]
    pub const fn id(&self) -> ChannelId {
        match self {
            Self::PrivateThread(c) => c.id,
            Self::PublicThread(c) => c.id,
            Self::Text(c) => c.id,
        }
    }

    #[inline]
    pub fn name(&self) -> &str {
        match self {
            Self::PrivateThread(c) => c.name.as_str(),
            Self::PublicThread(c) => c.name.as_str(),
            Self::Text(c) => c.name.as_str(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CachedTextChannel {
    #[serde(default, rename = "a", skip_serializing_if = "Option::is_none")]
    pub guild_id: Option<GuildId>,
    #[serde(rename = "b")]
    pub id: ChannelId,
    #[serde(rename = "c")]
    pub name: String,
    #[serde(default, rename = "d", skip_serializing_if = "Vec::is_empty")]
    pub permission_overwrites: Vec<PermissionOverwrite>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CachedThread {
    #[serde(default, rename = "a", skip_serializing_if = "Option::is_none")]
    pub guild_id: Option<GuildId>,
    #[serde(rename = "b")]
    pub id: ChannelId,
    #[serde(rename = "c")]
    pub name: String,
    #[serde(default, rename = "d", skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<ChannelId>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CachedGuild {
    #[serde(default, rename = "a", skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(rename = "b")]
    pub id: GuildId,
    #[serde(rename = "c")]
    pub name: String,
    #[serde(rename = "d")]
    pub owner_id: UserId,
}

#[derive(Clone, Default, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct CachedCurrentUser {
    #[serde(default, rename = "a", skip_serializing_if = "Option::is_none")]
    pub avatar: Option<String>,
    #[serde(rename = "b")]
    pub discriminator: String,
    #[serde(rename = "c")]
    pub id: UserId,
    #[serde(rename = "d")]
    pub name: String,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct CachedMember {
    #[serde(rename = "a")]
    pub guild_id: GuildId,
    #[serde(default, rename = "b", skip_serializing_if = "Option::is_none")]
    pub nick: Option<String>,
    #[serde(default, rename = "c", skip_serializing_if = "Vec::is_empty")]
    pub roles: Vec<RoleId>,
    #[serde(rename = "d")]
    pub user_id: UserId,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct CachedRole {
    #[serde(rename = "a")]
    pub id: RoleId,
    #[serde(rename = "b")]
    pub name: String,
    #[serde(rename = "c")]
    pub permissions: Permissions,
    #[serde(rename = "d")]
    pub position: i64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SessionInfo {
    #[serde(rename = "a")]
    pub session_id: String,
    #[serde(rename = "b")]
    pub sequence: u64,
}

pub struct IntoMemberIter {
    keys: Vec<RedisKey>,
}

impl IntoMemberIter {
    pub(crate) fn new(keys: Vec<RedisKey>) -> Self {
        Self { keys }
    }
}

impl IntoIterator for IntoMemberIter {
    type Item = UserId;

    #[allow(clippy::type_complexity)]
    type IntoIter = FilterMap<IntoIter<RedisKey>, fn(RedisKey) -> Option<UserId>>;

    fn into_iter(self) -> Self::IntoIter {
        self.keys.into_iter().filter_map(filter_member_key)
    }
}

fn filter_member_key(key: RedisKey) -> Option<UserId> {
    if let RedisKey::Member { user, .. } = key {
        Some(user)
    } else {
        None
    }
}

pub struct CacheStats {
    pub channels: usize,
    pub guilds: usize,
    pub members: usize,
    pub roles: usize,
}

pub enum MemberLookup {
    Found(CachedMember),
    NotChecked,
    NotFound,
}
