use sea_orm::entity::prelude::Uuid;

#[derive(PartialEq, Debug, Clone)]
pub enum EntityIdent {
    Article(Uuid),
    Comment(Uuid),
    Tag(Uuid),
    User(Uuid),
    None,
}