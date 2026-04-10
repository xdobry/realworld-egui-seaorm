use eframe::Storage;
use egui::{Align, Layout, global_theme_preference_switch};
use models::Uuid;
use command_bus::CommandBus;
use crate::{ 
    ui::{articles::pages::{ArticleNew, ArticleTable}, 
    core::page::{DbError, Page, PageAction}, 
    tags::pages::{TagNew, TagTable}, users::pages::{UserNew, UserTable}}};
use core::articles::dto::ArticleUI;
use core::tags::dto::TagUI;
use core::users::dto::UserUI;

// use models::entity::{articles, tags, users};

pub struct FormsApp {
    pub number: i32,
    command_tx: CommandBus,
    pub selected_page: Option<usize>,
    pub pages: Vec<Box<dyn Page>>,
    pub pending_actions: usize,
    pub about_window: bool,
}


impl FormsApp {
    pub fn swith_to_page<T: Page + 'static>(&mut self) -> bool {
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
    pub fn new(_storage: Option<&dyn Storage>, command_tx: CommandBus) -> Self {
        Self {
            number: 0,
            command_tx,
            pages: Vec::new(),
            selected_page: None,
            pending_actions: 0,
            about_window: false,
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
        egui::SidePanel::left("left_panel")
            .exact_width(100.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if ui.button("Help").clicked() {
                        self.about_window = true;
                    }
                    global_theme_preference_switch(ui);
                });
                egui::ScrollArea::vertical().show(ui, |ui| {
                    let resp = ui.button("Articles");
                    if resp.clicked() {
                        if !self.swith_to_page::<ArticleTable>() {
                            self.add_page(ArticleTable::new());
                        }
                    }                  
                    if resp.clicked_by(egui::PointerButton::Secondary) {
                        if !self.swith_to_page::<ArticleNew>() {
                            self.add_page(ArticleNew::new(ArticleUI::new()));
                        }
                    }
                    let resp = ui.button("Users");
                    if resp.clicked() {
                        if !self.swith_to_page::<UserTable>() {
                            self.add_page(UserTable::new());
                        }
                    }                  
                    if resp.clicked_by(egui::PointerButton::Secondary) {
                        if !self.swith_to_page::<UserNew>() {
                            let now = core::time_now();
                            self.add_page(UserNew::new(UserUI {
                                id: Uuid::new_v4(),
                                created_at: now,
                                updated_at: now,
                                ..Default::default()
                            }));
                        }
                    }
                    let resp = ui.button("Tags");
                    if resp.clicked() {
                        if !self.swith_to_page::<TagTable>() {
                            self.add_page(TagTable::new());
                        }
                    }                  
                    if resp.clicked_by(egui::PointerButton::Secondary) {
                        if !self.swith_to_page::<TagNew>() {
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
                match action {
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
                break;
            }
            ui.separator();
            if let Some(selected_page) = self.selected_page {
                let page_action = if let Some(page) = self.pages.get_mut(selected_page) {
                    page.show(ui, &mut self.command_tx)
                } else {
                    PageAction::None
                };
                match page_action {
                    PageAction::Close => {
                        self.pages.remove(selected_page);
                    }
                    PageAction::AddPage(page) => {
                        self.pages.push(page);
                        self.selected_page = Some(self.pages.len()-1);
                    },
                    PageAction::AddError(msg) => {
                        self.pages.push(Box::new(DbError::new(msg)));
                        self.selected_page = Some(self.pages.len()-1);
                    },
                    _ => {

                    }
                }

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
        if self.pending_actions>0 {
            ctx.request_repaint();
        }
    }
}
