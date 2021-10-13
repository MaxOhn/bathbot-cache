#![deny(clippy::all, nonstandard_style, rust_2018_idioms, unused, warnings)]

use std::fmt::Display;

use deadpool_redis::{Config, Pool, PoolConfig};

mod constants;
mod error;
mod fetch;
mod store;
mod util;

pub mod model;

pub use error::{CacheError, CacheResult};

pub struct Cache {
    redis: Pool,
}

impl Cache {
    pub fn new(host: impl Display, port: impl Display) -> CacheResult<Self> {
        let config = Config {
            url: Some(format!("redis://{}:{}", host, port)),
            connection: None,
            pool: Some(PoolConfig::new(4)),
        };

        let redis = config.create_pool()?;

        Ok(Self { redis })
    }
}
