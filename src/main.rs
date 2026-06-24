use std::{sync::Arc, time::Duration};
use tcp_reverse_proxy::{ProxyError, load_balancer::LoadBalancer, proxy};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::Mutex,
};

type BoxedError = Box<dyn std::error::Error + Send + Sync>;

fn recover_unhealthy_backends(
    load_balancer: Arc<Mutex<LoadBalancer>>,
) -> tokio::task::JoinHandle<()> {
    tokio::task::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(5)).await;

            let unhealthy_addresses: Vec<String> = {
                load_balancer
                    .lock()
                    .await
                    .unhealthy_addresses()
                    .map(str::to_owned)
                    .collect()
            };

            for address in unhealthy_addresses {
                if TcpStream::connect(&address).await.is_ok() {
                    load_balancer
                        .lock()
                        .await
                        .set_backend_health(&address, true);
                    eprintln!("backend {address} is healthy again");
                }
            }
        }
    })
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> std::result::Result<(), BoxedError> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    let load_balancer = Arc::new(Mutex::new(LoadBalancer::try_from_iter([
        "localhost:3001",
        "localhost:3002",
        "localhost:3003",
    ])?));

    recover_unhealthy_backends(load_balancer.clone());

    loop {
        let (client, _) = listener.accept().await?;
        let load_balancer = load_balancer.clone();

        tokio::task::spawn(async move {
            let server = {
                let load_balancer = load_balancer.lock().await;
                load_balancer.next()
            };
            let Some(server) = server else {
                eprintln!("no healthy backend available");
                return;
            };

            match proxy(client, &server).await {
                Ok(_) => {}
                Err(ProxyError::ConnectUpstream(err)) => {
                    eprintln!("backend {server} is unhealthy: {err:?}");
                    load_balancer
                        .lock()
                        .await
                        .set_backend_health(&server, false);
                }
                Err(err) => eprintln!("connection error: {err:?}"),
            };
        });
    }
}
