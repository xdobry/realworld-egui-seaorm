
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
    fn should_close(&self) -> bool {
        false
    }
}




pub struct DbError {
    pub msg: String
}

impl Page for DbError {
    fn show(&mut self, ui: &mut egui::Ui, _tx: &mut CommandBus) -> PageAction {
        let mut page_action = PageAction::None;
        if ui.button("Close").clicked() {
            page_action = PageAction::Close;
        }
        ui.label(self.msg.as_str());
        page_action
    }
    fn title(&self) -> &str {
        "Error"
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl DbError {
    pub fn new(msg: String) -> Self {
        Self {
            msg
        }
    }
}


pub enum PageAction {
    Close,
    AddPage(Box<dyn Page>),
    AddError(String),
    None,
}

pub trait Form {
    fn show_ui(&mut self, ui: &mut egui::Ui, tx: &mut CommandBus);
    fn update<F>(&mut self, _tx: &mut CommandBus, mut _emit: F) 
    where 
        F: FnMut(PageAction),
    {
        
    }
    fn init(&mut self, _tx: &mut CommandBus)  {
    }
}
