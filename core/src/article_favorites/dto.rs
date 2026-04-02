use sea_orm::{FromQueryResult, prelude::DateTimeWithTimeZone};
use sea_orm::entity::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(FromQueryResult, Default, Clone, Serialize, Deserialize, Debug)]
pub struct ArticleFavoriteUI {
    pub user_id: Uuid,
    pub article_id: Uuid,
    pub created_at: DateTimeWithTimeZone,
    pub user_name: String,
}

#[derive(FromQueryResult, Default, Clone, Serialize, Deserialize, Debug)]
pub struct UserFavoriteUI {
    pub user_id: Uuid,
    pub article_id: Uuid,
    pub created_at: DateTimeWithTimeZone,
    pub article_title: String,
}