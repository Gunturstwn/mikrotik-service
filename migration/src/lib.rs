pub use sea_orm_migration::prelude::*;

mod m20260403_000001_create_initial_tables;
mod m20260404_172022_create_audit_logs_table;
mod m20260404_172031_add_security_columns_to_users;
mod m20260404_172033_create_export_jobs_table;
mod m20260405_000001_create_permissions_tables;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260403_000001_create_initial_tables::Migration),
            Box::new(m20260404_172022_create_audit_logs_table::Migration),
            Box::new(m20260404_172031_add_security_columns_to_users::Migration),
            Box::new(m20260404_172033_create_export_jobs_table::Migration),
            Box::new(m20260405_000001_create_permissions_tables::Migration),
        ]
    }
}
