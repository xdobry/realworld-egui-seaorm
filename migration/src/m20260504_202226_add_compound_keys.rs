use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts

        manager.create_index(Index::create()
            .name("article_favorites_user_id_article_id_key")
            .table("article_favorites")
            .col("user_id")
            .col("article_id")
            .unique()
            .to_owned(),
        ).await?;

        manager.create_index(Index::create()
            .name("article_tags_tag_id_article_id_key")
            .table("article_tags")
            .col("tag_id")
            .col("article_id")
            .unique()
            .to_owned(),
        ).await?;

        manager.create_index(Index::create()
            .name("user_follows_follower_id_followee_id_key")
            .table("user_follows")
            .col("follower_id")
            .col("followee_id")
            .unique()
            .to_owned(),
        ).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {

        manager.drop_index(
            Index::drop()
                .name("article_favorites_user_id_article_id_key")
                .table("article_favorites")
                .to_owned(),
        ).await?;

        manager.drop_index(
            Index::drop()
                .name("article_tags_tag_id_article_id_key")
                .table("article_tags")
                .to_owned(),
        ).await?;

        manager.drop_index(
            Index::drop()
                .name("user_follows_follower_id_followee_id_key")
                .table("user_follows")
                .to_owned(),
        ).await?;

        Ok(())
    }
}