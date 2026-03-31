use sea_orm::entity::prelude::*;

use models::{entity::article_favorites::ActiveModel};
use crate::article_favorites::dto::UserFavoriteUI;

use super::dto::ArticleFavoriteUI;

pub enum ArticleFavoriteCommand {
    LoadByArticleId(Uuid),
    LoadByUserId(Uuid),
    Create(ActiveModel),
    Delete((Uuid,Uuid)),
}

pub enum ArticleFavoriteResult {
    ArticleFavorites(Vec<ArticleFavoriteUI>),
    UserFavorites(Vec<UserFavoriteUI>),
}