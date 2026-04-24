use core::entities::EntityIdent;
use std::any::Any;

use models::Uuid;
use models::entity::tags;
use core::api::{UICommand, UIResult};
use core::tags::api::{TagCommand, TagResult};
use core::tags::dto::TagUI;
use crate::ui::tags::forms::ui_tag;
use crate::ui::tags::tables::show_tags_table;
use crate::ui::core::page::{Page, PageAction, PageState};
use crate::ui::core::tables::{TableAction, TableMode};
use command_bus::{CommandBus, UIBus};

pub struct TagTable {
    tags: Vec<tags::Model>,
    event_bus: UIBus,
    should_close: bool,
}

impl Page for TagTable {
    fn show(&mut self, ui: &mut egui::Ui, tx: &mut CommandBus) -> PageAction {
        let mut page_action = PageAction::None;
        ui.horizontal(|ui| {
            if ui.button("Reload").clicked() {
                self.event_bus.send_task(tx, UICommand::Tag(TagCommand::Reload));
            }
            if ui.button("Create Tag").clicked() {
                page_action = PageAction::AddPage(Box::new(TagEdit::new_create(TagUI::new())));
            }
            if ui.button("Close").clicked() {
                self.should_close = true;
            }
        });
        let table_action = show_tags_table(ui, &self.tags, TableMode::EditDelete);
        match table_action {
            TableAction::SelectItem(tag_id, _label) => {
                page_action = PageAction::Navigate(EntityIdent::Tag(tag_id));
            }
            _ => {

            }
        }
        page_action
    }
    fn update(&mut self, _tx: &mut CommandBus,emit: &mut dyn FnMut(PageAction)) {
        if let Ok(msg) = self.event_bus.try_recv() {
            match msg {
                UIResult::Tag(TagResult::Tags(tags)) => {
                    self.tags = tags;
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
    fn should_close(&self) -> bool {
        self.should_close
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
            should_close: false,
        }
    }
}

pub struct TagEdit {
    ident: EntityIdent,
    tag: Option<TagUI>,
    orig_tag: Option<tags::Model>,
    page_state: PageState,
    event_bus: UIBus,
    should_close: bool,
}

impl Page for TagEdit {
    fn init(&mut self, tx: &mut CommandBus) {
        if self.tag.is_none() {
            if let EntityIdent::Tag(tag_id) = self.ident {
                self.event_bus.send_task(tx,UICommand::Tag(TagCommand::Load(tag_id)));
                self.page_state = PageState::Loading;
            }
        }
    }    
    fn show(&mut self, ui: &mut egui::Ui, tx: &mut CommandBus) -> PageAction {
        let mut page_action = PageAction::None;
        if self.tag.is_none() {
            ui.label("Loading...");
            return page_action;
        }
        if let Some(tag) = self.tag.as_mut() {
            ui.horizontal(|ui| {
                match self.page_state {
                    PageState::Update => {
                        if ui.button("Update").clicked() {
                            if let Some(orig_tag) = &self.orig_tag {
                                let _ = self.event_bus.send_task(tx, UICommand::Tag(TagCommand::Update(tag.to_change_record(orig_tag))));
                                self.page_state = PageState::Updating;
                            }
                        }
                    },
                    PageState::Loading => {
                        ui.label("Loading...");
                    }
                    PageState::Show => {
                        if ui.button("Start Updating").clicked() {
                            self.page_state = PageState::Update;
                        }
                    }
                    PageState::Create => {
                        if ui.button("Create").clicked() {
                            let _ = self.event_bus.send_task(tx, UICommand::Tag(TagCommand::Create(tag.to_model())));
                            self.page_state = PageState::Updating;
                        }
                    },
                    PageState::Updating => {
                        ui.label("Updating");
                    }
                    PageState::Creating => {
                        ui.label("Creating");
                    }
                    PageState::Final => {
                        ui.label("Updated");
                    }
                }
                if ui.button("Close").clicked() {
                    self.should_close = true;
                }
            });
            ui.add_enabled_ui(self.page_state.is_enabled(), |ui| {
                ui_tag(ui, tag);
            });
        }
        page_action
    }
    fn title(&self) -> &str {
        "Edit Tag"
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn should_close(&self) -> bool {
        self.should_close
    }
    fn update(&mut self, _tx: &mut CommandBus,emit: &mut dyn FnMut(PageAction)) {
        if let Ok(msg) = self.event_bus.try_recv() {
            match msg {
                UIResult::Updated(_) => {
                    if let Some(tag) = &self.tag {
                        self.orig_tag = Some(tag.to_model());
                        self.page_state = PageState::Show;
                    }
                },
                UIResult::Created => {
                    if let Some(tag) = &self.tag {
                        self.orig_tag = Some(tag.to_model());
                        self.page_state = PageState::Show;
                    }
                },
                UIResult::DbError(msg) => {
                    emit(PageAction::AddError(msg));
                },
                UIResult::Tag(TagResult::Tag(tag)) => {
                    self.tag = Some(TagUI::from_model(&tag));
                    self.orig_tag = Some(tag);
                    self.page_state = PageState::Show;
                },
                _ => {

                }
            }
        }
    }
}

impl TagEdit {
    pub fn new(tag_id: Uuid) -> Self {
        Self {
            ident: EntityIdent::Tag(tag_id),
            tag: None,
            orig_tag: None,
            event_bus: UIBus::default(),
            page_state: PageState::Show,
            should_close: false,
        }
    }
    pub fn new_create(tag: TagUI) -> Self {
        Self {
            ident: EntityIdent::Tag(tag.id),
            orig_tag: Some(tag.to_model()),
            tag: Some(tag),
            event_bus: UIBus::default(),
            page_state: PageState::Create,
            should_close: false,
        }
    }

}

