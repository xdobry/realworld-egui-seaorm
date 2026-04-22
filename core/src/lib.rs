use models::DateTimeWithTimeZone;
use uuid::Uuid;

pub mod articles;
pub mod users;
pub mod tags;
pub mod article_tags;
pub mod comments;
pub mod article_favorites;
pub mod user_follows;
pub mod api;
pub mod dto;
pub mod entities;

pub fn time_now() -> DateTimeWithTimeZone { 
    chrono::Local::now().with_timezone(&chrono::Local::now().offset())
}

pub fn new_uuid() -> models::Uuid {
    Uuid::new_v4()
}