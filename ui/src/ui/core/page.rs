
use core::{entities::EntityIdent, users::dto::LoginResponse};
use std::any::Any;

use command_bus::CommandBus;
use models::Uuid;

pub trait Page: Any {
    fn show(&mut self, ui: &mut egui::Ui,  tx: &mut CommandBus, ui_context: &UIContext) -> PageAction;
    fn title(&self, ui_context: &UIContext) -> &str;
    fn as_any(&self) -> &dyn Any;
    fn update(&mut self, _tx: &mut CommandBus, _uc: &UIContext,_emit: &mut dyn FnMut(PageAction)) 
    {        
    }
    fn init(&mut self,  _tx: &mut CommandBus, _uc: &UIContext)  {
    }
    fn should_close(&self) -> bool;
    fn entity_ident(&self) -> EntityIdent {
        EntityIdent::None
    }
}

pub struct UIContext {
    user_id: Option<Uuid>,
    is_admin: bool,
    is_edit: bool,
}

impl UIContext {
    pub fn is_anonymous(&self) -> bool {
        self.user_id.is_none()
    }
    pub fn is_admin(&self) -> bool {
        self.is_admin
    }
    pub fn is_user_or_admin(&self, user_id: Uuid) -> bool {
        self.is_admin || self.user_id == Some(user_id)
    }
    pub fn new(user_context: &Option<LoginResponse>) -> Self {
        if let Some(user_context) = &user_context {
            Self {
                user_id: Some(user_context.user_context.user_id),
                is_admin: user_context.user_context.is_admin,
                is_edit: false,
            }
        } else {
            Self {
                user_id: None,
                is_admin: false,
                is_edit: false,
            }
        }
    }
    pub fn user_id(&self) -> Uuid {
        if let Some(user_id) = self.user_id {
            user_id
        } else {
            Uuid::nil()
        }
    }
    pub fn is_edit(&self) -> bool {
        self.is_edit
    }
    pub fn as_admin(&self) -> Self {
        Self {
            user_id: self.user_id,
            is_admin: true,
            is_edit: self.is_edit,
        }
    }
    pub fn as_edit(&self) -> Self {
        Self {
            user_id: self.user_id,
            is_admin: self.is_admin,
            is_edit: true,
        }
    }
}


pub struct DbError {
    pub msg: String,
    should_close: bool,
}

impl Page for DbError {
    fn show(&mut self, ui: &mut egui::Ui, _tx: &mut CommandBus, _uc: &UIContext) -> PageAction {
        if ui.button("Close").clicked() {
            self.should_close = true;
        }
        ui.label(self.msg.as_str());
        PageAction::None
    }
    fn title(&self, _ui_context: &UIContext) -> &str {
        "Error"
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn should_close(&self) -> bool {
        self.should_close
    }
}

impl DbError {
    pub fn new(msg: String) -> Self {
        Self {
            msg,
            should_close: false,
        }
    }
}


pub enum PageAction {
    AddPage(Box<dyn Page>),
    AddError(String),
    Navigate(EntityIdent),
    None,
}

pub trait Form {
    fn show_ui(&mut self, ui: &mut egui::Ui, tx: &mut CommandBus, ui_context: &UIContext, page_action: &mut PageAction);
    fn update<F>(&mut self, _tx: &mut CommandBus, mut _emit: F) 
    where 
        F: FnMut(PageAction),
    {
        
    }
    fn init(&mut self, _tx: &mut CommandBus)  {
    }
}

pub enum PageState {
    Show,
    Update,
    Updating,
    Final,
    Create,
    Creating,
    Loading,
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
