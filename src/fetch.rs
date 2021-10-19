use deadpool_redis::redis::{AsyncCommands, FromRedisValue};
use hashbrown::HashMap;
use serde::de::DeserializeOwned;
use twilight_model::id::{ChannelId, GuildId, RoleId, UserId};

use crate::{
    constants::GUILD_KEYS,
    model::{
        CachedChannel, CachedCurrentUser, CachedGuild, CachedMember, CachedRole, IntoMemberIter,
        RedisKey, SessionInfo,
    },
    CacheResult,
};

use super::Cache;

type FetchResult<T> = CacheResult<Option<T>>;

impl Cache {
    #[inline]
    pub async fn channel(&self, channel: ChannelId) -> FetchResult<CachedChannel> {
        self.get(channel.into()).await
    }

    #[inline]
    pub async fn current_user(&self) -> FetchResult<CachedCurrentUser> {
        self.get(RedisKey::BotUser).await
    }

    #[inline]
    pub async fn guild(&self, guild: GuildId) -> FetchResult<CachedGuild> {
        self.get(guild.into()).await
    }

    #[inline]
    pub async fn member(&self, guild: GuildId, user: UserId) -> FetchResult<CachedMember> {
        self.get((guild, user).into()).await
    }

    #[inline]
    pub async fn members(&self, guild: GuildId) -> CacheResult<IntoMemberIter> {
        let key = format!("{}:{}", GUILD_KEYS, guild);
        let keys = self.get_members(key).await?;

        Ok(IntoMemberIter::new(keys))
    }

    #[inline]
    pub async fn role(&self, role: RoleId) -> FetchResult<CachedRole> {
        self.get(role.into()).await
    }

    #[inline]
    pub async fn shards(&self) -> FetchResult<u64> {
        self.get(RedisKey::Shards).await
    }

    #[inline]
    pub async fn sessions(&self) -> FetchResult<HashMap<String, SessionInfo>> {
        self.get(RedisKey::Sessions).await
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

        Ok(conn.smembers(key).await?)
    }
}
