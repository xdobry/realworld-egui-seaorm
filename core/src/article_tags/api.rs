use sea_orm::entity::prelude::*;
use serde::{Serialize, Deserialize};
use models::{entity::article_tags::Model};
use crate::api::AuthContext;

use super::dto::ArticleTagUI;

#[derive(Serialize, Deserialize, Debug)]
pub enum ArticleTagCommand {
    LoadByArticleId(Uuid),
    Create(Model),
    Delete((Uuid,Uuid)),
}

impl ArticleTagCommand {
    pub fn has_access(&self, auth_context: &AuthContext) -> bool {
        match self {
            ArticleTagCommand::LoadByArticleId(_) => {
                true
            }
            _ => {
                !auth_context.is_anonymous()
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ArticleTagResult {
    ArticleTags(Vec<ArticleTagUI>),
}