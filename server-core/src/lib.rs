use core::user_follows::api::{UserFollowerCommand, UserFollowerResult};
use core::user_follows::dto::UserFollowerName;
use core::api::{UICommand, UIResult};
use core::article_favorites::api::{ArticleFavoriteCommand, ArticleFavoriteResult};
use core::article_favorites::dto::{ArticleFavoriteUI, UserFavoriteUI};
use core::article_tags::api::{ArticleTagCommand, ArticleTagResult};
use core::article_tags::dto::ArticleTagUI;
use core::articles::api::{self, Api, ArticleCommand, ArticleResult};
use core::articles::dto::{ArticleUI};
use core::comments::api::{CommentCommand, CommentResult};
use core::comments::dto::{CommentArticle, CommentAuthor};
use core::tags::api::{TagCommand, TagResult};
use core::users::api::{UserCommand, UserResult};
use models::entity::{article_favorites, articles, tags, comments, article_tags, user_follows, users};
use sea_orm::{InsertResult, JoinType, prelude::*};
use command_bus::ResponseChannel;

struct DBApi<'a> {
    db: &'a DatabaseConnection
}

impl api::Api for DBApi<'_> {
    async fn load_articles(&self) -> Result<Vec<articles::Model>, DbErr> {
        articles::Entity::find().all(self.db).await
    }

    async fn update_article(&self, article: articles::ActiveModel) -> Result<(), DbErr> {
        let _l = articles::Entity::update(article).validate()?.exec(self.db).await?;
        Ok(())
    }

    async fn insert_article(&self, article: articles::ActiveModel) -> Result<(), DbErr> {
        let _l = articles::Entity::insert(article).exec(self.db).await?;
        Ok(())
    }

    async fn load_article(&self, id: Uuid) -> Result<ArticleUI, DbErr> {
        use sea_orm::QuerySelect;
            
        let article = articles::Entity::find_by_id(id)
            .select_only()
            .column(articles::Column::Id)
            .column(articles::Column::Title)
            .column(articles::Column::Slug)
            .column(articles::Column::Body)
            .column(articles::Column::Description)
            .column(articles::Column::CreatedAt)
            .column(articles::Column::UpdatedAt)
            .column(articles::Column::AuthorId)
            .column_as(users::Column::Username, "author_label")
            .join(JoinType::LeftJoin, articles::Relation::Author.def())
            .into_model::<ArticleUI>()
            .one(self.db)
            .await?;
        
        if let Some(article) = article {
            Ok(article)
        } else {
            Err(DbErr::Custom("article not found".into()))
        }
    }
    async fn delete_article(&self, id: Uuid) -> Result<(), DbErr> {
        let _l = articles::Entity::delete_by_id(id).exec(self.db).await?;
        Ok(())
    }
    
    async fn load_article_tags(&self, article_id: Uuid) -> Result<Vec<ArticleTagUI>,DbErr> {
        use sea_orm::QuerySelect;
        article_tags::Entity::find()
            .select_only()
            .filter(article_tags::Column::ArticleId.eq(article_id))
            .column(article_tags::Column::ArticleId)
            .column(article_tags::Column::TagId)
            .column(article_tags::Column::CreatedAt)
            .column_as(tags::Column::Name, "tag_name")
            .join(JoinType::LeftJoin, article_tags::Relation::Tags.def())
            .into_model::<ArticleTagUI>()
            .all(self.db)
            .await
    }
    
    async fn insert_article_tags(&self, article_tag: article_tags::ActiveModel) -> Result<(),DbErr> {
        let _l = article_tags::Entity::insert(article_tag).exec(self.db).await?;
        Ok(())
    }
}


pub async fn handle_ui_command(cmd: UICommand, result_tx: &mut ResponseChannel, db: &DatabaseConnection) -> Result<(), sea_orm::DbErr> {
    let db_api = DBApi { db };
    match cmd {
        UICommand::Article(article_command) => {
            match article_command {
                ArticleCommand::Reload => {
                    let articles = db_api.load_articles().await?;
                    result_tx.send(UIResult::Article(ArticleResult::Articles(articles)));
                },
                ArticleCommand::Create(article) => {
                    let _insert_res = db_api.insert_article(article.into()).await?;
                    result_tx.send(UIResult::Created);
                },
                ArticleCommand::Delete(id) => {
                    let _insert_res = db_api.delete_article(id).await?;
                    result_tx.send(UIResult::Deleted(id));
                },
                ArticleCommand::Load(uuid) => {
                    let article = db_api.load_article(uuid).await?;
                    result_tx.send(UIResult::Article(ArticleResult::Article(article)));
                },
                ArticleCommand::Update(article) => {
                    let id = article.key; 
                    let _update_res = db_api.update_article(article.to_active_model::<articles::Entity>()).await?;
                    result_tx.send(UIResult::Updated(id));
                },
            }
        },
        UICommand::ArticleTag(article_tag_command) => {
            match article_tag_command {
                ArticleTagCommand::LoadByArticleId(uuid) => {
                    let tags_article = db_api.load_article_tags(uuid).await?;
                    result_tx.send(UIResult::ArticleTag(ArticleTagResult::ArticleTags(tags_article)));
                },
                ArticleTagCommand::Create(article_tag) => {
                    let _insert_res = db_api.insert_article_tags(article_tag.into()).await?;
                    result_tx.send(UIResult::Created);
                },
                ArticleTagCommand::Delete(ids) => {
                    let _d = article_tags::Entity::delete_by_id(ids).exec(db).await?;
                    result_tx.send(UIResult::Deleted(ids.1));
                }
            }
        },
        UICommand::ArticleFavorite(article_tag_command) => {
            match article_tag_command {
                ArticleFavoriteCommand::LoadByArticleId(article_id) => {
                    use sea_orm::QuerySelect;
                    let article_favorites = article_favorites::Entity::find()
                        .select_only()
                        .filter(article_favorites::Column::ArticleId.eq(article_id))
                        .column(article_favorites::Column::ArticleId)
                        .column(article_favorites::Column::UserId)
                        .column(article_favorites::Column::CreatedAt)
                        .column_as(users::Column::Username, "user_name")
                        .join(JoinType::LeftJoin, article_favorites::Relation::Users.def())
                        .into_model::<ArticleFavoriteUI>()
                        .all(db)
                        .await?;
                    result_tx.send(UIResult::ArticleFavorite(ArticleFavoriteResult::ArticleFavorites(article_favorites)));
                },
                ArticleFavoriteCommand::LoadByUserId(user_id) => {
                    use sea_orm::QuerySelect;
                    let user_favorites = article_favorites::Entity::find()
                        .select_only()
                        .filter(article_favorites::Column::UserId.eq(user_id))
                        .column(article_favorites::Column::UserId)
                        .column(article_favorites::Column::ArticleId)
                        .column(article_favorites::Column::CreatedAt)
                        .column_as(articles::Column::Title, "article_title")
                        .join(JoinType::LeftJoin, article_favorites::Relation::Users.def())
                        .into_model::<UserFavoriteUI>()
                        .all(db)
                        .await?;
                    result_tx.send(UIResult::ArticleFavorite(ArticleFavoriteResult::UserFavorites(user_favorites)));
                },
                ArticleFavoriteCommand::Create(article_favorite) => {
                    let _insert_res = article_favorites::Entity::insert::<article_favorites::ActiveModel>(article_favorite.into()).exec(db).await?;
                    result_tx.send(UIResult::Created);
                },
                ArticleFavoriteCommand::Delete(ids) => {
                    let _d = article_favorites::Entity::delete_by_id(ids).exec(db).await?;
                    result_tx.send(UIResult::Deleted(ids.1));
                }
            }
        },
        UICommand::Comment(comment_command) => {
            match comment_command {
                CommentCommand::LoadByArticleId(article_id) => {
                    use sea_orm::QuerySelect;
                    let comments = comments::Entity::find()
                        .select_only()
                        .filter(comments::Column::ArticleId.eq(article_id))
                        .column(comments::Column::Id)
                        .column(comments::Column::ArticleId)
                        .column(comments::Column::AuthorId)
                        .column(comments::Column::Body)
                        .column(comments::Column::CreatedAt)
                        .column(comments::Column::UpdatedAt)
                        .column_as(users::Column::Username, "author_name")
                        .join(JoinType::LeftJoin, comments::Relation::Users.def())
                        .into_model::<CommentAuthor>()
                        .all(db)
                        .await?;
                    result_tx.send(UIResult::Comment(CommentResult::CommentsAuthor(comments)));
                },
                CommentCommand::LoadByUserId(user_id) => {
                    use sea_orm::QuerySelect;
                    let comments = comments::Entity::find()
                        .select_only()
                        .filter(comments::Column::AuthorId.eq(user_id))
                        .column(comments::Column::Id)
                        .column(comments::Column::ArticleId)
                        .column(comments::Column::AuthorId)
                        .column(comments::Column::Body)
                        .column(comments::Column::CreatedAt)
                        .column(comments::Column::UpdatedAt)
                        .column_as(articles::Column::Title, "article_title")
                        .join(JoinType::LeftJoin, comments::Relation::Articles.def())
                        .into_model::<CommentArticle>()
                        .all(db)
                        .await?;
                    result_tx.send(UIResult::Comment(CommentResult::CommentsArticle(comments)));
                }
                CommentCommand::Create(comment) => {
                    let _insert_res = comments::Entity::insert::<comments::ActiveModel>(comment.into()).exec(db).await?;
                    result_tx.send(UIResult::Created);
                }
                CommentCommand::Delete(id) => {
                    let _d = comments::Entity::delete_by_id(id).exec(db).await?;
                    result_tx.send(UIResult::Deleted(id));
                }
                CommentCommand::Update(comment) => {
                    let id = comment.key; 
                    let _d = comments::Entity::update::<comments::ActiveModel>(comment.to_active_model::<comments::Entity>()).exec(db).await?;
                    result_tx.send(UIResult::Updated(id));
                }
            }
        },
        UICommand::User(user_command) => {
            match user_command {
                UserCommand::Reload => {
                    let users = users::Entity::find().all(db).await?;
                    result_tx.send(UIResult::User(UserResult::Users(users)));
                },
                UserCommand::Create(user) => {
                    let _insert_res: InsertResult<users::ActiveModel> = users::Entity::insert(user.into()).exec(db).await?;
                    result_tx.send(UIResult::Created);
                },
                UserCommand::Load(uuid) => {
                    let user = users::Entity::find_by_id(uuid).one(db).await?;
                    if let Some(user) = user {
                        result_tx.send(UIResult::User(UserResult::User(user)));
                    }
                },
                UserCommand::Delete(id) => {
                    let _d = users::Entity::delete_by_id(id).exec(db).await?;
                    result_tx.send(UIResult::Deleted(id));
                },
                UserCommand::Update(user) => {
                    let id = user.key; 
                    let _update_res = users::Entity::update::<users::ActiveModel>(user.to_active_model::<users::Entity>()).validate()?.exec(db).await?;
                    result_tx.send(UIResult::Updated(id));
                },
            }
        }
        UICommand::UserFollower(user_follower_command) => {
            match user_follower_command {
                UserFollowerCommand::LoadByFolloweeId(followee_id) => {
                    use sea_orm::QuerySelect;
                    let followers = user_follows::Entity::find()
                        .select_only()
                        .filter(user_follows::Column::FolloweeId.eq(followee_id))
                        .column(user_follows::Column::FolloweeId)
                        .column(user_follows::Column::FollowerId)
                        .column(user_follows::Column::CreatedAt)
                        .column_as(users::Column::Username, "follower_name")
                        .join(JoinType::LeftJoin, user_follows::Relation::Users1.def())
                        .into_model::<UserFollowerName>()
                        .all(db)
                        .await?;
                    result_tx.send(UIResult::UserFollower(UserFollowerResult::Followers(followers)));
                },
                UserFollowerCommand::Create(user_follower) => {
                    let _insert_res: InsertResult<user_follows::ActiveModel> = user_follows::Entity::insert(user_follower.into()).exec(db).await?;
                    result_tx.send(UIResult::Created);
                }
                UserFollowerCommand::Delete(id) => {
                    let _d = user_follows::Entity::delete_by_id(id).exec(db).await?;
                    result_tx.send(UIResult::Deleted(id.0));
                }
            }
        }
        UICommand::Tag(tag_command) => {
            match tag_command {
                TagCommand::Reload => {
                    let tags = tags::Entity::find().all(db).await?;
                    result_tx.send(UIResult::Tag(TagResult::Tags(tags)));
                },
                TagCommand::Create(user) => {
                    let _insert_res: InsertResult<tags::ActiveModel> = tags::Entity::insert(user.into()).exec(db).await?;
                    result_tx.send(UIResult::Created);
                },
                TagCommand::Load(uuid) => {
                    let tag = tags::Entity::find_by_id(uuid).one(db).await?;
                    if let Some(tag) = tag {
                        result_tx.send(UIResult::Tag(TagResult::Tag(tag)));
                    }
                },
                TagCommand::Delete(id) => {
                    let _d = tags::Entity::delete_by_id(id).exec(db).await?;
                    result_tx.send(UIResult::Deleted(id));
                },
                TagCommand::Update(tag) => {
                    let id = tag.key;
                    let _update_res = tags::Entity::update::<tags::ActiveModel>(tag.to_active_model::<tags::Entity>()).validate()?.exec(db).await?;
                    result_tx.send(UIResult::Updated(id));
                },
            }
        }

    }
    Ok(())
}
