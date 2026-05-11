use models::entity::user_follows::{Model};
use sea_orm::entity::prelude::*;
use serde::{Serialize, Deserialize};

use crate::api::AuthContext;

use super::dto::UserFollowerName;

#[derive(Serialize, Deserialize, Debug)]
pub enum UserFollowerCommand {
    LoadByFolloweeId(Uuid),
    LoadByFollowerId(Uuid),
    Create(Model),
    Delete((Uuid, Uuid)),
}

impl UserFollowerCommand {
    pub fn has_access(&self, auth_context: &AuthContext) -> bool {
        match self {
            UserFollowerCommand::LoadByFolloweeId(_) | UserFollowerCommand::LoadByFollowerId(_) => {
                true
            }
            _ => {
                !auth_context.is_anonymous()
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum UserFollowerResult {
    Followers(Vec<UserFollowerName>),
}