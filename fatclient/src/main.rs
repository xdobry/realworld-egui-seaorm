use std::sync::{Arc, RwLock};
use std::{env, thread};

use egui::ViewportBuilder;
use sea_orm::Database;
use server_core::{CallContext, handle_ui_command};
use ui::app::FormsApp;
use core::api::{UIResult};
use tokio::runtime::Runtime;
use tokio::sync::mpsc::{self};
use command_bus::{CommandBus, UITask};
use argon2::{self, Config};
use dotenvy::dotenv;

fn main() -> Result<(), eframe::Error> {

    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default(),
        ..eframe::NativeOptions::default()
    };

    dotenv().ok();
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    eframe::run_native(
        "RealWorld App - Egui Fat Client",
        options,
        Box::new(|cc| {
            let egui_context = cc.egui_ctx.clone();
            let (command_tx, mut command_rx) = mpsc::channel::<UITask>(5);
            let call_context = MyCallContext::new();
            let shared_context = call_context.shared_context.clone();

            thread::spawn(move || {
                let rt = Runtime::new().unwrap();
                rt.block_on(async move {
                    // Example async task
                    let db = Database::connect(database_url).await;
                    if let Ok(db) = db {
                        while let Some(mut cmd) = command_rx.recv().await {
                            let result = handle_ui_command(cmd.command, &mut cmd.response, &db, &call_context).await;
                            if let Err(err) = result {
                                println!("db error {:?}",err);
                                cmd.response.send(UIResult::DbError(err.to_string()));
                            }
                            egui_context.request_repaint();
                        }
                    } else {
                        while let Some(mut cmd) = command_rx.recv().await {
                            cmd.response.send(UIResult::DbError("no db connection".to_string()));
                        }               
                    }
                });
            });
            let command_bus = CommandBus::new(command_tx);
           
            Ok(Box::new(FormsApp::new(cc.storage, command_bus, shared_context)))
        }),
    )
}

struct MyCallContext<'a> {
    password_salt: String,
    argo_config: Config<'a>,
    shared_context : ui::app::SharedContext,
}

impl MyCallContext<'_> {
    fn new() -> Self {
        MyCallContext { 
            password_salt: env::var("PASSWORD_SALT").expect("PASSWORD_SALT must be set"),
            argo_config: Config::default(),
            shared_context: Arc::new(RwLock::new(None)),
        }
    }
}

impl CallContext for MyCallContext<'_> {
    fn is_admin(&self) -> bool {
        if let Ok(shared_context) = self.shared_context.read() {
            if let Some(user_context) = &*shared_context {
                return user_context.user_context.is_admin;
            }
        }
        false
    }

    fn user_id(&self) -> Option<sea_orm::prelude::Uuid> {
        if let Ok(shared_context) = self.shared_context.read() {
            if let Some(user_context) = &*shared_context {
                return Some(user_context.user_context.user_id);
            }
        }
        None
    }

    fn encode_password(&self, password: &str) -> String {
        let password_bytes = password.as_bytes();
        let hashed_password = argon2::hash_encoded(password_bytes, self.password_salt.as_bytes(), &self.argo_config).unwrap();
        hashed_password
    }

    fn verify_password(&self, attempted_password: &str, hash: &str) -> bool {
        argon2::verify_encoded(hash, attempted_password.as_bytes()).unwrap()
    }
    
    fn create_token(&self, _user_context: &core::users::dto::UserContext) -> Vec<u8> {
        // fat client does not need a real token, here only dummy implementation
        vec![]
    }
}