use std::fmt;
use std::sync::atomic::AtomicUsize;

#[derive(Debug)]
struct Backend {
    address: String,
    healthy: bool,
}

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

#[derive(Debug)]
pub struct LoadBalancer {
    backends: Vec<Backend>,
    counter: AtomicUsize,
}

impl LoadBalancer {
    pub fn try_from_iter<'a, T>(iter: T) -> Result<LoadBalancer, LoadBalancerError>
    where
        T: IntoIterator<Item = &'a str>,
    {
        let backends = iter
            .into_iter()
            .map(|i| Backend {
                address: i.to_owned(),
                healthy: true,
            })
            .collect::<Vec<_>>();

        if backends.is_empty() {
            return Err(LoadBalancerError::EmptyBackends);
        }

        Ok(Self {
            backends,
            counter: AtomicUsize::new(0),
        })
    }

    pub fn next(&self) -> Option<String> {
        for _ in 0..self.backends.len() {
            let next = self
                .counter
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
                % self.backends.len();

            let backend = &self.backends[next];

            if backend.healthy {
                return Some(backend.address.clone());
            }
        }
        None
    }

    pub fn set_backend_health(&mut self, address: &str, healthy: bool) {
        if let Some(backend) = self.backends.iter_mut().find(|b| b.address == address) {
            backend.healthy = healthy;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{LoadBalancer, LoadBalancerError};

    #[test]
    fn returns_backends_in_round_robin_order() {
        let load_balancer =
            LoadBalancer::try_from_iter(["server-a", "server-b", "server-c"]).unwrap();

        assert_eq!(load_balancer.next().unwrap(), "server-a");
        assert_eq!(load_balancer.next().unwrap(), "server-b");
        assert_eq!(load_balancer.next().unwrap(), "server-c");
        assert_eq!(load_balancer.next().unwrap(), "server-a");
        assert_eq!(load_balancer.next().unwrap(), "server-b");
    }

    #[test]
    fn rejects_empty_backend_list() {
        let load_balancer = LoadBalancer::try_from_iter([]);

        assert!(matches!(
            load_balancer,
            Err(LoadBalancerError::EmptyBackends)
        ));
    }

    #[test]
    fn when_all_backends_are_unhealthy_next_returns_none() {
        let mut load_balancer = LoadBalancer::try_from_iter(["server-a", "server-b"]).unwrap();
        load_balancer.set_backend_health("server-a", false);
        load_balancer.set_backend_health("server-b", false);

        assert!(load_balancer.next().is_none());
    }

    #[test]
    fn dont_return_an_unhealthy_backend() {
        let mut load_balancer =
            LoadBalancer::try_from_iter(["server-a", "server-b", "server-c"]).unwrap();
        load_balancer.set_backend_health("server-b", false);

        dbg!(&load_balancer);

        assert_eq!(load_balancer.next(), Some("server-a".to_owned()));
        assert_eq!(load_balancer.next(), Some("server-c".to_owned()));
        assert_eq!(load_balancer.next(), Some("server-a".to_owned()));
        assert_eq!(load_balancer.next(), Some("server-c".to_owned()));
    }
}
