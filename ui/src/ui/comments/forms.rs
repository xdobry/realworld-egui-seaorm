use egui::{Id, Modal};

use crate::ui::{core::{page::{Form, PageAction, UIContext}, 
    tables::{TableAction, TableMode}}, users::tables::show_users_table, utils::date_time_ft};
use command_bus::{CommandBus, UIBus};    
use models::entity::users;
use core::users::api::{UserCommand, UserResult};
use core::comments::dto::CommentAuthor;
use core::api::{UICommand, UIResult};

#[derive(Default)]
pub struct CommentForm {
    pub comment: CommentAuthor,
    pub user_list: Option<Vec<users::Model>>,
    pub user_list_opened: bool,
    event_bus: UIBus,
}

impl Form for CommentForm {
    fn show_ui(&mut self, ui: &mut egui::Ui, tx: &mut CommandBus,ui_context: &UIContext, _page_action: &mut PageAction) {
        ui.label("body");
        ui.text_edit_multiline(&mut self.comment.body);
        ui.label("author id");
        ui.horizontal(|ui| {
            let id_string = self.comment.author_name.to_string();
            ui.label(id_string);
            if ui_context.is_admin() {
                if ui.button("...").clicked() {
                    self.user_list_opened = true;
                    if self.user_list.is_none() {
                        self.event_bus.send_task(tx, UICommand::User(UserCommand::Reload));
                    }
                }
                if self.user_list_opened {
                    let modal = Modal::new(Id::new("mod_select_user2")).show(ui.ctx(), |ui| {
                        ui.set_width(200.0);
                        if let Some(user_list) = &self.user_list {
                            let table_action = show_users_table(ui,user_list, TableMode::Select);
                            match table_action {
                                TableAction::SelectItem(uuid,label) => {
                                    self.comment.author_id = uuid;
                                    self.comment.author_name = label;
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
            ui.label(date_time_ft(self.comment.created_at));
            ui.strong("updated at");
            ui.label(date_time_ft(self.comment.updated_at));
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

impl CommentForm {
    pub fn new(comment: CommentAuthor) -> Self {
        Self {
            comment: comment,
            user_list: None,
            user_list_opened: false,   
            event_bus: UIBus::default(),        
        }
    }
}