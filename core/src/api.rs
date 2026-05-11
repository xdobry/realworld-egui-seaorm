use sea_orm::entity::prelude::*;
use serde::{Serialize, Deserialize};

use crate::{article_favorites::api::{ArticleFavoriteCommand, ArticleFavoriteResult}, article_tags::api::{ArticleTagCommand, ArticleTagResult}, articles::api::{ArticleCommand, ArticleResult}, comments::api::{CommentCommand, CommentResult}, tags::api::{TagCommand, TagResult}, user_follows::api::{UserFollowerCommand, UserFollowerResult}, users::api::{UserCommand, UserResult}};


#[derive(Serialize, Deserialize, Debug)]
pub enum UICommand {
    Article(ArticleCommand),
    User(UserCommand),
    Tag(TagCommand),
    ArticleTag(ArticleTagCommand),
    ArticleFavorite(ArticleFavoriteCommand),
    Comment(CommentCommand),
    UserFollower(UserFollowerCommand),
}

impl UICommand {
    pub fn has_access(&self, auth_context: &AuthContext) -> bool {
        match self {
            UICommand::Article(subcommand) => {
                subcommand.has_access(auth_context)
            }
            UICommand::User(subcommand) => {
                subcommand.has_access(auth_context)
            }
            UICommand::Tag(subcommand) => {
                subcommand.has_access(auth_context)
            }
            UICommand::ArticleTag(subcommand) => {
                subcommand.has_access(auth_context)
            }
            UICommand::ArticleFavorite(subcommand) => {
                subcommand.has_access(auth_context)
            }
            UICommand::Comment(subcommand) => {
                subcommand.has_access(auth_context)
            }
            UICommand::UserFollower(subcommand) => {
                subcommand.has_access(auth_context)
            }
        }
    }
}

pub struct AuthContext {
    user_id: Option<Uuid>,
    is_admin: bool,
}

impl AuthContext {
    pub fn is_anonymous(&self) -> bool {
        self.user_id.is_none()
    }
    pub fn is_admin(&self) -> bool {
        self.is_admin
    }
    pub fn new(user_id: Option<Uuid>, is_admin: bool) -> Self {
        Self {
            user_id,
            is_admin
        }
    }
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TokenClaims {
    pub user_id: Uuid,
    pub is_admin: bool,
    pub exp: u64,
}


#[derive(Serialize, Deserialize, Debug)]
pub struct RemoteMessage {
    pub token: Option<Vec<u8>>,
    pub command: UICommand,
}