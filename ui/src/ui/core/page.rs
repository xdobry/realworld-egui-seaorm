
use core::entities::EntityIdent;
use std::any::Any;

use command_bus::CommandBus;

pub trait Page: Any {
    fn show(&mut self, ui: &mut egui::Ui, tx: &mut CommandBus) -> PageAction;
    fn title(&self) -> &str;
    fn as_any(&self) -> &dyn Any;
    fn update(&mut self, _tx: &mut CommandBus,_emit: &mut dyn FnMut(PageAction)) 
    {        
    }
    fn init(&mut self, _tx: &mut CommandBus)  {
    }
    fn should_close(&self) -> bool;
    fn entity_ident(&self) -> EntityIdent {
        EntityIdent::None
    }
}




pub struct DbError {
    pub msg: String,
    should_close: bool,
}

impl Page for DbError {
    fn show(&mut self, ui: &mut egui::Ui, _tx: &mut CommandBus) -> PageAction {
        if ui.button("Close").clicked() {
            self.should_close = true;
        }
        ui.label(self.msg.as_str());
        PageAction::None
    }
    fn title(&self) -> &str {
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
    fn show_ui(&mut self, ui: &mut egui::Ui, tx: &mut CommandBus, page_action: &mut PageAction);
    fn update<F>(&mut self, _tx: &mut CommandBus, mut _emit: F) 
    where 
        F: FnMut(PageAction),
    {
        
    }
    fn init(&mut self, _tx: &mut CommandBus)  {
    }
}
