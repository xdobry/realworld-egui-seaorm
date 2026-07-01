
use sea_orm::entity::prelude::*;
use serde::{Serialize, Deserialize};

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "images")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub title: String,
    pub created_at: DateTimeWithTimeZone,
    pub mimetype: String,
    pub data: Vec<u8>,
    pub article_id: Uuid,
    #[sea_orm(belongs_to, from="article_id", to="id", on_update = "NoAction", on_delete = "Cascade" )]
    pub articles: HasOne<super::articles::Entity>,
}


impl ActiveModelBehavior for ActiveModel {}