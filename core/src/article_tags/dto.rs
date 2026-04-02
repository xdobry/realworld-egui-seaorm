use sea_orm::{FromQueryResult, prelude::DateTimeWithTimeZone};
use sea_orm::entity::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(FromQueryResult, Default, Clone, Serialize, Deserialize, Debug)]
pub struct ArticleTagUI {
    pub tag_id: Uuid,
    pub article_id: Uuid,
    pub created_at: DateTimeWithTimeZone,
    pub tag_name: String,
}