use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(MikrotikClients::Table)
                    .modify_column(ColumnDef::new(MikrotikClients::PortSsh).string().null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(MikrotikClients::Table)
                    .modify_column(ColumnDef::new(MikrotikClients::PortSsh).integer().null())
                    .to_owned(),
            )
            .await
    }
}

#[derive(Iden)]
pub enum MikrotikClients {
    Table,
    PortSsh,
}
