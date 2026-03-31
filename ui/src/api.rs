use core::api::{UICommand, UIResult};

use tokio::sync::mpsc;

pub struct CommandBus {
    tx: mpsc::Sender<UITask>
}



impl CommandBus {
    pub fn dispatch(&mut self, cmd: UITask) {
        let _rx = self.tx.try_send(cmd);
    }
    pub fn new(tx: mpsc::Sender<UITask>) -> Self {
        Self {
            tx
        }
    }
}

pub struct ResponseChannel {
    tx: mpsc::Sender<UIResult>,    
}


impl ResponseChannel {
    pub fn send(&mut self, result: UIResult) {
        let _rx = self.tx.try_send(result);
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
