use mikrotik_rs::MikrotikDevice;
use crate::models::mikrotik_clients::Model as MikrotikClient;
use crate::errors::app_error::AppError;

pub struct MikrotikConnection;

impl MikrotikConnection {
    /// Establishes a connection to the MikroTik device using decrypted credentials.
    /// Uses the port_api (encrypted) if available, otherwise defaults to 8728.
    pub async fn connect(client: &MikrotikClient, aes_key: &str) -> Result<MikrotikDevice, AppError> {
        let username = client.decrypt_username(aes_key)?;
        let password = client.decrypt_password(aes_key)?;
        
        let port = match client.decrypt_port_api(aes_key)? {
            Some(p) => p.parse::<u16>().map_err(|_| AppError::InternalServerError("Invalid API port value".to_string()))?,
            None => 8728, // Default RouterOS API port
        };

        let address = format!("{}:{}", client.host, port);
        
        // MikrotikDevice::connect expects address: &str, username: &str, password: Option<&str>
        let device = MikrotikDevice::connect(&address, &username, Some(password.as_str()))
            .await
            .map_err(|e| AppError::InternalServerError(format!("Failed to connect to MikroTik: {}", e)))?;

        Ok(device)
    }
}
