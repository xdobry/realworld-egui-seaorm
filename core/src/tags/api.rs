use sea_orm::entity::prelude::*;

use models::entity::tags::{ActiveModel, Model};

pub enum TagCommand {
    Reload,
    Create(ActiveModel),
    Update(ActiveModel),
    Load(Uuid),
    Delete(Uuid),
}

pub enum TagResult {
    Tags(Vec<Model>),
    Tag(Model),
}