use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(InterfaceMetrics::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(InterfaceMetrics::Id).uuid().not_null().primary_key().default(Expr::cust("gen_random_uuid()")))
                    .col(ColumnDef::new(InterfaceMetrics::MikrotikId).uuid().not_null())
                    .col(ColumnDef::new(InterfaceMetrics::InterfaceName).string().not_null())
                    .col(ColumnDef::new(InterfaceMetrics::RxByte).big_integer().not_null())
                    .col(ColumnDef::new(InterfaceMetrics::TxByte).big_integer().not_null())
                    .col(ColumnDef::new(InterfaceMetrics::RxPacket).big_integer().not_null())
                    .col(ColumnDef::new(InterfaceMetrics::TxPacket).big_integer().not_null())
                    .col(ColumnDef::new(InterfaceMetrics::CapturedAt).timestamp().not_null().default(Expr::current_timestamp()))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_interface_metrics_mikrotik_id")
                            .from(InterfaceMetrics::Table, InterfaceMetrics::MikrotikId)
                            .to(Alias::new("mikrotik_clients"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::Cascade)
                    )
                    .to_owned(),
            )
            .await?;

        // Create index for performance on time-series queries
        manager
            .create_index(
                Index::create()
                    .name("idx_interface_metrics_mikrotik_interface_time")
                    .table(InterfaceMetrics::Table)
                    .col(InterfaceMetrics::MikrotikId)
                    .col(InterfaceMetrics::InterfaceName)
                    .col(InterfaceMetrics::CapturedAt)
                    .to_owned()
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(InterfaceMetrics::Table).to_owned()).await
    }
}

#[derive(Iden)]
pub enum InterfaceMetrics {
    Table,
    Id,
    MikrotikId,
    InterfaceName,
    RxByte,
    TxByte,
    RxPacket,
    TxPacket,
    CapturedAt,
}
