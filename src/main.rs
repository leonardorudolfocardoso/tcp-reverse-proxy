use std::io;
use std::sync::Arc;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_rustls::TlsConnector;
use tokio_rustls::rustls::pki_types::InvalidDnsNameError;
use tokio_rustls::rustls::{ClientConfig, RootCertStore, pki_types::ServerName};

type BoxedError = Box<dyn std::error::Error + Send + Sync>;

#[derive(Debug)]
enum HandleError {
    Io(io::Error),
    Dns(InvalidDnsNameError),
}

impl From<io::Error> for HandleError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}
impl From<InvalidDnsNameError> for HandleError {
    fn from(value: InvalidDnsNameError) -> Self {
        Self::Dns(value)
    }
}

async fn handle(mut client: TcpStream, connector: TlsConnector) -> Result<u64, HandleError> {
    let upstream = TcpStream::connect("example.com:443").await?;

    let server_name = ServerName::try_from("example.com")?;
    let mut upstream_tls = connector.connect(server_name, upstream).await?;

    let mut buffer = vec![0u8; 8192];
    let n = client.read(&mut buffer).await?;

    let request = String::from_utf8_lossy(&buffer[..n]);
    let request = request.replace("Host: 127.0.0.1:8080", "Host: example.com");

    upstream_tls.write_all(request.as_bytes()).await?;

    let copied = tokio::io::copy(&mut upstream_tls, &mut client).await?;

    Ok(copied)
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> std::result::Result<(), BoxedError> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;

    let mut roots = RootCertStore::empty();
    roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

    let config = ClientConfig::builder()
        .with_root_certificates(roots)
        .with_no_client_auth();

    let connector = TlsConnector::from(Arc::new(config));

    loop {
        let (client, _) = listener.accept().await?;
        let connector = connector.clone();

        tokio::task::spawn(async move {
            if let Err(err) = handle(client, connector).await {
                eprintln!("connection error: {err:?}");
            }
        });
    }
}
