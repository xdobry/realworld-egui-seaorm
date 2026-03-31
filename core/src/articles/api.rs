use sea_orm::DbErr;
use sea_orm::entity::prelude::*;

use crate::{article_tags::dto::ArticleTagUI, articles::dto::ArticleUI};
use models::entity::{article_tags, articles::{self}};

pub trait Api {
    async fn load_articles(&self) -> Result<Vec<articles::Model>, DbErr>;
    async fn update_article(&self, article: articles::ActiveModel) -> Result<(), DbErr>;
    async fn insert_article(&self, article: articles::ActiveModel) -> Result<(), DbErr>;
    async fn load_article(&self, id: Uuid) -> Result<ArticleUI, DbErr>;
    async fn delete_article(&self, id: Uuid) -> Result<(), DbErr>;
    async fn load_article_tags(&self, article_id: Uuid) -> Result<Vec<ArticleTagUI>,DbErr>;
    async fn insert_article_tags(&self, article_tag: article_tags::ActiveModel) -> Result<(),DbErr>;
}

pub enum ArticleCommand {
    Reload,
    Create(articles::ActiveModel),
    Update(articles::ActiveModel),
    Load(Uuid),
    Delete(Uuid),
}

pub enum ArticleResult {
    Articles(Vec<articles::Model>),
    Article(ArticleUI),
}