use deadpool_redis::{Pool, redis::cmd, redis::Script};
use std::time::{SystemTime, UNIX_EPOCH};
use crate::errors::app_error::AppError;

#[derive(Clone)]
pub struct RedisClient {
    pool: Pool,
}

impl RedisClient {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }

    pub async fn ping(&self) -> Result<(), AppError> {
        let mut conn = self.pool.get().await
            .map_err(|e| AppError::RedisError(e.to_string()))?;

        cmd("PING")
            .query_async::<_, String>(&mut conn)
            .await
            .map_err(|e| AppError::RedisError(e.to_string()))?;

        Ok(())
    }

    pub async fn set(&self, key: &str, value: &str, ttl_seconds: usize) -> Result<(), AppError> {
        let mut conn = self.pool.get().await
            .map_err(|e| AppError::RedisError(e.to_string()))?;

        cmd("SET")
            .arg(key)
            .arg(value)
            .arg("EX")
            .arg(ttl_seconds)
            .query_async::<_, ()>(&mut conn)
            .await
            .map_err(|e| AppError::RedisError(e.to_string()))?;

        Ok(())
    }

    pub async fn get(&self, key: &str) -> Result<Option<String>, AppError> {
        let mut conn = self.pool.get().await
            .map_err(|e| AppError::RedisError(e.to_string()))?;

        let value: Option<String> = cmd("GET")
            .arg(key)
            .query_async(&mut conn)
            .await
            .map_err(|e| AppError::RedisError(e.to_string()))?;

        Ok(value)
    }

    pub async fn del(&self, key: &str) -> Result<(), AppError> {
        let mut conn = self.pool.get().await
            .map_err(|e| AppError::RedisError(e.to_string()))?;

        cmd("DEL")
            .arg(key)
            .query_async::<_, ()>(&mut conn)
            .await
            .map_err(|e| AppError::RedisError(e.to_string()))?;

        Ok(())
    }

    pub async fn incr(&self, key: &str, ttl_seconds: usize) -> Result<i64, AppError> {
        let mut conn = self.pool.get().await
            .map_err(|e| AppError::RedisError(e.to_string()))?;

        let val: i64 = cmd("INCR")
            .arg(key)
            .query_async(&mut conn)
            .await
            .map_err(|e| AppError::RedisError(e.to_string()))?;

        if val == 1 {
            cmd("EXPIRE")
                .arg(key)
                .arg(ttl_seconds)
                .query_async::<_, ()>(&mut conn)
                .await
                .map_err(|e| AppError::RedisError(e.to_string()))?;
        }

        Ok(val)
    }

    /// Check rate limit using a Token Bucket algorithm in Lua for atomicity.
    /// Returns true if allowed, false if rate limited.
    pub async fn check_rate_limit(
        &self,
        key: &str,
        rate: f64,
        burst: f64,
    ) -> Result<bool, AppError> {
        let script = r#"
            local key = KEYS[1]
            local rate = tonumber(ARGV[1])
            local burst = tonumber(ARGV[2])
            local now = tonumber(ARGV[3])
            local requested = 1

            local state = redis.call('HMGET', key, 'last_tokens', 'last_updated')
            local last_tokens = tonumber(state[1]) or burst
            local last_updated = tonumber(state[2]) or now

            local delta = math.max(0, now - last_updated)
            local extra = delta * rate
            local current_tokens = math.min(burst, last_tokens + extra)

            if current_tokens >= requested then
                current_tokens = current_tokens - requested
                redis.call('HMSET', key, 'last_tokens', current_tokens, 'last_updated', now)
                redis.call('EXPIRE', key, math.ceil(burst / rate) + 1)
                return 1
            else
                return 0
            end
        "#;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| AppError::InternalServerError(e.to_string()))?
            .as_secs_f64();

        let mut conn = self.pool.get().await
            .map_err(|e| AppError::RedisError(e.to_string()))?;

        let res: i32 = Script::new(script)
            .key(key)
            .arg(rate)
            .arg(burst)
            .arg(now)
            .invoke_async(&mut conn)
            .await
            .map_err(|e: deadpool_redis::redis::RedisError| AppError::RedisError(e.to_string()))?;

        Ok(res == 1)
    }

    pub async fn set_ex(&self, key: &str, value: &str, ttl_seconds: usize) -> Result<(), AppError> {
        self.set(key, value, ttl_seconds).await
    }

    pub async fn ttl(&self, key: &str) -> Result<Option<u64>, AppError> {
        let mut conn = self.pool.get().await
            .map_err(|e| AppError::RedisError(e.to_string()))?;

        let val: i64 = cmd("TTL")
            .arg(key)
            .query_async(&mut conn)
            .await
            .map_err(|e| AppError::RedisError(e.to_string()))?;

        if val < 0 {
            Ok(None)
        } else {
            Ok(Some(val as u64))
        }
    }
}
