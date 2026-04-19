use core::{api::{UICommand, UIResult}, 
    article_favorites::{api::{ArticleFavoriteCommand, ArticleFavoriteResult}, 
    dto::UserFavoriteUI}, articles::api::{ArticleCommand, ArticleResult}, 
    user_follows::{api::{UserFollowerCommand, UserFollowerResult}, dto::UserFollowerName}, 
    users::{api::{UserCommand, UserResult}}};

use egui::{Id, Modal};
use models::entity::{articles, users};
use models::entity::user_follows;
use models::{DateTimeWithTimeZone, Uuid};
use models::entity::article_favorites;

use crate::ui::{article_favorites::tables::show_user_favorites_table, articles::{pages::ArticleEdit, tables::show_articles_table}, core::{page::{Form, PageAction}, tables::{TableAction, TableMode}}, user_follows::tables::show_user_followers_table, users::tables::show_users_table};
use command_bus::{CommandBus, UIBus};

#[derive(Default)]
pub struct UserFollowersTab {
    user_id: Uuid,
    user_followers: Option<Vec<UserFollowerName>>,
    users: Option<Vec<users::Model>>,
    opened: bool,
    event_bus: UIBus,
    initialized: bool,
}

impl Form for UserFollowersTab {
    fn show_ui(&mut self, ui: &mut egui::Ui, tx: &mut CommandBus) {
        if !self.initialized {
            self.event_bus.send_task(tx,UICommand::UserFollower(UserFollowerCommand::LoadByFolloweeId(self.user_id)));
            self.initialized = true;
        }
        if let Some(user_followers) = &self.user_followers {
            if ui.button("Add Follower").clicked() {
                self.opened = true;
                if self.users.is_none() {
                    self.event_bus.send_task(tx,UICommand::User(UserCommand::Reload));
                }
            }
            let table_action = show_user_followers_table(ui, user_followers, TableMode::Delete);
            match table_action {
                TableAction::DeleteItem(ids) => {
                    self.event_bus.send_task(tx,UICommand::UserFollower(UserFollowerCommand::Delete(ids)));
                }
                _ => {
                    
                }
            }
            if self.opened {
                let modal = Modal::new(Id::new("mod_add_tag")).show(ui.ctx(), |ui| {
                    ui.set_width(200.0);
                    if let Some(users) = &self.users {
                        let table_action = show_users_table(ui,users, TableMode::Select);
                        match table_action {
                            TableAction::SelectItem(uuid,_label) => {
                                let user_follower = user_follows::Model {
                                    follower_id: uuid,
                                    followee_id: self.user_id,
                                    created_at: core::time_now(),
                                };
                                self.event_bus.send_task(tx,UICommand::UserFollower(UserFollowerCommand::Create(user_follower)));
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
                    self.opened = false;
                }
            }
        }
    }

    fn update<F>(&mut self, _tx: &mut CommandBus, mut emit: F) 
    where 
        F: FnMut(PageAction),
    {
        if let Ok(msg) = self.event_bus.try_recv() {
            match msg {
                UIResult::UserFollower(UserFollowerResult::Followers(user_followers)) => {
                   self.user_followers = Some(user_followers);
                },
                UIResult::User(UserResult::Users(users)) => {
                    self.users = Some(users);
                },
                UIResult::Created => {
                    self.initialized = false;
                },
                UIResult::Deleted(follower_id) => {
                    if let Some(user_followers) = self.user_followers.as_mut() {
                        user_followers.retain(| at | at.follower_id != follower_id);
                    }
                },
                UIResult::DbError(err) => {
                    emit(PageAction::AddError(err));
                }
                _ => {
                    println!("dropped message by UserFollowersTab")
                }
            }
        }
    }
}

impl UserFollowersTab {
    pub fn new(user_id: Uuid) -> Self {
        Self {
            user_id,
            user_followers: None,
            users: None,
            opened: false,
            event_bus: UIBus::default(),
            initialized: false,
        }
    }
}

#[derive(Default)]
pub struct UserFavoritesTab {
    user_id: Uuid,
    user_favorites: Option<Vec<UserFavoriteUI>>,
    articles: Option<Vec<articles::Model>>,
    opened: bool,
    event_bus: UIBus,
    initialized: bool,
}

impl Form for UserFavoritesTab {
    fn show_ui(&mut self, ui: &mut egui::Ui, tx: &mut CommandBus) {
        if !self.initialized {
            self.event_bus.send_task(tx,UICommand::ArticleFavorite(ArticleFavoriteCommand::LoadByUserId(self.user_id)));
            self.initialized = true;
        }
        if let Some(user_favorites) = &self.user_favorites {
            if ui.button("Add Favorite").clicked() {
                self.opened = true;
                if self.articles.is_none() {
                    self.event_bus.send_task(tx,UICommand::Article(ArticleCommand::Reload));
                }
            }
            let table_action = show_user_favorites_table(ui, user_favorites, TableMode::Delete);
            match table_action {
                TableAction::DeleteItem(ids) => {
                    self.event_bus.send_task(tx,UICommand::ArticleFavorite(ArticleFavoriteCommand::Delete(ids)));
                },
                TableAction::LinkItem(id) => {
                    self.event_bus.send_task(tx, UICommand::Article(ArticleCommand::Load(id)));
                }
                _ => {
                    
                }
            }
            if self.opened {
                let modal = Modal::new(Id::new("mod_add_tag")).show(ui.ctx(), |ui| {
                    ui.set_width(200.0);
                    if let Some(articles) = &self.articles {
                        let table_action = show_articles_table(ui,articles, TableMode::Select);
                        match table_action {
                            TableAction::SelectItem(uuid,_label) => {
                                let now = core::time_now();
                                let favorite = article_favorites::Model {
                                    article_id: uuid,
                                    user_id: self.user_id,
                                    created_at: now,
                                };
                                self.event_bus.send_task(tx,UICommand::ArticleFavorite(ArticleFavoriteCommand::Create(favorite)));
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
                    self.opened = false;
                }
            }
        }
    }

    fn update<F>(&mut self, _tx: &mut CommandBus, mut emit: F) 
    where 
        F: FnMut(PageAction),
    {
        if let Ok(msg) = self.event_bus.try_recv() {
            match msg {
                UIResult::ArticleFavorite(ArticleFavoriteResult::UserFavorites(user_favorites)) => {
                   self.user_favorites = Some(user_favorites);
                },
                UIResult::Article(ArticleResult::Articles(articles)) => {
                    self.articles = Some(articles);
                },
                UIResult::Article(ArticleResult::Article(article)) => {
                    emit(PageAction::AddPage(Box::new(ArticleEdit::new(article))));
                },
                UIResult::Created => {
                    self.initialized = false;
                },
                UIResult::ArticleFavorite(ArticleFavoriteResult::Deleted((_user_id, article_id))) => {
                    if let Some(user_favorites) = self.user_favorites.as_mut() {
                        user_favorites.retain(| at | at.article_id != article_id);
                    }
                },
                UIResult::DbError(err) => {
                    emit(PageAction::AddError(err));
                }
                _ => {
                    println!("dropped message by UserFollowersTab")
                }
            }
        }
    }
}

impl UserFavoritesTab {
    pub fn new(user_id: Uuid) -> Self {
        Self {
            user_id,
            user_favorites: None,
            articles: None,
            opened: false,
            event_bus: UIBus::default(),
            initialized: false,
        }
    }
}