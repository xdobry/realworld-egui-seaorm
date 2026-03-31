use sea_orm::entity::prelude::*;

use models::entity::users::{ActiveModel, Model};

pub enum UserCommand {
    Reload,
    Create(ActiveModel),
    Update(ActiveModel),
    Load(Uuid),
    Delete(Uuid),
}

pub enum UserResult {
    Users(Vec<Model>),
    User(Model),
}