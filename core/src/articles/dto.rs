use sea_orm::{ActiveValue, FromQueryResult, prelude::DateTimeWithTimeZone};
use sea_orm::entity::prelude::*;

use models::entity::articles::{ActiveModel, Model};

#[derive(FromQueryResult, Default, Clone)]
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

    pub fn to_active_model(&self) -> ActiveModel {
        ActiveModel {
            id: ActiveValue::Set(self.id),
            slug: ActiveValue::Set(self.slug.clone()),
            title: ActiveValue::Set(self.title.clone()),
            description: ActiveValue::Set(self.description.clone()),
            body: ActiveValue::Set(self.body.clone()),
            author_id: ActiveValue::Set(self.author_id),
            created_at: ActiveValue::Set(self.created_at),
            updated_at: ActiveValue::Set(self.updated_at),
            ..Default::default()
        }
    }

    pub fn to_active_model_update(&self, orig: &ArticleUI) -> ActiveModel {
        let mut am: ActiveModel =  <models::entity::articles::ActiveModel as sea_orm::ActiveModelTrait>::default();
        am.id = ActiveValue::Set(self.id);
        if orig.slug != self.slug {
            am.slug = ActiveValue::Set(self.slug.clone());
        }
        if orig.title != self.title {
            am.title = ActiveValue::Set(self.title.clone());
        }
        if orig.description != self.description {
            am.description = ActiveValue::Set(self.description.clone());
        }
        if orig.body != self.body {
            am.body = ActiveValue::Set(self.body.clone());
        }
        if orig.author_id != self.author_id {
            am.author_id = ActiveValue::Set(self.author_id);
        }
        if orig.created_at != self.created_at {
            am.created_at = ActiveValue::Set(self.created_at);
        }
        if orig.updated_at != self.updated_at {
            am.updated_at = ActiveValue::Set(self.updated_at);
        }
        am
    }

    pub fn new() -> Self {
        let now: DateTimeWithTimeZone = chrono::Local::now().with_timezone(&chrono::Local::now().offset());
        Self {
            id: Uuid::new_v4(),
            author_id: Uuid::nil(),
            created_at: now,
            updated_at: now,
            ..Default::default()
        }
    }

}

