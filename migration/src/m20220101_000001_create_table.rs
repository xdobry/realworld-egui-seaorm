use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {

        manager
            .create_table(
                Table::create()
                    .table("users")
                    .if_not_exists()
                    .col(uuid("id").primary_key())
                    .col(string("username").unique_key())
                    .col(string("email").unique_key())
                    .col(string("password_hash"))
                    .col(text_null("bio"))
                    .col(text_null("image"))
                    .col(timestamp_with_time_zone("created_at").default(Expr::current_timestamp()))
                    .col(timestamp_with_time_zone("updated_at").default(Expr::current_timestamp()))
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table("tags")
                    .if_not_exists()
                    .col(uuid("id").primary_key())
                    .col(string("name").unique_key())
                    .col(timestamp_with_time_zone("created_at").default(Expr::current_timestamp()))
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table("articles")
                    .if_not_exists()
                    .col(uuid("id").primary_key())
                    .col(string("slug").unique_key())
                    .col(string("title"))
                    .col(text("description"))
                    .col(text("body"))  
                    .col(uuid("author_id"))                 
                    .col(timestamp_with_time_zone("created_at").default(Expr::current_timestamp()))
                    .col(timestamp_with_time_zone("updated_at").default(Expr::current_timestamp()))
                    .foreign_key(
                        ForeignKey::create().name("fk_articles_author")
                            .from("articles", "author_id")
                            .to("users", "id")
                            .on_delete(ForeignKeyAction::Cascade))
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table("comments")
                    .if_not_exists()
                    .col(uuid("id").primary_key())
                    .col(text("body"))
                    .col(uuid("author_id"))                 
                    .col(uuid("article_id"))                 
                    .col(timestamp_with_time_zone("created_at").default(Expr::current_timestamp()))
                    .col(timestamp_with_time_zone("updated_at").default(Expr::current_timestamp()))
                    .foreign_key(
                        ForeignKey::create().name("fk_comments_author")
                            .from("comments", "author_id")
                            .to("users", "id")
                            .on_delete(ForeignKeyAction::Cascade))
                    .foreign_key(
                        ForeignKey::create().name("fk_comments_article")
                            .from("comments", "article_id")
                            .to("articles", "id")
                            .on_delete(ForeignKeyAction::Cascade))
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table("article_favorites")
                    .if_not_exists()
                    .col(uuid("user_id"))                 
                    .col(uuid("article_id"))                 
                    .col(timestamp_with_time_zone("created_at").default(Expr::current_timestamp()))
                    .foreign_key(
                        ForeignKey::create().name("fk_favorites_user")
                            .from("article_favorites", "user_id")
                            .to("users", "id")
                            .on_delete(ForeignKeyAction::Cascade))
                    .foreign_key(
                        ForeignKey::create().name("fk_favorites_article")
                            .from("article_favorites", "article_id")
                            .to("articles", "id")
                            .on_delete(ForeignKeyAction::Cascade))
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table("article_tags")
                    .if_not_exists()
                    .col(uuid("tag_id"))                 
                    .col(uuid("article_id"))                 
                    .col(timestamp_with_time_zone("created_at").default(Expr::current_timestamp()))
                    .foreign_key(
                        ForeignKey::create().name("fk_article_tags_tag")
                            .from("article_tags", "tag_id")
                            .to("tags", "id")
                            .on_delete(ForeignKeyAction::Cascade))
                    .foreign_key(
                        ForeignKey::create().name("fk_article_tag_article")
                            .from("article_tags", "article_id")
                            .to("articles", "id")
                            .on_delete(ForeignKeyAction::Cascade))
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table("user_follows")
                    .if_not_exists()
                    .col(uuid("follower_id"))                 
                    .col(uuid("followee_id"))                 
                    .col(timestamp_with_time_zone("created_at").default(Expr::current_timestamp()))
                    .foreign_key(
                        ForeignKey::create().name("fk_user_follows_follower")
                            .from("user_follows", "follower_id")
                            .to("users", "id")
                            .on_delete(ForeignKeyAction::Cascade))
                    .foreign_key(
                        ForeignKey::create().name("fk_user_follows_followee")
                            .from("user_follows", "followee_id")
                            .to("users", "id")
                            .on_delete(ForeignKeyAction::Cascade))
                    .to_owned(),
            )
            .await?;

        Ok(())

    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {

            manager
            .drop_table(Table::drop().table("user_follows").to_owned())
            .await?;


        manager
            .drop_table(Table::drop().table("article_tags").to_owned())
            .await?;
        
        manager
            .drop_table(Table::drop().table("article_favorites").to_owned())
            .await?;
        
        manager
            .drop_table(Table::drop().table("comments").to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table("tags").to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table("tags").to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table("users").to_owned())
            .await?;

        Ok(())

    }
}
