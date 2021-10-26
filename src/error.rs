use deadpool_redis::{redis::RedisError, CreatePoolError, PoolError};
use serde_cbor::Error as CborError;
use thiserror::Error;

pub type CacheResult<T> = Result<T, CacheError>;

#[derive(Debug, Error)]
pub enum CacheError {
    #[error("cbor error")]
    Cbor(#[from] CborError),
    #[error("failed to create redis pool")]
    CreatePool(#[from] CreatePoolError),
    #[error("guild is not cached")]
    MissingGuild,
    #[error("redis pool error")]
    Pool(#[from] PoolError),
    #[error("redis error")]
    Redis(#[from] RedisError),
}
