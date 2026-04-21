use eframe::Storage;
use egui::{Align, Layout, global_theme_preference_switch};
use models::Uuid;
use command_bus::CommandBus;
use crate::ui::{articles::pages::{ArticleNew, ArticleTable}, core::page::{DbError, Page, PageAction}, login::{LoginAction, LoginForm}, tags::pages::{TagNew, TagTable}, users::pages::{UserEdit, UserTable}};
use core::{articles::dto::ArticleUI, users::dto::LoginResponse};
use core::tags::dto::TagUI;
use core::users::dto::UserUI;
use std::sync::{Arc, RwLock};

pub type SharedContext = Arc<RwLock<Option<LoginResponse>>>;


pub struct FormsApp {
    pub number: i32,
    command_tx: CommandBus,
    pub selected_page: Option<usize>,
    pub pages: Vec<Box<dyn Page>>,
    pub about_window: bool,
    pub user_context: Option<LoginResponse>,
    pub login_form: Option<LoginForm>,
    shared_context: SharedContext,
}


impl FormsApp {
    pub fn switch_to_page<T: Page + 'static>(&mut self) -> bool {
        let pos = self.pages.iter().position(|p| p.as_any().is::<T>());
        if let Some(pos) = pos {
            self.selected_page  = Some(pos);
            true
        } else {
            false
        }
    }
}



impl FormsApp {
    pub fn new(_storage: Option<&dyn Storage>, command_tx: CommandBus, shared_context: SharedContext) -> Self {
        Self {
            number: 0,
            command_tx,
            pages: Vec::new(),
            selected_page: None,
            about_window: false,
            user_context: None,
            login_form: None,
            shared_context,
        }
    }
}

impl FormsApp {
    pub fn add_page<T: Page>(&mut self, mut page: T) {
        page.init(&mut self.command_tx);
        self.pages.push(Box::new(page));
        self.selected_page = Some(self.pages.len()-1);
    }
}

impl eframe::App for FormsApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.command_tx.update();
        if self.login_form.is_some() {
            let action = if let Some(login_form) = self.login_form.as_mut() {
                egui::CentralPanel::default().show(ctx, |ui| {
                    login_form.show(ui, &mut self.command_tx)
                }).inner
            } else {
                LoginAction::None
            };
            match action {
                LoginAction::LoggedIn(login_response) => {
                    self.login_form = None;
                    let mut ctx = self.shared_context.write().unwrap();
                    *ctx = Some(login_response.clone());
                    self.user_context = Some(login_response);
                }
                LoginAction::Cancel => {
                    self.login_form = None;
                }
                LoginAction::None => {

                }
            }
        } else {
            egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if let Some(user_context) = &self.user_context {
                        ui.label(&user_context.user_context.user_name);
                        if ui.button("Log out").clicked() {
                            self.user_context = None;
                            let mut ctx = self.shared_context.write().unwrap();
                            *ctx = None;
                        }
                    } else {
                        if ui.button("Login").clicked() {
                            self.login_form = Some(LoginForm::new());
                        }
                    }
                    if ui.button("Help").clicked() {
                        self.about_window = true;
                    }
                    global_theme_preference_switch(ui);
                });
            });           
            egui::SidePanel::left("left_panel")
                .exact_width(100.0)
                .show(ctx, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        let resp = ui.button("Articles");
                        if resp.clicked() {
                            if !self.switch_to_page::<ArticleTable>() {
                                self.add_page(ArticleTable::new());
                            }
                        }                  
                        if resp.clicked_by(egui::PointerButton::Secondary) {
                            if !self.switch_to_page::<ArticleNew>() {
                                self.add_page(ArticleNew::new(ArticleUI::new()));
                            }
                        }
                        let resp = ui.button("Users");
                        if resp.clicked() {
                            if !self.switch_to_page::<UserTable>() {
                                self.add_page(UserTable::new());
                            }
                        }                  
                        if resp.clicked_by(egui::PointerButton::Secondary) {
                            if !self.switch_to_page::<UserEdit>() {
                                let now = core::time_now();
                                self.add_page(UserEdit::new_create(UserUI {
                                    id: Uuid::new_v4(),
                                    created_at: now,
                                    updated_at: now,
                                    ..Default::default()
                                }));
                            }
                        }
                        let resp = ui.button("Tags");
                        if resp.clicked() {
                            if !self.switch_to_page::<TagTable>() {
                                self.add_page(TagTable::new());
                            }
                        }                  
                        if resp.clicked_by(egui::PointerButton::Secondary) {
                            if !self.switch_to_page::<TagNew>() {
                                let now = core::time_now();
                                self.add_page(TagNew::new(TagUI {
                                    id: core::new_uuid(),
                                    created_at: now,
                                    ..Default::default()
                                }));
                            }
                        }
                    });
            });
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.horizontal(|ui| {
                    for (page_num, page) in self.pages.iter().enumerate() {
                        if ui.selectable_label(Some(page_num) == self.selected_page, page.title()).clicked() {
                            self.selected_page = Some(page_num);
                        }
                    }
                });
                let mut actions = Vec::new();
                for page in self.pages.iter_mut() {
                    page.update(&mut self.command_tx, &mut |a| actions.push(a));
                }
                self.pages.retain(|p| !p.should_close());
                for action in actions {
                    self.apply_action(action);
                }
                ui.separator();
                if let Some(selected_page) = self.selected_page {
                    let page_action = if let Some(page) = self.pages.get_mut(selected_page) {
                        page.show(ui, &mut self.command_tx)
                    } else {
                        PageAction::None
                    };
                    self.apply_action(page_action);
                }
                if self.about_window {
                    egui::Window::new("About")
                        .collapsible(false)
                        .resizable(false)
                        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0]) // Center the modal
                        .show(ctx, |ui| {
                            ui.with_layout(Layout::top_down(Align::Center), |ui| {
                                ui.heading("RealWorld Demo");
                                ui.spacing();
                                ui.label("Realworld example app implemented in Rust using egui and SeaORM.");
                                ui.label("MIT License");
                                ui.spacing();
                                ui.hyperlink_to("GitHub Site", "https://github.com/xdobry/realworld-egui-seaorm");
                                ui.label("Author: Artur T. <mail@xdobry.de>");
                            });
                            ui.spacing();
                            if ui.button("Cancel").clicked() {
                                self.about_window = false;
                            }
                        });
                    ui.disable();
                }
            });
        }
    }
}

impl FormsApp {
    pub fn apply_action(&mut self, page_action: PageAction) {
        match page_action {
            PageAction::AddPage(page) => {
                self.pages.push(page);
                self.selected_page = Some(self.pages.len()-1);
            }
            PageAction::AddError(msg) => {
                self.pages.push(Box::new(DbError::new(msg)));
                self.selected_page = Some(self.pages.len()-1);
            },
            _ => {
            }
        }
    }
}
