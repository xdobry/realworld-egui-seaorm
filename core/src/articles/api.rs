use sea_orm::DbErr;
use sea_orm::entity::prelude::*;

use crate::{api::AuthContext, article_tags::dto::ArticleTagUI, articles::dto::{ArticleListItem, ArticlePopularListItem, ArticleUI}, dto::ChangeRecord};
use models::entity::{article_tags, articles::{self}};
use serde::{Serialize, Deserialize};

pub trait Api {
    async fn load_articles(&self) -> Result<Vec<ArticleListItem>, DbErr>;
    async fn update_article(&self, article: articles::ActiveModel) -> Result<(), DbErr>;
    async fn insert_article(&self, article: articles::ActiveModel) -> Result<(), DbErr>;
    async fn load_article(&self, id: Uuid) -> Result<ArticleUI, DbErr>;
    async fn delete_article(&self, id: Uuid) -> Result<(), DbErr>;
    async fn load_article_tags(&self, article_id: Uuid) -> Result<Vec<ArticleTagUI>,DbErr>;
    async fn insert_article_tags(&self, article_tag: article_tags::ActiveModel) -> Result<(),DbErr>;
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ArticleCommand {
    Reload,
    Create(articles::Model),
    Update(ChangeRecord),
    Load(Uuid),
    Delete(Uuid),
    ListAuthor(Uuid),
    ListFavorites(Uuid),
    ListFollowed(Uuid),
    ListTag(Uuid),
    ListPopular,
}

impl ArticleCommand {
    pub fn has_access(&self, auth_context: &AuthContext) -> bool {
        match self {
            ArticleCommand::Reload | ArticleCommand::Load(_) | ArticleCommand::ListAuthor(_) | ArticleCommand::ListTag(_) | ArticleCommand::ListPopular => {
                true
            }
            _ => {
                !auth_context.is_anonymous()
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ArticleResult {
    Articles(Vec<articles::Model>),
    ArticleList(Vec<ArticleListItem>),
    ArticlePopular(Vec<ArticlePopularListItem>),
    Article(ArticleUI),
}