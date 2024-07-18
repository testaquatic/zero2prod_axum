use tokio::net::TcpListener;
use zero2prod_axum::{
    database::{basic::Zero2ProdAxumDatabase, postgres::postgrespool::PostgresPool},
    settings::Settings,
};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    // 구성을 읽을 수 없으면 패닉에 빠진다.
    let settings = Settings::get_settings().expect("Failed to read configuration.");
    // 하드 코딩했던 `8000`을 제거한다.
    // 해당 값은 세팅에서 얻는다.
    let tcp_listener =
        TcpListener::bind(format!("127.0.0.1:{}", settings.application_port)).await?;
    let pool = PostgresPool::connect(&settings.database).expect("Failed to connect to Postgres.");
    zero2prod_axum::startup::run(tcp_listener, pool).await
}
