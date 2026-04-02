use sea_orm::entity::prelude::*;
use serde::{Serialize, Deserialize};
use models::{entity::article_tags::Model};
use super::dto::ArticleTagUI;

#[derive(Serialize, Deserialize, Debug)]
pub enum ArticleTagCommand {
    LoadByArticleId(Uuid),
    Create(Model),
    Delete((Uuid,Uuid)),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ArticleTagResult {
    ArticleTags(Vec<ArticleTagUI>),
}