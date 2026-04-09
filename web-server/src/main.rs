use app_core::api::{UIResult, UICommand};
use axum::{
    Router, body::Bytes, extract::State, response::IntoResponse, routing::post
};
use command_bus::{ResponseChannel};
use dotenvy::dotenv;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use std::{env, net::SocketAddr, sync::Arc, time::Duration};
use tokio::sync::mpsc;
use tower_http::cors::{Any, CorsLayer};
use axum::http::StatusCode;
use tower_http::services::{ServeDir, ServeFile};

#[derive(Clone)]
struct AppState {
    db: Arc<DatabaseConnection>,
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    let dist_folder = env::var("FRONTEND_DIST").expect("FRONTEND_DIST must be set");

    let addr = SocketAddr::from(([0, 0, 0, 0], 8081));
    println!("Server running on http://{}", addr);

    let mut opt = ConnectOptions::new(database_url);
    opt.max_connections(10)
        .min_connections(2)
        .connect_timeout(Duration::from_secs(10))
        .idle_timeout(Duration::from_secs(300))
        .sqlx_logging(true);

    let db = Database::connect(opt).await.unwrap();

    let state = AppState {
        db: Arc::new(db),
    };

    // TODO: This allow cors to anyone. Not for productive usage!
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/uicmd", post(handle_uicmd))
        .fallback_service(
            ServeDir::new(dist_folder.clone())
                .not_found_service(ServeFile::new(format!("{}/index.html",dist_folder)))
        )
        .with_state(state)
        .layer(cors);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn handle_uicmd(
    State(state): State<AppState>, 
    body: Bytes
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let data = body.to_vec();
    // TODO Pass internal errors to client (not so good for production). Several places
    let cmd: UICommand = postcard::from_bytes(&data).map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid body: {}",e)))?;

    let (result_tx, mut result_rx) = mpsc::channel::<UIResult>(1);
    let mut response_channel = ResponseChannel::new(result_tx);
    server_core::handle_ui_command(cmd, &mut response_channel, &state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Error during processing msg: {}",e)))?;
    if let Some(result) = result_rx.recv().await {
        let out_msg = postcard::to_stdvec(&result)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Error during serializing response: {}",e)))?;
        return Ok(out_msg);
    }
    return Err((StatusCode::INTERNAL_SERVER_ERROR,"can not get result".to_string()));
}