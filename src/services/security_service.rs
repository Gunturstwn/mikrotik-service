use crate::cache::redis_client::RedisClient;
use crate::errors::app_error::AppError;
use tracing::{info, warn};

pub enum SecurityStatus {
    Allowed,
    CaptchaRequired,
    Blocked(u64), // Locked for X seconds
}

pub struct SecurityService {
    redis: RedisClient,
}

impl SecurityService {
    pub fn new(redis: RedisClient) -> Self {
        Self { redis }
    }

    /// Check if an IP or Account is currently blocked.
    pub async fn check_status(&self, ip: &str, username: &str) -> Result<SecurityStatus, AppError> {
        let ip_block_key = format!("login:block:ip:{}", ip);
        let user_block_key = format!("login:block:user:{}", username);

        // 1. Check IP block
        if let Some(ttl) = self.get_ttl(&ip_block_key).await? {
            return Ok(SecurityStatus::Blocked(ttl));
        }

        // 2. Check User block
        if let Some(ttl) = self.get_ttl(&user_block_key).await? {
            return Ok(SecurityStatus::Blocked(ttl));
        }

        // 3. Check if CAPTCHA is required (e.g. > 3 failures)
        let ip_fail_key = format!("login:fail:ip:{}", ip);
        let user_fail_key = format!("login:fail:user:{}", username);

        let ip_fails = self.get_count(&ip_fail_key).await?;
        let user_fails = self.get_count(&user_fail_key).await?;

        if ip_fails >= 3 || user_fails >= 3 {
            return Ok(SecurityStatus::CaptchaRequired);
        }

        Ok(SecurityStatus::Allowed)
    }

    /// Track a failed login attempt and apply penalties.
    pub async fn track_failure(&self, ip: &str, username: &str) -> Result<u64, AppError> {
        let ip_fail_key = format!("login:fail:ip:{}", ip);
        let user_fail_key = format!("login:fail:user:{}", username);

        // Increment failure counters (valid for 24h to track patterns)
        let ip_count = self.redis.incr(&ip_fail_key, 86400).await? as u64;
        let user_count = self.redis.incr(&user_fail_key, 86400).await? as u64;

        warn!("Security: Login failure for IP: {} (count: {}) and User: {} (count: {})", ip, ip_count, username, user_count);

        let mut penalty_secs = 0;

        // Exponential Backoff Logic:
        // 10 failures -> 15m (900s)
        // 20 failures -> 1h (3600s)
        // 30 failures -> 6h (21600s)
        
        let max_count = std::cmp::max(ip_count, user_count);
        
        if max_count >= 30 {
            penalty_secs = 21600;
        } else if max_count >= 20 {
            penalty_secs = 3600;
        } else if max_count >= 10 {
            penalty_secs = 900;
        }

        if penalty_secs > 0 {
            let ip_block_key = format!("login:block:ip:{}", ip);
            let user_block_key = format!("login:block:user:{}", username);

            self.redis.set_ex(&ip_block_key, "1", penalty_secs as usize).await?;
            self.redis.set_ex(&user_block_key, "1", penalty_secs as usize).await?;
            
            info!("Security: IP {} and User {} BLOCKED for {}s", ip, username, penalty_secs);
        }

        Ok(penalty_secs)
    }

    /// Reset failure counters on successful login.
    pub async fn reset_failures(&self, ip: &str, username: &str) -> Result<(), AppError> {
        let ip_fail_key = format!("login:fail:ip:{}", ip);
        let user_fail_key = format!("login:fail:user:{}", username);
        
        let _ = self.redis.del(&ip_fail_key).await;
        let _ = self.redis.del(&user_fail_key).await;
        
        Ok(())
    }

    async fn get_count(&self, key: &str) -> Result<u64, AppError> {
        let val = self.redis.get(key).await?;
        Ok(val.and_then(|s| s.parse::<u64>().ok()).unwrap_or(0))
    }

    async fn get_ttl(&self, key: &str) -> Result<Option<u64>, AppError> {
        self.redis.ttl(key).await
    }
}
