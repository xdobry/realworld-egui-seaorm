use sea_orm::FromQueryResult;
use sea_orm::{prelude::DateTimeWithTimeZone};
use sea_orm::entity::prelude::*;
use serde::{Serialize, Deserialize};
use models::entity::tags::{Entity, Model};

use crate::dto::{ChangeRecord};

#[derive(Default, Clone, Serialize, Deserialize, Debug)]
pub struct TagUI {
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(FromQueryResult, Default, Clone, Serialize, Deserialize, Debug)]
pub struct TagListItem {
    pub id: Uuid,
    pub name: String,
    pub count: i64,
}

impl TagUI {
    pub fn from_model(model: &Model) -> Self {
        Self {
            id: model.id,
            name: model.name.clone(),
            created_at: model.created_at,
        }
    }

    pub fn to_model(&self) -> Model {
        Model {
            id: self.id,
            name: self.name.clone(),
            created_at: self.created_at,
        }
    }

    pub fn to_change_record(&self, orig: &Model) -> ChangeRecord {
        ChangeRecord::from_models::<Entity>(&self.to_model(), orig)
    }

    pub fn new () -> Self {
        let now = crate::time_now();
        Self {
            id: crate::new_uuid(),
            created_at: now,
            ..Default::default()
        }
    }

}
