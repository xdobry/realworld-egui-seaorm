use std::sync::mpsc;
use std::sync::{Arc, RwLock};
use wasm_bindgen_futures::wasm_bindgen::JsCast;
use app_core::api::UIResult;
use command_bus::{CommandBus, ResponseChannel, UITask};
use poll_promise::Promise;
use ui::app::FormsApp;

#[cfg(not(target_arch = "wasm32"))]
compile_error!("This crate is intended to be compiled to wasm32 only");

struct PendingRequest {
    pub poll_promise: poll_promise::Promise<UIResult>,
    pub response_channel: ResponseChannel,
}

fn main() {
    // Web start

    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window()
            .expect("No window")
            .document()
            .expect("No document");

        let canvas = document
            .get_element_by_id("the_canvas")
            .expect("Failed to find the_canvas_id")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("the_canvas_id was not a HtmlCanvasElement");
    
        
        eframe::WebRunner::new()
            .start(
                canvas, // matches id in index.html
                eframe::WebOptions::default(),
                Box::new(| cc | {
                    let egui_context = cc.egui_ctx.clone();
                    let (command_tx, command_rx) = mpsc::channel::<UITask>();
                    let mut command_bus = CommandBus::new(command_tx);
                    let shared_context: ui::app::SharedContext =  Arc::new(RwLock::new(None));
                    let mut pending_requests = Vec::new();
                    let client = reqwest::Client::new();
                    let shared_context_clone = shared_context.clone();

                    command_bus.update_call = Some(Box::new(move || {
                        if let Ok(task) = command_rx.try_recv() {
                            let client = client.clone();
                            let shared_context_clone = shared_context_clone.clone();
                            egui_context.request_repaint();
                            let egui_context = egui_context.clone();
                            let promise = Promise::spawn_local(async move {
                                let out_msg = postcard::to_stdvec(&task.command).unwrap();
                                let token = shared_context_clone.read().unwrap().as_ref().map(|c| c.token.clone());
                                // TODO take url from current page or config
                                let url = "http://localhost:8081/uicmd";
                                let request = match token {
                                    Some(token) => {
                                        client.post(url)
                                        .header("Content", "application/octet-stream")
                                        .header("Authorization", String::from_utf8(token).unwrap())
                                    },
                                    None => client.post(url)
                                        .header("Content", "application/octet-stream"),
                                };
                                let request = request.body(out_msg);
                                let request_result = request.send().await;
                                egui_context.request_repaint();
                                match request_result {
                                    Ok(resp) => {
                                        if resp.status().is_success() {
                                            if let Ok(bytes) = resp.bytes().await {
                                                let response: UIResult = postcard::from_bytes(&bytes).unwrap_or(UIResult::DbError("cannot deserialize server response".into()));
                                                return response;
                                            }
                                        } else {
                                            let status_str = resp.status().as_str().to_string();
                                            if let Ok(msg) = resp.text().await {
                                                return UIResult::DbError(format!("http error {}: msg {}",status_str,msg))
                                            }
                                        }
                                    }
                                    Err(err) => {
                                        return UIResult::DbError(format!("Error downloading from URL {}", err));
                                    }
                                }
                                UIResult::DbError("err during rest processing".into())
                            });
                            pending_requests.push(PendingRequest {
                                poll_promise: promise,
                                response_channel: task.response,
                            });
                        }
                        let ready: Vec<_> = pending_requests.extract_if(..,| f | {
                            f.poll_promise.ready().is_some()
                        }).collect();
                        for mut item in ready {
                            if let Ok(result) = item.poll_promise.try_take() {
                                item.response_channel.send(result);
                            }
                        }
                    }));

                    Ok(Box::new(FormsApp::new(cc.storage, command_bus, shared_context)))
                })
                ,
            )
            .await
            .expect("failed to start eframe");
    });
}

