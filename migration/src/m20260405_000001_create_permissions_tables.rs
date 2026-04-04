use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create permissions table
        manager
            .create_table(
                Table::create()
                    .table(Permissions::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Permissions::Id).uuid().not_null().primary_key().default(Expr::cust("gen_random_uuid()")))
                    .col(ColumnDef::new(Permissions::Name).string_len(100).not_null())
                    .col(ColumnDef::new(Permissions::Code).string_len(100).not_null().unique_key())
                    .col(ColumnDef::new(Permissions::CreatedAt).timestamp().not_null().default(Expr::current_timestamp()))
                    .col(ColumnDef::new(Permissions::UpdatedAt).timestamp().not_null().default(Expr::current_timestamp()))
                    .to_owned(),
            )
            .await?;

        // Create role_permissions table
        manager
            .create_table(
                Table::create()
                    .table(RolePermissions::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(RolePermissions::RoleId).uuid().not_null())
                    .col(ColumnDef::new(RolePermissions::PermissionId).uuid().not_null())
                    .primary_key(Index::create().col(RolePermissions::RoleId).col(RolePermissions::PermissionId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_role_permissions_role")
                            .from(RolePermissions::Table, RolePermissions::RoleId)
                            .to(Roles::Table, Roles::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_role_permissions_permission")
                            .from(RolePermissions::Table, RolePermissions::PermissionId)
                            .to(Permissions::Table, Permissions::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(RolePermissions::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(Permissions::Table).to_owned()).await?;
        Ok(())
    }
}

#[derive(Iden)]
enum Permissions {
    Table,
    Id,
    Name,
    Code,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum RolePermissions {
    Table,
    RoleId,
    PermissionId,
}

#[derive(Iden)]
enum Roles {
    Table,
    Id,
}
