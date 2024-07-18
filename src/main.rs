use tokio::net::TcpListener;
use zero2prod_axum::run;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let tcp_listener = TcpListener::bind("127.0.0.1:8000").await?;
    run(tcp_listener).await
}
