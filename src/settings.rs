use secrecy::{ExposeSecret, Secret};
use serde_aux::prelude::deserialize_number_from_string;
use sqlx::postgres::{PgConnectOptions, PgSslMode};
use tokio::net::TcpListener;

use crate::{
    database::{postgres::PostgresPool, Z2PADBError},
    domain::{InvalidNewSubscriber, SubscriberEmail},
    email_client::{EmailClientError, Postmark},
    startup::Server,
};

#[derive(serde::Deserialize)]
pub struct Settings {
    pub application: ApplicationSettings,
    pub database: DatabaseSettings,
    pub email_client: EmailClientSettings,
}

#[derive(serde::Deserialize)]
pub struct ApplicationSettings {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
    pub base_url: String,
    pub hmac_secret: Secret<String>,
}

#[derive(serde::Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: Secret<String>,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
    pub database_name: String,
    // 커넥션의 암호화 요청 여부를 결정한다.
    pub require_ssl: bool,
}

#[derive(serde::Deserialize)]
pub struct EmailClientSettings {
    pub base_url: String,
    pub sender_email: String,
    pub authorization_token: Secret<String>,
    pub timeout_milliseconds: u64,
}

/// 애플리케이션이 사용할 수 있는 런타임 환경
pub enum Envrionment {
    Local,
    Production,
}

// `Settings`를 이용해서 필요한 타입을 생성한다.
// 복잡함을 피하기 위해서 `get_settings`를 제외하고는 되도록이면 래퍼 함수로 작성한다.
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
                settings_directory.join(environment_filename),
            ))
            // 환경 변수로부터 설정에 추가한다.
            // `APP_APPLICATION__PORT=5001` => `Settings.application.port`
            .add_source(
                config::Environment::with_prefix("APP")
                    .prefix_separator("_")
                    .separator("__"),
            )
            .build()?;
        // 읽은 구성값을 Settings 타입으로 변환한다.
        settings.try_deserialize::<Settings>()
    }

    pub async fn build_server(&self) -> Result<Server, anyhow::Error> {
        Server::build(self).await
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

impl DatabaseSettings {
    pub fn connect_options_without_db(&self) -> PgConnectOptions {
        let ssl_mod = if self.require_ssl {
            PgSslMode::Require
        } else {
            // 암호화된 커넥션을 시도한다.
            // 실패하면 암호화하지 않은 커넥션을 사용한다.
            PgSslMode::Prefer
        };
        PgConnectOptions::new()
            .username(&self.username)
            .password(self.password.expose_secret())
            .host(&self.host)
            .port(self.port)
            .ssl_mode(ssl_mod)
    }
    pub fn connect_options_with_db(&self) -> PgConnectOptions {
        self.connect_options_without_db()
            .database(&self.database_name)
        // ``.log_statements`은 대한 부분은 저자의 예시 코드에도 보이지 않는다.
        // https://github.com/LukeMathWalker/zero-to-production/blob/root-chapter-05/src/configuration.rs
        // 노이즈를 줄이려고 INFO를 TRACE로 변경하는 것이 이해가 되지 않는다.
    }

    pub async fn get_pool(&self) -> Result<PostgresPool, Z2PADBError> {
        PostgresPool::connect(self)
    }
}

impl EmailClientSettings {
    pub fn get_sender_email(&self) -> Result<SubscriberEmail, InvalidNewSubscriber> {
        SubscriberEmail::try_from(self.sender_email.clone())
    }

    pub fn get_email_client(&self) -> Result<Postmark, EmailClientError> {
        Postmark::from_email_client_settings(self)
    }
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
