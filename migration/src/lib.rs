pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_table;
mod m20260429_091057_user_add_admin_flag;
mod m20260504_202226_add_compound_keys;


pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(m20220101_000001_create_table::Migration),
            Box::new(m20260429_091057_user_add_admin_flag::Migration),
            Box::new(m20260504_202226_add_compound_keys::Migration)]
    }
}
