use sea_orm::{FromQueryResult, prelude::DateTimeWithTimeZone};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use models::entity::{articles::Entity, articles::Model};

use crate::dto::{ChangeRecord};

#[derive(FromQueryResult, Default, Clone, Serialize, Deserialize, Debug)]
pub struct ArticleUI {
    pub id: Uuid,
    pub slug: String,
    pub title: String,
    pub description: String,
    pub body: String,
    pub author_id: Uuid,
    pub author_label: String,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(FromQueryResult, Default, Clone, Serialize, Deserialize, Debug)]
pub struct ArticleListItem {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub author_id: Uuid,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(FromQueryResult, Default, Clone, Serialize, Deserialize, Debug)]
pub struct ArticlePopularListItem {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub author_id: Uuid,
    pub created_at: DateTimeWithTimeZone,
    pub count: i64,
}

impl ArticleUI {
    pub fn from_model(model: &Model, author_label: String) -> Self {
        Self {
            id: model.id,
            slug: model.slug.clone(),
            title: model.title.clone(),
            description: model.description.clone(),
            body: model.body.clone(),
            author_id: model.author_id,
            author_label: author_label,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }

    pub fn to_model(&self) -> Model {
        Model {
            id: self.id,
            slug: self.slug.clone(),
            title: self.title.clone(),
            description: self.description.clone(),
            body: self.body.clone(),
            author_id: self.author_id,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }

    pub fn to_change_record(&self, orig: &ArticleUI) -> ChangeRecord {
        ChangeRecord::from_models::<Entity>(&self.to_model(), &orig.to_model())
    }

    pub fn new(author_id: Uuid) -> Self {
        let now: DateTimeWithTimeZone = chrono::Local::now().with_timezone(&chrono::Local::now().offset());
        Self {
            id: Uuid::new_v4(),
            author_id,
            created_at: now,
            updated_at: now,
            ..Default::default()
        }
    }

}
