use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TelegramBots::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(TelegramBots::Id).uuid().not_null().primary_key().default(Expr::cust("gen_random_uuid()")))
                    .col(ColumnDef::new(TelegramBots::Name).string().not_null())
                    .col(ColumnDef::new(TelegramBots::Token).string().not_null())
                    .col(ColumnDef::new(TelegramBots::ChatId).string().not_null())
                    .col(ColumnDef::new(TelegramBots::IsActive).boolean().not_null().default(true))
                    .col(ColumnDef::new(TelegramBots::Description).text().null())
                    .col(ColumnDef::new(TelegramBots::CreatedAt).timestamp().not_null().default(Expr::current_timestamp()))
                    .col(ColumnDef::new(TelegramBots::UpdatedAt).timestamp().not_null().default(Expr::current_timestamp()))
                    .col(ColumnDef::new(TelegramBots::DeletedAt).timestamp().null())
                    .col(ColumnDef::new(TelegramBots::CreatedBy).uuid().not_null())
                    .col(ColumnDef::new(TelegramBots::UpdatedBy).uuid().null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_telegram_bots_created_by")
                            .from(TelegramBots::Table, TelegramBots::CreatedBy)
                            .to(Alias::new("users"), Alias::new("id"))
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_telegram_bots_updated_by")
                            .from(TelegramBots::Table, TelegramBots::UpdatedBy)
                            .to(Alias::new("users"), Alias::new("id"))
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(TelegramBots::Table).to_owned()).await
    }
}

#[derive(Iden)]
pub enum TelegramBots {
    Table,
    Id,
    Name,
    Token,
    ChatId,
    IsActive,
    Description,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
    CreatedBy,
    UpdatedBy,
}
