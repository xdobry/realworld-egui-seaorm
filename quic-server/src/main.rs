//! This example demonstrates an HTTP server that serves files from a directory.
//!
//! Checkout the `README.md` for guidance.

use std::{
    env, fs, io, net::SocketAddr, path::PathBuf, sync::Arc, time::Duration
};

mod common;

use anyhow::{Context, Result, bail};
use app_core::{api::{RemoteMessage, TokenClaims, UIResult}, users::dto::UserContext};
use server_core::token::{expiration_time, is_token_expired, load_secret, sign_payload, verify_token};
use clap::Parser;
use dotenvy::dotenv;
use proto::crypto::rustls::QuicServerConfig;
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer, pem::PemObject};
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use server_core::CallContext;
use tokio::sync::mpsc;
use tracing::{error, info, info_span};
use tracing_futures::Instrument as _;
use command_bus::ResponseChannel;

#[derive(Parser, Debug)]
#[clap(name = "server")]
struct Opt {
    /// file to log TLS keys to for debugging
    #[clap(long = "keylog")]
    keylog: bool,
    /// TLS private key in PEM format
    #[clap(short = 'k', long = "key", requires = "cert")]
    key: Option<PathBuf>,
    /// TLS certificate in PEM format
    #[clap(short = 'c', long = "cert", requires = "key")]
    cert: Option<PathBuf>,
    /// Enable stateless retries
    #[clap(long = "stateless-retry")]
    stateless_retry: bool,
    /// Address to listen on
    #[clap(long = "listen", default_value = "127.0.0.1:4433")]
    listen: SocketAddr,
    /// Client address to block
    #[clap(long = "block")]
    block: Option<SocketAddr>,
    /// Maximum number of concurrent connections to allow
    #[clap(long = "connection-limit")]
    connection_limit: Option<usize>,
}

fn main() {
    tracing::subscriber::set_global_default(
        tracing_subscriber::FmtSubscriber::builder()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .finish(),
    )
    .unwrap();
    let opt = Opt::parse();
    let code = {
        if let Err(e) = run(opt) {
            eprintln!("ERROR: {e}");
            1
        } else {
            0
        }
    };
    ::std::process::exit(code);
}

#[tokio::main]
async fn run(options: Opt) -> Result<()> {
    let (certs, key) = if let (Some(key_path), Some(cert_path)) = (&options.key, &options.cert) {
        let key = if key_path.extension().is_some_and(|x| x == "der") {
            PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(
                fs::read(key_path).context("failed to read private key file")?,
            ))
        } else {
            PrivateKeyDer::from_pem_file(key_path)
                .context("failed to read PEM from private key file")?
        };

        let cert_chain = if cert_path.extension().is_some_and(|x| x == "der") {
            vec![CertificateDer::from(
                fs::read(cert_path).context("failed to read certificate chain file")?,
            )]
        } else {
            CertificateDer::pem_file_iter(cert_path)
                .context("failed to read PEM from certificate chain file")?
                .collect::<Result<_, _>>()
                .context("invalid PEM-encoded certificate")?
        };

        (cert_chain, key)
    } else {
        let dirs = directories_next::ProjectDirs::from("org", "quinn", "quinn-examples").unwrap();
        let path = dirs.data_local_dir();
        let cert_path = path.join("cert.der");
        let key_path = path.join("key.der");
        let (cert, key) = match fs::read(&cert_path).and_then(|x| Ok((x, fs::read(&key_path)?))) {
            Ok((cert, key)) => (
                CertificateDer::from(cert),
                PrivateKeyDer::try_from(key).map_err(anyhow::Error::msg)?,
            ),
            Err(ref e) if e.kind() == io::ErrorKind::NotFound => {
                info!("generating self-signed certificate");
                let cert = rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();
                let key = PrivatePkcs8KeyDer::from(cert.signing_key.serialize_der());
                let cert = cert.cert.into();
                fs::create_dir_all(path).context("failed to create certificate directory")?;
                fs::write(&cert_path, &cert).context("failed to write certificate")?;
                fs::write(&key_path, key.secret_pkcs8_der())
                    .context("failed to write private key")?;
                (cert, key.into())
            }
            Err(e) => {
                bail!("failed to read certificate: {}", e);
            }
        };

        (vec![cert], key)
    };

    let mut server_crypto = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)?;
    server_crypto.alpn_protocols = common::ALPN_QUIC_HTTP.iter().map(|&x| x.into()).collect();
    if options.keylog {
        server_crypto.key_log = Arc::new(rustls::KeyLogFile::new());
    }

    let mut server_config =
        quinn::ServerConfig::with_crypto(Arc::new(QuicServerConfig::try_from(server_crypto)?));
    let transport_config = Arc::get_mut(&mut server_config.transport).unwrap();
    transport_config.max_concurrent_uni_streams(0_u8.into());

    let endpoint = quinn::Endpoint::server(server_config, options.listen)?;
    eprintln!("listening on {}", endpoint.local_addr()?);

    dotenv().ok();
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    let mut opt = ConnectOptions::new(database_url);
    opt.max_connections(10)
        .min_connections(2)
        .connect_timeout(Duration::from_secs(10))
        .idle_timeout(Duration::from_secs(300))
        .sqlx_logging(true);

    let db = Database::connect(opt).await?;
    let db = Arc::new(db);
    let server_context = Arc::new(ServerContext::new());

    while let Some(conn) = endpoint.accept().await {
        if options
            .connection_limit
            .is_some_and(|n| endpoint.open_connections() >= n)
        {
            info!("refusing due to open connection limit");
            conn.refuse();
        } else if Some(conn.remote_address()) == options.block {
            info!("refusing blocked client IP address");
            conn.refuse();
        } else if options.stateless_retry && !conn.remote_address_validated() {
            info!("requiring connection to validate its address");
            conn.retry().unwrap();
        } else {
            info!("accepting connection");
            let fut = handle_connection(conn, db.clone(), server_context.clone());
            tokio::spawn(async move {
                if let Err(e) = fut.await {
                    error!("connection failed: {reason}", reason = e.to_string())
                }
            });
        }
    }

    Ok(())
}

async fn handle_connection(conn: quinn::Incoming, db: Arc<DatabaseConnection>, server_context: Arc<ServerContext>) -> Result<()> {
    let connection = conn.await?;
    let span = info_span!(
        "connection",
        remote = %connection.remote_address(),
        protocol = %connection
            .handshake_data()
            .unwrap()
            .downcast::<quinn::crypto::rustls::HandshakeData>().unwrap()
            .protocol
            .map_or_else(|| "<none>".into(), |x| String::from_utf8_lossy(&x).into_owned())
    );

    async {
        
        info!("established");

        // Each stream initiated by the client constitutes a new request.
        loop {
            let stream = connection.accept_bi().await;
            let stream = match stream {
                Err(quinn::ConnectionError::ApplicationClosed { .. }) => {
                    info!("connection closed");
                    return Ok(());
                }
                Err(e) => {
                    return Err(e);
                }
                Ok(s) => s,
            };
            let fut = handle_messages(stream, db.clone(), server_context.clone());
            tokio::spawn(
                async move {
                    if let Err(e) = fut.await {
                        error!("failed: {reason}", reason = e.to_string());
                    }
                }
                .instrument(info_span!("request")),
            );
        }
    }
    .instrument(span)
    .await?;
    Ok(())
}

async fn handle_messages(
    (mut send, mut recv): (quinn::SendStream, quinn::RecvStream), db: Arc<DatabaseConnection>, server_context: Arc<ServerContext>
) -> Result<()> {
    let msg = read_message(&mut recv).await?;
    let remote_msg: RemoteMessage = postcard::from_bytes(&msg)?;
    // println!("got command {:?}",cmd);
    let (result_tx, mut result_rx) = mpsc::channel::<UIResult>(1);
    let mut response_channel = ResponseChannel::new(result_tx);
    let token_claims = if let Some(token) = remote_msg.token {
        // TODO handle invalid token, expired, or can not be deserialized
        let token_payload = verify_token(&token, &server_context.token_secret);
        if let Some(token_payload) = token_payload {
            let claims: TokenClaims = postcard::from_bytes(&token_payload)?;
            if is_token_expired(claims.exp) {
                // Token Expired
                None
            } else {
                Some(claims)
            }
        } else {
            // Token not valid
            None
        }
    } else {
        None
    };
    let call_context = MyCallContext {
        server_context: server_context,
        token_claims: token_claims,
    };
    server_core::handle_ui_command(remote_msg.command, &mut response_channel, &db, &call_context).await?;
    if let Some(result) = result_rx.recv().await {
        // println!("result {:?}", result);
        let out_msg = postcard::to_stdvec(&result)?;
        write_message(&mut send, &out_msg).await?;
    }       
    send.finish().unwrap();
    Ok(())
}

pub async fn write_message(
    stream: &mut quinn::SendStream,
    payload: &[u8],
) -> io::Result<()> {
    let len = payload.len() as u32;

    // Convert length to big-endian bytes
    let len_bytes = len.to_be_bytes();

    // Write length prefix
    stream.write_all(&len_bytes).await?;

    // Write payload
    stream.write_all(payload).await?;

    Ok(())
}

pub async fn read_message(
    stream: &mut quinn::RecvStream,
) -> Result<Vec<u8>> {
    // Read exactly 4 bytes for length
    let mut len_bytes = [0u8; 4];
    stream.read_exact(&mut len_bytes).await?;

    let len = u32::from_be_bytes(len_bytes) as usize;
    const MAX_SIZE: usize = 10 * 1024 * 1024; // 10 MB

    if len > MAX_SIZE {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "message too large").into());
    }

    // Allocate buffer for payload
    let mut buffer = vec![0u8; len];

    // Read payload
    stream.read_exact(&mut buffer).await?;

    Ok(buffer)
}

struct ServerContext {
    password_salt: String,
    token_secret: Vec<u8>,
}

impl ServerContext {
    fn new() -> Self {
        ServerContext { 
            password_salt: env::var("PASSWORD_SALT").expect("PASSWORD_SALT must be set"),
            token_secret: load_secret(env::var("TOKEN_SECRET").expect("TOKEN_SECRET must be set").as_str()),
        }
    }
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
        sign_payload(&payload, &self.server_context.token_secret)
    }

    
}