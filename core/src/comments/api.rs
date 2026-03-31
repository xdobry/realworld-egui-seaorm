use sea_orm::entity::prelude::*;

use models::{entity::comments::ActiveModel};

use super::dto::CommentAuthor;
use super::dto::CommentArticle;

pub enum CommentCommand {
    LoadByArticleId(Uuid),
    LoadByUserId(Uuid),
    Create(ActiveModel),
    Delete(Uuid),
    Update(ActiveModel),
}

pub enum CommentResult {
    CommentsAuthor(Vec<CommentAuthor>),
    CommentsArticle(Vec<CommentArticle>),
}