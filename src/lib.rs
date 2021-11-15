#![deny(clippy::all, nonstandard_style, rust_2018_idioms, unused, warnings)]

use std::fmt::Display;

use deadpool_redis::{Config, Pool, PoolConfig};

mod constants;
mod error;
mod fetch;
mod store;
mod util;

pub mod model;

use model::CacheConfig;

pub use error::{CacheError, CacheResult};
use twilight_model::id::UserId;

pub struct Cache {
    redis: Pool,
    config: CacheConfig,
    bot_id: UserId,
}

impl Cache {
    pub fn new(host: impl Display, port: impl Display, bot_id: UserId) -> CacheResult<Self> {
        Self::with_config(host, port, bot_id, CacheConfig::default())
    }

    pub fn with_config(
        host: impl Display,
        port: impl Display,
        bot_id: UserId,
        config: CacheConfig,
    ) -> CacheResult<Self> {
        let redis_config = Config {
            url: Some(format!("redis://{}:{}", host, port)),
            connection: None,
            pool: Some(PoolConfig::new(4)),
        };

        let redis = redis_config.create_pool(None)?;

        Ok(Self {
            redis,
            config,
            bot_id,
        })
    }
}
