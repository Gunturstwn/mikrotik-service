use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(AuditLogs::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(AuditLogs::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(AuditLogs::UserId).uuid())
                    .col(ColumnDef::new(AuditLogs::Action).string().not_null())
                    .col(ColumnDef::new(AuditLogs::Method).string().not_null())
                    .col(ColumnDef::new(AuditLogs::Path).string().not_null())
                    .col(ColumnDef::new(AuditLogs::Status).integer().not_null())
                    .col(ColumnDef::new(AuditLogs::Ip).string().not_null())
                    .col(ColumnDef::new(AuditLogs::Metadata).json())
                    .col(ColumnDef::new(AuditLogs::CreatedAt).date_time().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AuditLogs::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum AuditLogs {
    Table,
    Id,
    UserId,
    Action,
    Method,
    Path,
    Status,
    Ip,
    Metadata,
    CreatedAt,
}
