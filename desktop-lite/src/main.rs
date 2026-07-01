use core::images::api::{ImageCommand, ImageResult};
use core::users::api::UserResult;
use core::users::dto::{LoginResponse, UserContext};
use std::sync::{Arc, RwLock};
use std::{env, thread};

use dashmap::DashMap;
use egui::ViewportBuilder;
use egui::load::{BytesLoadResult, BytesLoader, BytesPoll, LoadError};
use migration::Migrator;
use migration::MigratorTrait;
use models::Uuid;
use models::entity::users;
use sea_orm::ActiveValue::Set;
use sea_orm::{Database, DatabaseConnection};
use server_core::{CallContext, handle_ui_command};
use ui::app::FormsApp;
use core::api::{UICommand, UIResult};
use tokio::runtime::Runtime;
use tokio::sync::mpsc::{self, Sender};
use command_bus::{CommandBus, CommandBusUpdate, ResponseChannel, UITask};
use argon2::{self, Config};
use dotenvy::dotenv;

fn main() -> Result<(), eframe::Error> {

    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default().with_inner_size([1024.0,800.0]),
        ..eframe::NativeOptions::default()
    };

    dotenv().ok();
    env_logger::init();

    let database_url = env::var("SQLITE_DATABASE_URL")
        .expect("DATABASE_URL must be set");

    eframe::run_native(
        "RealWorld App - Egui Standalone Client",
        options,
        Box::new(|cc| {
            let egui_context = cc.egui_ctx.clone();
            let (command_tx, mut command_rx) = mpsc::channel::<UITask>(5);

            let call_context = MyCallContext::new();
            let shared_context = call_context.shared_context.clone();
            let loader_bus = CommandBus::new(command_tx.clone());
            let command_bus = CommandBusUpdate::new(command_tx);
            let form_app = FormsApp::new(cc.storage, command_bus, shared_context);
            let ui_result_sender = form_app.ui_bus.result_tx.clone();

            let (image_result_tx, mut image_result_rx) = mpsc::channel::<UIResult>(5);
            let blob_loader = BlobLoader::new( loader_bus, image_result_tx);
            let blob_cache = blob_loader.cache.clone();
            egui_context.add_bytes_loader(Arc::new(blob_loader));

            
            thread::spawn(move || {
                let rt = Runtime::new().unwrap();
                rt.block_on(async move {
                    tokio::spawn(async move {
                        while let Some(msg) = image_result_rx.recv().await {
                            if let UIResult::Image(ImageResult::Image(model)) = msg {
                                let blob_uri = format!("blob://{}", model.id.to_string());
                                blob_cache.insert(blob_uri, BlobEntry::Ready(Arc::from(model.data)));
                            }
                        }
                    });
                    // Example async task
                    let db = Database::connect(database_url).await;
                    // cli::run_cli_with_connection(migration::Migrator, db).await;
                    if let Ok(db) = db {
                        let init_res = init_db(&db).await;
                        match init_res {
                            Err(err) => {
                                println!("db init error {:?}",err);
                                return;
                            }
                            Ok(user_context) => {
                                let login_response = LoginResponse {
                                    token: call_context.create_token(&user_context),
                                    user_context,
                                };
                                let _s = ui_result_sender.send(UIResult::User(UserResult::Login(login_response))).await;
                            }
                        }
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
           
            Ok(Box::new(form_app))
        }),
    )
}

async fn init_db(db: &DatabaseConnection) -> Result<UserContext,sea_orm::DbErr>  {
    Migrator::up(db, None).await?;
    let admin_user = users::Entity::find_by_username("admin").one(db).await?;
    if let Some(admin_user) = admin_user {
        return Ok(UserContext {
            is_admin: true,
            user_name: admin_user.username,
            user_email: admin_user.email,
            user_id: admin_user.id,
        });
    } else {
        use sea_orm::EntityTrait;
        let uuid = Uuid::new_v4();
        let email = "standalone@db.com".to_string();
        let name = "admin".to_string();
        let admin = users::ActiveModel {
            id: Set(uuid.clone()),
            email: Set(email.clone()),
            username: Set(name.clone()),
            password_hash: Set("no_pass".to_string()),
            is_admin: Set(true),
            ..Default::default()
        };
        let user_context = UserContext { 
            is_admin: true,
            user_email: email,
            user_name: name,
            user_id: uuid,
        };
        users::Entity::insert(admin).exec(db).await?;
        return Ok(user_context);
    }
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

struct BlobLoader {
    bus: CommandBus,
    cache: Arc<DashMap<String, BlobEntry>>,
    image_response_tx: Sender<UIResult>,
}

enum BlobEntry {
    Loading,
    Ready(Arc<[u8]>),
}

impl BlobLoader {
    pub fn new(command_bus: CommandBus, image_response_tx: Sender<UIResult>) -> Self {
        Self {
            bus: command_bus,
            cache: Arc::new(DashMap::new()),
            image_response_tx,
        }
    }
}

// 2. Implement the BytesLoader trait
impl BytesLoader for BlobLoader {
    fn id(&self) -> &str {
        "blob_loader"
    }
    fn load(&self, _ctx: &egui::Context, uri: &str) -> BytesLoadResult {
        if uri.starts_with("blob://") {
            let data = self.cache.get(uri);
            if let Some(data) = data {
                match data.value() {
                    BlobEntry::Loading => {
                        return Ok(BytesPoll::Pending { size: None })                
                    },
                    BlobEntry::Ready(data) => {
                        return Ok(BytesPoll::Ready { size: None, bytes: egui::load::Bytes::Shared(data.clone()), mime: None })
                    }
                }
            } else {
                if let Ok(blob_id) = Uuid::try_parse(&uri[7..]) {
                    let response_channel = ResponseChannel::new(self.image_response_tx.clone());
                    self.bus.dispatch(UITask { 
                        command: UICommand::Image(ImageCommand::Load(blob_id)),
                        response: response_channel });
                    let _r = self.cache.insert(uri.into(), BlobEntry::Loading);
                    return Ok(BytesPoll::Pending { size: None })      
                } else {
                    log::error!("can not parse uuid {} from {}", &uri[7..],uri);
                    return Err(LoadError::FormatNotSupported { detected_format: None })
                }
            }
        } else {
            Err(LoadError::NotSupported)
        }
    }
    fn forget(&self, uri: &str) {
        log::info!("forget {}",uri);
    }
    fn forget_all(&self) {
        log::info!("forget all");
    }
    fn byte_size(&self) -> usize {
        0
    }
}