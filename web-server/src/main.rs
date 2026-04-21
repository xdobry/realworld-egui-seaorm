use app_core::{api::{TokenClaims, UICommand, UIResult}, users::dto::UserContext};
use axum::{
    Extension, Router, body::{Body, Bytes}, extract::State, http::{Request, Response}, middleware::{self, Next}, response::IntoResponse, routing::post
};
use command_bus::{ResponseChannel};
use dotenvy::dotenv;
use sea_orm::{ConnectOptions, Database, DatabaseConnection, sea_query::Token};
use server_core::{CallContext, token::{expiration_time, is_token_expired, load_secret, sign_payload, verify_token}};
use std::{env, net::SocketAddr, sync::Arc, time::Duration};
use tokio::sync::mpsc;
use tower_http::cors::{Any, CorsLayer};
use axum::http::StatusCode;
use tower_http::services::{ServeDir, ServeFile};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};

#[derive(Clone)]
struct AppState {
    db: Arc<DatabaseConnection>,
    server_context: Arc<ServerContext>,
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
        server_context: Arc::new(ServerContext {
            password_salt: env::var("PASSWORD_SALT").expect("PASSWORD_SALT must be set"),
            token_secret: load_secret(env::var("TOKEN_SECRET").expect("TOKEN_SECRET must be set").as_str()),
        }),  
    };
    // check if dist folder exists
    if !std::path::Path::new(&dist_folder).is_dir() {
        println!("Dist folder does not exist: {}", dist_folder);
    }
    // create absolute path to dist folder
    // let dist_folder_abs = std::fs::canonicalize(dist_folder.clone()).expect("Failed to get absolute path to dist folder");
    // println!("Serving frontend from: {}", dist_folder_abs.display());

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
        .layer(middleware::from_fn_with_state(state.clone(),auth_middleware))
        .with_state(state)
        .layer(cors);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn handle_uicmd(
    State(state): State<AppState>,
    Extension(token_claims): Extension<Option<TokenClaims>>,
    body: Bytes
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let data = body.to_vec();
    // TODO Pass internal errors to client (not so good for production). Several places
    let cmd: UICommand = postcard::from_bytes(&data).map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid body: {}",e)))?;

    let (result_tx, mut result_rx) = mpsc::channel::<UIResult>(1);
    let mut response_channel = ResponseChannel::new(result_tx);
    let call_context = MyCallContext {
        server_context: state.server_context.clone(),
        token_claims: token_claims,
    };
    server_core::handle_ui_command(cmd, &mut response_channel, &state.db, &call_context).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Error during processing msg: {}",e)))?;
    if let Some(result) = result_rx.recv().await {
        let out_msg = postcard::to_stdvec(&result)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Error during serializing response: {}",e)))?;
        return Ok(out_msg);
    }
    return Err((StatusCode::INTERNAL_SERVER_ERROR,"can not get result".to_string()));
}

async fn auth_middleware(
    State(app): State<AppState>,
    mut req: Request<Body>,
    next: Next,
) -> Response<Body> {
    let token_opt = req
        .headers()
        .get("authorization")
        .and_then(|h| h.to_str().ok());

    if let Some(token) = token_opt {
        let token_bytes = decode_token(token);
        let token_payload = verify_token(&token_bytes, &app.server_context.token_secret);
        if let Some(token_payload) = token_payload {
            let claims= postcard::from_bytes::<TokenClaims>(&token_payload);
            match claims {
                Ok(claims) => {
                    if is_token_expired(claims.exp) {
                        return (StatusCode::UNAUTHORIZED, "TOKEN_EXPIRED").into_response();
                    } else {
                        req.extensions_mut().insert(Some(claims));
                    }
                },
                Err(_) => {
                    return (StatusCode::UNAUTHORIZED, "INVALID_TOKEN").into_response();
                }
            }
        } else {
            return (StatusCode::UNAUTHORIZED, "INVALID_TOKEN").into_response();
        }
    } else {
        req.extensions_mut().insert(None::<TokenClaims>);
    }
    next.run(req).await
}

fn decode_token(s: &str) -> Vec<u8> {
    URL_SAFE_NO_PAD.decode(s).expect("invalid base64 token")
}

fn encode_token(token: &[u8]) -> String {
    URL_SAFE_NO_PAD.encode(token)
}

struct ServerContext {
    password_salt: String,
    token_secret: Vec<u8>,
}

struct MyCallContext {
    server_context: Arc<ServerContext>,
    token_claims: Option<TokenClaims>,
}

impl CallContext for MyCallContext {
    fn is_admin(&self) -> bool {
        if let Some(token_claims) = &self.token_claims {
            return token_claims.is_admin;
        }
        false
    }

    fn user_id(&self) -> Option<sea_orm::prelude::Uuid> {
        if let Some(token_claims) = &self.token_claims {
            return Some(token_claims.user_id);
        }
        None
    }

    fn encode_password(&self, password: &str) -> String {
        let password_bytes = password.as_bytes();
        let argo_config = argon2::Config::default();
        let hashed_password = argon2::hash_encoded(password_bytes, self.server_context.password_salt.as_bytes(), &argo_config).unwrap();
        hashed_password
    }

    fn verify_password(&self, attempted_password: &str, hash: &str) -> bool {
        argon2::verify_encoded(hash, attempted_password.as_bytes()).unwrap()
    }
    
    fn create_token(&self, user_context: &UserContext) -> Vec<u8> {
        let token_claim = TokenClaims {
            user_id: user_context.user_id,
            is_admin: user_context.is_admin,
            exp: expiration_time(360)
        };
        let payload = postcard::to_stdvec(&token_claim).unwrap();
        let token = sign_payload(&payload, &self.server_context.token_secret);
        let token_base64 = encode_token(&token);
        token_base64.into_bytes()
    }   
}