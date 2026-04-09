use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "mikrotik_config_snapshots")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub mikrotik_id: Uuid,
    pub config_content: String,
    pub config_hash: String,
    pub created_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::mikrotik_clients::Entity",
        from = "Column::MikrotikId",
        to = "super::mikrotik_clients::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    MikrotikClient,
}

impl Related<super::mikrotik_clients::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::MikrotikClient.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
