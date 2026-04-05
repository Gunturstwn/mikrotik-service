use axum::extract::{Request, ConnectInfo};
use axum::http::HeaderMap;
use std::net::SocketAddr;

/// Securely extracts the client's IP address.
/// 
/// 1. If TRUST_FORWARDED_FOR is "true", it checks X-Forwarded-For and X-Real-IP.
/// 2. Otherwise, it uses ConnectInfo<SocketAddr> from the underlying connection.
/// 3. Falls back to "unknown" if no IP can be determined.
pub fn extract_ip(req: &Request) -> String {
    // 1. Check for trusted headers (only if TRUST_FORWARDED_FOR is "true")
    if is_trust_forwarded() {
        if let Some(ip) = get_ip_from_headers(req.headers()) {
            return ip;
        }
    }

    // 2. Fallback to socket IP via ConnectInfo
    req.extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|info| info.0.ip().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

/// Helper for handlers that only have access to HeaderMap (less secure than Request extension).
/// This should only be used as a last resort if ConnectInfo is not available.
pub fn extract_ip_from_headers(headers: &HeaderMap) -> String {
    if is_trust_forwarded() {
        if let Some(ip) = get_ip_from_headers(headers) {
            return ip;
        }
    }
    "unknown".to_string()
}

fn is_trust_forwarded() -> bool {
    std::env::var("TRUST_FORWARDED_FOR")
        .map(|v| v == "true")
        .unwrap_or(false)
}

fn get_ip_from_headers(headers: &HeaderMap) -> Option<String> {
    headers
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.split(',').next().unwrap_or(s).trim().to_string())
        .or_else(|| {
            headers
                .get("x-real-ip")
                .and_then(|h| h.to_str().ok())
                .map(|s| s.to_string())
        })
}
