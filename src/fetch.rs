use deadpool_redis::redis::{AsyncCommands, FromRedisValue};
use serde::de::DeserializeOwned;
use twilight_model::id::{ChannelId, GuildId, RoleId, UserId};

use crate::{
    constants::{GUILD_KEY, KEYS_SUFFIX},
    model::{
        CachedCurrentUser, CachedGuild, CachedGuildChannel, CachedMember, CachedRole, CachedUser,
        IntoMemberIter, RedisKey,
    },
    CacheResult,
};

use super::Cache;

type FetchResult<T> = CacheResult<Option<T>>;

impl Cache {
    pub async fn member(&self, guild: GuildId, user: UserId) -> FetchResult<CachedMember> {
        self.get((guild, user).into()).await
    }

    pub async fn user(&self, user: UserId) -> FetchResult<CachedUser> {
        self.get(user.into()).await
    }

    pub async fn guild(&self, guild: GuildId) -> FetchResult<CachedGuild> {
        self.get(guild.into()).await
    }

    pub async fn current_user(&self) -> FetchResult<CachedCurrentUser> {
        self.get(RedisKey::BotUser).await
    }

    pub async fn channel(&self, channel: ChannelId) -> FetchResult<CachedGuildChannel> {
        self.get(channel.into()).await
    }

    pub async fn role(&self, role: RoleId) -> FetchResult<CachedRole> {
        self.get(role.into()).await
    }

    pub async fn guild_members(&self, guild: GuildId) -> CacheResult<IntoMemberIter> {
        let keys = self
            .get_members::<RedisKey>(format!("{}{}:{}", GUILD_KEY, KEYS_SUFFIX, guild))
            .await?;

        Ok(IntoMemberIter::new(keys))
    }

    async fn get<T>(&self, key: RedisKey) -> FetchResult<T>
    where
        T: DeserializeOwned,
    {
        let mut conn = self.redis.get().await?;
        let res: Option<Vec<u8>> = conn.get(key).await?;
        let opt = res.map(|value| serde_cbor::from_slice(&value));

        Ok(opt.transpose()?)
    }

    pub(crate) async fn get_members<T>(&self, key: String) -> CacheResult<Vec<T>>
    where
        T: FromRedisValue,
    {
        let mut conn = self.redis.get().await?;
        let res = conn.smembers(key).await?;

        Ok(res)
    }
}
