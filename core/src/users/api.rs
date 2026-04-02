use sea_orm::entity::prelude::*;
use serde::{Serialize, Deserialize};
use models::entity::users::Model;
use crate::dto::ChangeRecord;


#[derive(Serialize, Deserialize, Debug)]
pub enum UserCommand {
    Reload,
    Create(Model),
    Update(ChangeRecord),
    Load(Uuid),
    Delete(Uuid),
}


#[derive(Serialize, Deserialize, Debug)]
pub enum UserResult {
    Users(Vec<Model>),
    User(Model),
}