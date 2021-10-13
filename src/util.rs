use deadpool_redis::redis::AsyncCommands;
use twilight_model::id::{GuildId, UserId};

use crate::{
    constants::{CHANNEL_KEY, GUILD_KEY, KEYS_SUFFIX, MEMBER_KEY, ROLE_KEY, USER_KEY},
    model::CacheStats,
    CacheError, CacheResult,
};

use super::Cache;

impl Cache {
    pub async fn is_guild_owner(&self, guild: GuildId, user: UserId) -> CacheResult<bool> {
        let guild = self.guild(guild).await?.ok_or(CacheError::MissingGuild)?;

        Ok(guild.owner_id == user)
    }

    pub async fn stats(&self) -> CacheResult<CacheStats> {
        let mut conn = self.redis.get().await?;

        let stats = CacheStats {
            channels: conn
                .scard(format!("{}{}", CHANNEL_KEY, KEYS_SUFFIX))
                .await?,
            guilds: conn.scard(format!("{}{}", GUILD_KEY, KEYS_SUFFIX)).await?,
            members: conn.scard(format!("{}{}", MEMBER_KEY, KEYS_SUFFIX)).await?,
            roles: conn.scard(format!("{}{}", ROLE_KEY, KEYS_SUFFIX)).await?,
            users: conn.scard(format!("{}{}", USER_KEY, KEYS_SUFFIX)).await?,
        };

        Ok(stats)
    }
}
