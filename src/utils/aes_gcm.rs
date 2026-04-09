use aes_gcm::{
    aead::{Aead, KeyInit, Payload},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose, Engine as _};
use rand::{rngs::OsRng, RngCore};
use crate::errors::app_error::AppError;
use hex;

pub fn encrypt(data: &str, key: &str) -> Result<String, AppError> {
    let key_bytes = if key.len() == 64 {
        hex::decode(key).map_err(|e| AppError::InternalServerError(format!("Invalid AES key hex: {}", e)))?
    } else {
        key.as_bytes().to_vec()
    };

    if key_bytes.len() != 32 {
        return Err(AppError::InternalServerError("AES key must be exactly 32 bytes for AES-256".to_string()));
    }

    let cipher = Aes256Gcm::new_from_slice(&key_bytes)
        .map_err(|e| AppError::InternalServerError(format!("AES cipher Init failed: {}", e)))?;

    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, Payload { msg: data.as_bytes(), aad: &[] })
        .map_err(|e| AppError::InternalServerError(format!("AES encryption failed: {}", e)))?;

    // Prepend nonce to ciphertext
    let mut combined = nonce_bytes.to_vec();
    combined.extend_from_slice(&ciphertext);

    Ok(general_purpose::STANDARD.encode(combined))
}

pub fn decrypt(encrypted_base64: &str, key: &str) -> Result<String, AppError> {
    let key_bytes = if key.len() == 64 {
        hex::decode(key).map_err(|e| AppError::InternalServerError(format!("Invalid AES key hex: {}", e)))?
    } else {
        key.as_bytes().to_vec()
    };

    if key_bytes.len() != 32 {
        return Err(AppError::InternalServerError("AES key must be exactly 32 bytes for AES-256".to_string()));
    }

    let combined = general_purpose::STANDARD
        .decode(encrypted_base64)
        .map_err(|e| AppError::InternalServerError(format!("Base64 decode failed: {}", e)))?;

    if combined.len() < 12 {
        return Err(AppError::InternalServerError("Invalid encrypted data: too short".to_string()));
    }

    let (nonce_bytes, ciphertext) = combined.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);

    let cipher = Aes256Gcm::new_from_slice(&key_bytes)
        .map_err(|e| AppError::InternalServerError(format!("AES cipher Init failed: {}", e)))?;

    let decrypted_bytes = cipher
        .decrypt(nonce, Payload { msg: ciphertext, aad: &[] })
        .map_err(|e| AppError::InternalServerError(format!("AES decryption failed: {}", e)))?;

    String::from_utf8(decrypted_bytes)
        .map_err(|e| AppError::InternalServerError(format!("UTF-8 decode failed: {}", e)))
}
