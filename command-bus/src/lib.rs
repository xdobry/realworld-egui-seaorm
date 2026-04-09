use core::api::{UICommand, UIResult};
#[cfg(not(target_arch = "wasm32"))]
use tokio::sync::mpsc::{self, error::TryRecvError};
#[cfg(target_arch = "wasm32")]
use std::sync::mpsc::{self, TryRecvError};

pub struct CommandBus {
    tx: mpsc::Sender<UITask>,
    pub update_call: Option<Box<dyn FnMut() -> ()>>
}

impl CommandBus {
    pub fn dispatch(&mut self, cmd: UITask) {
        #[cfg(not(target_arch = "wasm32"))]
        let _rx = self.tx.try_send(cmd);
        #[cfg(target_arch = "wasm32")]
        let _rx = self.tx.send(cmd);
    }
    pub fn new(tx: mpsc::Sender<UITask>) -> Self {
        Self {
            tx,
            update_call: None
        }
    }
    pub fn update(&mut self) {
        if let Some(update_call) = self.update_call.as_mut() {
            update_call();
        }
    }
}

pub struct ResponseChannel {
    tx: mpsc::Sender<UIResult>,    
}


impl ResponseChannel {
    pub fn send(&mut self, result: UIResult) {
        #[cfg(not(target_arch = "wasm32"))]
        let _rx = self.tx.try_send(result);
        #[cfg(target_arch = "wasm32")]
        let _rx = self.tx.send(result);
    }
    pub fn new(tx: mpsc::Sender<UIResult>) -> Self {
        Self {
            tx
        }
    }
}



pub struct UITask {
    pub command: UICommand,
    pub response: ResponseChannel,
}

impl UITask {
    pub fn new(command: UICommand, response: ResponseChannel) -> Self {
        Self {
            command,
            response,
        }
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
        #[cfg(not(target_arch = "wasm32"))]
        let (result_tx, result_rx) = mpsc::channel::<UIResult>(5);
        #[cfg(target_arch = "wasm32")]
        let (result_tx, result_rx) = mpsc::channel::<UIResult>();
        Self { result_rx, result_tx }
    }
}
