use std::fmt;

use deadpool_redis::redis::{
    ErrorKind, FromRedisValue, RedisError, RedisResult, RedisWrite, ToRedisArgs, Value,
};
use twilight_model::{
    guild::Member,
    id::{ChannelId, GuildId, RoleId, UserId},
};

use crate::constants::{
    BOT_USER_KEY, CHANNEL_KEY, GUILD_KEY, MEMBER_KEY, ROLE_KEY, SESSIONS_KEY, SHARDS_KEY,
};

use super::{BasicGuildChannel, CachedChannel, MemberWrapper};

#[derive(Copy, Clone, Debug)]
pub enum RedisKey {
    BotUser,
    Channel {
        guild: Option<GuildId>,
        channel: ChannelId,
    },
    Guild {
        guild: GuildId,
    },
    Member {
        guild: GuildId,
        user: UserId,
    },
    Role {
        guild: Option<GuildId>,
        role: RoleId,
    },
    Sessions,
    Shards,
}

impl RedisKey {
    pub(crate) fn user_id(&self) -> Option<UserId> {
        if let RedisKey::Member { user, .. } = self {
            Some(*user)
        } else {
            None
        }
    }
}

impl fmt::Display for RedisKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BotUser => f.write_str(BOT_USER_KEY),
            Self::Channel { channel, .. } => write!(f, "{}:{}", CHANNEL_KEY, channel),
            Self::Guild { guild } => write!(f, "{}:{}", GUILD_KEY, guild),
            Self::Member { guild, user } => write!(f, "{}:{}:{}", MEMBER_KEY, guild, user),
            Self::Role { role, .. } => write!(f, "{}:{}", ROLE_KEY, role),
            Self::Sessions => f.write_str(SESSIONS_KEY),
            Self::Shards => f.write_str(SHARDS_KEY),
        }
    }
}

impl ToRedisArgs for RedisKey {
    fn write_redis_args<W: ?Sized + RedisWrite>(&self, out: &mut W) {
        out.write_arg_fmt(self)
    }
}

impl From<&CachedChannel> for RedisKey {
    fn from(channel: &CachedChannel) -> Self {
        Self::Channel {
            guild: channel.guild_id(),
            channel: channel.id(),
        }
    }
}

impl<'c> From<&BasicGuildChannel<'c>> for RedisKey {
    fn from(channel: &BasicGuildChannel<'c>) -> Self {
        Self::Channel {
            guild: channel.guild_id(),
            channel: channel.id(),
        }
    }
}

impl From<&Member> for RedisKey {
    fn from(member: &Member) -> Self {
        Self::Member {
            guild: member.guild_id,
            user: member.user.id,
        }
    }
}

impl<'m> From<&MemberWrapper<'m>> for RedisKey {
    fn from(member: &MemberWrapper<'m>) -> Self {
        Self::Member {
            guild: member.0.guild_id,
            user: member.0.user.id,
        }
    }
}

impl From<(GuildId, RoleId)> for RedisKey {
    fn from((guild, role): (GuildId, RoleId)) -> Self {
        Self::Role {
            guild: Some(guild),
            role,
        }
    }
}

impl From<RoleId> for RedisKey {
    fn from(role: RoleId) -> Self {
        Self::Role { guild: None, role }
    }
}

impl From<ChannelId> for RedisKey {
    fn from(channel: ChannelId) -> Self {
        Self::Channel {
            guild: None,
            channel,
        }
    }
}

impl From<GuildId> for RedisKey {
    fn from(guild: GuildId) -> Self {
        Self::Guild { guild }
    }
}

impl From<(GuildId, UserId)> for RedisKey {
    fn from((guild, user): (GuildId, UserId)) -> Self {
        Self::Member { guild, user }
    }
}

impl FromRedisValue for RedisKey {
    fn from_redis_value(v: &Value) -> RedisResult<Self> {
        if let Value::Data(data) = v {
            let s = std::str::from_utf8(data).map_err(|_| {
                let kind = ErrorKind::ResponseError;
                let description = "Response was invalid utf8 data";

                RedisError::from((kind, description))
            })?;

            let mut split = s.split(':');

            match split.next() {
                Some(CHANNEL_KEY) => {
                    let parse = split
                        .next()
                        .map(str::parse)
                        .map(|res| res.map(ChannelId))
                        .filter(|_| split.next().is_none());

                    if let Some(Ok(channel)) = parse {
                        let key = RedisKey::Channel {
                            guild: None,
                            channel,
                        };

                        return Ok(key);
                    }
                }
                Some(GUILD_KEY) => {
                    let parse = split
                        .next()
                        .map(str::parse)
                        .map(|res| res.map(GuildId))
                        .filter(|_| split.next().is_none());

                    if let Some(Ok(guild)) = parse {
                        return Ok(RedisKey::Guild { guild });
                    }
                }
                Some(MEMBER_KEY) => {
                    let guild = split.next().map(str::parse).map(|res| res.map(GuildId));

                    let user = split
                        .next()
                        .map(str::parse)
                        .map(|res| res.map(UserId))
                        .filter(|_| split.next().is_none());

                    if let (Some(Ok(guild)), Some(Ok(user))) = (guild, user) {
                        return Ok(RedisKey::Member { guild, user });
                    }
                }
                Some(ROLE_KEY) => {
                    let parse = split
                        .next()
                        .map(str::parse)
                        .map(|res| res.map(RoleId))
                        .filter(|_| split.next().is_none());

                    if let Some(Ok(role)) = parse {
                        return Ok(RedisKey::Role { guild: None, role });
                    }
                }
                _ => {}
            }

            let kind = ErrorKind::TypeError;
            let description = "Response string was of incompatible format";
            let detail = format!(
                r#"Response string could not be parsed as RedisKey (response was "{}")"#,
                s
            );

            Err((kind, description, detail).into())
        } else {
            let kind = ErrorKind::TypeError;
            let description = "Response was of incompatible type";
            let detail = format!(
                "Response type not RedisKey compatible (response was {:?})",
                v
            );

            Err((kind, description, detail).into())
        }
    }
}
