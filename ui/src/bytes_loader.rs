use core::{api::{UICommand, UIResult}, images::api::ImageCommand};
use std::sync::Arc;
use dashmap::DashMap;

use command_bus::{CommandBus, ResponseChannel, UITask};
use egui::load::{BytesLoadResult, BytesLoader, BytesPoll, LoadError};
use models::Uuid;
#[cfg(not(target_arch = "wasm32"))]
use tokio::sync::mpsc::{self, error::TryRecvError};
#[cfg(target_arch = "wasm32")]
use std::sync::mpsc::{self, TryRecvError};


pub struct BlobLoader {
    bus: CommandBus,
    pub cache: Arc<DashMap<String, BlobEntry>>,
    image_response_tx: mpsc::Sender<UIResult>,
}

pub enum BlobEntry {
    Loading,
    Ready(Arc<[u8]>),
}

impl BlobLoader {
    pub fn new(command_bus: CommandBus, image_response_tx: mpsc::Sender<UIResult>) -> Self {
        Self {
            bus: command_bus,
            cache: Arc::new(DashMap::new()),
            image_response_tx,
        }
    }
}

// 2. Implement the BytesLoader trait
impl BytesLoader for BlobLoader {
    fn id(&self) -> &str {
        "blob_loader"
    }
    fn load(&self, _ctx: &egui::Context, uri: &str) -> BytesLoadResult {
        if uri.starts_with("blob://") {
            let data = self.cache.get(uri);
            if let Some(data) = data {
                match data.value() {
                    BlobEntry::Loading => {
                        return Ok(BytesPoll::Pending { size: None })                
                    },
                    BlobEntry::Ready(data) => {
                        return Ok(BytesPoll::Ready { size: None, bytes: egui::load::Bytes::Shared(data.clone()), mime: None })
                    }
                }
            } else {
                if let Ok(blob_id) = Uuid::try_parse(&uri[7..]) {
                    let response_channel = ResponseChannel::new(self.image_response_tx.clone());
                    self.bus.dispatch(UITask { 
                        command: UICommand::Image(ImageCommand::Load(blob_id)),
                        response: response_channel });
                    let _r = self.cache.insert(uri.into(), BlobEntry::Loading);
                    return Ok(BytesPoll::Pending { size: None })      
                } else {
                    log::error!("can not parse uuid {} from {}", &uri[7..],uri);
                    return Err(LoadError::FormatNotSupported { detected_format: None })
                }
            }
        } else {
            Err(LoadError::NotSupported)
        }
    }
    fn forget(&self, uri: &str) {
        log::info!("forget {}",uri);
    }
    fn forget_all(&self) {
        log::info!("forget all");
    }
    fn byte_size(&self) -> usize {
        0
    }
}