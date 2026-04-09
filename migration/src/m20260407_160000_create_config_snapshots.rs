use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(MikrotikConfigSnapshots::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(MikrotikConfigSnapshots::Id).uuid().not_null().primary_key().default(Expr::cust("gen_random_uuid()")))
                    .col(ColumnDef::new(MikrotikConfigSnapshots::MikrotikId).uuid().not_null())
                    .col(ColumnDef::new(MikrotikConfigSnapshots::ConfigContent).text().not_null())
                    .col(ColumnDef::new(MikrotikConfigSnapshots::ConfigHash).string().not_null())
                    .col(ColumnDef::new(MikrotikConfigSnapshots::CreatedAt).timestamp().not_null().default(Expr::current_timestamp()))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_mikrotik_config_snapshots_mikrotik_id")
                            .from(MikrotikConfigSnapshots::Table, MikrotikConfigSnapshots::MikrotikId)
                            .to(Alias::new("mikrotik_clients"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::Cascade)
                    )
                    .to_owned(),
            )
            .await?;

        // Composite index for fast historical lookup per device
        manager
            .create_index(
                Index::create()
                    .name("idx_mikrotik_config_snapshots_client_time")
                    .table(MikrotikConfigSnapshots::Table)
                    .col(MikrotikConfigSnapshots::MikrotikId)
                    .col(MikrotikConfigSnapshots::CreatedAt)
                    .to_owned()
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(MikrotikConfigSnapshots::Table).to_owned()).await
    }
}

#[derive(Iden)]
pub enum MikrotikConfigSnapshots {
    Table,
    Id,
    MikrotikId,
    ConfigContent,
    ConfigHash,
    CreatedAt,
}
