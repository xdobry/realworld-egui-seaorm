use sea_orm::entity::prelude::*;
use serde::{Serialize, Deserialize};
use models::entity::tags::{ActiveModel, Model};

use crate::dto::ChangeRecord;

#[derive(Serialize, Deserialize, Debug)]
pub enum TagCommand {
    Reload,
    Create(Model),
    Update(ChangeRecord),
    Load(Uuid),
    Delete(Uuid),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum TagResult {
    Tags(Vec<Model>),
    Tag(Model),
}