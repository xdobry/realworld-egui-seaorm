use models::entity::comments::ActiveModel;
use sea_orm::{ActiveValue, FromQueryResult, prelude::DateTimeWithTimeZone};
use sea_orm::entity::prelude::*;

#[derive(FromQueryResult, Default, Clone)]
pub struct CommentAuthor {
    pub id: Uuid,
    pub body: String,
    pub article_id: Uuid,
    pub author_id: Uuid,
    pub author_name: String,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

impl CommentAuthor {
    pub fn to_active_model(&self) -> ActiveModel {
        ActiveModel {
            id: ActiveValue::Set(self.id),
            article_id: ActiveValue::Set(self.article_id),
            author_id: ActiveValue::Set(self.author_id),
            body: ActiveValue::Set(self.body.clone()),
            created_at: ActiveValue::Set(self.created_at),
            updated_at: ActiveValue::Set(self.updated_at),
        }
    }
}

#[derive(FromQueryResult, Default, Clone)]
pub struct CommentArticle {
    pub id: Uuid,
    pub body: String,
    pub article_id: Uuid,
    pub author_id: Uuid,
    pub article_title: String,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}