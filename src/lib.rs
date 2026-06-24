use std::fmt;
use std::io;
use std::sync::atomic::AtomicUsize;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadBalancerError {
    EmptyBackends,
}

impl fmt::Display for LoadBalancerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyBackends => write!(f, "load balancer requires at least one backend"),
        }
    }
}

impl std::error::Error for LoadBalancerError {}

pub struct LoadBalancer {
    backends: Vec<String>,
    counter: AtomicUsize,
}

impl LoadBalancer {
    pub fn try_from_iter<'a, T>(iter: T) -> Result<LoadBalancer, LoadBalancerError>
    where
        T: IntoIterator<Item = &'a str>,
    {
        let backends = iter.into_iter().map(|i| i.to_owned()).collect::<Vec<_>>();

        if backends.is_empty() {
            return Err(LoadBalancerError::EmptyBackends);
        }

        Ok(Self {
            backends,
            counter: AtomicUsize::new(0),
        })
    }

    pub fn next(&self) -> String {
        let index = self
            .counter
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            % self.backends.len();
        self.backends[index].clone()
    }
}

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

#[cfg(test)]
mod tests {
    use super::{LoadBalancer, LoadBalancerError};

    #[test]
    fn returns_backends_in_round_robin_order() {
        let load_balancer =
            LoadBalancer::try_from_iter(["server-a", "server-b", "server-c"]).unwrap();

        assert_eq!(load_balancer.next(), "server-a");
        assert_eq!(load_balancer.next(), "server-b");
        assert_eq!(load_balancer.next(), "server-c");
        assert_eq!(load_balancer.next(), "server-a");
        assert_eq!(load_balancer.next(), "server-b");
    }

    #[test]
    fn rejects_empty_backend_list() {
        let load_balancer = LoadBalancer::try_from_iter([]);

        assert!(matches!(
            load_balancer,
            Err(LoadBalancerError::EmptyBackends)
        ));
    }
}
