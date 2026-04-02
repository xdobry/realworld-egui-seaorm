use models::entity::comments::{ActiveModel, Entity, Column, Model};
use sea_orm::Iterable;
use sea_orm::{FromQueryResult, prelude::DateTimeWithTimeZone};
use sea_orm::entity::prelude::*;
use serde::{Serialize, Deserialize};

use crate::dto::{ChangeRecord};

#[derive(FromQueryResult, Default, Clone, Serialize, Deserialize, Debug)]
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
    pub fn to_model(&self) -> Model {
        Model {
            id: self.id,
            article_id: self.article_id,
            author_id: self.author_id,
            body: self.body.clone(),
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }

    pub fn to_change_record(&self, orig: &Model) -> ChangeRecord {
        ChangeRecord::from_models::<Entity>(&self.to_model(), orig)
    }
}

#[derive(FromQueryResult, Default, Clone, Serialize, Deserialize, Debug)]
pub struct CommentArticle {
    pub id: Uuid,
    pub body: String,
    pub article_id: Uuid,
    pub author_id: Uuid,
    pub article_title: String,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}
