use core::tags::dto::TagListItem;
use core::user_follows::api::{UserFollowerCommand, UserFollowerResult};
use core::user_follows::dto::UserFollowerName;
use core::api::{AuthContext, UICommand, UIResult};
use core::article_favorites::api::{ArticleFavoriteCommand, ArticleFavoriteResult};
use core::article_favorites::dto::{ArticleFavoriteUI, UserFavoriteUI};
use core::article_tags::api::{ArticleTagCommand, ArticleTagResult};
use core::article_tags::dto::ArticleTagUI;
use core::articles::api::{self, Api, ArticleCommand, ArticleResult};
use core::articles::dto::{ArticleListItem, ArticlePopularListItem, ArticleUI};
use core::comments::api::{CommentCommand, CommentResult};
use core::comments::dto::{CommentArticle, CommentAuthor};
use core::tags::api::{TagCommand, TagResult};
use core::users::api::{UserCommand, UserResult};
use core::users::dto::{LoginResponse, UserContext};
use models::entity::{article_favorites, articles, tags, comments, article_tags, user_follows, users};
use sea_orm::sea_query::{PostgresQueryBuilder, Query, SqliteQueryBuilder};
use sea_orm::{ActiveValue, DatabaseBackend, EntityTrait, ExprTrait, FromQueryResult, InsertResult, IntoActiveModel, JoinType, Order, QueryOrder, QuerySelect, Statement, prelude::*};
use command_bus::ResponseChannel;

pub mod token;

const DEFAULT_LIMIT: u64 = 500;

struct DBApi<'a> {
    db: &'a DatabaseConnection
}

impl api::Api for DBApi<'_> {
    async fn load_articles(&self) -> Result<Vec<ArticleListItem>, DbErr> {
        articles::Entity::find().order_by_desc(articles::Column::CreatedAt)
            .limit(DEFAULT_LIMIT)
            .into_model::<ArticleListItem>()
            .all(self.db).await
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
            .limit(DEFAULT_LIMIT)
            .order_by_asc(tags::Column::Name)
            .into_model::<ArticleTagUI>()
            .all(self.db)
            .await
    }
    
    async fn insert_article_tags(&self, article_tag: article_tags::ActiveModel) -> Result<(),DbErr> {
        let _l = article_tags::Entity::insert(article_tag).exec(self.db).await?;
        Ok(())
    }
}

async fn get_article_user(article_id: Uuid, db: &DatabaseConnection) -> Result<Option<Uuid>,DbErr> {
    articles::Entity::find_by_id(article_id).select_only().column(articles::Column::AuthorId).into_tuple::<Uuid>().one(db).await
}

async fn get_comment_user(comment_id: Uuid, db: &DatabaseConnection) -> Result<Option<Uuid>,DbErr> {
    comments::Entity::find_by_id(comment_id).select_only().column(comments::Column::AuthorId).into_tuple::<Uuid>().one(db).await
}


pub async fn handle_ui_command<T: CallContext>(cmd: UICommand, result_tx: &mut ResponseChannel, db: &DatabaseConnection, call_context: &T) -> Result<(), sea_orm::DbErr> {
    let db_api = DBApi { db };
    let auth_context = AuthContext::new(call_context.user_id(), call_context.is_admin());
    if !cmd.has_access(&auth_context) {
        result_tx.send(UIResult::DbError("no access".into()));
        return Ok(());
    }
    match cmd {
        UICommand::Article(article_command) => {
            match article_command {
                ArticleCommand::Reload => {
                    let articles = db_api.load_articles().await?;
                    result_tx.send(UIResult::Article(ArticleResult::ArticleList(articles)));
                },
                ArticleCommand::ListAuthor(author_id) => {
                    let articles =  articles::Entity::find()
                        .filter(articles::Column::AuthorId.eq(author_id))
                        .limit(DEFAULT_LIMIT)
                        .order_by_desc(articles::Column::CreatedAt)
                        .into_model::<ArticleListItem>()
                        .all(db).await?;
                    result_tx.send(UIResult::Article(ArticleResult::ArticleList(articles)));
                }
                ArticleCommand::ListFavorites(user_id) => {
                    let articles = articles::Entity::find()
                        .join(JoinType::LeftJoin, articles::Relation::ArticleFavorites.def())
                        .filter(article_favorites::Column::UserId.eq(user_id))
                        .limit(DEFAULT_LIMIT)
                        .order_by_desc(articles::Column::CreatedAt)
                        .into_model::<ArticleListItem>()
                        .all(db)
                        .await?;
                    result_tx.send(UIResult::Article(ArticleResult::ArticleList(articles)));                
                }
                ArticleCommand::ListTag(tag_id) => {
                    let articles = articles::Entity::find()
                        .join(JoinType::LeftJoin, articles::Relation::ArticleTags.def())
                        .filter(article_tags::Column::TagId.eq(tag_id))
                        .limit(DEFAULT_LIMIT)
                        .order_by_desc(articles::Column::CreatedAt)
                        .into_model::<ArticleListItem>()
                        .all(db)
                        .await?;
                    result_tx.send(UIResult::Article(ArticleResult::ArticleList(articles)));
                }
                ArticleCommand::ListFollowed(follower_id) => {
                    let backend = db.get_database_backend();
                    let query = Query::select()
                        .column(articles::Column::Id)
                        .column(articles::Column::Slug)
                        .column(articles::Column::Title)
                        .column(articles::Column::Description)
                        .column(articles::Column::AuthorId)
                        .column((articles::Entity, articles::Column::CreatedAt))
                        .from(articles::Entity)
                        .join(JoinType::InnerJoin, user_follows::Entity, 
                            Expr::col((articles::Entity, articles::Column::AuthorId))
                                .equals((user_follows::Entity, user_follows::Column::FolloweeId))
                        )
                        .and_where(
                               Expr::col((user_follows::Entity, user_follows::Column::FollowerId))
                                .eq(follower_id)
    )
                        .order_by((articles::Entity, articles::Column::CreatedAt), sea_orm::Order::Desc)
                        .limit(DEFAULT_LIMIT)
                        .to_owned();

                    let (sql, values) = build_query(backend, &query);
                    let stmt = Statement::from_sql_and_values(backend, sql, values);
                    let articles = ArticleListItem::find_by_statement(stmt).all(db).await?;
                    result_tx.send(UIResult::Article(ArticleResult::ArticleList(articles)));                   
                }
                ArticleCommand::ListPopular => {
                    let backend = db.get_database_backend();
                    let query = Query::select()
                        .column(articles::Column::Id)
                        .column(articles::Column::Title)
                        .column(articles::Column::Slug)
                        .column(articles::Column::Description)
                        .column(articles::Column::AuthorId)
                        .column((articles::Entity, articles::Column::CreatedAt))
                        .expr_as(Expr::cust("count(*)"), "count")
                        .from(articles::Entity)
                        .join(JoinType::LeftJoin, article_favorites::Entity, 
                            Expr::col((articles::Entity, articles::Column::Id))
                                .equals((article_favorites::Entity, article_favorites::Column::ArticleId))
                        )
                        .group_by_col((articles::Entity, articles::Column::Id))
                        .group_by_col((articles::Entity, articles::Column::Slug))
                        .group_by_col((articles::Entity, articles::Column::Title))
                        .group_by_col((articles::Entity, articles::Column::Description))
                        .group_by_col((articles::Entity, articles::Column::AuthorId))
                        .group_by_col((articles::Entity, articles::Column::CreatedAt))
                        .order_by("count", Order::Desc)
                        .limit(DEFAULT_LIMIT)
                        .to_owned();

                    let (sql, values) = build_query(backend, &query);
                    let stmt = Statement::from_sql_and_values(backend, sql, values);
                    let articles = ArticlePopularListItem::find_by_statement(stmt).all(db).await?;
                    result_tx.send(UIResult::Article(ArticleResult::ArticlePopular(articles)));                     
                }
                ArticleCommand::Create(article) => {
                    if call_context.is_admin_or_user(Some(article.author_id)) {
                        let _insert_res = db_api.insert_article(article.into()).await?;
                        result_tx.send(UIResult::Created);
                    } else {
                        result_tx.send(UIResult::DbError("no access".into()));
                    }
                },
                ArticleCommand::Delete(id) => {
                    let author_id= get_article_user(id, db).await?;
                    if call_context.is_admin_or_user(author_id) {
                        let _insert_res = db_api.delete_article(id).await?;
                        result_tx.send(UIResult::Deleted(id));
                    } else {
                        result_tx.send(UIResult::DbError("no access".into()));
                    }
                },
                ArticleCommand::Load(uuid) => {
                    let article = db_api.load_article(uuid).await?;
                    result_tx.send(UIResult::Article(ArticleResult::Article(article)));
                },
                ArticleCommand::Update(article) => {
                    let id = article.key; 
                    let author_id= get_article_user(id, db).await?;
                    if call_context.is_admin_or_user(author_id) {
                        let article_model = article.to_active_model::<articles::Entity>();
                        if !call_context.is_admin() {
                            match article_model.author_id {
                                ActiveValue::Set(changed_author_id) => {
                                    if call_context.user_id() != Some(changed_author_id) {
                                        result_tx.send(UIResult::DbError("no access".into()));
                                        return Ok(());
                                    }
                                }
                                _ => {

                                }
                            }
                        }
                        let _update_res = db_api.update_article(article_model).await?;
                        result_tx.send(UIResult::Updated(id));
                    } else {
                        result_tx.send(UIResult::DbError("no access".into()));
                    }
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
                    let author_id = get_article_user(article_tag.article_id, db).await?;
                    if call_context.is_admin_or_user(author_id) {
                        let _insert_res = db_api.insert_article_tags(article_tag.into()).await?;
                        result_tx.send(UIResult::Created);
                    } else {
                        result_tx.send(UIResult::DbError("no access".into()));
                    }
                },
                ArticleTagCommand::Delete(ids) => {
                    let author_id = get_article_user(ids.0, db).await?;
                    if call_context.is_admin_or_user(author_id) {
                        let _d = article_tags::Entity::delete_by_id(ids).exec(db).await?;
                        result_tx.send(UIResult::Deleted(ids.1));
                    } else {
                        result_tx.send(UIResult::DbError("no access".into()));
                    }
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
                        .order_by_desc(article_favorites::Column::CreatedAt)
                        .limit(DEFAULT_LIMIT)
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
                        .join(JoinType::LeftJoin, article_favorites::Relation::Articles.def())
                        .order_by_desc(article_favorites::Column::CreatedAt)
                        .limit(DEFAULT_LIMIT)
                        .into_model::<UserFavoriteUI>()
                        .all(db)
                        .await?;
                    result_tx.send(UIResult::ArticleFavorite(ArticleFavoriteResult::UserFavorites(user_favorites)));
                },
                ArticleFavoriteCommand::Create(article_favorite) => {
                    if call_context.is_admin_or_user(Some(article_favorite.user_id)) {
                        let _insert_res = article_favorites::Entity::insert::<article_favorites::ActiveModel>(article_favorite.into()).exec(db).await?;
                        result_tx.send(UIResult::Created);
                    } else {
                        result_tx.send(UIResult::DbError("no access".into()));
                    }
                },
                ArticleFavoriteCommand::Delete(ids) => {
                    if call_context.is_admin_or_user(Some(ids.0)) {
                        let _d = article_favorites::Entity::delete_by_id(ids).exec(db).await?;
                        result_tx.send(UIResult::ArticleFavorite(ArticleFavoriteResult::Deleted(ids)));
                    } else {
                        result_tx.send(UIResult::DbError("no access".into()));
                    }
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
                        .order_by_desc(comments::Column::CreatedAt)
                        .limit(DEFAULT_LIMIT)
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
                        .order_by_desc(comments::Column::CreatedAt)
                        .limit(DEFAULT_LIMIT)
                        .into_model::<CommentArticle>()
                        .all(db)
                        .await?;
                    result_tx.send(UIResult::Comment(CommentResult::CommentsArticle(comments)));
                }
                CommentCommand::Create(comment) => {
                    if call_context.is_admin_or_user(Some(comment.author_id)) {
                        let _insert_res = comments::Entity::insert::<comments::ActiveModel>(comment.into()).exec(db).await?;
                        result_tx.send(UIResult::Created);
                    } else {
                        result_tx.send(UIResult::DbError("no access".into()));
                    }
                }
                CommentCommand::Delete(id) => {
                    let author_id = get_comment_user(id, db).await?;
                    if call_context.is_admin_or_user(author_id) {
                        let _d = comments::Entity::delete_by_id(id).exec(db).await?;
                        result_tx.send(UIResult::Deleted(id));
                    } else {
                        result_tx.send(UIResult::DbError("no access".into()));
                    }
                }
                CommentCommand::Update(comment) => {
                    let id = comment.key;
                    let author_id = get_comment_user(id, db).await?;
                    if call_context.is_admin_or_user(author_id) {
                        let comment_model = comment.to_active_model::<comments::Entity>();
                        if !call_context.is_admin() {
                            match comment_model.author_id {
                                ActiveValue::Set(changed_author_id) => {
                                    if call_context.user_id() != Some(changed_author_id) {
                                        result_tx.send(UIResult::DbError("no access".into()));
                                        return Ok(());
                                    }
                                }
                                _ => {

                                }
                            }
                        }
                        let _d = comments::Entity::update::<comments::ActiveModel>(comment_model).exec(db).await?;
                        result_tx.send(UIResult::Updated(id));
                    } else {
                        result_tx.send(UIResult::DbError("no access".into()));
                    }
                }
            }
        },
        UICommand::User(user_command) => {
            match user_command {
                UserCommand::Reload => {
                    let users = users::Entity::find()
                        .order_by_asc(users::Column::Username)
                        .limit(DEFAULT_LIMIT)
                        .all(db).await?;
                    result_tx.send(UIResult::User(UserResult::Users(users)));
                },
                UserCommand::Create(user) => {
                    let mut user_model: users::ActiveModel = user.into();
                    if let Some(password) = user_model.password_hash.take() {
                       user_model.password_hash = ActiveValue::Set(call_context.encode_password(password.as_str()));
                    }
                    let _insert_res: InsertResult<users::ActiveModel> = users::Entity::insert(user_model).exec(db).await?;
                    result_tx.send(UIResult::Created);
                },
                UserCommand::Load(uuid) => {
                    let user = users::Entity::find_by_id(uuid).one(db).await?;
                    if let Some(user) = user {
                        result_tx.send(UIResult::User(UserResult::User(user)));
                    } else {
                        result_tx.send(UIResult::DbError("user not found".into()));
                    }
                },
                UserCommand::Delete(id) => {
                    if call_context.is_admin() {
                        let _d = users::Entity::delete_by_id(id).exec(db).await?;
                        result_tx.send(UIResult::Deleted(id));
                    } else {
                        result_tx.send(UIResult::DbError("need admin".into()))
                    }
                },
                UserCommand::Update(user) => {
                    let id = user.key; 
                    if call_context.is_admin_or_user(Some(id)) {
                        let mut user_model = user.to_active_model::<users::Entity>();
                        if let Some(password) = user_model.password_hash.take() {
                        user_model.password_hash = ActiveValue::Set(call_context.encode_password(password.as_str()));
                        }
                        let _update_res = users::Entity::update::<users::ActiveModel>(user_model).validate()?.exec(db).await?;
                        result_tx.send(UIResult::Updated(id));
                    } else {
                        result_tx.send(UIResult::DbError("no access".into()));
                    }
                },
                UserCommand::Login(login_request) => {
                    let user = users::Entity::find().filter(users::Column::Email.eq(login_request.email)).one(db).await?;
                    if let Some(user) = user {
                        if call_context.verify_password(&login_request.password, &user.password_hash) {
                            let user_context = UserContext {
                                    user_id: user.id,
                                    is_admin: user.is_admin,
                                    user_name: user.username,
                                    user_email: user.email,
                            };
                            let token = call_context.create_token(&user_context);
                            result_tx.send(UIResult::User(UserResult::Login(LoginResponse {
                                user_context: user_context,
                                token: token,
                            })));
                            return Ok(());
                        }
                    }
                    result_tx.send(UIResult::User(UserResult::LoginFailed("user not found or password wrong".into())));
                }
                UserCommand::Register(register_request) => {
                    let user_id = Uuid::new_v4();
                    let now = core::time_now();
                    let new_user = users::Model {
                        id: user_id,
                        username: register_request.name.clone(),
                        password_hash: call_context.encode_password(&register_request.password),
                        email: register_request.email.clone(),
                        bio: None,
                        image: None,
                        is_admin: false,
                        created_at: now,
                        updated_at: now,
                    };
                    let _insert_res: InsertResult<users::ActiveModel> = users::Entity::insert(new_user.into_active_model()).exec(db).await?;
                    let user_context = UserContext {
                        user_id: user_id,
                        is_admin: false,
                        user_name: register_request.name.clone(),
                        user_email: register_request.email.clone(),
                    };
                    let token = call_context.create_token(&user_context);
                    result_tx.send(UIResult::User(UserResult::Login(LoginResponse {
                        user_context: user_context,
                        token: token,
                    })));
                }
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
                        .order_by_asc(users::Column::Username)
                        .limit(DEFAULT_LIMIT)
                        .into_model::<UserFollowerName>()
                        .all(db)
                        .await?;
                    result_tx.send(UIResult::UserFollower(UserFollowerResult::Followers(followers)));
                },
                UserFollowerCommand::LoadByFollowerId(follower_id) => {
                    use sea_orm::QuerySelect;
                    let followers = user_follows::Entity::find()
                        .select_only()
                        .filter(user_follows::Column::FollowerId.eq(follower_id))
                        .column(user_follows::Column::FolloweeId)
                        .column(user_follows::Column::FollowerId)
                        .column(user_follows::Column::CreatedAt)
                        .column_as(users::Column::Username, "follower_name")
                        .join(JoinType::LeftJoin, user_follows::Relation::Users2.def())
                        .order_by_asc(users::Column::Username)
                        .limit(DEFAULT_LIMIT)
                        .into_model::<UserFollowerName>()
                        .all(db)
                        .await?;
                    result_tx.send(UIResult::UserFollower(UserFollowerResult::Followers(followers)));
                },
                UserFollowerCommand::Create(user_follower) => {
                    if call_context.is_admin_or_user(Some(user_follower.follower_id)) {
                        let _insert_res: InsertResult<user_follows::ActiveModel> = user_follows::Entity::insert(user_follower.into()).exec(db).await?;
                        result_tx.send(UIResult::Created);
                    } else {
                        result_tx.send(UIResult::DbError("no access".into()));
                    }
                }
                UserFollowerCommand::Delete(id) => {
                    if call_context.is_admin_or_user(Some(id.0)) {
                        let _d = user_follows::Entity::delete_by_id(id).exec(db).await?;
                        result_tx.send(UIResult::Deleted(id.0));
                    } else {
                        result_tx.send(UIResult::DbError("no access".into()));
                    }
                }
            }
        }
        UICommand::Tag(tag_command) => {
            match tag_command {
                TagCommand::Reload => {
                    let tags = tags::Entity::find()
                        .order_by_asc(tags::Column::Name)
                        .limit(DEFAULT_LIMIT).all(db).await?;
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
                TagCommand::PopularTags => {
                    let backend = db.get_database_backend();
                    let query = Query::select()
                        .column(tags::Column::Id)
                        .column(tags::Column::Name)
                        .expr_as(Expr::cust("count(*)"), "count")
                        .from(tags::Entity)
                        .join(JoinType::Join, article_tags::Entity, 
                            Expr::col((tags::Entity, tags::Column::Id))
                                .equals((article_tags::Entity, article_tags::Column::TagId))
                        )
                        .group_by_col((tags::Entity, tags::Column::Id))
                        .group_by_col((tags::Entity, tags::Column::Name))
                        .order_by("count", Order::Desc)
                        .limit(32)
                        .to_owned();

                    let (sql, values) = build_query(backend, &query);
                    let stmt = Statement::from_sql_and_values(backend, sql, values);
                    let popular_tags = TagListItem::find_by_statement(stmt).all(db).await?;
                    result_tx.send(UIResult::Tag(TagResult::PopularTags(popular_tags)));
                }
            }
        }

    }
    Ok(())
}

fn build_query(backend: DatabaseBackend, query: &sea_orm::sea_query::SelectStatement) -> (String, sea_orm::Values) {
    let (sql, values) = match backend {
        DatabaseBackend::Postgres => {
            query.build(PostgresQueryBuilder)
        },
        DatabaseBackend::Sqlite => {
            query.build(SqliteQueryBuilder)
        },
        _ => {
            panic!("unsupported DB")
        }
    };
    (sql, values)
}

pub trait CallContext {
    fn is_admin(&self) -> bool;
    fn user_id(&self) -> Option<Uuid>;
    fn encode_password(&self, password: &str) -> String;
    fn verify_password(&self, password: &str, hash: &str) -> bool;
    fn create_token(&self, user_context: &UserContext) -> Vec<u8>;
    fn is_admin_or_user(&self, user_id: Option<Uuid>) -> bool {
        self.is_admin() || Some(user_id) == Some(self.user_id())
    }
}
