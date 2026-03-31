use egui::{Id, Modal};
use sea_orm::{ActiveValue, prelude::DateTimeWithTimeZone};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::{api::CommandBus, ui::{article_favorites::tables::show_article_favorites_table, 
    article_tags::tables::show_article_tags_table, 
    comments::{forms::CommentForm, tables::show_comments_author_table}, 
    core::{page::{Form, PageAction, UIBus}, tables::{TableAction, TableMode}}, 
    tags::tables::show_tags_table, users::tables::show_users_table}};

use models::entity::{article_favorites, article_tags, tags, users};
use core::comments::api::{CommentCommand, CommentResult};
use core::article_tags::dto::ArticleTagUI;
use core::comments::dto::CommentAuthor;
use core::article_favorites::api::{ArticleFavoriteCommand, ArticleFavoriteResult};
use core::article_favorites::dto::ArticleFavoriteUI;
use core::article_tags::api::{ArticleTagCommand, ArticleTagResult};
use core::tags::api::{TagCommand, TagResult};
use core::users::api::{UserCommand, UserResult};
use core::api::{UICommand, UIResult};

#[derive(Default)]
pub struct ArticleTagsTab {
    article_id: Uuid,
    article_tags: Option<Vec<ArticleTagUI>>,
    tags: Option<Vec<tags::Model>>,
    opened: bool,
    event_bus: UIBus,
    initialized: bool,
}

impl Form for ArticleTagsTab {
    fn show_ui(&mut self, ui: &mut egui::Ui, tx: &mut CommandBus) {
        if !self.initialized {
            self.event_bus.send_task(tx,UICommand::ArticleTag(ArticleTagCommand::LoadByArticleId(self.article_id)));
            self.initialized = true;
        }
        if let Some(article_tags) = &self.article_tags {
            if ui.button("Add Tag").clicked() {
                self.opened = true;
                if self.tags.is_none() {
                    self.event_bus.send_task(tx,UICommand::Tag(TagCommand::Reload));
                }
            }
            let table_action = show_article_tags_table(ui, article_tags, TableMode::Delete);
            match table_action {
                TableAction::DeleteItem(ids) => {
                    self.event_bus.send_task(tx,UICommand::ArticleTag(ArticleTagCommand::Delete(ids)));
                }
                _ => {
                    
                }
            }
            if self.opened {
                let modal = Modal::new(Id::new("mod_add_tag")).show(ui.ctx(), |ui| {
                    ui.set_width(200.0);
                    if let Some(tags) = &self.tags {
                        let table_action = show_tags_table(ui,tags, TableMode::Select);
                        match table_action {
                            TableAction::SelectItem(uuid,_label) => {
                                let now: DateTimeWithTimeZone = chrono::Local::now().with_timezone(&chrono::Local::now().offset());
                                let article_tag = article_tags::ActiveModel {
                                    tag_id: ActiveValue::Set(uuid),
                                    article_id: ActiveValue::Set(self.article_id),
                                    created_at: ActiveValue::Set(now),
                                    ..Default::default()
                                };
                                self.event_bus.send_task(tx,UICommand::ArticleTag(ArticleTagCommand::Create(article_tag)));
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
                UIResult::ArticleTag(ArticleTagResult::ArticleTags(article_tags)) => {
                   self.article_tags = Some(article_tags);
                },
                UIResult::Tag(TagResult::Tags(tags)) => {
                    self.tags = Some(tags);
                },
                UIResult::Created => {
                    self.initialized = false;
                },
                UIResult::Deleted(tag_id) => {
                    if let Some(article_tags) = self.article_tags.as_mut() {
                        article_tags.retain(| at | at.tag_id != tag_id);
                    }
                },
                UIResult::DbError(err) => {
                    emit(PageAction::AddError(err));
                }
                _ => {
                    
                }
            }
        }
    }
}

impl ArticleTagsTab {
    pub fn new(article_id: Uuid) -> Self {
        Self {
            article_id,
            article_tags: None,
            tags: None,
            opened: false,
            event_bus: UIBus::default(),
            initialized: false,
        }
    }
}

#[derive(Default)]
pub struct ArticleFavoriteTab {
    article_id: Uuid,
    article_favorites: Option<Vec<ArticleFavoriteUI>>,
    users: Option<Vec<users::Model>>,
    opened: bool,
    event_bus: UIBus,
    initialized: bool,
}

impl Form for ArticleFavoriteTab {
    fn show_ui(&mut self, ui: &mut egui::Ui, tx: &mut CommandBus) {
        if !self.initialized {
            self.event_bus.send_task(tx,UICommand::ArticleFavorite(ArticleFavoriteCommand::LoadByArticleId(self.article_id)));
            self.initialized = true;
        }
        if let Some(article_favorites) = &self.article_favorites {
            if ui.button("Add Favorite").clicked() {
                self.opened = true;
                if self.users.is_none() {
                    self.event_bus.send_task(tx,UICommand::User(UserCommand::Reload));
                }
            }
            let table_action = show_article_favorites_table(ui, article_favorites, TableMode::Delete);
            match table_action {
                TableAction::DeleteItem(ids) => {
                    self.event_bus.send_task(tx,UICommand::ArticleFavorite(ArticleFavoriteCommand::Delete(ids)));
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
                                let now: DateTimeWithTimeZone = chrono::Local::now().with_timezone(&chrono::Local::now().offset());
                                let article_tag = article_favorites::ActiveModel {
                                    user_id: ActiveValue::Set(uuid),
                                    article_id: ActiveValue::Set(self.article_id),
                                    created_at: ActiveValue::Set(now),
                                    ..Default::default()
                                };
                                self.event_bus.send_task(tx,UICommand::ArticleFavorite(ArticleFavoriteCommand::Create(article_tag)));
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

    fn update<F>(&mut self, _tx: &mut CommandBus, mut _emit: F) 
    where 
        F: FnMut(PageAction),
    {
        if let Ok(msg) = self.event_bus.try_recv() {
            match msg {
                UIResult::ArticleFavorite(ArticleFavoriteResult::ArticleFavorites(article_favorites)) => {
                   self.article_favorites = Some(article_favorites);
                },
                UIResult::User(UserResult::Users(users)) => {
                    self.users = Some(users);
                },
                UIResult::Created => {
                    self.initialized = false;
                },
                UIResult::Deleted(tag_id) => {
                    if let Some(article_tags) = self.article_favorites.as_mut() {
                        article_tags.retain(| at | at.user_id != tag_id);
                    }
                },
                _ => {
                    
                }
            }
        }
    }
}

impl ArticleFavoriteTab {
    pub fn new(article_id: Uuid) -> Self {
        Self {
            article_id,
            article_favorites: None,
            users: None,
            opened: false,
            event_bus: UIBus::default(),
            initialized: false,
        }
    }
}

#[derive(Default)]
pub struct ArticleCommentsTab {
    article_id: Uuid,
    article_comments: Option<Vec<CommentAuthor>>,
    comment_form: Option<CommentForm>,
    event_bus: UIBus,
    initialized: bool,
    comment_edit: bool,
}

impl Form for ArticleCommentsTab {
    fn show_ui(&mut self, ui: &mut egui::Ui, tx: &mut CommandBus) {
        if !self.initialized {
            self.event_bus.send_task(tx,UICommand::Comment(CommentCommand::LoadByArticleId(self.article_id)));
            self.initialized = true;
        }
        if let Some(article_comments) = &self.article_comments {
            if ui.button("Add Comment").clicked() {
                let now: DateTimeWithTimeZone = chrono::Local::now().with_timezone(&chrono::Local::now().offset());
                let comment_author = CommentAuthor {
                    id: Uuid::new_v4(),
                    body: "".to_string(),
                    article_id: self.article_id,
                    author_id: Uuid::nil(),
                    author_name: "".to_string(),
                    created_at: now,
                    updated_at: now,
                };
                self.comment_edit = false;
                self.comment_form = Some(CommentForm::new(comment_author))
            }
            let table_action = show_comments_author_table(ui, article_comments, TableMode::EditDelete);
            match table_action {
                TableAction::DeleteItem(id) => {
                    self.event_bus.send_task(tx,UICommand::Comment(CommentCommand::Delete(id)));
                }
                TableAction::SelectItem(id,_label) => {
                    let comment_to_edit = article_comments.iter().find(|c| c.id == id);
                    if let Some(comment_to_edit) = comment_to_edit {
                        self.comment_form = Some(CommentForm::new(comment_to_edit.clone()));
                        self.comment_edit = true;
                    }
                }
                _ => {
                    
                }
            }
            if let Some(comment_form) = self.comment_form.as_mut() {
                let modal = Modal::new(Id::new("mod_add_tag")).show(ui.ctx(), |ui| {
                    ui.set_width(350.0);
                    comment_form.show_ui(ui, tx);
                    egui::Sides::new().show(
                        ui,
                        |_ui| {},
                        |ui| {
                            if ui.button("Cancel").clicked() {
                                ui.close();
                            }
                            if self.comment_edit {
                                if ui.button("Edit").clicked() {
                                    let comment_model = comment_form.comment.to_active_model();
                                    self.event_bus.send_task(tx,UICommand::Comment(CommentCommand::Update(comment_model)));
                                    ui.close();
                                }
                            } else {
                                if ui.button("Create").clicked() {
                                    let comment_model = comment_form.comment.to_active_model();
                                    self.event_bus.send_task(tx,UICommand::Comment(CommentCommand::Create(comment_model)));
                                    ui.close();
                                }
                            }
                        },
                    );
                });
                if modal.should_close() {
                    self.comment_form = None;
                }
            }
        }
    }

    fn update<F>(&mut self, tx: &mut CommandBus, mut emit: F) 
    where 
        F: FnMut(PageAction),
    {
        if let Ok(msg) = self.event_bus.try_recv() {
            match msg {
                UIResult::Comment(CommentResult::CommentsAuthor(article_comments)) => {
                    self.article_comments = Some(article_comments);
                },
                UIResult::Updated(_) | UIResult::Created => {
                    self.article_comments = None;
                    self.initialized = false;
                },
                UIResult::Deleted(id) => {
                    if let Some(article_comments) = self.article_comments.as_mut() {
                        article_comments.retain(| at | at.id != id);
                    }
                },
                UIResult::DbError(err) => {
                    emit(PageAction::AddError(err));
                }
                _ => {
                    println!("drop result->article comments");
                }
            }
        }
        if let Some(comment_form) = self.comment_form.as_mut() {
            comment_form.update(tx, emit);
        }
    }
}

impl ArticleCommentsTab {
    pub fn new(article_id: Uuid) -> Self {
        Self {
            article_id,
            article_comments: None,
            comment_form: None,
            event_bus: UIBus::default(),
            initialized: false,
            comment_edit: false,
        }
    }
}