use crate::dto::mikrotik::{
    MikrotikClientRequest, MikrotikClientResponse, MikrotikResourceResponse, MikrotikInterfaceResponse, 
    MikrotikMonitorResponse, MikrotikTorchResponse, MikrotikConfigSnapshotResponse, MikrotikConfigViewResponse,
    MikrotikConfigDiffResponse, MikrotikConfigDiffItem, BackupAndSendResponse
};
use crate::models::mikrotik_clients::{Entity as MikrotikClient, ActiveModel as MikrotikClientActiveModel, Model};
use crate::models::mikrotik_config_snapshots::{Entity as ConfigSnapshot, ActiveModel as ConfigSnapshotActiveModel};
use mikrotik_rs::MikrotikDevice;
use crate::errors::app_error::AppError;
use crate::pool::MikrotikPool;
use sea_orm::*;
use uuid::Uuid;
use chrono::Utc;
use sha2::{Sha256, Digest};
use similar::{ChangeTag, TextDiff};

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
            username: "".to_string(),
            password: "".to_string(),
            port_winbox: None,
            port_api: None,
            port_ftp: None,
            port_ssh: None,
            location: req.location,
            latitude: req.latitude,
            longitude: req.longitude,
            timezone: req.timezone,
            telegram_bot_id: req.telegram_bot_id,
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
            req.port_ssh.as_deref(),
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
        client.host = Set(req.host.clone());
        client.location = Set(req.location);
        client.latitude = Set(req.latitude);
        client.longitude = Set(req.longitude);
        client.timezone = Set(req.timezone);
        client.telegram_bot_id = Set(req.telegram_bot_id);
        client.updated_at = Set(Utc::now().naive_utc());
        client.updated_by = Set(Some(updated_by));

        let mut temp_model = Model {
            id: Uuid::nil(),
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
            telegram_bot_id: None,
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
            req.port_ssh.as_deref(),
            aes_key,
        )?;

        client.username = Set(temp_model.username);
        client.password = Set(temp_model.password);
        client.port_winbox = Set(temp_model.port_winbox);
        client.port_api = Set(temp_model.port_api);
        client.port_ftp = Set(temp_model.port_ftp);
        client.port_ssh = Set(temp_model.port_ssh);

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
        user_id: Option<Uuid>,
    ) -> Result<MikrotikResourceResponse, AppError> {
        let device_mutex = pool.get_connection(id, db, aes_key, user_id).await?;
        let device = device_mutex.lock().await;

        let cmd = mikrotik_rs::protocol::command::CommandBuilder::new()
            .command("/system/resource/print")
            .build();

        let mut receiver = device.send_command(cmd)
            .await
            .map_err(|e| AppError::InternalServerError(format!("MikroTik communication error: {}", e)))?;

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

    pub async fn get_interfaces(
        db: &DatabaseConnection,
        pool: &MikrotikPool,
        id: Uuid,
        aes_key: &str,
        user_id: Option<Uuid>,
    ) -> Result<Vec<MikrotikInterfaceResponse>, AppError> {
        let device_mutex = pool.get_connection(id, db, aes_key, user_id).await?;
        let device = device_mutex.lock().await;

        let cmd = mikrotik_rs::protocol::command::CommandBuilder::new()
            .command("/interface/print")
            .attribute(".proplist", Some("name,default-name,type,mtu,actual-mtu,mac-address,last-link-up-time,link-downs,rx-byte,tx-byte,rx-packet,tx-packet,rx-error,tx-error,rx-drop,tx-drop,running,disabled"))
            .build();

        let mut receiver = device.send_command(cmd)
            .await
            .map_err(|e| AppError::InternalServerError(format!("MikroTik communication error: {}", e)))?;

        let mut interfaces = Vec::new();

        while let Some(result) = receiver.recv().await {
            let response = result.map_err(|e| AppError::InternalServerError(format!("MikroTik response error: {}", e)))?;
            
            match response {
                mikrotik_rs::protocol::CommandResponse::Reply(re) => {
                    let attr = re.attributes;
                    
                    let interface = MikrotikInterfaceResponse {
                        name: attr.get("name").and_then(|v| v.clone()).unwrap_or_default(),
                        default_name: attr.get("default-name").and_then(|v| v.clone()),
                        type_name: attr.get("type").and_then(|v| v.clone()),
                        mtu: attr.get("mtu").and_then(|v| v.as_ref().and_then(|s| s.parse::<i32>().ok())),
                        actual_mtu: attr.get("actual-mtu").and_then(|v| v.as_ref().and_then(|s| s.parse::<i32>().ok())),
                        mac_address: attr.get("mac-address").and_then(|v| v.clone()),
                        last_link_up_time: attr.get("last-link-up-time").and_then(|v| v.clone()),
                        link_downs: attr.get("link-downs").and_then(|v| v.as_ref().and_then(|s| s.parse::<i32>().ok())),
                        rx_byte: attr.get("rx-byte").and_then(|v| v.as_ref().and_then(|s| s.parse::<u64>().ok())),
                        tx_byte: attr.get("tx-byte").and_then(|v| v.as_ref().and_then(|s| s.parse::<u64>().ok())),
                        rx_packet: attr.get("rx-packet").and_then(|v| v.as_ref().and_then(|s| s.parse::<u64>().ok())),
                        tx_packet: attr.get("tx-packet").and_then(|v| v.as_ref().and_then(|s| s.parse::<u64>().ok())),
                        rx_error: attr.get("rx-error").and_then(|v| v.as_ref().and_then(|s| s.parse::<u64>().ok())),
                        tx_error: attr.get("tx-error").and_then(|v| v.as_ref().and_then(|s| s.parse::<u64>().ok())),
                        rx_drop: attr.get("rx-drop").and_then(|v| v.as_ref().and_then(|s| s.parse::<u64>().ok())),
                        tx_drop: attr.get("tx-drop").and_then(|v| v.as_ref().and_then(|s| s.parse::<u64>().ok())),
                        running: attr.get("running").and_then(|v| v.as_ref().map(|s| s == "true")).unwrap_or(false),
                        disabled: attr.get("disabled").and_then(|v| v.as_ref().map(|s| s == "true")).unwrap_or(false),
                    };
                    interfaces.push(interface);
                }
                mikrotik_rs::protocol::CommandResponse::Trap(trap) => {
                    return Err(AppError::InternalServerError(format!("MikroTik Trap: {}", trap.message)));
                }
                mikrotik_rs::protocol::CommandResponse::Fatal(fatal) => {
                    return Err(AppError::InternalServerError(format!("MikroTik Fatal: {}", fatal)));
                }
                mikrotik_rs::protocol::CommandResponse::Done(_) => {
                    return Ok(interfaces);
                }
                mikrotik_rs::protocol::CommandResponse::Empty(_) => {
                    // Just continue
                }
            }
        }

        Ok(interfaces)
    }

    pub async fn monitor_interfaces(
        db: DatabaseConnection,
        pool: std::sync::Arc<MikrotikPool>,
        id: Uuid,
        aes_key: String,
        user_id: Option<Uuid>,
        interfaces: Option<String>,
    ) -> Result<impl futures_util::Stream<Item = Result<Vec<MikrotikMonitorResponse>, AppError>> + use<>, AppError> {
        let device_mutex = pool.get_connection(id, &db, &aes_key, user_id).await?;
        
        let mut receiver = {
            let device = device_mutex.lock().await;
            
            let mut cmd_builder = mikrotik_rs::protocol::command::CommandBuilder::new()
                .command("/interface/monitor-traffic")
                .attribute(".proplist", Some("name,rx-bits-per-second,tx-bits-per-second"));
            
            if let Some(ifs) = &interfaces {
                cmd_builder = cmd_builder.attribute("interface", Some(ifs));
            }

            let cmd = cmd_builder.build();
            device.send_command(cmd)
                .await
                .map_err(|e| AppError::InternalServerError(format!("MikroTik communication error: {}", e)))?
        };

        let stream = async_stream::try_stream! {
            while let Some(result) = receiver.recv().await {
                let response = result.map_err(|e| AppError::InternalServerError(format!("MikroTik response error: {}", e)))?;
                
                match response {
                    mikrotik_rs::protocol::CommandResponse::Reply(re) => {
                        let attr = re.attributes;
                        let monitor = MikrotikMonitorResponse {
                            name: attr.get("name").and_then(|v| v.clone()).unwrap_or_default(),
                            rx_bits_per_second: attr.get("rx-bits-per-second").and_then(|v| v.as_ref().map(|s| Self::parse_mikrotik_rate(s))).unwrap_or_default(),
                            tx_bits_per_second: attr.get("tx-bits-per-second").and_then(|v| v.as_ref().map(|s| Self::parse_mikrotik_rate(s))).unwrap_or_default(),
                        };
                        yield vec![monitor];
                    }
                    mikrotik_rs::protocol::CommandResponse::Trap(trap) => {
                        Err(AppError::InternalServerError(format!("MikroTik Trap: {}", trap.message)))?;
                    }
                    mikrotik_rs::protocol::CommandResponse::Fatal(fatal) => {
                        Err(AppError::InternalServerError(format!("MikroTik Fatal: {}", fatal)))?;
                    }
                    mikrotik_rs::protocol::CommandResponse::Done(_) => break,
                    _ => continue,
                }
            }
        };

        Ok(stream)
    }

    pub async fn get_torch(
        db: DatabaseConnection,
        pool: std::sync::Arc<MikrotikPool>,
        id: Uuid,
        aes_key: String,
        user_id: Option<Uuid>,
        interface: String,
    ) -> Result<impl futures_util::Stream<Item = Result<Vec<MikrotikTorchResponse>, AppError>> + use<>, AppError> {
        let device_mutex = pool.get_connection(id, &db, &aes_key, user_id).await?;
        
        let mut receiver = {
            let device = device_mutex.lock().await;
            
            let cmd = mikrotik_rs::protocol::command::CommandBuilder::new()
                .command("/tool/torch")
                .attribute("interface", Some(&interface))
                .attribute("src-address", Some("0.0.0.0/0"))
                .attribute("duration", Some("60s"))
                .build();

            device.send_command(cmd)
                .await
                .map_err(|e| AppError::InternalServerError(format!("MikroTik communication error: {}", e)))?
        };

        let stream = async_stream::try_stream! {
            while let Some(result) = receiver.recv().await {
                let response = result.map_err(|e| AppError::InternalServerError(format!("MikroTik response error: {}", e)))?;
                
                match response {
                    mikrotik_rs::protocol::CommandResponse::Reply(re) => {
                        let attr = re.attributes;
                        let torch = MikrotikTorchResponse {
                            source_address: attr.get("src-address").and_then(|v| v.clone()),
                            destination_address: attr.get("dst-address").and_then(|v| v.clone()),
                            protocol: attr.get("protocol").and_then(|v| v.clone()),
                            port: attr.get("port").and_then(|v| v.clone()),
                            tx_rate: attr.get("tx-rate").and_then(|v| v.as_ref().map(|s| Self::parse_mikrotik_rate(s))).unwrap_or_default(),
                            rx_rate: attr.get("rx-rate").and_then(|v| v.as_ref().map(|s| Self::parse_mikrotik_rate(s))).unwrap_or_default(),
                        };
                        yield vec![torch];
                    }
                    mikrotik_rs::protocol::CommandResponse::Trap(trap) => {
                        Err(AppError::InternalServerError(format!("MikroTik Trap: {}", trap.message)))?;
                    }
                    mikrotik_rs::protocol::CommandResponse::Fatal(fatal) => {
                        Err(AppError::InternalServerError(format!("MikroTik Fatal: {}", fatal)))?;
                    }
                    mikrotik_rs::protocol::CommandResponse::Done(_) => break,
                    _ => continue,
                }
            }
        };

        Ok(stream)
    }

    // ─── BACKUP: Create binary .backup file via /system/backup/save ──────

    pub async fn trigger_backup(
        db: &DatabaseConnection,
        pool: &MikrotikPool,
        id: Uuid,
        aes_key: &str,
        user_id: Option<Uuid>,
        name: Option<&str>,
        password: Option<&str>,
    ) -> Result<String, AppError> {
        let device_mutex = pool.get_connection(id, db, aes_key, user_id).await?;
        let device = device_mutex.lock().await;

        let mut cmd_builder = mikrotik_rs::protocol::command::CommandBuilder::new()
            .command("/system/backup/save");

        if let Some(n) = name {
            cmd_builder = cmd_builder.attribute("name", Some(n));
        }
        if let Some(pwd) = password {
            cmd_builder = cmd_builder.attribute("password", Some(pwd));
        } else {
            cmd_builder = cmd_builder.attribute("dont-encrypt", Some("yes"));
        }

        let cmd = cmd_builder.build();
        let mut receiver = device.send_command(cmd)
            .await
            .map_err(|e| AppError::InternalServerError(format!("MikroTik communication error: {}", e)))?;

        let mut filename = String::new();
        let mut success = false;

        while let Some(result) = receiver.recv().await {
            let response = result.map_err(|e| AppError::InternalServerError(format!("MikroTik response error: {}", e)))?;
            match response {
                mikrotik_rs::protocol::CommandResponse::Reply(re) => {
                    if let Some(msg) = re.attributes.get("message").and_then(|v| v.clone()) {
                        filename = msg;
                    }
                    if let Some(name_val) = re.attributes.get("name").and_then(|v| v.clone()) {
                        filename = name_val;
                    }
                }
                mikrotik_rs::protocol::CommandResponse::Done(_) => {
                    success = true;
                    break;
                }
                mikrotik_rs::protocol::CommandResponse::Trap(trap) => {
                    return Err(AppError::InternalServerError(format!("MikroTik backup failed: {}", trap.message)));
                }
                mikrotik_rs::protocol::CommandResponse::Fatal(fatal) => {
                    return Err(AppError::InternalServerError(format!("MikroTik Fatal: {}", fatal)));
                }
                _ => continue,
            }
        }

        if success {
            Ok(filename)
        } else {
            Err(AppError::InternalServerError("Backup creation did not complete successfully".to_string()))
        }
    }

    // ─── BACKUP: Fetch config inline via /export (for .rsc send) ─────

    pub async fn fetch_current_config(
        db: &DatabaseConnection,
        pool: &MikrotikPool,
        id: Uuid,
        aes_key: &str,
        user_id: Option<Uuid>,
    ) -> Result<String, AppError> {
        let device_mutex = pool.get_connection(id, db, aes_key, user_id).await?;
        let device = device_mutex.lock().await;

        let cmd = mikrotik_rs::protocol::command::CommandBuilder::new()
            .command("/export")
            .attribute("hide-sensitive", None)
            .build();

        let mut receiver = device.send_command(cmd)
            .await
            .map_err(|e| AppError::InternalServerError(format!("MikroTik communication error: {}", e)))?;

        let mut config = String::new();

        while let Some(result) = receiver.recv().await {
            let response = result.map_err(|e| AppError::InternalServerError(format!("MikroTik response error: {}", e)))?;
            
            match response {
                mikrotik_rs::protocol::CommandResponse::Reply(re) => {
                    if let Some(msg) = re.attributes.get("").and_then(|v| v.clone()) {
                        config.push_str(&msg);
                    }
                }
                mikrotik_rs::protocol::CommandResponse::Done(_) => {
                    return Ok(config);
                }
                _ => continue,
            }
        }

        Ok(config)
    }

    // ─── BACKUP: List files on device ───────────────────────────────

    pub async fn list_backup_files(
        db: &DatabaseConnection,
        pool: &MikrotikPool,
        id: Uuid,
        aes_key: &str,
        user_id: Option<Uuid>,
    ) -> Result<Vec<crate::dto::mikrotik::BackupFileResponse>, AppError> {
        let device_mutex = pool.get_connection(id, db, aes_key, user_id).await?;
        let device = device_mutex.lock().await;

        let cmd = mikrotik_rs::protocol::command::CommandBuilder::new()
            .command("/file/print")
            .attribute(".proplist", Some("name,size,type,creation-time"))
            .build();

        let mut receiver = device.send_command(cmd)
            .await
            .map_err(|e| AppError::InternalServerError(format!("MikroTik communication error: {}", e)))?;

        let mut files = Vec::new();

        while let Some(result) = receiver.recv().await {
            let response = result.map_err(|e| AppError::InternalServerError(format!("MikroTik response error: {}", e)))?;
            match response {
                mikrotik_rs::protocol::CommandResponse::Reply(re) => {
                    let attr = re.attributes;
                    let file_type = attr.get("type").and_then(|v| v.as_deref()).unwrap_or("");
                    let name = attr.get("name").and_then(|v| v.clone()).unwrap_or_default();
                    
                    // Filter .backup and .rsc files
                    if name.ends_with(".backup") || name.ends_with(".rsc") || file_type == "backup" {
                        files.push(crate::dto::mikrotik::BackupFileResponse {
                            name: name.clone(),
                            size: attr.get("size").and_then(|v| v.as_ref().and_then(|s| s.parse::<i64>().ok())).unwrap_or(0),
                            creation_time: attr.get("creation-time").and_then(|v| v.clone()).unwrap_or_default(),
                        });
                    }
                }
                mikrotik_rs::protocol::CommandResponse::Done(_) => break,
                mikrotik_rs::protocol::CommandResponse::Trap(trap) => {
                    return Err(AppError::InternalServerError(format!("MikroTik file listing failed: {}", trap.message)));
                }
                mikrotik_rs::protocol::CommandResponse::Fatal(fatal) => {
                    return Err(AppError::InternalServerError(format!("MikroTik Fatal: {}", fatal)));
                }
                _ => continue,
            }
        }

        // Sort by creation time descending (newest first)
        files.sort_by(|a, b| b.creation_time.cmp(&a.creation_time));
        Ok(files)
    }

    // ─── BACKUP: Download file via SSH/SCP ─────────────────────────

    /// Download a file from the MikroTik device using SCP over SSH.
    /// Uses the device's stored SSH credentials and port.
    async fn download_file_via_scp(
        db: &DatabaseConnection,
        id: Uuid,
        aes_key: &str,
        filename: &str,
    ) -> Result<Vec<u8>, AppError> {
        let client = MikrotikClient::find_by_id(id)
            .filter(crate::models::mikrotik_clients::Column::DeletedAt.is_null())
            .one(db)
            .await?
            .ok_or_else(|| AppError::NotFound("Device not found".to_string()))?;

        let username = client.decrypt_username(aes_key)?;
        let password = client.decrypt_password(aes_key)?;
        let ssh_port = match client.decrypt_port_ssh(aes_key)? {
            Some(p) => p.parse::<u16>().map_err(|_| AppError::InternalServerError("Invalid SSH port value".to_string()))?,
            None => 22,
        };

        let addr = format!("{}:{}", client.host, ssh_port);
        let f_username = username.clone();
        let f_password = password.clone();
        let f_filename = filename.to_string();
        let f_host = client.host.clone();
        let f_addr = addr.clone();

        let bytes = tokio::task::spawn_blocking(move || -> Result<Vec<u8>, String> {
            use std::io::Read;
            use std::net::TcpStream;
            use std::path::Path;

            // Connect TCP
            let tcp = TcpStream::connect(&f_addr)
                .map_err(|e| format!(
                    "SSH connection failed to {}: {}",
                    f_addr, e
                ))?;

            // Create SSH session
            let mut sess = ssh2::Session::new()
                .map_err(|e| format!("Failed to create SSH session: {}", e))?;

            sess.set_tcp_stream(tcp);
            sess.handshake()
                .map_err(|e| format!("SSH handshake failed with '{}': {}", f_host, e))?;

            // Authenticate
            sess.userauth_password(&f_username, &f_password)
                .map_err(|e| format!("SSH authentication failed for '{}': {}", f_host, e))?;

            if !sess.authenticated() {
                return Err(format!("SSH authentication failed for '{}': invalid credentials", f_host));
            }

            // SCP receive the file
            let (mut channel, _stat) = sess.scp_recv(Path::new(&f_filename))
                .map_err(|e| format!(
                    "SCP download failed for '{}' from '{}': {}. Ensure SSH server and SCP are enabled on the device.",
                    f_filename, f_host, e
                ))?;

            let mut data = Vec::new();
            channel.read_to_end(&mut data)
                .map_err(|e| format!("Failed to read SCP data for '{}': {}", f_filename, e))?;

            // Drop the channel - this sends EOF and closes it automatically
            // Do NOT call wait_close() as it can fail with "channel not in EOF state"
            // on some RouterOS versions. The channel destructor handles cleanup.
            drop(channel);

            Ok(data)
        })
        .await
        .map_err(|e| AppError::InternalServerError(format!("SCP download thread panicked: {}", e)))?
        .map_err(|e| AppError::InternalServerError(e))?;
        
        if bytes.is_empty() {
            return Err(AppError::NotFound(format!("File '{}' not found or empty on device", filename)));
        }

        Ok(bytes)
    }

    // ─── BACKUP: Download file (public wrapper) ──────────────────────

    pub async fn download_backup_file(
        db: &DatabaseConnection,
        _pool: &MikrotikPool,
        id: Uuid,
        aes_key: &str,
        _user_id: Option<Uuid>,
        filename: &str,
    ) -> Result<Vec<u8>, AppError> {
        // Use FTP to download the backup file
        // RouterOS API does NOT support /file/get-contents (no such command)
        // FTP is the standard way to transfer files from MikroTik
        Self::download_file_via_scp(db, id, aes_key, filename).await
    }

    // ─── BACKUP: Delete file from device ───────────────────────────

    pub async fn delete_file(
        db: &DatabaseConnection,
        pool: &MikrotikPool,
        id: Uuid,
        aes_key: &str,
        user_id: Option<Uuid>,
        filename: &str,
    ) -> Result<(), AppError> {
        let device_mutex = pool.get_connection(id, db, aes_key, user_id).await?;
        let device = device_mutex.lock().await;

        let cmd = mikrotik_rs::protocol::command::CommandBuilder::new()
            .command("/file/remove")
            .attribute("numbers", Some(filename))
            .build();

        let mut receiver = device.send_command(cmd)
            .await
            .map_err(|e| AppError::InternalServerError(format!("MikroTik communication error: {}", e)))?;

        while let Some(result) = receiver.recv().await {
            let response = result.map_err(|e| AppError::InternalServerError(format!("MikroTik response error: {}", e)))?;
            match response {
                mikrotik_rs::protocol::CommandResponse::Done(_) => {
                    return Ok(());
                }
                mikrotik_rs::protocol::CommandResponse::Trap(trap) => {
                    return Err(AppError::InternalServerError(format!("MikroTik file remove failed: {}", trap.message)));
                }
                mikrotik_rs::protocol::CommandResponse::Fatal(fatal) => {
                    return Err(AppError::InternalServerError(format!("MikroTik Fatal: {}", fatal)));
                }
                _ => continue,
            }
        }

        Ok(())
    }

    // ─── BACKUP: Orchestration - Backup & Send to Telegram ───────────

    pub async fn backup_and_send(
        db: &DatabaseConnection,
        pool: &MikrotikPool,
        id: Uuid,
        aes_key: &str,
        user_id: Option<Uuid>,
        name: Option<&str>,
        password: Option<&str>,
        format: &crate::dto::mikrotik::BackupFormat,
        telegram_bot_id: Uuid,
        delete_after_send: bool,
    ) -> Result<BackupAndSendResponse, AppError> {
        use crate::services::telegram_service::TelegramService;

        match format {
            // ── .backup: Create file, download via SCP, send, delete ──
            crate::dto::mikrotik::BackupFormat::Backup => {
                let api_filename = Self::trigger_backup(db, pool, id, aes_key, user_id, name, password).await?;

                // RouterOS adds .backup extension automatically.
                // The API response may not always return the full filename,
                // so we construct it from the 'name' parameter if needed.
                let filename = if api_filename.is_empty() || !api_filename.ends_with(".backup") {
                    if let Some(n) = name {
                        format!("{}.backup", n)
                    } else {
                        return Err(AppError::InternalServerError(
                            "Could not determine backup filename from API response. 
                             Please provide a 'name' parameter.".to_string()
                        ));
                    }
                } else {
                    api_filename
                };

                // Brief wait for file to be written
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;

                // Download via SCP over SSH
                let contents = Self::download_file_via_scp(db, id, aes_key, &filename).await?;

                // Send to Telegram
                let send_result = TelegramService::send_document(
                    db, telegram_bot_id, &contents, &filename,
                    &format!("📦 *Backup*: `{}`\n📁 *Format*: `.backup`", filename),
                    aes_key,
                ).await;

                let telegram_success = match &send_result {
                    Ok((true, _)) => true,
                    _ => false,
                };
                let telegram_message = match &send_result {
                    Ok((true, _)) => Some("Backup berhasil dikirim ke Telegram.".to_string()),
                    Ok((false, Some(err))) => Some(format!("Telegram API: {}", err)),
                    Ok((false, None)) => Some("Telegram API menolak file (mungkin terlalu besar atau format tidak didukung).".to_string()),
                    Err(e) => Some(format!("Gagal kirim ke Telegram: {}", e)),
                };

                // Delete file if requested and successful
                let mut deleted_from_device = false;
                if delete_after_send && telegram_success {
                    match Self::delete_file(db, pool, id, aes_key, user_id, &filename).await {
                        Ok(_) => {
                            deleted_from_device = true;
                            tracing::info!("Deleted backup file '{}' from device {}", filename, id);
                        }
                        Err(e) => tracing::warn!("Failed to delete '{}' from device {}: {}", filename, id, e),
                    }
                }

                Ok(BackupAndSendResponse {
                    filename,
                    format: "backup".to_string(),
                    telegram_bot_id,
                    telegram_success,
                    telegram_message,
                    deleted_from_device,
                })
            }

            // ── .rsc: Export to file via /export file=, download via SCP, send ──
            crate::dto::mikrotik::BackupFormat::Rsc => {
                let base_name = name.unwrap_or("export");
                let rsc_filename = format!("{}.rsc", base_name);

                // Create the .rsc file on the device via /export file=basename
                // RouterOS automatically adds .rsc extension, so we use just the base name
                let device_mutex = pool.get_connection(id, db, aes_key, user_id).await?;
                let device = device_mutex.lock().await;

                let cmd = mikrotik_rs::protocol::command::CommandBuilder::new()
                    .command("/export")
                    .attribute("file", Some(base_name))
                    .attribute("hide-sensitive", Some("yes"))
                    .build();

                let mut receiver = device.send_command(cmd)
                    .await
                    .map_err(|e| AppError::InternalServerError(format!("MikroTik communication error: {}", e)))?;

                let mut export_success = false;
                while let Some(result) = receiver.recv().await {
                    let response = result.map_err(|e| AppError::InternalServerError(format!("MikroTik response error: {}", e)))?;
                    match response {
                        mikrotik_rs::protocol::CommandResponse::Done(_) => {
                            export_success = true;
                            break;
                        }
                        mikrotik_rs::protocol::CommandResponse::Trap(trap) => {
                            return Err(AppError::InternalServerError(format!("MikroTik export failed: {}", trap.message)));
                        }
                        mikrotik_rs::protocol::CommandResponse::Fatal(fatal) => {
                            return Err(AppError::InternalServerError(format!("MikroTik Fatal: {}", fatal)));
                        }
                        _ => continue,
                    }
                }
                // Drop device lock before SCP download
                drop(device);

                if !export_success {
                    return Err(AppError::InternalServerError("Export command did not complete".to_string()));
                }

                // Brief wait for file to be written
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;

                // Download via SCP over SSH
                let contents = Self::download_file_via_scp(db, id, aes_key, &rsc_filename).await?;

                // Drop the DB lock before potentially slow SCP
                // Send to Telegram
                let send_result = TelegramService::send_document(
                    db, telegram_bot_id, &contents, &rsc_filename,
                    &format!("📄 *Config Export*: `{}`\n📁 *Format*: `.rsc`", rsc_filename),
                    aes_key,
                ).await;

                let telegram_success = match &send_result {
                    Ok((true, _)) => true,
                    _ => false,
                };
                let telegram_message = match &send_result {
                    Ok((true, _)) => Some("Export berhasil dikirim ke Telegram.".to_string()),
                    Ok((false, Some(err))) => Some(format!("Telegram API: {}", err)),
                    Ok((false, None)) => Some("Telegram API menolak file.".to_string()),
                    Err(e) => Some(format!("Gagal kirim ke Telegram: {}", e)),
                };

                // Delete file from device if requested and successful
                let mut deleted_from_device = false;
                if delete_after_send && telegram_success {
                    match Self::delete_file(db, pool, id, aes_key, user_id, &rsc_filename).await {
                        Ok(_) => {
                            deleted_from_device = true;
                            tracing::info!("Deleted export file '{}' from device {}", rsc_filename, id);
                        }
                        Err(e) => tracing::warn!("Failed to delete '{}' from device {}: {}", rsc_filename, id, e),
                    }
                }

                Ok(BackupAndSendResponse {
                    filename: rsc_filename,
                    format: "rsc".to_string(),
                    telegram_bot_id,
                    telegram_success,
                    telegram_message,
                    deleted_from_device,
                })
            }
        }
    }

    // ─── CONFIGURATION SNAPSHOTS ────────────────────────────────────

    async fn enforce_snapshot_retention(db: &DatabaseConnection, mikrotik_id: Uuid) -> Result<(), AppError> {
        let count = ConfigSnapshot::find()
            .filter(crate::models::mikrotik_config_snapshots::Column::MikrotikId.eq(mikrotik_id))
            .count(db)
            .await?;

        if count > 50 {
            let excess = count - 50;
            let old_snapshots = ConfigSnapshot::find()
                .filter(crate::models::mikrotik_config_snapshots::Column::MikrotikId.eq(mikrotik_id))
                .order_by_asc(crate::models::mikrotik_config_snapshots::Column::CreatedAt)
                .limit(excess)
                .all(db)
                .await?;

            for s in old_snapshots {
                let _ = ConfigSnapshot::delete_by_id(s.id)
                    .exec(db)
                    .await;
            }
            tracing::info!("Cleaned up {} old snapshots for device {}", excess, mikrotik_id);
        }
        Ok(())
    }

    pub async fn perform_versioned_backup(
        db: &DatabaseConnection,
        pool: &MikrotikPool,
        id: Uuid,
        aes_key: &str,
        user_id: Option<Uuid>,
    ) -> Result<MikrotikConfigSnapshotResponse, AppError> {
        Self::enforce_snapshot_retention(db, id).await?;

        let content = Self::fetch_current_config(db, pool, id, aes_key, user_id).await?;
        
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        let hash_hex = hex::encode(hasher.finalize());

        let latest = ConfigSnapshot::find()
            .filter(crate::models::mikrotik_config_snapshots::Column::MikrotikId.eq(id))
            .order_by_desc(crate::models::mikrotik_config_snapshots::Column::CreatedAt)
            .one(db)
            .await?;

        if let Some(last) = latest {
            if last.config_hash == hash_hex {
                return Ok(MikrotikConfigSnapshotResponse {
                    id: last.id,
                    config_hash: last.config_hash,
                    created_at: last.created_at,
                });
            }
        }

        let new_snapshot = ConfigSnapshotActiveModel {
            id: Set(Uuid::new_v4()),
            mikrotik_id: Set(id),
            config_content: Set(content),
            config_hash: Set(hash_hex.clone()),
            created_at: Set(Utc::now().naive_utc()),
        };

        let result = new_snapshot.insert(db).await?;

        Ok(MikrotikConfigSnapshotResponse {
            id: result.id,
            config_hash: result.config_hash,
            created_at: result.created_at,
        })
    }

    pub async fn get_config_history(
        db: &DatabaseConnection,
        id: Uuid,
    ) -> Result<Vec<MikrotikConfigSnapshotResponse>, AppError> {
        let snapshots = ConfigSnapshot::find()
            .filter(crate::models::mikrotik_config_snapshots::Column::MikrotikId.eq(id))
            .order_by_desc(crate::models::mikrotik_config_snapshots::Column::CreatedAt)
            .all(db)
            .await?;

        Ok(snapshots.into_iter().map(|s| MikrotikConfigSnapshotResponse {
            id: s.id,
            config_hash: s.config_hash,
            created_at: s.created_at,
        }).collect())
    }

    pub async fn get_config_snapshot(
        db: &DatabaseConnection,
        snapshot_id: Uuid,
    ) -> Result<MikrotikConfigViewResponse, AppError> {
        let snapshot = ConfigSnapshot::find_by_id(snapshot_id)
            .one(db)
            .await?
            .ok_or_else(|| AppError::NotFound("Snapshot not found".to_string()))?;

        Ok(MikrotikConfigViewResponse {
            id: snapshot.id,
            config_content: snapshot.config_content,
            created_at: snapshot.created_at,
        })
    }

    pub async fn get_config_diff(
        db: &DatabaseConnection,
        snapshot_id_1: Uuid,
        snapshot_id_2: Uuid,
    ) -> Result<MikrotikConfigDiffResponse, AppError> {
        let s1 = ConfigSnapshot::find_by_id(snapshot_id_1)
            .one(db)
            .await?
            .ok_or_else(|| AppError::NotFound("First snapshot not found".to_string()))?;

        let s2 = ConfigSnapshot::find_by_id(snapshot_id_2)
            .one(db)
            .await?
            .ok_or_else(|| AppError::NotFound("Second snapshot not found".to_string()))?;

        let diff = TextDiff::from_lines(&s1.config_content, &s2.config_content);
        let mut diff_items = Vec::new();

        for change in diff.iter_all_changes() {
            let status = match change.tag() {
                ChangeTag::Delete => "removed",
                ChangeTag::Insert => "added",
                ChangeTag::Equal => "unchanged",
            };

            diff_items.push(MikrotikConfigDiffItem {
                status: status.to_string(),
                content: change.value().to_string(),
            });
        }

        Ok(MikrotikConfigDiffResponse { diffs: diff_items })
    }

    pub async fn check_connectivity(
        db: &DatabaseConnection,
        pool: &MikrotikPool,
        id: Uuid,
        aes_key: &str,
        user_id: Option<Uuid>,
    ) -> Result<bool, AppError> {
        match pool.get_connection(id, db, aes_key, user_id).await {
            Ok(device_mutex) => {
                let _ = Self::sync_identity(db, device_mutex.clone(), id).await;
                Ok(true)
            }
            Err(_) => Ok(false)
        }
    }

    async fn sync_identity(
        db: &DatabaseConnection,
        device_mutex: std::sync::Arc<tokio::sync::Mutex<MikrotikDevice>>,
        id: Uuid,
    ) -> Result<(), AppError> {
        let device = device_mutex.lock().await;
        
        let cmd = mikrotik_rs::protocol::command::CommandBuilder::new()
            .command("/system/identity/print")
            .build();

        let mut receiver = device.send_command(cmd)
            .await
            .map_err(|e| AppError::InternalServerError(format!("MikroTik communication error: {}", e)))?;

        if let Some(result) = receiver.recv().await {
            let response = result.map_err(|e| AppError::InternalServerError(format!("MikroTik response error: {}", e)))?;
            
            if let mikrotik_rs::protocol::CommandResponse::Reply(re) = response {
                if let Some(name) = re.attributes.get("name").and_then(|v| v.clone()) {
                    let mut client: MikrotikClientActiveModel = MikrotikClient::find_by_id(id)
                        .one(db)
                        .await?
                        .ok_or_else(|| AppError::NotFound("Client not found".to_string()))?
                        .into();
                    
                    client.name_device = Set(name);
                    client.updated_at = Set(Utc::now().naive_utc());
                    client.update(db).await?;
                }
            }
        }

        Ok(())
    }

    fn map_to_response(model: Model) -> MikrotikClientResponse {
         MikrotikClientResponse {
            id: model.id,
            name_device: model.name_device,
            host: model.host,
            username: "********".to_string(),
            port_ssh: model.port_ssh.as_ref().map(|_| "********".to_string()),
            port_winbox: model.port_winbox.as_ref().map(|_| "********".to_string()),
            port_api: model.port_api.as_ref().map(|_| "********".to_string()),
            port_ftp: model.port_ftp.as_ref().map(|_| "********".to_string()),
            location: model.location,
            latitude: model.latitude,
            longitude: model.longitude,
            timezone: model.timezone,
            telegram_bot_id: model.telegram_bot_id,
            created_at: model.created_at,
            updated_at: model.updated_at,
            created_by: model.created_by,
            updated_by: model.updated_by,
        }
    }

    fn parse_mikrotik_rate(input: &str) -> u64 {
        if input.is_empty() { return 0; }
        
        let normalized = input.to_lowercase();
        let last_char = normalized.chars().last().unwrap_or(' ');
        
        if last_char.is_ascii_digit() {
            return normalized.parse::<u64>().unwrap_or(0);
        }
        
        let numeric_part = &normalized[..normalized.len()-1];
        let base_val = numeric_part.parse::<f64>().unwrap_or(0.0);
        
        match last_char {
            'k' => (base_val * 1000.0) as u64,
            'm' => (base_val * 1000000.0) as u64,
            'g' => (base_val * 1000000000.0) as u64,
            _ => base_val as u64,
        }
    }
}
