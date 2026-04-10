use std::any::Any;

use models::entity::{articles};
use core::api::{UICommand, UIResult};
use core::articles::api::{ArticleCommand, ArticleResult};
use core::articles::dto::{ArticleUI};
use command_bus::{CommandBus, UIBus};
use crate::ui::articles::forms::ArticleForm;
use crate::ui::articles::tables::{show_articles_table};
use crate::ui::articles::tabs::{ArticleCommentsTab, ArticleFavoriteTab, ArticleTagsTab};
use crate::ui::core::page::{Form, Page, PageAction};
use crate::ui::core::tables::{TableAction, TableMode};

pub struct ArticleTable {
    articles: Vec<articles::Model>,
    event_bus: UIBus,
}

impl Page for ArticleTable {
    fn show(&mut self, ui: &mut egui::Ui, tx: &mut CommandBus) -> PageAction {
        let mut page_action = PageAction::None;
        ui.horizontal(|ui| {
            if ui.button("Reload").clicked() {
                self.event_bus.send_task(tx, UICommand::Article(ArticleCommand::Reload));
            }
            if ui.button("New").clicked() {
                page_action = PageAction::AddPage(Box::new(ArticleNew::new(ArticleUI::new())));
            }
            if ui.button("Close").clicked() {
                page_action = PageAction::Close;
            }
        });
        let table_action = show_articles_table(ui, &self.articles, TableMode::EditDelete);
        match table_action {
            TableAction::SelectItem(article_id, _label) => {
                self.event_bus.send_task(tx, UICommand::Article(ArticleCommand::Load(article_id)));
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
                        ArticleResult::Article(article) => {
                            emit(PageAction::AddPage(Box::new(ArticleEdit::new(article))));
                        }
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
}

impl ArticleTable {
    pub fn new() -> Self {
        Self {
            articles: Vec::new(),
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

pub struct ArticleNew {
    article_form: ArticleForm,
    page_state: PageState,
    event_bus: UIBus,
}

impl Page for ArticleNew {
    fn show(&mut self, ui: &mut egui::Ui, tx: &mut CommandBus) -> PageAction {
        let mut page_action = PageAction::None;
        ui.horizontal(|ui| {
            match self.page_state {
                PageState::Initial => {
                    if ui.button("Create").clicked() {
                        self.event_bus.send_task(tx,UICommand::Article(ArticleCommand::Create(self.article_form.article.to_model())));
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
            self.article_form.show_ui(ui, tx);
        });
        page_action

    }
    fn title(&self) -> &str {
        "New Article"
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn update(&mut self, tx: &mut CommandBus,emit: &mut dyn FnMut(PageAction)) {
        if let Ok(msg) = self.event_bus.try_recv() {
            match msg {
                UIResult::Created => {
                    self.page_state = PageState::Final;
                },
                UIResult::DbError(msg) => {
                    self.page_state = PageState::Initial;
                    emit(PageAction::AddError(msg));
                },
                 _ => {
                }
            }
        }
        self.article_form.update(tx, &mut *emit);
    }
}

impl ArticleNew {
    pub fn new(article: ArticleUI) -> Self {       
        Self {
            article_form: ArticleForm::new(article),
            page_state: PageState::Initial,
            event_bus: UIBus::default(),
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
    article_form: ArticleForm,
    orig_article: ArticleUI,
    current_tab: ArticleTab,
    page_state: PageState,
    article_tags_tab: ArticleTagsTab,
    article_favorites_tab: ArticleFavoriteTab,
    article_comments_tab: ArticleCommentsTab,
    event_bus: UIBus,
}

impl Page for ArticleEdit {
    fn show(&mut self, ui: &mut egui::Ui, tx: &mut CommandBus) -> PageAction {
        let mut page_action = PageAction::None;
        ui.horizontal(|ui| {
            match self.page_state {
                PageState::Initial => {
                    if ui.button("Update").clicked() {
                        self.event_bus.send_task(tx,UICommand::Article(ArticleCommand::Update(self.article_form.article.to_change_record(&self.orig_article))));
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
            if ui.selectable_label(matches!(self.current_tab, ArticleTab::Details), "Details").clicked() {
                self.current_tab = ArticleTab::Details;
            }
            if ui.selectable_label(matches!(self.current_tab, ArticleTab::Tags), "Tags").clicked() {
                self.current_tab = ArticleTab::Tags;
            }
            if ui.selectable_label(matches!(self.current_tab, ArticleTab::Favorites), "Favorites").clicked() {
                self.current_tab = ArticleTab::Favorites;
            }
            if ui.selectable_label(matches!(self.current_tab, ArticleTab::Comments), "Comments").clicked() {
                self.current_tab = ArticleTab::Comments;
            }

        });
        match self.current_tab {
            ArticleTab::Details => {
                ui.add_enabled_ui(self.page_state.is_initial(), |ui| {
                    self.article_form.show_ui(ui, tx);
                });
            }
            ArticleTab::Tags => {
                self.article_tags_tab.show_ui(ui, tx);
            }
            ArticleTab::Favorites => {
                self.article_favorites_tab.show_ui(ui, tx);
            }
            ArticleTab::Comments => {
                self.article_comments_tab.show_ui(ui, tx);
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
        self.article_form.update(tx, &mut *emit);
        self.article_comments_tab.update(tx, &mut *emit);
        self.article_favorites_tab.update(tx, &mut *emit);
        self.article_tags_tab.update(tx, &mut *emit);
    }
}

impl ArticleEdit {
    pub fn new(orig_article: ArticleUI) -> Self {
        Self {
            article_tags_tab: ArticleTagsTab::new(orig_article.id),
            article_favorites_tab: ArticleFavoriteTab::new(orig_article.id),
            article_comments_tab: ArticleCommentsTab::new(orig_article.id),
            article_form: ArticleForm::new(orig_article.clone()),
            current_tab: ArticleTab::Details,
            orig_article: orig_article,
            page_state: PageState::Initial,
            event_bus: UIBus::default(),
        }
    }

}

