
use egui::{Id, Modal, TextEdit};
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};

use crate::ui::{ 
    core::{page::{Form, PageAction, UIContext}, 
    tables::{TableAction, TableMode}}, 
    users::tables::show_users_table, utils::date_time_ft
};
use command_bus::{CommandBus, UIBus};
use models::entity::users;
use core::{articles::dto::ArticleUI, entities::EntityIdent};
use core::users::api::{UserCommand, UserResult};
use core::api::{UICommand, UIResult};

#[derive(Default)]
pub struct ArticleForm {
    pub article: ArticleUI,
    pub user_list: Option<Vec<users::Model>>,
    pub user_list_opened: bool,
    event_bus: UIBus,
}

impl ArticleForm {
    pub fn show_ui(&mut self, ui: &mut egui::Ui, tx: &mut CommandBus, ui_context: &UIContext, page_action: &mut PageAction, cache: &mut CommonMarkCache) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            if ui_context.is_edit() {
                ui.label("title");
                TextEdit::singleline(&mut self.article.title).desired_width(f32::INFINITY).show(ui);
                ui.label("description");
                TextEdit::singleline(&mut self.article.description).desired_width(f32::INFINITY).show(ui);
                ui.label("body");
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
        }
    }

}

