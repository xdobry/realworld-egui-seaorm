use sea_orm::{FromQueryResult, entity::prelude::*};
use serde::{Serialize, Deserialize};

#[derive(FromQueryResult, Default, Clone, Serialize, Deserialize, Debug)]
pub struct UserFollowerName {
    pub follower_id: Uuid,
    pub followee_id: Uuid,
    pub follower_name: String,
    pub created_at: DateTimeWithTimeZone,
}