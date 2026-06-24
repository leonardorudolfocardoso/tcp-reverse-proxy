use std::io;
use std::sync::atomic::AtomicUsize;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

type BoxedError = Box<dyn std::error::Error + Send + Sync>;

async fn handle(mut client: TcpStream, server: &str) -> Result<u64, io::Error> {
    let mut upstream = TcpStream::connect(server).await?;

    let mut buffer = vec![0u8; 8192];
    let n = client.read(&mut buffer).await?;

    let request = String::from_utf8_lossy(&buffer[..n]);
    let request = request.replace("Host: 127.0.0.1:8080", format!("Host: {server}").as_str());

    upstream.write_all(request.as_bytes()).await?;

    let copied = tokio::io::copy(&mut upstream, &mut client).await?;

    Ok(copied)
}

struct LoadBalancer {
    backends: Vec<String>,
    counter: AtomicUsize,
}

impl LoadBalancer {
    fn new(backends: Vec<String>) -> LoadBalancer {
        LoadBalancer {
            backends,
            counter: AtomicUsize::new(0),
        }
    }
    fn next(&self) -> String {
        let index = self
            .counter
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            % self.backends.len();
        self.backends[index].clone()
    }
}

impl<'a> FromIterator<&'a str> for LoadBalancer {
    fn from_iter<T: IntoIterator<Item = &'a str>>(iter: T) -> Self {
        Self::new(iter.into_iter().map(|i| i.to_owned()).collect())
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> std::result::Result<(), BoxedError> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;

    let lb = LoadBalancer::from_iter(["localhost:3001", "localhost:3002", "localhost:3003"]);

    loop {
        let (client, _) = listener.accept().await?;
        let server = lb.next();

        tokio::task::spawn(async move {
            if let Err(err) = handle(client, &server).await {
                eprintln!("connection error: {err:?}");
            }
        });
    }
}
