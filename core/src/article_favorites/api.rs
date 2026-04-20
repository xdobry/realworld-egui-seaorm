use sea_orm::entity::prelude::*;
use serde::{Serialize, Deserialize};
use models::entity::article_favorites::Model;
use crate::article_favorites::dto::UserFavoriteUI;

use super::dto::ArticleFavoriteUI;

#[derive(Serialize, Deserialize, Debug)]
pub enum ArticleFavoriteCommand {
    LoadByArticleId(Uuid),
    LoadByUserId(Uuid),
    Create(Model),
    Delete((Uuid,Uuid)),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ArticleFavoriteResult {
    ArticleFavorites(Vec<ArticleFavoriteUI>),
    UserFavorites(Vec<UserFavoriteUI>),
    Deleted((Uuid,Uuid)),
}