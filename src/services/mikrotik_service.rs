use crate::dto::mikrotik::{MikrotikClientRequest, MikrotikClientResponse, MikrotikResourceResponse};
use crate::models::mikrotik_clients::{Entity as MikrotikClient, ActiveModel as MikrotikClientActiveModel, Model};
use crate::errors::app_error::AppError;
use crate::pool::MikrotikPool;
use sea_orm::*;
use uuid::Uuid;
use chrono::Utc;

pub struct MikrotikService;

impl MikrotikService {
    pub async fn create_client(
        db: &DatabaseConnection,
        created_by: Uuid,
        req: MikrotikClientRequest,
        aes_key: &str,
    ) -> Result<MikrotikClientResponse, AppError> {
        let mut model = Model {
            id: Uuid::new_v4(),
            name_device: req.name_device,
            host: req.host,
            username: "".to_string(), // Will be set via helper
            password: "".to_string(), // Will be set via helper
            port_winbox: None,
            port_api: None,
            port_ftp: None,
            port_ssh: req.port_ssh,
            location: req.location,
            latitude: req.latitude,
            longitude: req.longitude,
            timezone: req.timezone,
            created_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
            deleted_at: None,
            created_by,
            updated_by: None,
            deleted_by: None,
        };

        model.set_encrypted_fields(
            &req.username,
            &req.password,
            req.port_winbox.as_deref(),
            req.port_api.as_deref(),
            req.port_ftp.as_deref(),
            aes_key,
        )?;

        let active_model: MikrotikClientActiveModel = model.into();
        let result = active_model.insert(db).await?;

        Ok(Self::map_to_response(result))
    }

    pub async fn list_clients(
        db: &DatabaseConnection,
    ) -> Result<Vec<MikrotikClientResponse>, AppError> {
        let clients = MikrotikClient::find()
            .filter(crate::models::mikrotik_clients::Column::DeletedAt.is_null())
            .all(db)
            .await?;

        Ok(clients.into_iter().map(Self::map_to_response).collect())
    }

    pub async fn get_client(
        db: &DatabaseConnection,
        id: Uuid,
    ) -> Result<MikrotikClientResponse, AppError> {
        let client = MikrotikClient::find_by_id(id)
            .filter(crate::models::mikrotik_clients::Column::DeletedAt.is_null())
            .one(db)
            .await?
            .ok_or_else(|| AppError::NotFound("MikroTik client not found".to_string()))?;

        Ok(Self::map_to_response(client))
    }

    pub async fn update_client(
        db: &DatabaseConnection,
        id: Uuid,
        updated_by: Uuid,
        req: MikrotikClientRequest,
        aes_key: &str,
    ) -> Result<MikrotikClientResponse, AppError> {
        let mut client: MikrotikClientActiveModel = MikrotikClient::find_by_id(id)
            .filter(crate::models::mikrotik_clients::Column::DeletedAt.is_null())
            .one(db)
            .await?
            .ok_or_else(|| AppError::NotFound("MikroTik client not found".to_string()))?
            .into();

        client.name_device = Set(req.name_device);
        client.host = Set(req.host);
        client.port_ssh = Set(req.port_ssh);
        client.location = Set(req.location);
        client.latitude = Set(req.latitude);
        client.longitude = Set(req.longitude);
        client.timezone = Set(req.timezone);
        client.updated_at = Set(Utc::now().naive_utc());
        client.updated_by = Set(Some(updated_by));

        // Use a temporary model to leverage encryption helper
        let mut temp_model = Model {
            id: Uuid::nil(), // Not used
            name_device: "".to_string(),
            host: "".to_string(),
            username: "".to_string(),
            password: "".to_string(),
            port_winbox: None,
            port_api: None,
            port_ftp: None,
            port_ssh: None,
            location: None,
            latitude: None,
            longitude: None,
            timezone: None,
            created_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
            deleted_at: None,
            created_by: Uuid::nil(),
            updated_by: None,
            deleted_by: None,
        };

        temp_model.set_encrypted_fields(
            &req.username,
            &req.password,
            req.port_winbox.as_deref(),
            req.port_api.as_deref(),
            req.port_ftp.as_deref(),
            aes_key,
        )?;

        client.username = Set(temp_model.username);
        client.password = Set(temp_model.password);
        client.port_winbox = Set(temp_model.port_winbox);
        client.port_api = Set(temp_model.port_api);
        client.port_ftp = Set(temp_model.port_ftp);

        let result = client.update(db).await?;
        Ok(Self::map_to_response(result))
    }

    pub async fn delete_client(
        db: &DatabaseConnection,
        id: Uuid,
        deleted_by: Uuid,
    ) -> Result<(), AppError> {
        let mut client: MikrotikClientActiveModel = MikrotikClient::find_by_id(id)
            .filter(crate::models::mikrotik_clients::Column::DeletedAt.is_null())
            .one(db)
            .await?
            .ok_or_else(|| AppError::NotFound("MikroTik client not found".to_string()))?
            .into();

        client.deleted_at = Set(Some(Utc::now().naive_utc()));
        client.deleted_by = Set(Some(deleted_by));
        client.update(db).await?;

        Ok(())
    }

    pub async fn get_system_resource(
        db: &DatabaseConnection,
        pool: &MikrotikPool,
        id: Uuid,
        aes_key: &str,
    ) -> Result<MikrotikResourceResponse, AppError> {
        let device_mutex = pool.get_connection(id, db, aes_key).await?;
        let device = device_mutex.lock().await;

        // Correct command construction using CommandBuilder
        let cmd = mikrotik_rs::protocol::command::CommandBuilder::new()
            .command("/system/resource/print")
            .build();

        let mut receiver = device.send_command(cmd)
            .await
            .map_err(|e| AppError::InternalServerError(format!("MikroTik communication error: {}", e)))?;

        // Iterate through the responses until we get a Reply (!re) or a terminal response
        while let Some(result) = receiver.recv().await {
            let response = result.map_err(|e| AppError::InternalServerError(format!("MikroTik response error: {}", e)))?;
            
            match response {
                mikrotik_rs::protocol::CommandResponse::Reply(re) => {
                    let attr = re.attributes;
                    
                    return Ok(MikrotikResourceResponse {
                        uptime: attr.get("uptime").and_then(|v| v.clone()).unwrap_or_default(),
                        cpu_load: attr.get("cpu-load").and_then(|v| v.as_ref().and_then(|s| s.parse::<i32>().ok())).unwrap_or_default(),
                        free_memory: attr.get("free-memory").and_then(|v| v.as_ref().and_then(|s| s.parse::<i64>().ok())).unwrap_or_default(),
                        total_memory: attr.get("total-memory").and_then(|v| v.as_ref().and_then(|s| s.parse::<i64>().ok())).unwrap_or_default(),
                        free_hdd_space: attr.get("free-hdd-space").and_then(|v| v.as_ref().and_then(|s| s.parse::<i64>().ok())).unwrap_or_default(),
                        total_hdd_space: attr.get("total-hdd-space").and_then(|v| v.as_ref().and_then(|s| s.parse::<i64>().ok())).unwrap_or_default(),
                    });
                }
                mikrotik_rs::protocol::CommandResponse::Trap(trap) => {
                    return Err(AppError::InternalServerError(format!("MikroTik Trap: {}", trap.message)));
                }
                mikrotik_rs::protocol::CommandResponse::Fatal(fatal) => {
                    return Err(AppError::InternalServerError(format!("MikroTik Fatal: {}", fatal)));
                }
                mikrotik_rs::protocol::CommandResponse::Done(_) | mikrotik_rs::protocol::CommandResponse::Empty(_) => {
                    break;
                }
            }
        }

        Err(AppError::InternalServerError("No resource data returned from MikroTik".to_string()))
    }

    fn map_to_response(model: Model) -> MikrotikClientResponse {
        MikrotikClientResponse {
            id: model.id,
            name_device: model.name_device,
            host: model.host,
            username: "********".to_string(), // Masked
            port_ssh: model.port_ssh,
            location: model.location,
            latitude: model.latitude,
            longitude: model.longitude,
            timezone: model.timezone,
            created_at: model.created_at,
            updated_at: model.updated_at,
            created_by: model.created_by,
            updated_by: model.updated_by,
        }
    }
}
