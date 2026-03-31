use std::any::Any;

use tokio::sync::mpsc;

use models::entity::tags;
use core::api::{UICommand, UIResult};
use core::tags::api::{TagCommand, TagResult};
use core::tags::dto::TagUI;
use crate::ui::tags::forms::ui_tag;
use crate::ui::tags::tables::show_tags_table;
use crate::ui::core::page::{Page, PageAction, UIBus};
use crate::ui::core::tables::{TableAction, TableMode};
use crate::api::{CommandBus, UITask};

pub struct TagTable {
    tags: Vec<tags::Model>,
    event_bus: UIBus,
}

impl Page for TagTable {
    fn show(&mut self, ui: &mut egui::Ui, tx: &mut CommandBus) -> PageAction {
        let mut page_action = PageAction::None;
        ui.horizontal(|ui| {
            if ui.button("Reload").clicked() {
                self.event_bus.send_task(tx, UICommand::Tag(TagCommand::Reload));
            }
            if ui.button("Close").clicked() {
                page_action = PageAction::Close;
            }
        });
        let table_action = show_tags_table(ui, &self.tags, TableMode::EditDelete);
        match table_action {
            TableAction::SelectItem(tag_id, _label) => {
                self.event_bus.send_task(tx, UICommand::Tag(TagCommand::Load(tag_id)));
            }
            _ => {

            }
        }
        page_action
    }
    fn update(&mut self, _tx: &mut CommandBus,emit: &mut dyn FnMut(PageAction)) {
        if let Ok(msg) = self.event_bus.try_recv() {
            match msg {
                UIResult::Tag(tag_result) => {
                    match tag_result {
                        TagResult::Tag(tag) => {
                            emit(PageAction::AddPage(Box::new(TagEdit::new(tag))));
                        },
                        TagResult::Tags(tags) => {
                            self.tags = tags;
                        },
                    }
                }
                UIResult::DbError(msg) => {
                    emit(PageAction::AddError(msg));
                },
                _ => {

                }
            }
        }
    }
    fn title(&self) -> &str {
        "Tags"
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn init(&mut self, tx: &mut CommandBus) {
        self.event_bus.send_task(tx, UICommand::Tag(TagCommand::Reload));
    }
}

impl TagTable {
    pub fn new() -> Self {
        Self {
            tags: Vec::new(),
            event_bus: UIBus::default(),
        }
    }
}

pub enum PageState {
    Initial,
    Running,
    Final,
}

impl PageState {
    pub fn is_initial(&self) -> bool {
        match self {
            PageState::Initial => {
                true
            }
            _ => {
                false
            }
        }
    }
}

pub struct TagNew {
    tag: TagUI,
    page_state: PageState,
    event_bus: UIBus,
}

impl Page for TagNew {
    fn show(&mut self, ui: &mut egui::Ui, tx: &mut CommandBus) -> PageAction {
        let mut page_action = PageAction::None;
        ui.horizontal(|ui| {
            match self.page_state {
                PageState::Initial => {
                    if ui.button("Create").clicked() {
                        self.event_bus.send_task(tx, UICommand::Tag(TagCommand::Create(self.tag.to_active_model())));
                        self.page_state = PageState::Running;
                    }
                },
                PageState::Running => {
                    ui.label("Creating");
                }
                PageState::Final => {
                    ui.label("Created");
                }
            }
            if ui.button("Close").clicked() {
                page_action = PageAction::Close;
            }
        });
        ui.add_enabled_ui(self.page_state.is_initial(), |ui| {
            ui_tag(ui, &mut self.tag);
        });
        page_action

    }
    fn title(&self) -> &str {
        "New Tag"
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn update(&mut self, _tx: &mut CommandBus,emit: &mut dyn FnMut(PageAction)) {
        if let Ok(msg) = self.event_bus.try_recv() {
            match msg {
                UIResult::Created => {
                    self.page_state = PageState::Final;
                },
                UIResult::DbError(msg) => {
                    emit(PageAction::AddError(msg));
                },
                _ => {

                }
            }
        }
    }
}

impl TagNew {
    pub fn new(tag: TagUI) -> Self {       
        Self {
            tag,
            page_state: PageState::Initial,
            event_bus: UIBus::default(),
        }
    }
}

pub struct TagEdit {
    tag: TagUI,
    orig_tag: tags::Model,
    page_state: PageState,
    event_bus: UIBus,
}

impl Page for TagEdit {
    fn show(&mut self, ui: &mut egui::Ui, tx: &mut CommandBus) -> PageAction {
        let mut page_action = PageAction::None;
        ui.horizontal(|ui| {
            match self.page_state {
                PageState::Initial => {
                    if ui.button("Update").clicked() {
                        let _ = self.event_bus.send_task(tx, UICommand::Tag(TagCommand::Update(self.tag.to_active_model_update(&self.orig_tag))));
                        self.page_state = PageState::Running;
                    }
                },
                PageState::Running => {
                    ui.label("Updating");
                }
                PageState::Final => {
                    ui.label("Updated");
                }
            }
            if ui.button("Close").clicked() {
                page_action = PageAction::Close;
            }
        });
        ui.add_enabled_ui(self.page_state.is_initial(), |ui| {
            ui_tag(ui, &mut self.tag);
        });
        page_action
    }
    fn title(&self) -> &str {
        "Edit Tag"
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn update(&mut self, _tx: &mut CommandBus,emit: &mut dyn FnMut(PageAction)) {
        if let Ok(msg) = self.event_bus.try_recv() {
            match msg {
                UIResult::Updated(_) => {
                    self.page_state = PageState::Final;
                },
                UIResult::DbError(msg) => {
                    emit(PageAction::AddError(msg));
                },
                _ => {

                }
            }
        }
    }
}

impl TagEdit {
    pub fn new(orig_tag: tags::Model) -> Self {
        Self {
            tag: TagUI::from_model(&orig_tag),
            orig_tag,
            event_bus: UIBus::default(),
            page_state: PageState::Initial,
        }
    }

}

