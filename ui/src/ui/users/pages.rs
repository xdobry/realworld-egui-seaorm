use core::entities::EntityIdent;
use std::any::Any;

use core::api::{UICommand, UIResult};
use core::users::api::{UserCommand, UserResult};
use core::users::dto::UserUI;
use command_bus::{CommandBus, UIBus};
use egui::Key::N;
use egui::{Context, TextureHandle};
use egui_commonmark::CommonMarkCache;
use crate::ui::notebook::NoteBook;
use crate::ui::users::forms::ui_user;
use crate::ui::users::tables::show_users_table;
use crate::ui::core::page::{Page, PageAction, PageState, UIContext};
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
    fn show(&mut self, ui: &mut egui::Ui, tx: &mut CommandBus, ui_context: &UIContext, _cache: &mut CommonMarkCache) -> PageAction {
        let mut page_action = PageAction::None;
        ui.horizontal(|ui| {
            if ui.button("Reload").clicked() {
                self.event_bus.send_task(tx, UICommand::User(UserCommand::Reload));
            }
            if ui_context.is_admin() {
                if ui.button("Create User").clicked() {
                    page_action = PageAction::AddPage(Box::new(UserEdit::new_create()));
                }
            }
        });
        let table_action = show_users_table(ui, &self.users, if ui_context.is_admin() {TableMode::EditDelete} else {TableMode::Select});
        match table_action {
            TableAction::SelectItem(user_id, _label) => {
                page_action = PageAction::Navigate(EntityIdent::User(user_id));
            }
            TableAction::DeleteItem(user_id) => {
                self.event_bus.send_task(tx, UICommand::User(UserCommand::Delete(user_id)));
            }
            _ => {

            }
        }
        page_action
    }
    fn update(&mut self, _tx: &mut CommandBus, _ui_context: &UIContext, emit: &mut dyn FnMut(PageAction)) {
        if let Ok(msg) = self.event_bus.try_recv() {
            match msg {
                UIResult::User(user_result) => {
                    match user_result {
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
    fn title(&self, _ui_context: &UIContext) -> &str {
        "Users"
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn should_close(&self) -> bool {
        self.should_close
    }
    fn init(&mut self, tx: &mut CommandBus, _ui_context: &UIContext) {
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

enum UserTab {
    Details,
    FollowedUsers,
    Followers,
    Favorites,
}

impl TryFrom<usize> for UserTab {
    type Error = ();
    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Details),
            1 => Ok(Self::FollowedUsers),
            2 => Ok(Self::Followers),
            3 => Ok(Self::Favorites),
            _ => Err(()),
        }
    }
}

static USER_TABS: [&str; 4]  = ["Details", "Followed", "Followers", "Favorites"];

pub struct UserEdit {
    ident: EntityIdent,
    user: Option<UserUI>,
    title: Option<String>,
    orig_user: Option<users::Model>,
    user_followers_tab: UserFollowersTab,
    followed_users_tab: UserFollowersTab,
    user_favorites_tab: UserFavoritesTab,
    page_state: PageState,
    current_tab: usize,
    event_bus: UIBus,
    should_close: bool,
    user_image: Option<TextureHandle>,
}

impl Page for UserEdit {
    fn init(&mut self, tx: &mut CommandBus, _ui_context: &UIContext) {
        if self.user.is_none() {
            if let EntityIdent::User(user_id) = self.ident {
                self.event_bus.send_task(tx,UICommand::User(UserCommand::Load(user_id)));
                self.page_state = PageState::Loading;
            }
        }
    }
    fn show(&mut self, ui: &mut egui::Ui, tx: &mut CommandBus, ui_context: &UIContext, cache: &mut CommonMarkCache) -> PageAction {
        let mut page_action = PageAction::None;
        ui.horizontal(|ui| {
            match self.page_state {
                PageState::Update => {
                    if ui.button("Update").clicked() {
                        if let Some(user) = self.user.as_mut() {
                            user.updated_at = core::time_now();
                            if let Some(orig_user) = &self.orig_user {
                                self.event_bus.send_task(tx,UICommand::User(UserCommand::Update(user.to_change_record(orig_user))));
                                self.page_state = PageState::Updating;
                            }
                        }
                    }
                    if ui.button("Cancel").clicked() {
                        self.page_state = PageState::Show;
                    }
                },
                PageState::Show => {
                    if let EntityIdent::User(edited_user_id) = self.ident {
                        if ui_context.is_user_or_admin(edited_user_id) {
                            if ui.button("Start Update").clicked() {
                                self.page_state = PageState::Update;
                            }
                        }
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
                        if let Some(user) = &self.user {
                            self.event_bus.send_task(tx,UICommand::User(UserCommand::Create(user.to_model())));
                            self.page_state = PageState::Creating;
                        }
                    }
                },
                PageState::Loading => {
                    ui.label("Loading...");
                }
            }
            if ui.button("Show Articles").clicked() {
                if let Some(user) = &self.user {
                    page_action = PageAction::Navigate(EntityIdent::ArticleListAuthor(user.id));
                }
            }
        });
        if let Some(user) = self.user.as_mut() {
            // perhaps the image should be loaded and converter to texture handler in separate thread if platform allows it
            if user.image.is_some() && self.user_image.is_none() {
                if let Some(bytes) = &user.image {
                    self.user_image = load_image(ui.ctx(), bytes);
                    if self.user_image.is_none() {
                        // If data corrupt do not try to create texture in each loop
                        user.image = None;
                    }
                }
            }
            ui.horizontal(|ui| {
                match self.page_state {
                    PageState::Create | PageState::Creating => {
                    },
                    _ => {
                        NoteBook::new(4, &mut self.current_tab, |i| USER_TABS[i].into(),200.0, false).show(ui);
                    }
                }
            });
            if let Ok(current_tab) = UserTab::try_from(self.current_tab) {
                match current_tab {
                    UserTab::Details => {
                        let ui_context = if self.page_state.is_enabled() { &ui_context.as_edit() } else { ui_context};
                        let mut new_image = false;
                        ui_user(ui, user, &ui_context, cache, &self.user_image, &mut new_image);
                        if new_image {
                            self.user_image = None;
                        }
                    },
                    UserTab::Followers => {
                        self.user_followers_tab.show_ui(ui, tx, ui_context, &mut page_action);
                    },
                    UserTab::FollowedUsers => {
                        self.followed_users_tab.show_ui(ui, tx, ui_context, &mut page_action);
                    },
                    UserTab::Favorites => {
                        self.user_favorites_tab.show_ui(ui, tx, ui_context, &mut page_action);
                    }
                }
            }
        }
        page_action
    }
    fn title(&self, _ui_context: &UIContext) -> &str {
        if let Some(title) = &self.title {
            title.as_str()
        } else {
            "User"
        }
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn should_close(&self) -> bool {
        self.should_close
    }

    fn update(&mut self, tx: &mut CommandBus, _ui_context: &UIContext, emit: &mut dyn FnMut(PageAction)) {
        if let Ok(msg) = self.event_bus.try_recv() {
            match msg {
                UIResult::Updated(_) => {
                    self.page_state = PageState::Show;
                    if let Some(user) = &self.user {
                        self.orig_user = Some(user.to_model());
                        self.title = Some(format!("\u{1F464} {}",user.username));
                    }
                },
                UIResult::Created => {
                    self.page_state = PageState::Show;
                    if let Some(user) = &self.user {
                        self.orig_user = Some(user.to_model());
                        self.title = Some(format!("\u{1F464} {}",user.username));
                    }
                },
                UIResult::User(UserResult::User(user)) => {
                    self.user = Some(UserUI::from_model(&user));
                    self.title = Some(format!("\u{1F464} {}",user.username));
                    self.orig_user = Some(user);
                    self.page_state = PageState::Show;
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
        self.followed_users_tab.update(tx, &mut *emit);
        self.user_favorites_tab.update(tx, &mut *emit);
    }
    fn entity_ident(&self) -> EntityIdent {
        self.ident.clone()
    }
}

impl UserEdit {
    pub fn new(user_id: Uuid) -> Self {
        Self {
            ident: EntityIdent::User(user_id),
            user: None,
            user_followers_tab: UserFollowersTab::new(user_id, true),
            followed_users_tab: UserFollowersTab::new(user_id, false),
            user_favorites_tab: UserFavoritesTab::new(user_id),
            orig_user: None,
            current_tab: 0,
            event_bus: UIBus::default(),
            page_state: PageState::Loading,
            should_close: false,
            title: None,
            user_image: None,
        }
    }

    pub fn new_create() -> Self {
        let user = UserUI::new_create();
        Self {
            ident: EntityIdent::User(user.id),
            user_followers_tab: UserFollowersTab::new(user.id, true),
            followed_users_tab: UserFollowersTab::new(user.id, false),
            user_favorites_tab: UserFavoritesTab::new(user.id),
            orig_user: Some(user.to_model()),
            user: Some(user),
            current_tab: 0,
            event_bus: UIBus::default(),
            page_state: PageState::Create,
            should_close: false,
            title: Some("Create User".into()),
            user_image: None,
        }
    }

}

pub fn load_image(ctx: &Context, bytes: &Vec<u8>) -> Option<TextureHandle> {
    let image = egui_extras::image::load_image_bytes(bytes);
    if let Ok(image) = image {
        Some(ctx.load_texture("user", image, egui::TextureOptions::default()))
    } else {
        None
    }
}

