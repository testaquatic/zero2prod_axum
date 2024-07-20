use secrecy::Secret;
use sqlx::Postgres;

use crate::database::postgres::postgrespool::PostgresPool;

pub type DefaultDBPool = PostgresPool;
pub type DefaultDB = Postgres;

#[derive(serde::Deserialize)]
pub struct Settings {
    pub application_port: u16,
    pub database: DatabaseSettings,
}

#[derive(serde::Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: Secret<String>,
    pub port: u16,
    pub host: String,
    pub database_name: String,
}

impl Settings {
    pub fn get_settings() -> Result<Self, config::ConfigError> {
        // 구성 읽기를 초기화한다.
        let settings = config::Config::builder()
            // `configuration.json`부터 구성값을 추가한다.
            .add_source(config::File::new("settings.json", config::FileFormat::Json))
            .build()?;
        // 읽은 구성값을 Settings 타입으로 변환한다.
        settings.try_deserialize::<Settings>()
    }
}
