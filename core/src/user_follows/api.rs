use models::entity::user_follows::{Model};
use sea_orm::entity::prelude::*;
use serde::{Serialize, Deserialize};

use super::dto::UserFollowerName;

#[derive(Serialize, Deserialize, Debug)]
pub enum UserFollowerCommand {
    LoadByFolloweeId(Uuid),
    Create(Model),
    Delete((Uuid, Uuid)),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum UserFollowerResult {
    Followers(Vec<UserFollowerName>),
}