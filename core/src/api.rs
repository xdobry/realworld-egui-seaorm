use sea_orm::entity::prelude::*;
use serde::{Serialize, Deserialize};

use crate::{article_favorites::api::{ArticleFavoriteCommand, ArticleFavoriteResult}, article_tags::api::{ArticleTagCommand, ArticleTagResult}, articles::api::{ArticleCommand, ArticleResult}, comments::api::{CommentCommand, CommentResult}, tags::api::{TagCommand, TagResult}, user_follows::api::{UserFollowerCommand, UserFollowerResult}, users::api::{UserCommand, UserResult}};
pub enum UICommand {
    Article(ArticleCommand),
    User(UserCommand),
    Tag(TagCommand),
    ArticleTag(ArticleTagCommand),
    ArticleFavorite(ArticleFavoriteCommand),
    Comment(CommentCommand),
    UserFollower(UserFollowerCommand),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum UIResult {
    Article(ArticleResult),
    User(UserResult),
    Tag(TagResult),
    ArticleTag(ArticleTagResult),
    ArticleFavorite(ArticleFavoriteResult),
    Comment(CommentResult),
    UserFollower(UserFollowerResult),
    DbError(String),
    Created,
    Updated(Uuid),
    Deleted(Uuid),
}