use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ExportJobs::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(ExportJobs::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(ExportJobs::UserId).uuid().not_null())
                    .col(ColumnDef::new(ExportJobs::ExportType).string().not_null())
                    .col(ColumnDef::new(ExportJobs::Status).string().not_null())
                    .col(ColumnDef::new(ExportJobs::FilePath).string())
                    .col(ColumnDef::new(ExportJobs::Error).text())
                    .col(ColumnDef::new(ExportJobs::CreatedAt).date_time().not_null())
                    .col(ColumnDef::new(ExportJobs::FinishedAt).date_time())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ExportJobs::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum ExportJobs {
    Table,
    Id,
    UserId,
    ExportType,
    Status,
    FilePath,
    Error,
    CreatedAt,
    FinishedAt,
}
