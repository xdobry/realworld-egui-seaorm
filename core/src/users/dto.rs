use models::entity::users::{ActiveModel, Entity, Column};
use sea_orm::Iterable;
use sea_orm::{prelude::DateTimeWithTimeZone};
use sea_orm::entity::prelude::*;

use models::entity::users::{Model};

use crate::dto::ChangeRecord;

#[derive(Default, Clone)]
pub struct UserUI {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub bio: String,
    pub image: String,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

impl UserUI {
    pub fn from_model(model: &Model) -> Self {
        Self {
            id: model.id,
            username: model.username.clone(),
            email: model.email.clone(),
            password_hash: model.password_hash.clone(),
            bio: model.bio.clone().unwrap_or(String::new()),
            image: model.image.clone().unwrap_or(String::new()),
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }

    pub fn to_model(&self) -> Model {
        Model {
            id: self.id,
            username: self.username.clone(),
            email: self.email.clone(),
            password_hash: self.password_hash.clone(),
            bio: if self.bio.len()>0 {Some(self.bio.clone())} else {None},
            image: if self.image.len()>0 {Some(self.image.clone())} else {None},
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }

    pub fn to_change_record(&self, orig: &Model) -> ChangeRecord {
        ChangeRecord::from_models::<Entity>(&self.to_model(), orig)
    }

}


