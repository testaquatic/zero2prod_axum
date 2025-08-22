use sea_orm::sqlx::{
    ConnectOptions,
    postgres::{PgConnectOptions, PgSslMode},
};
use secrecy::{ExposeSecret, SecretString};
use serde_aux::field_attributes::deserialize_number_from_string;

use crate::domain::SubscriberEmail;

/// 설정을 저장하는 구조체이다.
#[derive(serde::Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application: ApplicationSettings,
    pub email_client: EmailClientSettings,
}

/// 애플리케이션 설정을 저장하는 구조체이다.
#[derive(serde::Deserialize)]
pub struct ApplicationSettings {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
    pub base_url: String,
    pub hmac_secret: SecretString,
}

/// 데이터베이스 연결 정보를 포함한다.
#[derive(serde::Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: SecretString,
    pub host: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub database_name: String,
    pub require_ssl: bool,
}

/// 이메일 클라이언트 설정을 저장하는 구조체이다.
#[derive(serde::Deserialize)]
pub struct EmailClientSettings {
    pub base_url: String,
    pub sender_email: String,
    pub authorization_token: SecretString,
    pub timeout_milliseconds: u64,
}

/// 구성 파일(configuration.json5)을 읽어 Settings 구조체로 변환한다.
pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let base_path = std::env::current_dir().expect("Failed to determine the current directory");
    // 설정 파일은 ./configuration/에 저장한다.
    let configuration_directory = base_path.join("configuration");
    let environment: Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or("local".into())
        .try_into()
        .expect("Failed to parse APP_ENVIRONMENT");
    let environment_filename = format!("{}.json5", environment.as_str());

    config::Config::builder()
        .add_source(config::File::from(
            configuration_directory.join("base.json5"),
        ))
        .add_source(config::File::from(
            configuration_directory.join(&environment_filename),
        ))
        // `APP_APPLICATION__PORT` => `Settings.application.port`
        .add_source(
            config::Environment::with_prefix("APP")
                .prefix_separator("_")
                .separator("__"),
        )
        .build()?
        .try_deserialize()
}

/// 런타임 환경을 나타낸다.
enum Environment {
    Local,
    Production,
}

impl Environment {
    /// Environment를 &'static str로 변환한다.
    fn as_str(&self) -> &'static str {
        match self {
            Environment::Local => "local",
            Environment::Production => "production",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    /// String을 Environment로 변환한다.
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Production),
            other => Err(format!(
                "{other} is not a supported environment. Use either `local` or `production`.",
            )),
        }
    }
}

impl DatabaseSettings {
    /// 데이터베이스 이름이 포함된 `PgConnectOptions`을 반환한다.
    pub fn with_db(&self) -> PgConnectOptions {
        self.without_db()
            .database(&self.database_name)
            .log_statements(tracing_log::log::LevelFilter::Trace)
    }

    /// 데이터베이스 이름을 제외한 `PgConnectOptions`를 반환한다.
    /// 테스트 함수의 초기화용으로 주로 사용한다.
    pub fn without_db(&self) -> PgConnectOptions {
        let ssl_mode = if self.require_ssl {
            PgSslMode::Require
        } else {
            PgSslMode::Prefer
        };

        PgConnectOptions::new()
            .host(&self.host)
            .port(self.port)
            .username(&self.username)
            .password(self.password.expose_secret())
            .ssl_mode(ssl_mode)
    }
}

impl EmailClientSettings {
    /// 발신자 주소를 반환한다.
    pub fn sender(&self) -> Result<SubscriberEmail, String> {
        SubscriberEmail::parse(self.sender_email.clone())
    }

    /// `EmailClientSettings`로부터 `std::time::Duration`을 생성한다.
    pub fn timeout(&self) -> std::time::Duration {
        std::time::Duration::from_millis(self.timeout_milliseconds)
    }
}
