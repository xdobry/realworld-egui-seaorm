use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {

        manager
            .create_table(
                Table::create()
                    .table("images")
                    .if_not_exists()
                    .col(uuid("id").primary_key())
                    .col(string("title").not_null())
                    .col(blob("data").not_null())
                    .col(uuid("article_id").not_null())
                    .col(string("mimetype").not_null())
                    .col(timestamp_with_time_zone("created_at").default(Expr::current_timestamp()))
                    .foreign_key(
                        ForeignKey::create().name("fk_images_article")
                            .from("images", "article_id")
                            .to("articles", "id")
                            .on_delete(ForeignKeyAction::Cascade))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {

        manager
            .drop_table(Table::drop().table("images").to_owned())
            .await
    }
}
