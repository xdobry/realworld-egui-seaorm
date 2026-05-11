use sea_orm::entity::prelude::*;
use serde::{Serialize, Deserialize};
use models::entity::tags::{Model};

use crate::{api::AuthContext, dto::ChangeRecord, tags::dto::TagListItem};

#[derive(Serialize, Deserialize, Debug)]
pub enum TagCommand {
    Reload,
    PopularTags,
    Create(Model),
    Update(ChangeRecord),
    Load(Uuid),
    Delete(Uuid),
}

impl TagCommand {
    pub fn has_access(&self, auth_context: &AuthContext) -> bool {
        match self {
            TagCommand::Reload | TagCommand::Load(_) | TagCommand::PopularTags => {
                true
            }
            _ => {
                auth_context.is_admin()
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum TagResult {
    Tags(Vec<Model>),
    PopularTags(Vec<TagListItem>),
    Tag(Model),
}