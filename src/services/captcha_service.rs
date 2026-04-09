use async_trait::async_trait;
use serde::Deserialize;
use std::env;
use crate::errors::app_error::AppError;
use reqwest::Client;

#[async_trait]
pub trait CaptchaProvider: Send + Sync {
    async fn verify(&self, token: &str, ip: Option<&str>) -> Result<bool, AppError>;
}

pub struct TurnstileProvider {
    secret_key: String,
    client: Client,
}

impl TurnstileProvider {
    pub fn new() -> Self {
        let secret_key = env::var("TURNSTILE_SECRET_KEY")
            .unwrap_or_else(|_| "1x00000000000000000000AA".to_string()); // Default testing key
        Self {
            secret_key,
            client: Client::new(),
        }
    }
}

#[derive(Deserialize)]
struct TurnstileResponse {
    success: bool,
    #[serde(rename = "error-codes")]
    _error_codes: Option<Vec<String>>,
}

#[async_trait]
impl CaptchaProvider for TurnstileProvider {
    async fn verify(&self, token: &str, ip: Option<&str>) -> Result<bool, AppError> {
        let mut params = [
            ("secret", self.secret_key.as_str()),
            ("response", token),
        ].to_vec();

        if let Some(ip_addr) = ip {
            if ip_addr != "unknown" {
                params.push(("remoteip", ip_addr));
            }
        }

        let res = self.client.post("https://challenges.cloudflare.com/turnstile/v0/siteverify")
            .form(&params)
            .send()
            .await
            .map_err(|e| AppError::InternalServerError(format!("CAPTCHA request failed: {}", e)))?;

        let verification = res.json::<TurnstileResponse>()
            .await
            .map_err(|e| AppError::InternalServerError(format!("CAPTCHA parsing failed: {}", e)))?;

        Ok(verification.success)
    }
}

pub struct CaptchaService {
    provider: Box<dyn CaptchaProvider + Send + Sync>,
}

impl CaptchaService {
    pub fn new() -> Self {
        let provider: Box<dyn CaptchaProvider + Send + Sync> = Box::new(TurnstileProvider::new());
        Self { provider }
    }

    pub async fn verify(&self, token: &str, ip: Option<&str>) -> Result<bool, AppError> {
        self.provider.verify(token, ip).await
    }
}
