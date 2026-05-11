
use egui::{Id, Label, Modal, Sense};

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

impl Form for ArticleForm {
    fn show_ui(&mut self, ui: &mut egui::Ui, tx: &mut CommandBus, ui_context: &UIContext, page_action: &mut PageAction) {
        ui.label("slug");
        ui.add_enabled_ui(ui_context.is_edit(), |ui| {
            ui.text_edit_singleline(&mut self.article.slug);
        });
        ui.label("title");
        ui.add_enabled_ui(ui_context.is_edit(), |ui| {
            ui.text_edit_singleline(&mut self.article.title);
        });
        ui.label("description");
        ui.add_enabled_ui(ui_context.is_edit(), |ui| {
            ui.text_edit_multiline(&mut self.article.description);
        });
        ui.label("body");
        ui.add_enabled_ui(ui_context.is_edit(), |ui| {
            ui.text_edit_multiline(&mut self.article.body);
        });
        ui.label("author id");
        ui.horizontal(|ui| {
            let user_name = self.article.author_label.to_string();
            if ui.link(user_name).clicked() {
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
        ui.horizontal(|ui| {
            ui.strong("created at");
            ui.label(date_time_ft(self.article.created_at));
            ui.strong("updated at");
            ui.label(date_time_ft(self.article.updated_at));
        });
    }

    fn update<F>(&mut self, _tx: &mut CommandBus, mut _emit: F) 
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
}

impl ArticleForm {
    pub fn new(article: ArticleUI) -> Self {
        Self {
            article: article,
            user_list: None,
            user_list_opened: false, 
            event_bus: UIBus::default(),          
        }
    }
}