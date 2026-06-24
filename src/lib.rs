use std::io;

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

pub mod load_balancer;

#[derive(Debug)]
pub enum ProxyError {
    ConnectUpstream(io::Error),
    ReadClient(io::Error),
    WriteUpstream(io::Error),
    CopyResponse(io::Error),
}

pub async fn proxy(mut client: TcpStream, server: &str) -> Result<u64, ProxyError> {
    let mut upstream = TcpStream::connect(server)
        .await
        .map_err(ProxyError::ConnectUpstream)?;

    let mut buffer = vec![0u8; 8192];
    let n = client
        .read(&mut buffer)
        .await
        .map_err(ProxyError::ReadClient)?;

    let request = String::from_utf8_lossy(&buffer[..n]);
    let request = request.replace("Host: 127.0.0.1:8080", format!("Host: {server}").as_str());

    upstream
        .write_all(request.as_bytes())
        .await
        .map_err(ProxyError::WriteUpstream)?;

    let copied = tokio::io::copy(&mut upstream, &mut client)
        .await
        .map_err(ProxyError::CopyResponse)?;

    Ok(copied)
}
