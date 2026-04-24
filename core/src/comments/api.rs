use sea_orm::entity::prelude::*;
use serde::{Serialize, Deserialize};
use models::entity::comments::Model;

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

#[derive(Serialize, Deserialize, Debug)]
pub enum CommentResult {
    CommentsAuthor(Vec<CommentAuthor>),
    CommentsArticle(Vec<CommentArticle>),
}