use sea_orm::{FromQueryResult, entity::prelude::*};


#[derive(FromQueryResult, Default, Clone)]
pub struct UserFollowerName {
    pub follower_id: Uuid,
    pub followee_id: Uuid,
    pub follower_name: String,
    pub created_at: DateTimeWithTimeZone,
}