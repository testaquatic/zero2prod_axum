use sea_orm::sqlx::postgres::PgPoolOptions;
use zero2prod_axum::{
    configuration::get_configuration,
    startup::run,
    telemetry::{get_subscriber, init_subscriber},
};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    // tracing 관련 초기화를 한다.
    let subscriber = get_subscriber("info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration");

    // https://www.sea-ql.org/sea-orm-cookbook/015-lazy-connection.html 이 문서를 참고로 했다.
    let pg_pool = PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(configuration.database.with_db());
    let connection_pool = sea_orm::SqlxPostgresConnector::from_sqlx_postgres_pool(pg_pool);

    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );
    let listener = tokio::net::TcpListener::bind(address).await?;
    run(listener, connection_pool).await
}
