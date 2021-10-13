use std::{error::Error, fmt};

use deadpool_redis::{redis::RedisError, CreatePoolError, PoolError};
use serde_cbor::Error as CborError;

pub type CacheResult<T> = Result<T, CacheError>;

#[derive(Debug)]
pub enum CacheError {
    Cbor(CborError),
    CreatePool(CreatePoolError),
    MissingGuild,
    ParseRedisKey(String),
    Pool(PoolError),
    Redis(RedisError),
}

impl fmt::Display for CacheError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cbor(_) => f.write_str("cbor error"),
            Self::CreatePool(_) => f.write_str("failed to create redis pool"),
            Self::MissingGuild => f.write_str("guild is not cached"),
            Self::ParseRedisKey(key) => write!(f, "failed to parse `{}` into RedisKey", key),
            Self::Pool(_) => f.write_str("redis pool error"),
            Self::Redis(_) => f.write_str("redis error"),
        }
    }
}

impl Error for CacheError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Cbor(e) => Some(e),
            Self::CreatePool(e) => Some(e),
            Self::MissingGuild => None,
            Self::ParseRedisKey(_) => None,
            Self::Pool(e) => Some(e),
            Self::Redis(e) => Some(e),
        }
    }
}

impl From<CborError> for CacheError {
    fn from(e: CborError) -> Self {
        Self::Cbor(e)
    }
}

impl From<CreatePoolError> for CacheError {
    fn from(e: CreatePoolError) -> Self {
        Self::CreatePool(e)
    }
}

impl From<PoolError> for CacheError {
    fn from(e: PoolError) -> Self {
        Self::Pool(e)
    }
}

impl From<RedisError> for CacheError {
    fn from(e: RedisError) -> Self {
        Self::Redis(e)
    }
}
