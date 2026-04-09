use std::any::Any;

use core::api::{UICommand, UIResult};
use core::users::api::{UserCommand, UserResult};
use core::users::dto::UserUI;
use command_bus::{CommandBus, UIBus};
use crate::ui::users::forms::ui_user;
use crate::ui::users::tables::show_users_table;
use crate::ui::core::page::{Page, PageAction};
use crate::ui::core::tables::{TableAction, TableMode};
use crate::ui::users::tabs::{UserFavoritesTab, UserFollowersTab};
use crate::ui::core::page::Form;

use models::entity::users;
use models::{DateTimeWithTimeZone, Uuid};

pub struct UserTable {
    users: Vec<users::Model>,
    event_bus: UIBus,
}

impl Page for UserTable {
    fn show(&mut self, ui: &mut egui::Ui, tx: &mut CommandBus) -> PageAction {
        let mut page_action = PageAction::None;
        ui.horizontal(|ui| {
            if ui.button("Reload").clicked() {
                self.event_bus.send_task(tx, UICommand::User(UserCommand::Reload));
            }
            if ui.button("Create User").clicked() {
                let now = core::time_now();
                let new_user = UserUI {
                    id: Uuid::new_v4(),
                    created_at: now,
                    updated_at: now,
                    ..Default::default()
                };               
                page_action = PageAction::AddPage(Box::new(UserNew::new(new_user)));
            }
            if ui.button("Close").clicked() {
                page_action = PageAction::Close;
            }
        });
        let table_action = show_users_table(ui, &self.users, TableMode::EditDelete);
        match table_action {
            TableAction::SelectItem(user_id, _label) => {
                self.event_bus.send_task(tx, UICommand::User(UserCommand::Load(user_id)));
            }
            _ => {

            }
        }
        page_action
    }
    fn update(&mut self, _tx: &mut CommandBus,emit: &mut dyn FnMut(PageAction)) {
        if let Ok(msg) = self.event_bus.try_recv() {
            match msg {
                UIResult::User(user_result) => {
                    match user_result {
                        UserResult::User(user) => {
                            emit(PageAction::AddPage(Box::new(UserEdit::new(user))));
                        }
                        UserResult::Users(users) => {
                            self.users = users;
                        }
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
        "Users"
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn init(&mut self, tx: &mut CommandBus) {
        self.event_bus.send_task(tx, UICommand::User(UserCommand::Reload));
    }
}

impl UserTable {
    pub fn new() -> Self {
        Self {
            users: Vec::new(),
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

pub struct UserNew {
    user: UserUI,
    page_state: PageState,
    event_bus: UIBus,
}

impl Page for UserNew {
    fn show(&mut self, ui: &mut egui::Ui, tx: &mut CommandBus) -> PageAction {
        let mut page_action = PageAction::None;
        ui.horizontal(|ui| {
            match self.page_state {
                PageState::Initial => {
                    if ui.button("Create").clicked() {
                        self.event_bus.send_task(tx,UICommand::User(UserCommand::Create(self.user.to_model())));
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
            ui_user(ui, &mut self.user);
        });
        page_action

    }
    fn title(&self) -> &str {
        "New User"
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

impl UserNew {
    pub fn new(user: UserUI) -> Self {       
        Self {
            event_bus: UIBus::default(),
            user,
            page_state: PageState::Initial,
        }
    }
}

pub enum UserTab {
    Details,
    Favorites,
    Followers,
}

pub struct UserEdit {
    user: UserUI,
    orig_user: users::Model,
    user_followers_tab: UserFollowersTab,
    user_favorites_tab: UserFavoritesTab,
    page_state: PageState,
    current_tab: UserTab,
    event_bus: UIBus,
}

impl Page for UserEdit {
    fn show(&mut self, ui: &mut egui::Ui, tx: &mut CommandBus) -> PageAction {
        let mut page_action = PageAction::None;
        ui.horizontal(|ui| {
            match self.page_state {
                PageState::Initial => {
                    if ui.button("Update").clicked() {
                        self.event_bus.send_task(tx,UICommand::User(UserCommand::Update(self.user.to_change_record(&self.orig_user))));
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
        ui.horizontal(|ui| {
            if ui.selectable_label(matches!(self.current_tab, UserTab::Details), "Details").clicked() {
                self.current_tab = UserTab::Details;
            }
            if ui.selectable_label(matches!(self.current_tab, UserTab::Followers), "Followers").clicked() {
                self.current_tab = UserTab::Followers;
            }
            if ui.selectable_label(matches!(self.current_tab, UserTab::Favorites), "Favorites").clicked() {
                self.current_tab = UserTab::Favorites;
            }
        });
        match self.current_tab {
            UserTab::Details => {
                ui.add_enabled_ui(self.page_state.is_initial(), |ui| {
                    ui_user(ui, &mut self.user);
                });
            },
            UserTab::Followers => {
                self.user_followers_tab.show_ui(ui, tx);
            },
            UserTab::Favorites => {
                self.user_favorites_tab.show_ui(ui, tx);
            }
        }
        page_action
    }
    fn title(&self) -> &str {
        "Edit User"
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn update(&mut self, tx: &mut CommandBus,emit: &mut dyn FnMut(PageAction)) {
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
        self.user_followers_tab.update(tx, &mut *emit);
        self.user_favorites_tab.update(tx, &mut *emit);
    }
}

impl UserEdit {
    pub fn new(orig_user: users::Model) -> Self {
        Self {
            user: UserUI::from_model(&orig_user),
            user_followers_tab: UserFollowersTab::new(orig_user.id),
            user_favorites_tab: UserFavoritesTab::new(orig_user.id),
            orig_user,
            current_tab: UserTab::Details,
            event_bus: UIBus::default(),
            page_state: PageState::Initial,
        }
    }

}

