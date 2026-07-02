
use egui::{Id, Modal, TextEdit};
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};

use crate::ui::{ 
    core::{page::{Form, PageAction, UIContext}, 
    tables::{TableAction, TableMode}}, 
    users::tables::show_users_table, utils::date_time_ft
};
use command_bus::{CommandBus, UIBus};
use models::entity::{images, users};
use core::{articles::dto::ArticleUI, entities::EntityIdent, images::api::ImageCommand, new_uuid, time_now};
use core::users::api::{UserCommand, UserResult};
use core::api::{UICommand, UIResult};
use std::fs;
#[cfg(not(target_arch = "wasm32"))]
use rfd::FileDialog;

#[derive(Default)]
pub struct ArticleForm {
    pub article: ArticleUI,
    pub user_list: Option<Vec<users::Model>>,
    pub user_list_opened: bool,
    event_bus: UIBus,
    #[cfg(target_arch = "wasm32")]
    pub file_upload: Option<poll_promise::Promise<Result<FileData, anyhow::Error>>>,
}

#[cfg(target_arch = "wasm32")]
struct FileData {
    pub path: String,
    pub data: Vec<u8>,
}

impl ArticleForm {
    pub fn show_ui(&mut self, ui: &mut egui::Ui, tx: &mut CommandBus, ui_context: &UIContext, page_action: &mut PageAction, cache: &mut CommonMarkCache) {
        #[cfg(target_arch = "wasm32")]
        if let Some(promise) = self.file_upload.take() {
            if promise.ready().is_some() {
                match promise.block_and_take() {
                    Ok(FileData { path, data }) => {
                        let uuid = new_uuid();
                        self.article.body.push_str(format!("![{}](blob://{})",path,uuid).as_str());
                        let image = images::Model {
                            id: uuid,
                            mimetype: "img".into(),
                            title: path,
                            created_at: time_now(),
                            article_id: self.article.id,
                            data: data,
                        };
                        self.event_bus.send_task(tx, UICommand::Image(ImageCommand::Create(image)));
                        self.file_upload = None;                  
                    }
                    Err(e) => {
                        self.file_upload = None;                  
                    }
                }
            } else {
                self.file_upload = Some(promise);
            }
        }
        egui::ScrollArea::vertical().show(ui, |ui| {
            if ui_context.is_edit() {
                ui.label("title");
                TextEdit::singleline(&mut self.article.title).desired_width(f32::INFINITY).show(ui);
                ui.label("description");
                TextEdit::singleline(&mut self.article.description).desired_width(f32::INFINITY).show(ui);
                ui.horizontal(|ui| {
                    ui.label("body");
                    if ui.button("Add Image").clicked() {
                        #[cfg(not(target_arch = "wasm32"))]
                        if let Some(path) = FileDialog::new()
                            .add_filter("Image", &["png","jpg","gif"])
                            .pick_file()
                        {
                            let bytes = fs::read(&path);
                            if let Ok(bytes) = bytes {
                                let title = path.file_name().unwrap().display().to_string();
                                let uuid = new_uuid();
                                self.article.body.push_str(format!("![{}](blob://{})",title,uuid).as_str());
                                let image = images::Model {
                                    id: uuid,
                                    mimetype: "img".into(),
                                    title: title,
                                    created_at: time_now(),
                                    article_id: self.article.id,
                                    data: bytes,
                                };
                                self.event_bus.send_task(tx, UICommand::Image(ImageCommand::Create(image))); 
                            } else {
                                println!("no bytes");
                            }
                        }
                        #[cfg(target_arch = "wasm32")]
                        {
                            use poll_promise::Promise;
                            self.file_upload = Some(Promise::spawn_local(async {
                                let file_selected = rfd::AsyncFileDialog::new()
                                    .add_filter("Image", &["png","jpg","gif"])
                                    .pick_file()
                                    .await;
                                if let Some(curr_file) = file_selected {
                                    let buf = curr_file.read().await;
                                    return Ok(FileData {
                                        path: curr_file.file_name(),
                                        data: buf,
                                    });
                                }
                                // no file selected
                                Err(anyhow::anyhow!("Upload: no file Selected"))
                            }));
                        }
                    }
                });
                TextEdit::multiline(&mut self.article.body).desired_width(f32::INFINITY).desired_rows(50).show(ui);
                ui.horizontal(|ui| {
                    ui.label("author:");
                    ui.horizontal(|ui| {
                        if ui.link(&self.article.author_label).clicked() {
                            *page_action = PageAction::Navigate(EntityIdent::User(self.article.author_id));
                        }
                        if ui_context.is_admin() && ui_context.is_edit() {
                            if ui.button("...").clicked() {
                                self.user_list_opened = true;
                                if self.user_list.is_none() {
                                    self.event_bus.send_task(tx, UICommand::User(UserCommand::Reload));
                                }
                            }
                            if self.user_list_opened {
                                let modal = Modal::new(Id::new("mod_select_user")).show(ui.ctx(), |ui| {
                                    ui.set_width(200.0);
                                    if let Some(user_list) = &self.user_list {
                                        let table_action = show_users_table(ui,user_list, TableMode::Select);
                                        match table_action {
                                            TableAction::SelectItem(uuid,label) => {
                                                self.article.author_id = uuid;
                                                self.article.author_label = label;
                                                ui.close();
                                            }
                                            _ => {

                                            }
                                        }
                                    }
                                    egui::Sides::new().show(
                                        ui,
                                        |_ui| {},
                                        |ui| {
                                            if ui.button("Cancel").clicked() {
                                                ui.close();
                                            }
                                        },
                                    );
                                });
                                if modal.should_close() {
                                    self.user_list_opened = false;
                                }
                            }
                        }
                    });
                });
            } else {
                ui.heading(&self.article.title);
                ui.label(&self.article.description);
                if ui.link(&self.article.author_label).clicked() {
                    *page_action = PageAction::Navigate(EntityIdent::User(self.article.author_id));
                }
                ui.separator();
                CommonMarkViewer::new().show(ui, cache, &self.article.body);
                ui.separator();           
            }
            ui.horizontal(|ui| {
                ui.strong("created at");
                ui.label(date_time_ft(self.article.created_at));
                ui.strong("updated at");
                ui.label(date_time_ft(self.article.updated_at));
            });
        });
    }

    pub fn update<F>(&mut self, _tx: &mut CommandBus, mut _emit: F) 
    where 
        F: FnMut(PageAction),
    {
        if let Ok(msg) = self.event_bus.try_recv() {
            match msg {
                UIResult::User(UserResult::Users(users)) => {
                    self.user_list = Some(users);
                }
                _ => {

                }
            }
        }
    }

    pub fn new(article: ArticleUI) -> Self {
        Self {
            article: article,
            user_list: None,
            user_list_opened: false, 
            event_bus: UIBus::default(),
            #[cfg(target_arch = "wasm32")]
            file_upload: None,       
        }
    }

}

