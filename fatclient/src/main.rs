use core::user_follows::api::{UserFollowerCommand, UserFollowerResult};
use core::user_follows::dto::UserFollowerName;
use std::thread;

use egui::ViewportBuilder;
use ui::app::FormsApp;
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
use sea_orm::{JoinType, prelude::*};
use sea_orm::{Database, EntityTrait};
use models::entity::{
    article_favorites, article_tags, articles, comments, tags, user_follows, users
};
use tokio::runtime::Runtime;
use tokio::sync::mpsc::{self};
use ui::api::{CommandBus, ResponseChannel, UITask};

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


async fn handle_ui_command(cmd: UICommand, result_tx: &mut ResponseChannel, db: &DatabaseConnection) -> Result<(), sea_orm::DbErr> {
    let db_api = DBApi { db };
    match cmd {
        UICommand::Article(article_command) => {
            match article_command {
                ArticleCommand::Reload => {
                    let articles = db_api.load_articles().await?;
                    result_tx.send(UIResult::Article(ArticleResult::Articles(articles)));
                },
                ArticleCommand::Create(article) => {
                    let _insert_res = db_api.insert_article(article).await?;
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
                    let id = article.id.as_ref().clone(); 
                    let _update_res = db_api.update_article(article).await?;
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
                    let _insert_res = db_api.insert_article_tags(article_tag).await?;
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
                ArticleFavoriteCommand::Create(article_tag) => {
                    let _insert_res = article_favorites::Entity::insert(article_tag).exec(db).await?;
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
                    let _insert_res = comments::Entity::insert(comment).exec(db).await?;
                    result_tx.send(UIResult::Created);
                }
                CommentCommand::Delete(id) => {
                    let _d = comments::Entity::delete_by_id(id).exec(db).await?;
                    result_tx.send(UIResult::Deleted(id));
                }
                CommentCommand::Update(comment) => {
                    let id = comment.id.as_ref().clone(); 
                    let _d = comments::Entity::update(comment).exec(db).await?;
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
                    let _insert_res = users::Entity::insert(user).exec(db).await?;
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
                    let id = user.id.as_ref().clone(); 
                    let _update_res = users::Entity::update(user).validate()?.exec(db).await?;
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
                    let _insert_res = user_follows::Entity::insert(user_follower).exec(db).await?;
                    result_tx.send(UIResult::Created);
                }
                UserFollowerCommand::Delete(id) => {
                    let _d = user_follows::Entity::delete_by_id(id).exec(db).await?;
                    result_tx.send(UIResult::Deleted(id.0));
                }
                UserFollowerCommand::Update(user_follower) => {
                    let id = user_follower.follower_id.as_ref().clone(); 
                    let _d = user_follows::Entity::update(user_follower).exec(db).await?;
                    result_tx.send(UIResult::Updated(id));
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
                    let _insert_res = tags::Entity::insert(user).exec(db).await?;
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
                TagCommand::Update(user) => {
                    let id = user.id.as_ref().clone();
                    let _update_res = tags::Entity::update(user).validate()?.exec(db).await?;
                    result_tx.send(UIResult::Updated(id));
                },
            }
        }

    }
    Ok(())
}

fn main() -> Result<(), eframe::Error> {

    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default(),
        ..eframe::NativeOptions::default()
    };
    let args = std::env::args().skip(1).collect();

    let (command_tx, mut command_rx) = mpsc::channel::<UITask>(5);

    thread::spawn(move || {
        let rt = Runtime::new().unwrap();
        rt.block_on(async move {
            // Example async task
            let db = Database::connect("postgres://realworld:realworld@localhost/realworld").await;
            if let Ok(db) = db {
                while let Some(mut cmd) = command_rx.recv().await {
                    let result = handle_ui_command(cmd.command, &mut cmd.response, &db).await;
                    if let Err(err) = result {
                        println!("db error {:?}",err);
                        cmd.response.send(UIResult::DbError(err.to_string()));
                    }
                }
            } else {
                while let Some(mut cmd) = command_rx.recv().await {
                    cmd.response.send(UIResult::DbError("no db connection".to_string()));
                }               
            }
        });
    });

    let command_bus = CommandBus::new(command_tx);

    eframe::run_native(
        "RealWorld App - Egui",
        options,
        Box::new(|cc| 
            Ok(Box::new(FormsApp::new(cc.storage, args, command_bus)))),
    )
}