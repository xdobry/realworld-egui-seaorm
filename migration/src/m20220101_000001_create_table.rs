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
                    .col(string("username").not_null().unique_key())
                    .col(string("email").not_null().unique_key())
                    .col(string("password_hash").not_null())
                    .col(text("bio"))
                    .col(text("image"))
                    .col(timestamp_with_time_zone("created_at").not_null().default(Expr::current_timestamp()))
                    .col(timestamp_with_time_zone("updated_at").not_null().default(Expr::current_timestamp()))
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table("tags")
                    .if_not_exists()
                    .col(uuid("id").primary_key())
                    .col(string("name").not_null().unique_key())
                    .col(timestamp_with_time_zone("created_at").not_null().default(Expr::current_timestamp()))
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table("articles")
                    .if_not_exists()
                    .col(uuid("id").primary_key())
                    .col(string("slug").not_null().unique_key())
                    .col(string("title").not_null())
                    .col(text("description").not_null())
                    .col(text("body").not_null())  
                    .col(uuid("author_id").not_null())                 
                    .col(timestamp_with_time_zone("created_at").not_null().default(Expr::current_timestamp()))
                    .col(timestamp_with_time_zone("updated_at").not_null().default(Expr::current_timestamp()))
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
                    .col(text("body").not_null())
                    .col(uuid("author_id").not_null())                 
                    .col(uuid("article_id").not_null())                 
                    .col(timestamp_with_time_zone("created_at").not_null().default(Expr::current_timestamp()))
                    .col(timestamp_with_time_zone("updated_at").not_null().default(Expr::current_timestamp()))
                    .foreign_key(
                        ForeignKey::create().name("fk_comments_author")
                            .from("articles", "author_id")
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
                    .col(uuid("user_id").not_null())                 
                    .col(uuid("article_id").not_null())                 
                    .col(timestamp_with_time_zone("created_at").not_null().default(Expr::current_timestamp()))
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

        Ok(())

    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {

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
