use eframe::Storage;
use egui::{Align, Layout, global_theme_preference_switch};
use command_bus::{CommandBus, UIBus};
use crate::ui::{articles::pages::{ArticleEdit, ArticlePopularTable, ArticleTable}, core::page::{DbError, Page, PageAction, UIContext}, login::{LoginAction, LoginForm}, register::{RegisterAction, RegisterForm}, tags::pages::{TagEdit, TagTable}, users::pages::{UserEdit, UserTable}};
use core::{api::{UICommand, UIResult}, articles::dto::ArticleUI, entities::EntityIdent, tags::{api::{TagCommand, TagResult}, dto::TagListItem}, users::dto::LoginResponse};
use core::tags::dto::TagUI;
use std::sync::{Arc, RwLock};

pub type SharedContext = Arc<RwLock<Option<LoginResponse>>>;


pub struct FormsApp {
    pub number: i32,
    command_tx: CommandBus,
    ui_bus: UIBus,
    pub selected_page: Option<usize>,
    pub pages: Vec<Box<dyn Page>>,
    pub about_window: bool,
    pub user_context: Option<LoginResponse>,
    pub login_form: Option<LoginForm>,
    pub register_form: Option<RegisterForm>,
    pub popular_tags: Vec<TagListItem>,
    pub tags_requested: bool,
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
            register_form: None,
            popular_tags: Vec::new(),
            shared_context,
            ui_bus: UIBus::default(),
            tags_requested: false,
        }
    }
}

impl FormsApp {
    pub fn add_page<T: Page>(&mut self, mut page: T, ui_context: &UIContext) {
        page.init(&mut self.command_tx, ui_context);
        self.pages.push(Box::new(page));
        self.selected_page = Some(self.pages.len()-1);
    }
}

impl eframe::App for FormsApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.command_tx.update();
        if !self.tags_requested {
            self.ui_bus.send_task(&mut self.command_tx, UICommand::Tag(TagCommand::PopularTags));
            self.tags_requested = true;
        } else {
            if let Ok(result) = self.ui_bus.try_recv() {
                match result {
                    UIResult::Tag(TagResult::PopularTags(tags)) => {
                        self.popular_tags = tags;
                    },
                    _ => {

                    }
                }
            }
        }
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
        } else if self.register_form.is_some() {
            let action = if let Some(register_form) = self.register_form.as_mut() {
                egui::CentralPanel::default().show(ctx, |ui| {
                    register_form.show(ui, &mut self.command_tx)
                }).inner
            } else {
                RegisterAction::None
            };
            match action {
                RegisterAction::LoggedIn(login_response) => {
                    self.register_form = None;
                    let mut ctx = self.shared_context.write().unwrap();
                    *ctx = Some(login_response.clone());
                    self.user_context = Some(login_response);
                }
                RegisterAction::Cancel => {
                    self.register_form = None;
                }
                RegisterAction::None => {

                }
            }
        } else {
            let ui_context = UIContext::new(&self.user_context);
            egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    global_theme_preference_switch(ui);
                    if ui.button("Help").clicked() {
                        self.about_window = true;
                    }
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let mut page_action = PageAction::None;
                        if let Some(user_context) = &self.user_context {
                            if ui.button("\u{1F464}").clicked() {
                                page_action = PageAction::Navigate(EntityIdent::User(ui_context.user_id()));
                            }
                            if ui_context.is_admin() {
                                ui.strong(format!("admin: {}",&user_context.user_context.user_name));
                            } else {
                                ui.label(&user_context.user_context.user_name);
                            }
                           
                            if ui.button("Log out").clicked() {
                                self.user_context = None;
                                let mut ctx = self.shared_context.write().unwrap();
                                *ctx = None;
                            }
                        } else {
                            if ui.button("Register").clicked() {
                                self.register_form = Some(RegisterForm::new());
                            }
                            if ui.button("Login").clicked() {
                                self.login_form = Some(LoginForm::new());
                            }
                        }
                        self.apply_action(page_action, &ui_context);
                    });
                });
            });           
            egui::SidePanel::left("left_panel")
                .exact_width(100.0)
                .show(ctx, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.strong("Articles");
                        ui.indent("articles", |ui| {
                            let resp = ui.button("Newest");
                            if resp.clicked() {
                                if !self.switch_to_page::<ArticleTable>() {
                                    self.apply_action(PageAction::Navigate(EntityIdent::ArticleList), &ui_context);
                                }
                            }
                            if ui_context.is_admin() {
                                if resp.clicked_by(egui::PointerButton::Secondary) {
                                    self.add_page(ArticleEdit::new_create(ArticleUI::new(ui_context.user_id())), &ui_context);
                                }
                            }
                            if ui.button("Popular").clicked() {
                                self.apply_action(PageAction::Navigate(EntityIdent::ArticlePopularList), &ui_context);
                            }
                            if !ui_context.is_anonymous() {
                                if ui.button("Feed").clicked() {
                                    self.apply_action(PageAction::Navigate(EntityIdent::ArticleListFollowed(ui_context.user_id())), &ui_context);
                                }
                                if ui.button("Favorites").clicked() {
                                    self.apply_action(PageAction::Navigate(EntityIdent::ArticleListFavorites(ui_context.user_id())), &ui_context);
                                }
                                if ui.button("My").clicked() {
                                    self.apply_action(PageAction::Navigate(EntityIdent::ArticleListAuthor(ui_context.user_id())), &ui_context);
                                }
                            }
                        });
                        let resp = ui.button("Users");
                        if resp.clicked() {
                            if !self.switch_to_page::<UserTable>() {
                                self.add_page(UserTable::new(), &ui_context);
                            }
                        }     
                        if ui_context.is_admin() {
                            if resp.clicked_by(egui::PointerButton::Secondary) {
                                if !self.switch_to_page::<UserEdit>() {
                                    self.add_page(UserEdit::new_create(), &ui_context);
                                }
                            }
                        }
                        let resp = ui.button("Tags");
                        if resp.clicked() {
                            if !self.switch_to_page::<TagTable>() {
                                self.add_page(TagTable::new(), &ui_context);
                            }
                        }
                        if self.popular_tags.len()>0 {
                            ui.indent("tags", |ui| {
                                let mut page_action = PageAction::None;
                                for tag in self.popular_tags.iter() {
                                    let resp = ui.button(format!("{} {}",tag.name.as_str(),tag.count));
                                    if resp.clicked() {
                                        page_action = PageAction::Navigate(EntityIdent::ArticleListTag(tag.id));
                                    }
                                }
                                self.apply_action(page_action, &ui_context);
                            });                           
                        }
                        if ui_context.is_admin() {
                            if resp.clicked_by(egui::PointerButton::Secondary) {
                                self.add_page(TagEdit::new_create(TagUI::new()), &ui_context);
                            }
                        }
                    });
            });
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.horizontal(|ui| {
                    for (page_num, page) in self.pages.iter().enumerate() {
                        if ui.selectable_label(Some(page_num) == self.selected_page, page.title(&ui_context)).clicked() {
                            self.selected_page = Some(page_num);
                        }
                    }
                });
                let mut actions = Vec::new();
                for page in self.pages.iter_mut() {
                    page.update(&mut self.command_tx, &ui_context, &mut |a| actions.push(a));
                }
                self.pages.retain(|p| !p.should_close());
                if let Some(selected_page) = self.selected_page {
                    if selected_page>=self.pages.len() && self.pages.len()>0 {
                        self.selected_page = Some(self.pages.len()-1)
                    }
                }
                for action in actions {
                    self.apply_action(action, &ui_context);
                }
                ui.separator();
                if let Some(selected_page) = self.selected_page {
                    let page_action = if let Some(page) = self.pages.get_mut(selected_page) {
                        page.show(ui, &mut self.command_tx, &ui_context)
                    } else {
                        PageAction::None
                    };
                    self.apply_action(page_action, &ui_context);
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
    pub fn apply_action(&mut self, page_action: PageAction, ui_context: &UIContext) {
        match page_action {
            PageAction::AddPage(page) => {
                self.pages.push(page);
                self.selected_page = Some(self.pages.len()-1);
            }
            PageAction::AddError(msg) => {
                self.pages.push(Box::new(DbError::new(msg)));
                self.selected_page = Some(self.pages.len()-1);
            },
            PageAction::Navigate(entity_ident) => {
                let pos = self.pages.iter().position(|p| p.entity_ident()==entity_ident);
                if let Some(pos) = pos {
                    self.selected_page = Some(pos);
                } else {
                    match entity_ident {
                        EntityIdent::Article(uuid) => {
                            self.add_page(ArticleEdit::new(uuid), &ui_context);
                        }
                        EntityIdent::Tag(uuid) => {
                            self.add_page(TagEdit::new(uuid), &ui_context);
                        }
                        EntityIdent::User(uuid) => {
                            self.add_page(UserEdit::new(uuid), &ui_context);
                        },
                        EntityIdent::Comment(_uuid) => {
                            // Where is no standalone comment page
                        },
                        EntityIdent::ArticleList | EntityIdent::ArticleListAuthor(_) | EntityIdent::ArticleListFavorites(_) | EntityIdent::ArticleListFollowed(_) 
                        | EntityIdent::ArticleListTag(_) => {
                            self.add_page(ArticleTable::new(entity_ident), &ui_context);
                        },
                        EntityIdent::ArticlePopularList => {
                            self.add_page(ArticlePopularTable::new(entity_ident), &ui_context);
                        },
                        EntityIdent::None => {

                        }
                    }
                }
            },            
            PageAction::None => {
            }
        }
    }
}
