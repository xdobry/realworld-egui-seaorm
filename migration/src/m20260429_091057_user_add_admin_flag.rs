use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts

        manager.alter_table(Table::alter().table("users")
            .add_column(ColumnDef::new("is_admin").boolean().not_null().default(false))
            .to_owned()).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {

        manager.alter_table(Table::alter().table("users")
            .drop_column("is_admin")
            .to_owned()).await
    }
}
