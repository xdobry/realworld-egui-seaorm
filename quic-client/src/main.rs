//! This example demonstrates an HTTP client that requests files from a server.
//!
//! Checkout the `README.md` for guidance.

use std::{
    fs,
    io::{self},
    net::{SocketAddr, ToSocketAddrs},
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};

use anyhow::{Result, anyhow};
use app_core::{api::{UICommand, UIResult}, users::api::{UserCommand, UserResult}};
use clap::Parser;
use proto::crypto::rustls::QuicClientConfig;
use rustls::pki_types::CertificateDer;
use tracing::{error, info};
use url::Url;

mod common;

/// HTTP/0.9 over QUIC client
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

    let start = Instant::now();
    let rebind = options.rebind;
    let host = options.host.as_deref().unwrap_or(url_host);

    eprintln!("connecting to {host} at {remote}");
    let conn = endpoint
        .connect(remote, host)?
        .await
        .map_err(|e| anyhow!("failed to connect: {}", e))?;
    eprintln!("connected at {:?}", start.elapsed());
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

    let ui_command = UICommand::User(UserCommand::Reload);
    let out_msg = postcard::to_stdvec(&ui_command)?;
    write_message(&mut send, &out_msg).await.map_err(|e| anyhow!("failed to send request: {}", e))?;
    send.finish().unwrap();
    
    let response_start = Instant::now();
    eprintln!("request sent at {:?}", response_start - start);
    let msg = read_message(&mut recv)
        .await
        .map_err(|e| anyhow!("failed to read response: {}", e))?;
    let duration = response_start.elapsed();
    eprintln!(
        "response received in {:?} - {} KiB/s",
        duration,
        msg.len() as f32 / (duration_secs(&duration) * 1024.0)
    );
    let response: UIResult = postcard::from_bytes(&msg)?;
    match response {
        UIResult::User(UserResult::Users(users)) => {
            for user in users {
                println!("user {:?}", user);
            }
        },
        _ => {
            println!("unknown response");
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

fn duration_secs(x: &Duration) -> f32 {
    x.as_secs() as f32 + x.subsec_nanos() as f32 * 1e-9
}