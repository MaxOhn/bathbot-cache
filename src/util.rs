use twilight_model::id::{GuildId, UserId};

use crate::{CacheError, CacheResult};

use super::Cache;

impl Cache {
    pub async fn is_guild_owner(&self, guild: GuildId, user: UserId) -> CacheResult<bool> {
        let guild = self.guild(guild).await?.ok_or(CacheError::MissingGuild)?;

        Ok(guild.owner_id == user)
    }
}
