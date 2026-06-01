use core::entities::EntityIdent;
use std::any::Any;

use egui_commonmark::CommonMarkCache;
use models::Uuid;
use core::api::{UICommand, UIResult};
use core::articles::api::{ArticleCommand, ArticleResult};
use core::articles::dto::{ArticleListItem, ArticlePopularListItem, ArticleUI};
use command_bus::{CommandBus, UIBus};
use crate::ui::articles::forms::ArticleForm;
use crate::ui::articles::tables::{show_articles_table, show_popular_articles_table};
use crate::ui::articles::tabs::{ArticleCommentsTab, ArticleFavoriteTab, ArticleTagsTab};
use crate::ui::core::page::{Form, Page, PageAction, PageState, UIContext};
use crate::ui::core::tables::{TableAction};
use crate::ui::notebook::NoteBook;

fn entity_ident_to_command(entity_ident: &EntityIdent) -> ArticleCommand {
    match entity_ident {
        EntityIdent::ArticleList => {
            ArticleCommand::Reload
        }
        EntityIdent::ArticleListAuthor(author_id) => {
            ArticleCommand::ListAuthor(*author_id)
        }
        EntityIdent::ArticleListFavorites(user_id) => {
            ArticleCommand::ListFavorites(*user_id)
        }
        EntityIdent::ArticleListFollowed(user_id) => {
            ArticleCommand::ListFollowed(*user_id)
        }
        EntityIdent::ArticleListTag(tag_id, _tag_name) => {
            ArticleCommand::ListTag(*tag_id)
        }
        EntityIdent::ArticlePopularList => {
            ArticleCommand::ListPopular
        }
        _ => {
            panic!("not list identity")
        }
    }
}

pub struct ArticleTable {
    list_identity: EntityIdent,
    title: Option<String>,
    articles: Vec<ArticleListItem>,
    event_bus: UIBus,
    should_close: bool,
}

impl Page for ArticleTable {
    fn show(&mut self, ui: &mut egui::Ui, tx: &mut CommandBus, ui_context: &UIContext, _cache: &mut CommonMarkCache) -> PageAction {
        let mut page_action = PageAction::None;
        ui.horizontal(|ui| {
            if ui.button("Reload").clicked() {
                let load_command = entity_ident_to_command(&self.list_identity);
                self.event_bus.send_task(tx, UICommand::Article(load_command));
            }
            if !ui_context.is_anonymous() {
                if ui.button("New").clicked() {
                    page_action = PageAction::AddPage(Box::new(ArticleEdit::new_create(ArticleUI::new(ui_context.user_id()))));
                }
            }
        });
        let table_action = show_articles_table(ui, &self.articles, ui_context, false);
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
    fn init(&mut self, tx: &mut CommandBus, _ui_context: &UIContext) {
        self.event_bus.send_task(tx, UICommand::Article(entity_ident_to_command(&self.list_identity)));
    }
    fn update(&mut self, _tx: &mut CommandBus, _ui_context: &UIContext, emit: &mut dyn FnMut(PageAction)) {
        if let Ok(msg) = self.event_bus.try_recv() {
            match msg {
                UIResult::Article(ArticleResult::ArticleList(article_result)) => {
                    self.articles = article_result;
                    if let EntityIdent::ArticleListTag(_tag_id,tag_name ) = &self.list_identity {
                        self.title = Some(format!("Articles: {}", tag_name));
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
    fn title(&self, ui_context: &UIContext) -> &str {
        match &self.list_identity {
            EntityIdent::ArticleListAuthor(author_id) => {
                if ui_context.user_id() == *author_id {
                    "My Articles"
                } else {
                    "Authors Articles"
                }
            }
            EntityIdent::ArticleListFavorites(_user_id) => {
                "My Favorites"
            }
            EntityIdent::ArticleListFollowed(_user_id) => {
                "Feed"
            }
            EntityIdent::ArticleListTag(_tag_id, _tag_name) => {
                if let Some(title) = &self.title {
                    title
                } else {
                    "Article Tags"
                }
            }
            EntityIdent::ArticlePopularList => {
                "Popular"
            }
            _ => {
                "Newest Articles"
            } 
        }
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn should_close(&self) -> bool {
        self.should_close
    }
    fn entity_ident(&self) -> EntityIdent {
        self.list_identity.clone()
    }
}

impl ArticleTable {
    pub fn new(list_identity: EntityIdent) -> Self {
        Self {
            list_identity,
            title: None,
            articles: Vec::new(),
            event_bus: UIBus::default(),
            should_close: false,
        }
    }
}

pub struct ArticlePopularTable {
    list_identity: EntityIdent,
    articles: Vec<ArticlePopularListItem>,
    event_bus: UIBus,
    should_close: bool,
}

impl Page for ArticlePopularTable {
    fn show(&mut self, ui: &mut egui::Ui, tx: &mut CommandBus, ui_context: &UIContext, _cache: &mut CommonMarkCache) -> PageAction {
        let mut page_action = PageAction::None;
        ui.horizontal(|ui| {
            if ui.button("Reload").clicked() {
                let load_command = entity_ident_to_command(&self.list_identity);
                self.event_bus.send_task(tx, UICommand::Article(load_command));
            }
            if !ui_context.is_anonymous() {
                if ui.button("New").clicked() {
                    page_action = PageAction::AddPage(Box::new(ArticleEdit::new_create(ArticleUI::new(ui_context.user_id()))));
                }
            }
        });
        let table_action = show_popular_articles_table(ui, &self.articles, ui_context);
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
    fn init(&mut self, tx: &mut CommandBus, _ui_context: &UIContext) {
        self.event_bus.send_task(tx, UICommand::Article(entity_ident_to_command(&self.list_identity)));
    }
    fn update(&mut self, _tx: &mut CommandBus, _ui_context: &UIContext, emit: &mut dyn FnMut(PageAction)) {
        if let Ok(msg) = self.event_bus.try_recv() {
            match msg {
                UIResult::Article(ArticleResult::ArticlePopular(article_result)) => {
                    self.articles = article_result;
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
    fn title(&self, _ui_context: &UIContext) -> &str {
        "Popular"
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn should_close(&self) -> bool {
        self.should_close
    }
    fn entity_ident(&self) -> EntityIdent {
        self.list_identity.clone()
    }
}

impl ArticlePopularTable {
    pub fn new(list_identity: EntityIdent) -> Self {
        Self {
            list_identity,
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

impl TryFrom<usize> for ArticleTab {
    type Error = ();
    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Details),
            1 => Ok(Self::Tags),
            2 => Ok(Self::Favorites),
            3 => Ok(Self::Comments),
            _ => Err(()),
        }
    }
}

pub struct ArticleEdit {
    ident: EntityIdent,
    title: Option<String>,
    article_form: Option<ArticleForm>,
    orig_article: Option<ArticleUI>,
    current_tab: usize,
    page_state: PageState,
    article_tags_tab: ArticleTagsTab,
    article_favorites_tab: ArticleFavoriteTab,
    article_comments_tab: ArticleCommentsTab,
    event_bus: UIBus,
    should_close: bool,
}

static ARTICLE_TABS: [&str; 4]  = ["Details", "Tags", "Favorites", "Comments"];

impl Page for ArticleEdit {
    fn init(&mut self, tx: &mut CommandBus, _ui_context: &UIContext) {
        if self.article_form.is_none() {
            if let EntityIdent::Article(article_id) = self.ident {
                self.event_bus.send_task(tx,UICommand::Article(ArticleCommand::Load(article_id)));
                self.page_state = PageState::Loading;
            }
        }
    }
    fn show(&mut self, ui: &mut egui::Ui, tx: &mut CommandBus, ui_context: &UIContext, cache: &mut CommonMarkCache) -> PageAction {
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
                            article_form.article.updated_at = core::time_now();
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
                        if let Some(orig_article) = &self.orig_article {
                            if ui_context.is_user_or_admin(orig_article.author_id) {
                                if ui.button("Start Update").clicked() {
                                    self.page_state = PageState::Update;
                                }
                            }
                        }
                    }
                }
            });
            if !matches!(self.page_state,PageState::Create) {
                NoteBook::new(4, &mut self.current_tab, |i| ARTICLE_TABS[i].into(),200.0, false).show(ui);
            }
            if let Ok(current_tab) = ArticleTab::try_from(self.current_tab) {
                match current_tab {
                    ArticleTab::Details => {
                        let ui_context = if self.page_state.is_enabled() { &ui_context.as_edit() } else { ui_context};                   
                        article_form.show_ui(ui, tx, ui_context, &mut page_action, cache);
                    }
                    ArticleTab::Tags => {
                        if article_form.article.author_id==ui_context.user_id() {
                            let admin_context = ui_context.as_admin();
                            self.article_tags_tab.show_ui(ui, tx, &admin_context, &mut page_action);
                        } else {
                            self.article_tags_tab.show_ui(ui, tx, ui_context, &mut page_action);
                        }
                    }
                    ArticleTab::Favorites => {
                        self.article_favorites_tab.show_ui(ui, tx, ui_context, &mut page_action);
                    }
                    ArticleTab::Comments => {
                        self.article_comments_tab.show_ui(ui, tx, ui_context, &mut page_action);
                    }
                }
            } 
        }
        page_action
    }
    fn title(&self, _ui_context: &UIContext) -> &str {
        if let Some(title) = &self.title {
            title
        } else {
            "Article"
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
                    if let Some(article_form) = &self.article_form {
                        self.orig_article = Some(article_form.article.clone());
                        self.page_state = PageState::Show;
                        self.title = Some(format!("\u{1f5b9} {}", article_form.article.title))
                    }
                },
                UIResult::Created => {
                    if let Some(article_form) = &self.article_form {
                        self.orig_article = Some(article_form.article.clone());
                        self.page_state = PageState::Show;
                        self.title = Some(format!("\u{1f5b9} {}", article_form.article.title))
                    }
                },
                UIResult::DbError(msg) => {
                    emit(PageAction::AddError(msg));
                },
                UIResult::Article(ArticleResult::Article(article)) => {
                    self.title = Some(format!("\u{1f5b9} {}", article.title));
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
            current_tab: 0,
            orig_article: None,
            page_state: PageState::Show,
            event_bus: UIBus::default(),
            should_close: false,
            title: None,
        }
    }

    pub fn new_create(article: ArticleUI) -> Self {
        Self {
            ident: EntityIdent::Article(article.id),
            article_tags_tab: ArticleTagsTab::new(article.id),
            article_favorites_tab: ArticleFavoriteTab::new(article.id),
            article_comments_tab: ArticleCommentsTab::new(article.id),
            article_form: Some(ArticleForm::new(article)),
            current_tab: 0,
            orig_article: None,
            page_state: PageState::Create,
            event_bus: UIBus::default(),
            should_close: false,
            title: Some("New Article".into())
        }
    }

}

