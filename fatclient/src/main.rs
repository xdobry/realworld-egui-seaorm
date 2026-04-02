use std::thread;

use egui::ViewportBuilder;
use sea_orm::Database;
use server_core::handle_ui_command;
use ui::app::FormsApp;
use core::api::{UIResult};
use tokio::runtime::Runtime;
use tokio::sync::mpsc::{self};
use ui::api::{CommandBus, UITask};

fn main() -> Result<(), eframe::Error> {

    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default(),
        ..eframe::NativeOptions::default()
    };
    let args = std::env::args().skip(1).collect();

    let (command_tx, mut command_rx) = mpsc::channel::<UITask>(5);

    thread::spawn(move || {
        let rt = Runtime::new().unwrap();
        rt.block_on(async move {
            // Example async task
            let db = Database::connect("postgres://realworld:realworld@localhost/realworld").await;
            if let Ok(db) = db {
                while let Some(mut cmd) = command_rx.recv().await {
                    let result = handle_ui_command(cmd.command, &mut cmd.response, &db).await;
                    if let Err(err) = result {
                        println!("db error {:?}",err);
                        cmd.response.send(UIResult::DbError(err.to_string()));
                    }
                }
            } else {
                while let Some(mut cmd) = command_rx.recv().await {
                    cmd.response.send(UIResult::DbError("no db connection".to_string()));
                }               
            }
        });
    });

    let command_bus = CommandBus::new(command_tx);

    eframe::run_native(
        "RealWorld App - Egui",
        options,
        Box::new(|cc| 
            Ok(Box::new(FormsApp::new(cc.storage, args, command_bus)))),
    )
}