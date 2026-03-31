use models::entity::user_follows::ActiveModel;
use sea_orm::entity::prelude::*;


use super::dto::UserFollowerName;

pub enum UserFollowerCommand {
    LoadByFolloweeId(Uuid),
    Create(ActiveModel),
    Delete((Uuid, Uuid)),
    Update(ActiveModel),
}

pub enum UserFollowerResult {
    Followers(Vec<UserFollowerName>),
}