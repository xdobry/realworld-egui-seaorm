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
use models::Uuid;

pub struct UserTable {
    users: Vec<users::Model>,
    event_bus: UIBus,
    should_close: bool,

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
                page_action = PageAction::AddPage(Box::new(UserEdit::new_create(new_user)));
            }
            if ui.button("Close").clicked() {
                self.should_close = true;
            }
        });
        let table_action = show_users_table(ui, &self.users, TableMode::EditDelete);
        match table_action {
            TableAction::SelectItem(user_id, _label) => {
                self.event_bus.send_task(tx, UICommand::User(UserCommand::Load(user_id)));
            }
            TableAction::DeleteItem(user_id) => {
                self.event_bus.send_task(tx, UICommand::User(UserCommand::Delete(user_id)));
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
                        },
                        _ => {

                        }
                    }
                }
                UIResult::Deleted(id) => {
                    self.users.retain(|u| u.id != id);
                },
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
    fn should_close(&self) -> bool {
        self.should_close
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
            should_close: false,
        }
    }
}

pub enum PageState {
    Show,
    Update,
    Updating,
    Final,
    Create,
    Creating,
}

impl PageState {
    pub fn is_enabled(&self) -> bool {
        match self {
            PageState::Update | PageState::Create => {
                true
            }
            _ => {
                false
            }
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
    should_close: bool,
}

impl Page for UserEdit {
    fn show(&mut self, ui: &mut egui::Ui, tx: &mut CommandBus) -> PageAction {
        let mut page_action = PageAction::None;
        ui.horizontal(|ui| {
            match self.page_state {
                PageState::Update => {
                    if ui.button("Update").clicked() {
                        self.user.updated_at = core::time_now();
                        self.event_bus.send_task(tx,UICommand::User(UserCommand::Update(self.user.to_change_record(&self.orig_user))));
                        self.page_state = PageState::Updating;
                    }
                    if ui.button("Cancel").clicked() {
                        self.page_state = PageState::Show;
                    }
                },
                PageState::Show => {
                    if ui.button("Start Update").clicked() {
                        self.page_state = PageState::Update;
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
                },
                PageState::Create => {
                     if ui.button("Create").clicked() {
                        self.event_bus.send_task(tx,UICommand::User(UserCommand::Create(self.user.to_model())));
                        self.page_state = PageState::Creating;
                    }
                }
            }
            if ui.button("Close").clicked() {
                self.should_close = true;
            }
        });
        ui.horizontal(|ui| {
            if ui.selectable_label(matches!(self.current_tab, UserTab::Details), "Details").clicked() {
                self.current_tab = UserTab::Details;
            }
            match self.page_state {
                PageState::Create | PageState::Creating => {
                },
                _ => {
                    if ui.selectable_label(matches!(self.current_tab, UserTab::Followers), "Followers").clicked() {
                        self.current_tab = UserTab::Followers;
                    }
                    if ui.selectable_label(matches!(self.current_tab, UserTab::Favorites), "Favorites").clicked() {
                        self.current_tab = UserTab::Favorites;
                    }
                }
            }
        });
        match self.current_tab {
            UserTab::Details => {
                ui.add_enabled_ui(self.page_state.is_enabled(), |ui| {
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
    fn should_close(&self) -> bool {
        self.should_close
    }
    fn update(&mut self, tx: &mut CommandBus,emit: &mut dyn FnMut(PageAction)) {
        if let Ok(msg) = self.event_bus.try_recv() {
            match msg {
                UIResult::Updated(_) => {
                    self.page_state = PageState::Show;
                    self.orig_user = self.user.to_model();
                },
                UIResult::Created => {
                    self.page_state = PageState::Show;
                    self.orig_user = self.user.to_model();
                },
                UIResult::DbError(msg) => {
                    match self.page_state {
                        PageState::Updating => {
                            self.page_state = PageState::Update;
                        },
                        PageState::Creating => {
                            self.page_state = PageState::Create;
                        },
                        _ => {

                        }
                    }
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
            page_state: PageState::Show,
            should_close: false,
        }
    }

    pub fn new_create(user: UserUI) -> Self {
        Self {
            user_followers_tab: UserFollowersTab::new(user.id),
            user_favorites_tab: UserFavoritesTab::new(user.id),
            orig_user: user.to_model(),
            user,
            current_tab: UserTab::Details,
            event_bus: UIBus::default(),
            page_state: PageState::Create,
            should_close: false,
        }
    }

}

