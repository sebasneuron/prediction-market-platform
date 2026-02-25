use crate::{
    log_info,
    message_pack_helper::{deserialize_from_message_pack, serialize_to_message_pack},
    redis::keys::RedisKey,
};
use deadpool_redis::{Config, Pool, Runtime, redis::AsyncCommands};
use serde::{Serialize, de::DeserializeOwned};
use std::future::Future;

pub mod keys;

#[derive(Clone, Debug)]
pub struct RedisHelper {
    pool: Pool,
    cache_expiry: u64,
}

impl RedisHelper {
    pub async fn new(url: &str, cache_expiry: u64) -> Result<Self, Box<dyn std::error::Error>> {
        let cfg = Config::from_url(url);
        let pool = cfg.create_pool(Some(Runtime::Tokio1))?;
        Ok(RedisHelper { pool, cache_expiry })
    }

    /// Retrieves data from the cache or computes it using the provided callback.
    ///
    /// If the data is not found in the cache, it will call the callback to get
    ///
    /// It uses MessagePack for serialization and deserialization.
    ///
    /// Parameters
    /// - `key`: The key to retrieve data from the cache.
    /// - `callback`: A function that computes the data if it's not found in the cache
    /// - `cache_expiry`: Optional cache expiry time in seconds. If not provided, uses the default expiry set during initialization.
    pub async fn get_or_set_cache<T, F, Fut>(
        &self,
        key: RedisKey,
        callback: F,
        cache_expiry: Option<u64>,
    ) -> Result<T, Box<dyn std::error::Error>>
    where
        T: DeserializeOwned + Serialize,
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T, Box<dyn std::error::Error>>>,
    {
        let mut conn = self.pool.get().await?;

        if let Ok(data) = conn.get::<_, Vec<u8>>(key.to_string()).await {
            if !data.is_empty() {
                match deserialize_from_message_pack(&data) {
                    Ok(decoded) => {
                        log_info!("Cache hit for key: {}", key);
                        return Ok(decoded);
                    }
                    Err(e) => {
                        log_info!("Cache corruption detected for key: {}, error: {}", key, e);
                        let _: () = conn.del(key.to_string()).await.unwrap_or(());
                    }
                }
            }
        }

        let fresh_data = callback().await?;
        if let Some(true) = is_empty_array(&fresh_data) {
            log_info!("Data for key: {} is an empty array, skipping cache", key);
            return Ok(fresh_data);
        }

        let expiry = cache_expiry.unwrap_or(self.cache_expiry);
        match serialize_to_message_pack(&fresh_data) {
            Ok(encoded) => {
                if !encoded.is_empty() {
                    if let Err(e) = conn
                        .set_ex::<&str, Vec<u8>, String>(&key.to_string(), encoded, expiry)
                        .await
                    {
                        log_info!("Failed to set cache for key: {}, error: {}", key, e);
                    } else {
                        log_info!("Cache miss, set new value for key: {}", key);
                    }
                }
            }
            Err(e) => {
                log_info!("Failed to serialize data for key: {}, error: {}", key, e);
            }
        }

        Ok(fresh_data)
    }

    pub async fn clear_cache(&self, key: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut conn = self.pool.get().await?;
        let _: () = conn.del(key).await?;
        log_info!("Cleared cache for key: {}", key);
        Ok(())
    }

    pub async fn is_cache_valid<T>(&self, key: &str) -> bool
    where
        T: DeserializeOwned,
    {
        let mut conn = match self.pool.get().await {
            Ok(conn) => conn,
            Err(_) => return false,
        };

        if let Ok(data) = conn.get::<_, Vec<u8>>(key).await {
            if !data.is_empty() {
                return deserialize_from_message_pack::<T>(&data).is_ok();
            }
        }
        false
    }
}

fn is_empty_array<T: ?Sized>(data: &T) -> Option<bool>
where
    T: Serialize,
{
    let mut buf = Vec::new();
    if serde_json::to_writer(&mut buf, data).is_ok() {
        if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&buf) {
            if let serde_json::Value::Array(arr) = json {
                return Some(arr.is_empty());
            }
        }
    }
    None
}
