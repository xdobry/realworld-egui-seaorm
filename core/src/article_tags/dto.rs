use sea_orm::{FromQueryResult, prelude::DateTimeWithTimeZone};
use sea_orm::entity::prelude::*;

#[derive(FromQueryResult, Default, Clone)]
pub struct ArticleTagUI {
    pub tag_id: Uuid,
    pub article_id: Uuid,
    pub created_at: DateTimeWithTimeZone,
    pub tag_name: String,
}