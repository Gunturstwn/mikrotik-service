use deadpool_redis::{Pool, redis::cmd};
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
}
