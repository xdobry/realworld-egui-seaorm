use core::entities::EntityIdent;
use std::any::Any;

use models::Uuid;
use models::entity::{articles};
use core::api::{UICommand, UIResult};
use core::articles::api::{ArticleCommand, ArticleResult};
use core::articles::dto::{ArticleUI};
use command_bus::{CommandBus, UIBus};
use crate::ui::articles::forms::ArticleForm;
use crate::ui::articles::tables::{show_articles_table};
use crate::ui::articles::tabs::{ArticleCommentsTab, ArticleFavoriteTab, ArticleTagsTab};
use crate::ui::core::page::{Form, Page, PageAction, PageState};
use crate::ui::core::tables::{TableAction, TableMode};

pub struct ArticleTable {
    articles: Vec<articles::Model>,
    event_bus: UIBus,
    should_close: bool,
}

impl Page for ArticleTable {
    fn show(&mut self, ui: &mut egui::Ui, tx: &mut CommandBus) -> PageAction {
        let mut page_action = PageAction::None;
        ui.horizontal(|ui| {
            if ui.button("Reload").clicked() {
                self.event_bus.send_task(tx, UICommand::Article(ArticleCommand::Reload));
            }
            if ui.button("New").clicked() {
                page_action = PageAction::AddPage(Box::new(ArticleEdit::new_create(ArticleUI::new())));
            }
            if ui.button("Close").clicked() {
                self.should_close = true;
            }
        });
        let table_action = show_articles_table(ui, &self.articles, TableMode::EditDelete);
        match table_action {
            TableAction::SelectItem(article_id, _label) => {
                page_action = PageAction::Navigate(EntityIdent::Article(article_id));
            }
            TableAction::DeleteItem(article_id) => {
                self.event_bus.send_task(tx, UICommand::Article(ArticleCommand::Delete(article_id)));
            }
            _ => {

            }
        }
        page_action
    }
    fn init(&mut self, tx: &mut CommandBus) {
        self.event_bus.send_task(tx, UICommand::Article(ArticleCommand::Reload));
    }
    fn update(&mut self, _tx: &mut CommandBus,emit: &mut dyn FnMut(PageAction)) {
        if let Ok(msg) = self.event_bus.try_recv() {
            match msg {
                UIResult::Article(article_result) => {
                    match article_result {
                        ArticleResult::Articles(articles) => {
                            self.articles = articles;
                        },
                        _ => {

                        }
                    }
                }
                UIResult::Deleted(id) => {
                    self.articles.retain(|a| a.id != id);
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
        "Articles"
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn should_close(&self) -> bool {
        self.should_close
    }
}

impl ArticleTable {
    pub fn new() -> Self {
        Self {
            articles: Vec::new(),
            event_bus: UIBus::default(),
            should_close: false,
        }
    }
}
pub enum ArticleTab {
    Details,
    Tags,
    Favorites,
    Comments,
}

pub struct ArticleEdit {
    ident: EntityIdent,
    article_form: Option<ArticleForm>,
    orig_article: Option<ArticleUI>,
    current_tab: ArticleTab,
    page_state: PageState,
    article_tags_tab: ArticleTagsTab,
    article_favorites_tab: ArticleFavoriteTab,
    article_comments_tab: ArticleCommentsTab,
    event_bus: UIBus,
    should_close: bool,
}

impl Page for ArticleEdit {
    fn init(&mut self, tx: &mut CommandBus) {
        if self.article_form.is_none() {
            if let EntityIdent::Article(article_id) = self.ident {
                self.event_bus.send_task(tx,UICommand::Article(ArticleCommand::Load(article_id)));
                self.page_state = PageState::Loading;
            }
        }
    }
    fn show(&mut self, ui: &mut egui::Ui, tx: &mut CommandBus) -> PageAction {
        let mut page_action = PageAction::None;
        if self.article_form.is_none() {
            ui.label("Loading...");
            return page_action;
        }
        if let Some(article_form) = self.article_form.as_mut() {
            ui.horizontal(|ui| {
                match self.page_state {
                    PageState::Update => {
                        if ui.button("Update").clicked() {
                            if let Some(orig_article) = &self.orig_article {
                                self.event_bus.send_task(tx,UICommand::Article(ArticleCommand::Update(article_form.article.to_change_record(orig_article))));
                                self.page_state = PageState::Updating;
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
                    }
                    PageState::Loading => {
                        ui.label("Loading...");
                    }
                    PageState::Create => {
                        if ui.button("Create").clicked() {
                        self.event_bus.send_task(tx,UICommand::Article(ArticleCommand::Create(article_form.article.to_model())));
                        self.page_state = PageState::Show;

                        }
                    }
                    PageState::Show => {
                        if ui.button("Start Update").clicked() {
                            self.page_state = PageState::Update;
                        }
                    }
                }
                if ui.button("Close").clicked() {
                    self.should_close = true;
                }
            });
            ui.horizontal(|ui| {
                if ui.selectable_label(matches!(self.current_tab, ArticleTab::Details), "Details").clicked() {
                    self.current_tab = ArticleTab::Details;
                }
                if !matches!(self.page_state,PageState::Create) {
                    if ui.selectable_label(matches!(self.current_tab, ArticleTab::Tags), "Tags").clicked() {
                        self.current_tab = ArticleTab::Tags;
                    }
                    if ui.selectable_label(matches!(self.current_tab, ArticleTab::Favorites), "Favorites").clicked() {
                        self.current_tab = ArticleTab::Favorites;
                    }
                    if ui.selectable_label(matches!(self.current_tab, ArticleTab::Comments), "Comments").clicked() {
                        self.current_tab = ArticleTab::Comments;
                    }
                }
            });
            match self.current_tab {
                ArticleTab::Details => {
                    ui.add_enabled_ui(self.page_state.is_enabled(), |ui| {
                        article_form.show_ui(ui, tx, &mut page_action);
                    });
                }
                ArticleTab::Tags => {
                    self.article_tags_tab.show_ui(ui, tx, &mut page_action);
                }
                ArticleTab::Favorites => {
                    self.article_favorites_tab.show_ui(ui, tx, &mut page_action);
                }
                ArticleTab::Comments => {
                    self.article_comments_tab.show_ui(ui, tx, &mut page_action);
                }
            }
        }
        page_action
    }
    fn title(&self) -> &str {
        "Edit Article"
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
                    if let Some(article_form) = &self.article_form {
                        self.orig_article = Some(article_form.article.clone());
                        self.page_state = PageState::Show;
                    }
                },
                UIResult::Created => {
                    if let Some(article_form) = &self.article_form {
                        self.orig_article = Some(article_form.article.clone());
                        self.page_state = PageState::Show;
                    }
                },
                UIResult::DbError(msg) => {
                    emit(PageAction::AddError(msg));
                },
                UIResult::Article(ArticleResult::Article(article)) => {
                    self.article_form = Some(ArticleForm::new(article.clone()));
                    self.orig_article = Some(article);
                    self.page_state = PageState::Show;
                },
                _ => {
                }
            }
        }
        if let Some(article_form) = self.article_form.as_mut() {
            article_form.update(tx, &mut *emit);
        }
        self.article_comments_tab.update(tx, &mut *emit);
        self.article_favorites_tab.update(tx, &mut *emit);
        self.article_tags_tab.update(tx, &mut *emit);
    }
    fn entity_ident(&self) -> EntityIdent {
        self.ident.clone()
    }
}

impl ArticleEdit {
    pub fn new(article_id: Uuid) -> Self {
        Self {
            ident: EntityIdent::Article(article_id),
            article_tags_tab: ArticleTagsTab::new(article_id),
            article_favorites_tab: ArticleFavoriteTab::new(article_id),
            article_comments_tab: ArticleCommentsTab::new(article_id),
            article_form: None,
            current_tab: ArticleTab::Details,
            orig_article: None,
            page_state: PageState::Show,
            event_bus: UIBus::default(),
            should_close: false,
        }
    }

    pub fn new_create(article: ArticleUI) -> Self {
        Self {
            ident: EntityIdent::Article(article.id),
            article_tags_tab: ArticleTagsTab::new(article.id),
            article_favorites_tab: ArticleFavoriteTab::new(article.id),
            article_comments_tab: ArticleCommentsTab::new(article.id),
            article_form: Some(ArticleForm::new(article)),
            current_tab: ArticleTab::Details,
            orig_article: None,
            page_state: PageState::Create,
            event_bus: UIBus::default(),
            should_close: false,
        }
    }

}

