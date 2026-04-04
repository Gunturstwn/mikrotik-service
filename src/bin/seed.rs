use mikrotik_service::config;
use mikrotik_service::models::{roles, user_roles, users, permissions, role_permissions};
use mikrotik_service::utils::encryption::hash_password;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use uuid::Uuid;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    println!("Connecting to the database...");
    let db = config::database::connect().await;

    let role_names = vec![
        "Super Admin",
        "Admin",
        "Finance",
        "Teknisi",
        "Customer",
    ];

    println!("Seeding Roles...");
    let mut super_admin_role_id = Uuid::nil();
    let mut admin_role_id = Uuid::nil();
    let mut finance_role_id = Uuid::nil();
    let mut teknisi_role_id = Uuid::nil();
    let mut customer_role_id = Uuid::nil();

    for role_name in &role_names {
        let existing_role = roles::Entity::find()
            .filter(roles::Column::Name.eq(*role_name))
            .one(&db)
            .await
            .unwrap();

        let r_id = match existing_role {
            Some(role) => {
                println!("Role '{}' already exists.", role_name);
                role.id
            }
            None => {
                let id = Uuid::new_v4();
                let new_role = roles::ActiveModel {
                    id: Set(id),
                    name: Set(role_name.to_string()),
                    created_at: Set(chrono::Utc::now().naive_utc()),
                    updated_at: Set(chrono::Utc::now().naive_utc()),
                    ..Default::default()
                };
                new_role.insert(&db).await.unwrap();
                println!("Inserted role '{}'.", role_name);
                id
            }
        };

        match *role_name {
            "Super Admin" => super_admin_role_id = r_id,
            "Admin" => admin_role_id = r_id,
            "Finance" => finance_role_id = r_id,
            "Teknisi" => teknisi_role_id = r_id,
            "Customer" => customer_role_id = r_id,
            _ => {}
        }
    }

    println!("Seeding Super Admin User...");
    let email = "gntr4rs@gmail.com";
    let password = "numbernine9";

    let existing_user = users::Entity::find()
        .filter(users::Column::Email.eq(email))
        .one(&db)
        .await
        .unwrap();

    let user_id = match existing_user {
        Some(user) => {
            println!("User '{}' already exists.", email);
            user.id
        }
        None => {
            let id = Uuid::new_v4();
            let hashed_password = hash_password(password).expect("Failed to hash password");
            
            let new_user = users::ActiveModel {
                id: Set(id),
                name: Set("Super Administrator".to_string()),
                email: Set(email.to_string()),
                password: Set(hashed_password),
                is_verified: Set(true),
                created_at: Set(chrono::Utc::now().naive_utc()),
                updated_at: Set(chrono::Utc::now().naive_utc()),
                ..Default::default()
            };
            
            new_user.insert(&db).await.unwrap();
            println!("Inserted User '{}'.", email);
            id
        }
    };

    println!("Linking User to Super Admin Role...");
    let existing_user_role = user_roles::Entity::find()
        .filter(user_roles::Column::UserId.eq(user_id))
        .filter(user_roles::Column::RoleId.eq(super_admin_role_id))
        .one(&db)
        .await
        .unwrap();

    if existing_user_role.is_none() {
        let new_user_role = user_roles::ActiveModel {
            user_id: Set(user_id),
            role_id: Set(super_admin_role_id),
            ..Default::default()
        };
        new_user_role.insert(&db).await.unwrap();
        println!("User '{}' linked to Role 'Super Admin'.", email);
    } else {
        println!("User '{}' is already linked to Role 'Super Admin'.", email);
    }

    // ═══════════════════════════════════════════════
    //  PERMISSIONS SEEDING
    // ═══════════════════════════════════════════════
    println!("\nSeeding Permissions...");

    let permission_codes = vec![
        ("users.list",       "List all users"),
        ("users.detail",     "View user detail"),
        ("users.create",     "Register / create user"),
        ("users.update",     "Update user profile"),
        ("users.delete",     "Soft-delete user"),
        ("users.verify",     "Verify user email"),
        ("export.csv",       "Export users to CSV"),
        ("export.xlsx",      "Export users to XLSX"),
        ("audit.view",       "View audit logs"),
        ("billing.view",     "View billing data"),
        ("billing.create",   "Create billing invoice"),
        ("billing.update",   "Update billing invoice"),
        ("device.view",      "View MikroTik devices"),
        ("device.manage",    "Manage MikroTik devices"),
        ("profile.update",   "Update own profile"),
        ("profile.photo",    "Upload own photo"),
    ];

    let mut perm_ids: std::collections::HashMap<String, Uuid> = std::collections::HashMap::new();

    for (code, name) in &permission_codes {
        let existing = permissions::Entity::find()
            .filter(permissions::Column::Code.eq(*code))
            .one(&db)
            .await
            .unwrap();

        let p_id = match existing {
            Some(p) => {
                println!("  Permission '{}' exists.", code);
                p.id
            }
            None => {
                let id = Uuid::new_v4();
                let perm = permissions::ActiveModel {
                    id: Set(id),
                    name: Set(name.to_string()),
                    code: Set(code.to_string()),
                    created_at: Set(chrono::Utc::now().naive_utc()),
                    updated_at: Set(chrono::Utc::now().naive_utc()),
                };
                perm.insert(&db).await.unwrap();
                println!("  Inserted permission '{}'.", code);
                id
            }
        };
        perm_ids.insert(code.to_string(), p_id);
    }

    // ═══════════════════════════════════════════════
    //  ROLE ↔ PERMISSION ASSIGNMENTS
    // ═══════════════════════════════════════════════
    println!("\nAssigning Permissions to Roles...");

    // Super Admin → ALL permissions
    let super_admin_perms: Vec<&str> = permission_codes.iter().map(|(c, _)| *c).collect();

    // Admin → most except verify, export, audit
    let admin_perms = vec![
        "users.list", "users.detail", "users.create", "users.update",
        "billing.view", "billing.create", "billing.update",
        "device.view", "device.manage",
        "profile.update", "profile.photo",
    ];

    // Finance → billing + profile
    let finance_perms = vec![
        "billing.view", "billing.create", "billing.update",
        "users.list", "export.csv", "export.xlsx",
        "profile.update", "profile.photo",
    ];

    // Teknisi → device + profile
    let teknisi_perms = vec![
        "device.view", "device.manage",
        "users.list",
        "profile.update", "profile.photo",
    ];

    // Customer → own profile only
    let customer_perms = vec![
        "profile.update", "profile.photo",
        "billing.view",
    ];

    let role_perm_map = vec![
        (super_admin_role_id, super_admin_perms),
        (admin_role_id,       admin_perms),
        (finance_role_id,     finance_perms),
        (teknisi_role_id,     teknisi_perms),
        (customer_role_id,    customer_perms),
    ];

    for (role_id, codes) in role_perm_map {
        for code in codes {
            if let Some(&p_id) = perm_ids.get(code) {
                let exists = role_permissions::Entity::find()
                    .filter(role_permissions::Column::RoleId.eq(role_id))
                    .filter(role_permissions::Column::PermissionId.eq(p_id))
                    .one(&db)
                    .await
                    .unwrap();

                if exists.is_none() {
                    let rp = role_permissions::ActiveModel {
                        role_id: Set(role_id),
                        permission_id: Set(p_id),
                    };
                    rp.insert(&db).await.unwrap();
                }
            }
        }
    }
    println!("Role-Permission assignments completed.");

    println!("\n✅ Seeding process completed successfully!");
}

