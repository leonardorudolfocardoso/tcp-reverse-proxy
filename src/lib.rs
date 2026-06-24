use std::io;

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

mod load_balancer;

pub async fn proxy(mut client: TcpStream, server: &str) -> Result<u64, io::Error> {
    let mut upstream = TcpStream::connect(server).await?;

    let mut buffer = vec![0u8; 8192];
    let n = client.read(&mut buffer).await?;

    let request = String::from_utf8_lossy(&buffer[..n]);
    let request = request.replace("Host: 127.0.0.1:8080", format!("Host: {server}").as_str());

    upstream.write_all(request.as_bytes()).await?;

    let copied = tokio::io::copy(&mut upstream, &mut client).await?;

    Ok(copied)
}
