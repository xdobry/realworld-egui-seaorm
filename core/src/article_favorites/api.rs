use sea_orm::entity::prelude::*;
use serde::{Serialize, Deserialize};
use models::entity::article_favorites::Model;
use crate::{api::AuthContext, article_favorites::dto::UserFavoriteUI};

use super::dto::ArticleFavoriteUI;

#[derive(Serialize, Deserialize, Debug)]
pub enum ArticleFavoriteCommand {
    LoadByArticleId(Uuid),
    LoadByUserId(Uuid),
    Create(Model),
    Delete((Uuid,Uuid)),
}

impl ArticleFavoriteCommand {
    pub fn has_access(&self, auth_context: &AuthContext) -> bool {
        match self {
            ArticleFavoriteCommand::LoadByArticleId(_) | ArticleFavoriteCommand::LoadByUserId(_) => {
                true
            }
            _ => {
                !auth_context.is_anonymous()
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ArticleFavoriteResult {
    ArticleFavorites(Vec<ArticleFavoriteUI>),
    UserFavorites(Vec<UserFavoriteUI>),
    Deleted((Uuid,Uuid)),
}