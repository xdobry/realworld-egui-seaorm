use sea_orm::{ActiveValue, prelude::DateTimeWithTimeZone};
use sea_orm::entity::prelude::*;

use models::entity::tags::{ActiveModel, Model};

#[derive(Default, Clone)]
pub struct TagUI {
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTimeWithTimeZone,
}

impl TagUI {
    pub fn from_model(model: &Model) -> Self {
        Self {
            id: model.id,
            name: model.name.clone(),
            created_at: model.created_at,
        }
    }

    pub fn to_active_model(&self) -> ActiveModel {
        ActiveModel {
            id: ActiveValue::Set(self.id),
            name: ActiveValue::Set(self.name.clone()),
            created_at: ActiveValue::Set(self.created_at),
            ..Default::default()
        }
    }

    pub fn to_active_model_update(&self, orig: &Model) -> ActiveModel {
        let mut am: ActiveModel = orig.clone().into();
        if orig.name != self.name {
            am.name = ActiveValue::Set(self.name.clone());
        }
        if orig.created_at != self.created_at {
            am.created_at = ActiveValue::Set(self.created_at);
        }
        am
    }

}