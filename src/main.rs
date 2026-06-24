use std::sync::Arc;
use tcp_reverse_proxy::{ProxyError, load_balancer::LoadBalancer, proxy};
use tokio::{net::TcpListener, sync::Mutex};

type BoxedError = Box<dyn std::error::Error + Send + Sync>;

#[tokio::main(flavor = "current_thread")]
async fn main() -> std::result::Result<(), BoxedError> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    let load_balancer = Arc::new(Mutex::new(LoadBalancer::try_from_iter([
        "localhost:3001",
        "localhost:3002",
        "localhost:3003",
    ])?));

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
