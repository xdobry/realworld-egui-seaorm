use eframe::Storage;
use sea_orm::{prelude::{DateTimeWithTimeZone, Uuid}};
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
        }
    }
}

impl FormsApp {
    pub fn add_page<T: Page>(&mut self, mut page: T) {
        println!("add page");
        page.init(&mut self.command_tx);
        println!("page initialized");
        self.pages.push(Box::new(page));
        self.selected_page = Some(self.pages.len()-1);
    }
}

impl eframe::App for FormsApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::left("left_panel")
            .exact_width(100.0)
            .show(ctx, |ui| {
                ui.strong("Navigation");
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
                            let now: DateTimeWithTimeZone = chrono::Local::now().with_timezone(&chrono::Local::now().offset());
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
                            let now: DateTimeWithTimeZone = chrono::Local::now().with_timezone(&chrono::Local::now().offset());
                            self.add_page(TagNew::new(TagUI {
                                id: Uuid::new_v4(),
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
        });
        if self.pending_actions>0 {
            ctx.request_repaint();
        }
    }
}
