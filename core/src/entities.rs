use sea_orm::entity::prelude::Uuid;

#[derive(PartialEq, Debug, Clone)]
pub enum EntityIdent {
    Article(Uuid),
    ArticleList,
    ArticlePopularList,
    ArticleListAuthor(Uuid),
    ArticleListFavorites(Uuid),
    ArticleListFollowed(Uuid),
    ArticleListTag(Uuid),
    Comment(Uuid),
    Tag(Uuid),
    User(Uuid),
    None,
}