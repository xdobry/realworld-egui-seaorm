use tokio::sync::mpsc::{self, error::TryRecvError};
use std::any::Any;

use core::api::{UICommand, UIResult};

use crate::api::{CommandBus, ResponseChannel, UITask};

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

pub struct UIBus {
    result_rx: mpsc::Receiver<UIResult>,
    result_tx: mpsc::Sender<UIResult>,
}

impl UIBus {
    pub fn send_task(&self, command_bus: &mut CommandBus, ui_command: UICommand) {
        command_bus.dispatch(UITask::new(ui_command, ResponseChannel::new(self.result_tx.clone())));
    }
    pub fn try_recv(&mut self) -> Result<UIResult, TryRecvError> {
        self.result_rx.try_recv()
    }
}

impl Default for UIBus {
    fn default() -> Self {
        let (result_tx, result_rx) = mpsc::channel::<UIResult>(5);
        Self { result_rx, result_tx }
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
