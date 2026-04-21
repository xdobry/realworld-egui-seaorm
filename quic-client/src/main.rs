//! This example demonstrates an HTTP client that requests files from a server.
//!
//! Checkout the `README.md` for guidance.

use std::{
    fs, io::{self}, net::{SocketAddr, ToSocketAddrs}, path::PathBuf, sync::{Arc, RwLock}, thread
};

use anyhow::{Result, anyhow};
use app_core::api::{RemoteMessage, UIResult};
use clap::Parser;
use command_bus::{CommandBus, UITask};
use egui::{Context, ViewportBuilder};
use proto::crypto::rustls::QuicClientConfig;
use rustls::pki_types::CertificateDer;
use tokio::{runtime::Runtime, sync::mpsc};
use tracing::{error, info};
use ui::app::{FormsApp, SharedContext};
use url::Url;

mod common;

#[derive(Parser, Debug)]
#[clap(name = "client")]
struct Opt {
    /// Perform NSS-compatible TLS key logging to the file specified in `SSLKEYLOGFILE`.
    #[clap(long = "keylog")]
    keylog: bool,

    url: Url,

    /// Override hostname used for certificate verification
    #[clap(long = "host")]
    host: Option<String>,

    /// Custom certificate authority to trust, in DER format
    #[clap(long = "ca")]
    ca: Option<PathBuf>,

    /// Simulate NAT rebinding after connecting
    #[clap(long = "rebind")]
    rebind: bool,

    /// Address to bind on
    #[clap(long = "bind", default_value = "[::]:0")]
    bind: SocketAddr,
}

fn main() -> Result<(), eframe::Error> {
    tracing::subscriber::set_global_default(
        tracing_subscriber::FmtSubscriber::builder()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .finish(),
    )
    .unwrap();
    let opt = Opt::parse();
        let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default(),
        ..eframe::NativeOptions::default()
    };

    let (command_tx, mut command_rx) = mpsc::channel::<UITask>(5);

    eframe::run_native(
        "RealWorld App - Egui Quic Client",
        options,
        Box::new(|cc| {
            let egui_context = cc.egui_ctx.clone();
            let shared_context: SharedContext =  Arc::new(RwLock::new(None));
            let shared_context_clone = shared_context.clone();
            
            thread::spawn(move || {
                let rt = Runtime::new().unwrap();
                rt.block_on(async move {
                    // Example async task
                    let r = run(opt, &mut command_rx, egui_context, shared_context_clone).await;
                    if let Err(e) = r {
                        println!("error {:?}",e);
                    }
                });
            });

            let command_bus = CommandBus::new(command_tx);

            Ok(Box::new(FormsApp::new(cc.storage, command_bus, shared_context)))
        }),
    )
    
}

async fn run(options: Opt, commands: &mut mpsc::Receiver<UITask>, egui_context: Context, shared_context: SharedContext) -> Result<()> {
    let url = options.url;
    let url_host = strip_ipv6_brackets(url.host_str().unwrap());
    let remote = (url_host, url.port().unwrap_or(4433))
        .to_socket_addrs()?
        .next()
        .ok_or_else(|| anyhow!("couldn't resolve to an address"))?;

    let mut roots = rustls::RootCertStore::empty();
    if let Some(ca_path) = options.ca {
        roots.add(CertificateDer::from(fs::read(ca_path)?))?;
    } else {
        let dirs = directories_next::ProjectDirs::from("org", "quinn", "quinn-examples").unwrap();
        match fs::read(dirs.data_local_dir().join("cert.der")) {
            Ok(cert) => {
                roots.add(CertificateDer::from(cert))?;
            }
            Err(ref e) if e.kind() == io::ErrorKind::NotFound => {
                info!("local server certificate not found");
            }
            Err(e) => {
                error!("failed to open local server certificate: {}", e);
            }
        }
    }
    let mut client_crypto = rustls::ClientConfig::builder()
        .with_root_certificates(roots)
        .with_no_client_auth();

    client_crypto.alpn_protocols = common::ALPN_QUIC_HTTP.iter().map(|&x| x.into()).collect();
    if options.keylog {
        client_crypto.key_log = Arc::new(rustls::KeyLogFile::new());
    }

    let client_config =
        quinn::ClientConfig::new(Arc::new(QuicClientConfig::try_from(client_crypto)?));
    let mut endpoint = quinn::Endpoint::client(options.bind)?;
    endpoint.set_default_client_config(client_config);

    let rebind = options.rebind;
    let host = options.host.as_deref().unwrap_or(url_host);

    eprintln!("connecting to {host} at {remote}");
    let conn = endpoint
        .connect(remote, host)?
        .await
        .map_err(|e| anyhow!("failed to connect: {}", e))?;
    loop {
        let task = commands.recv().await;
        if let Some(mut task) = task {
            let remote_msg = RemoteMessage { 
                token: shared_context.read().unwrap().as_ref().map(|c| c.token.clone()),
                command: task.command,
            };
            let out_msg = postcard::to_stdvec(&remote_msg)?;
            let (mut send, mut recv) = conn
                .open_bi()
                .await
                .map_err(|e| anyhow!("failed to open stream: {}", e))?;
            if rebind {
                let socket = std::net::UdpSocket::bind("[::]:0").unwrap();
                let addr = socket.local_addr().unwrap();
                eprintln!("rebinding to {addr}");
                endpoint.rebind(socket).expect("rebind failed");
            }
            write_message(&mut send, &out_msg).await.map_err(|e| anyhow!("failed to send request: {}", e))?;
            send.finish().unwrap();
            let msg = read_message(&mut recv)
                .await
                .map_err(|e| anyhow!("failed to read response: {}", e))?;
            let response: UIResult = postcard::from_bytes(&msg)?;
            task.response.send(response);
            egui_context.request_repaint();
        } else {
            break;
        }
    }
    conn.close(0u32.into(), b"done");
    // Give the server a fair chance to receive the close packet
    endpoint.wait_idle().await;

    Ok(())
}

fn strip_ipv6_brackets(host: &str) -> &str {
    // An ipv6 url looks like eg https://[::1]:4433/Cargo.toml, wherein the host [::1] is the
    // ipv6 address ::1 wrapped in brackets, per RFC 2732. This strips those.
    if host.starts_with('[') && host.ends_with(']') {
        &host[1..host.len() - 1]
    } else {
        host
    }
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


#[cfg(test)]
mod tests {
    use app_core::dto::ChangeRecord;
    use uuid::Uuid;
     use sea_orm::prelude::DateTimeWithTimeZone;

    #[test]
    fn test_serialization() {
        use models::entity::articles;
        
        let now: DateTimeWithTimeZone = chrono::Local::now().with_timezone(chrono::Local::now().offset());
        let a1 = articles::Model {
            id: Uuid::new_v4(),
            slug: "slug".into(),
            title: "tilte".into(),
            description: "description".into(),
            body: "body".into(),
            author_id: Uuid::new_v4(),
            created_at: now,
            updated_at: now,
        };
        let mut a2 = a1.clone();
        a2.title = "title2".into();
        a2.slug = "s1".into();
        let change_record = ChangeRecord::from_models::<articles::Entity>(&a2, &a1);
        assert_eq!(2, change_record.changes.len());
        let out_msg = postcard::to_stdvec(&change_record).unwrap();
        let cr2: ChangeRecord = postcard::from_bytes(&out_msg).unwrap();
        assert_eq!(2, cr2.changes.len())
    }
}