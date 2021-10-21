#![deny(clippy::all, nonstandard_style, rust_2018_idioms, unused, warnings)]

use std::fmt::Display;

use deadpool_redis::{Config, Pool, PoolConfig};

mod config;
mod constants;
mod error;
mod fetch;
mod store;
mod util;

pub mod model;

pub use config::CacheConfig;
pub use error::{CacheError, CacheResult};

pub struct Cache {
    redis: Pool,
    config: CacheConfig,
}

impl Cache {
    pub fn new(host: impl Display, port: impl Display, config: CacheConfig) -> CacheResult<Self> {
        let redis_config = Config {
            url: Some(format!("redis://{}:{}", host, port)),
            connection: None,
            pool: Some(PoolConfig::new(4)),
        };

        let redis = redis_config.create_pool(None)?;

        Ok(Self { redis, config })
    }
}
