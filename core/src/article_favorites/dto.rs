use sea_orm::{FromQueryResult, prelude::DateTimeWithTimeZone};
use sea_orm::entity::prelude::*;

#[derive(FromQueryResult, Default, Clone)]
pub struct ArticleFavoriteUI {
    pub user_id: Uuid,
    pub article_id: Uuid,
    pub created_at: DateTimeWithTimeZone,
    pub user_name: String,
}

#[derive(FromQueryResult, Default, Clone)]
pub struct UserFavoriteUI {
    pub user_id: Uuid,
    pub article_id: Uuid,
    pub created_at: DateTimeWithTimeZone,
    pub article_title: String,
}