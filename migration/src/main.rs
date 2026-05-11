use std::env;

use argon2::Config;
use dotenvy::dotenv;
use migration::sea_orm::{ActiveValue::Set, Database};
use models::entity::users;
use sea_orm_migration::prelude::*;
use migration::sea_orm::EntityTrait;
use uuid::Uuid;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    // Example:
    // cargo run -- add-admin admin@example.com secret123
    if args.len() >= 4 && args[1] == "add-admin" {
        let email = &args[2];
        let password = &args[3];

        add_admin(email, password).await;
    } else {
        cli::run_cli(migration::Migrator).await;
    }
}

async fn add_admin(email: &str, password: &str) {
    println!("Creating admin user:");
    println!("email: {}", email);
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let password_salt = env::var("PASSWORD_SALT").expect("PASSWORD_SALT must be set");

    let db = Database::connect(database_url).await.unwrap();
    let argo_config = Config::default();

    let hashed_password = argon2::hash_encoded(password.as_bytes(), password_salt.as_bytes(), &argo_config).unwrap();

    let admin = users::ActiveModel {
        id: Set(Uuid::new_v4()),
        email: Set(email.to_string()),
        username: Set("admin".to_string()),
        password_hash: Set(hashed_password),
        is_admin: Set(true),
        ..Default::default()
    };
    users::Entity::insert(admin).exec(&db).await.unwrap();
    println!("Admin creation finished");
}