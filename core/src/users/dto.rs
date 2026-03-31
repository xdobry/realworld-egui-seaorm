use sea_orm::{ActiveValue, prelude::DateTimeWithTimeZone};
use sea_orm::entity::prelude::*;

use models::entity::users::{ActiveModel, Model};

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

    pub fn to_active_model(&self) -> ActiveModel {
        ActiveModel {
            id: ActiveValue::Set(self.id),
            username: ActiveValue::Set(self.username.clone()),
            email: ActiveValue::Set(self.email.clone()),
            password_hash: ActiveValue::Set(self.password_hash.clone()),
            bio: ActiveValue::Set(if self.bio.len()>0 {Some(self.bio.clone())} else {None}),
            image: ActiveValue::Set(if self.image.len()>0 {Some(self.image.clone())} else {None}),
            created_at: ActiveValue::Set(self.created_at),
            updated_at: ActiveValue::Set(self.updated_at),
            ..Default::default()
        }
    }

    pub fn to_active_model_update(&self, orig: &Model) -> ActiveModel {
        let mut am: ActiveModel = orig.clone().into();
        if orig.username != self.username {
            am.username = ActiveValue::Set(self.username.clone());
        }
        if orig.email != self.email {
            am.email = ActiveValue::Set(self.email.clone());
        }
        if orig.password_hash != self.password_hash {
            am.password_hash = ActiveValue::Set(self.password_hash.clone());
        }
        if self.bio.len()>0 {
            if let Some(bio) = &orig.bio {
                if *bio != self.bio {
                    am.bio = ActiveValue::Set(Some(self.bio.clone()));    
                }
            } else {
                am.bio = ActiveValue::Set(Some(self.bio.clone()));
            }
        } else {
            if orig.bio.is_some() {
                am.bio = ActiveValue::Set(None);
            }
        }
        if self.image.len()>0 {
            if let Some(image) = &orig.image {
                if *image != self.image {
                    am.image = ActiveValue::Set(Some(self.image.clone()));    
                }
            } else {
                am.image = ActiveValue::Set(Some(self.image.clone()));
            }
        } else {
            if orig.image.is_some() {
                am.image = ActiveValue::Set(None);
            }
        }
        if orig.created_at != self.created_at {
            am.created_at = ActiveValue::Set(self.created_at);
        }
        if orig.updated_at != self.updated_at {
            am.updated_at = ActiveValue::Set(self.updated_at);
        }
        am
    }

}

