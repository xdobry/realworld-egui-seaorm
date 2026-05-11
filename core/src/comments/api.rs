use sea_orm::entity::prelude::*;
use serde::{Serialize, Deserialize};
use models::entity::comments::Model;

use crate::api::AuthContext;
use crate::dto::ChangeRecord;

use super::dto::CommentAuthor;
use super::dto::CommentArticle;

#[derive(Serialize, Deserialize, Debug)]
pub enum CommentCommand {
    LoadByArticleId(Uuid),
    LoadByUserId(Uuid),
    Create(Model),
    Delete(Uuid),
    Update(ChangeRecord),
}

impl CommentCommand {
    pub fn has_access(&self, auth_context: &AuthContext) -> bool {
        match self {
            CommentCommand::LoadByArticleId(_) | CommentCommand::LoadByUserId(_) => {
                true
            }
            _ => {
                !auth_context.is_anonymous()
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum CommentResult {
    CommentsAuthor(Vec<CommentAuthor>),
    CommentsArticle(Vec<CommentArticle>),
}