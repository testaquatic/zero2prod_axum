use secrecy::Secret;
use sqlx::Postgres;
use tokio::net::TcpListener;

use crate::database::postgres::postgrespool::PostgresPool;

pub type DefaultDBPool = PostgresPool;
pub type DefaultDB = Postgres;

#[derive(serde::Deserialize)]
pub struct Settings {
    pub application: ApplicationSettings,
    pub database: DatabaseSettings,
}

#[derive(serde::Deserialize)]
pub struct ApplicationSettings {
    pub port: u16,
    pub host: String,
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
        let base_path =
            std::env::current_dir().expect("Failed to determine the current directory.");
        let settings_directory = base_path.join("settings");

        // 실행 환경을 식별한다.
        // 지정되지 않았으면 `local`로 기본 설정한다.
        let environment: Envrionment = std::env::var("APP_ENVIRONMENT")
            .unwrap_or("local".into())
            .as_str()
            .try_into()
            .expect("Failed to parse APP_ENVIRONMENT.");
        let environment_filename = format!("{}.json", environment.as_str());

        // 구성 읽기를 초기화한다.
        let settings = config::Config::builder()
            // `configuration.json`부터 구성값을 추가한다.
            .add_source(config::File::from(settings_directory.join("base.json")))
            .add_source(config::File::from(
                settings_directory.join(&environment_filename),
            ))
            .build()?;
        // 읽은 구성값을 Settings 타입으로 변환한다.
        settings.try_deserialize::<Settings>()
    }
}

impl ApplicationSettings {
    pub fn get_address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    pub async fn get_listener(&self) -> Result<TcpListener, std::io::Error> {
        TcpListener::bind(self.get_address()).await
    }
}

/// 애플리케이션이 사용할 수 있는 런타임 환경
pub enum Envrionment {
    Local,
    Production,
}

impl Envrionment {
    fn as_str(&self) -> &'static str {
        match self {
            Envrionment::Local => "local",
            Envrionment::Production => "production",
        }
    }
}

impl TryFrom<&str> for Envrionment {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Production),
            other => Err(format!(
                "{} is not a supported environment. Use either `local` or `production`.",
                other
            )),
        }
    }
}
