use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(MikrotikClients::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(MikrotikClients::Id).uuid().not_null().primary_key().default(Expr::cust("gen_random_uuid()")))
                    .col(ColumnDef::new(MikrotikClients::NameDevice).string().not_null())
                    .col(ColumnDef::new(MikrotikClients::Host).string().not_null())
                    .col(ColumnDef::new(MikrotikClients::Username).string().not_null())
                    .col(ColumnDef::new(MikrotikClients::Password).string().not_null())
                    .col(ColumnDef::new(MikrotikClients::PortWinbox).string().null())
                    .col(ColumnDef::new(MikrotikClients::PortApi).string().null())
                    .col(ColumnDef::new(MikrotikClients::PortFtp).string().null())
                    .col(ColumnDef::new(MikrotikClients::PortSsh).integer().null())
                    .col(ColumnDef::new(MikrotikClients::Location).text().null())
                    .col(ColumnDef::new(MikrotikClients::Latitude).decimal_len(10, 8).null())
                    .col(ColumnDef::new(MikrotikClients::Longitude).decimal_len(11, 8).null())
                    .col(ColumnDef::new(MikrotikClients::Timezone).string().null())
                    .col(ColumnDef::new(MikrotikClients::CreatedAt).timestamp().not_null().default(Expr::current_timestamp()))
                    .col(ColumnDef::new(MikrotikClients::UpdatedAt).timestamp().not_null().default(Expr::current_timestamp()))
                    .col(ColumnDef::new(MikrotikClients::DeletedAt).timestamp().null())
                    .col(ColumnDef::new(MikrotikClients::CreatedBy).uuid().not_null())
                    .col(ColumnDef::new(MikrotikClients::UpdatedBy).uuid().null())
                    .col(ColumnDef::new(MikrotikClients::DeletedBy).uuid().null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_mikrotik_clients_created_by")
                            .from(MikrotikClients::Table, MikrotikClients::CreatedBy)
                            .to(Alias::new("users"), Alias::new("id"))
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_mikrotik_clients_updated_by")
                            .from(MikrotikClients::Table, MikrotikClients::UpdatedBy)
                            .to(Alias::new("users"), Alias::new("id"))
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_mikrotik_clients_deleted_by")
                            .from(MikrotikClients::Table, MikrotikClients::DeletedBy)
                            .to(Alias::new("users"), Alias::new("id"))
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(MikrotikClients::Table).to_owned()).await
    }
}

#[derive(Iden)]
pub enum MikrotikClients {
    Table,
    Id,
    NameDevice,
    Host,
    Username,
    Password,
    PortWinbox,
    PortApi,
    PortFtp,
    PortSsh,
    Location,
    Latitude,
    Longitude,
    Timezone,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
    CreatedBy,
    UpdatedBy,
    DeletedBy,
}
