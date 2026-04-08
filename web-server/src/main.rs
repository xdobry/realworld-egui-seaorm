use app_core::api::{UIResult, UICommand};
use axum::{
    Router, body::Bytes, extract::State, response::IntoResponse, routing::post
};
use command_bus::{ResponseChannel, UITask};
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tokio::sync::mpsc;

#[derive(Clone)]
struct AppState {
    db: Arc<DatabaseConnection>,
}


#[tokio::main]
async fn main() {

    let addr = SocketAddr::from(([0, 0, 0, 0], 8081));
    println!("Server running on http://{}", addr);

    let mut opt = ConnectOptions::new("postgres://realworld:realworld@localhost/realworld");
    opt.max_connections(10)
        .min_connections(2)
        .connect_timeout(Duration::from_secs(10))
        .idle_timeout(Duration::from_secs(300))
        .sqlx_logging(true);

    let db = Database::connect(opt).await.unwrap();

    let state = AppState {
        db: Arc::new(db),
    };

    let app = Router::new().route("/uicmd", post(handle_uicmd)).with_state(state);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn handle_uicmd(
    State(state): State<AppState>, 
    body: Bytes
) -> impl IntoResponse {
    let db: &sea_orm::DatabaseConnection = &state.db;

    let mut data = body.to_vec();
    let cmd: UICommand = postcard::from_bytes(&data).unwrap();

    let (result_tx, mut result_rx) = mpsc::channel::<UIResult>(1);
    let mut response_channel = ResponseChannel::new(result_tx);
    server_core::handle_ui_command(cmd, &mut response_channel, &db).await.unwrap();
    if let Some(result) = result_rx.recv().await {
        // println!("result {:?}", result);
        let out_msg = postcard::to_stdvec(&result).unwrap();
        return out_msg;
    }
    return data;
}