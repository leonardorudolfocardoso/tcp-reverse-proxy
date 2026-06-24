use tcp_reverse_proxy::{LoadBalancer, proxy};
use tokio::net::TcpListener;

type BoxedError = Box<dyn std::error::Error + Send + Sync>;

#[tokio::main(flavor = "current_thread")]
async fn main() -> std::result::Result<(), BoxedError> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    let load_balancer =
        LoadBalancer::try_from_iter(["localhost:3001", "localhost:3002", "localhost:3003"])?;

    loop {
        let (client, _) = listener.accept().await?;
        let server = load_balancer.next();

        tokio::task::spawn(async move {
            if let Err(err) = proxy(client, &server).await {
                eprintln!("connection error: {err:?}");
            }
        });
    }
}
