use secrecy::{ExposeSecret, SecretString};
use serde_aux::prelude::deserialize_number_from_string;
use sqlx::{
    ConnectOptions,
    postgres::{PgConnectOptions, PgSslMode},
};

/// 애플리케이션 설정
#[derive(serde::Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    /// 포트
    pub application: ApplicationSettings,
}

/// 애플리케이션 설정을 읽는다.
/// 설정은 JSON5이다.
pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let base_path = std::env::current_dir().expect("Failed to determine the current directory.");
    let configuration_directory = base_path.join("configuration");

    let environment: Environment = std::env::var("APP_ENVIRONMENT")
        // APP_ENVIRONMENT가 환경변수에 없으면 local.json5를 읽는다.
        .unwrap_or("local".into())
        .try_into()
        .expect("Failed to parse APP_ENVIRONMENT.");

    let environment_filename = format!("{}.json5", environment.as_str());

    config::Config::builder()
        .add_source(config::File::from(
            configuration_directory.join("base.json5"),
        ))
        .add_source(config::File::from(
            configuration_directory.join(&environment_filename),
        ))
        .add_source(
            config::Environment::with_prefix("APP")
                .prefix_separator("_")
                .separator("__"),
        )
        .build()?
        .try_deserialize()
}

// 애플리케이션 설정
#[derive(serde::Deserialize)]
pub struct ApplicationSettings {
    /// 포트
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
}

/// 데이터베이스 연결 파라미터
#[derive(serde::Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: SecretString,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
    pub database_name: String,
    // 암호화
    pub require_ssl: bool,
}

impl DatabaseSettings {
    /// postgres://username:password@host:port/db_name 형식의 문자열을 얻는다.
    pub fn with_db(&self) -> PgConnectOptions {
        self.without_db()
            .database(&self.database_name)
            .log_statements(tracing_log::log::LevelFilter::Trace)
    }

    /// postgres://username:password@host:port 형식의 문자열을 얻는다.
    pub fn without_db(&self) -> PgConnectOptions {
        let ssl_mode = if self.require_ssl {
            PgSslMode::Require
        } else {
            PgSslMode::Prefer
        };

        PgConnectOptions::new()
            .host(&self.host)
            .username(&self.username)
            .password(&self.password.expose_secret())
            .port(self.port)
            .ssl_mode(ssl_mode)
    }
}

/// 애플리케이션이 사용할 수 있는 런타임 환경
enum Environment {
    Local,
    Production,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Local => "local",
            Environment::Production => "production",
        }
    }
}

impl TryFrom<&str> for Environment {
    type Error = String;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Production),
            other => Err(format!(
                "{} is not a supported environment. Use either `local` or `production`.",
                other,
            )),
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        s.as_str().try_into()
    }
}
