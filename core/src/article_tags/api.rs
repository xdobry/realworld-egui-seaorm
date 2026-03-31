use sea_orm::entity::prelude::*;

use models::{entity::article_tags::ActiveModel};
use super::dto::ArticleTagUI;

pub enum ArticleTagCommand {
    LoadByArticleId(Uuid),
    Create(ActiveModel),
    Delete((Uuid,Uuid)),
}

pub enum ArticleTagResult {
    ArticleTags(Vec<ArticleTagUI>),
}