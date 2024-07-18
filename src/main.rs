use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let tcp_listener = TcpListener::bind("127.0.0.1:8000").await?;
    zero2prod_axum::startup::run(tcp_listener).await
}
