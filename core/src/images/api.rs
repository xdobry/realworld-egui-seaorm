use sea_orm::entity::prelude::*;
use serde::{Serialize, Deserialize};
use models::entity::images::{Model};

use crate::{api::AuthContext};

#[derive(Serialize, Deserialize, Debug)]
pub enum ImageCommand {
    Create(Model),
    Delete(Uuid),
    Load(Uuid),
}

impl ImageCommand {
    pub fn has_access(&self, auth_context: &AuthContext) -> bool {
        match self {
            ImageCommand::Load(_) | ImageCommand::Create(_) | ImageCommand::Delete(_) => {
                true
            }
            _ => {
                auth_context.is_admin()
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ImageResult {
    Image(Model),
}