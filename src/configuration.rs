use secrecy::{ExposeSecret, SecretString};

/// 애플리케이션 설정
#[derive(serde::Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    /// 포트
    pub application_port: u16,
}

/// 데이터베이스 연결 파라미터
#[derive(serde::Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: SecretString,
    pub port: u16,
    pub host: String,
    pub database_name: String,
}

/// 애플리케이션 설정을 읽는다.
/// 설정은 JSON5이다.
pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    config::Config::builder()
        .add_source(config::File::new(
            "configuration.json5",
            config::FileFormat::Json5,
        ))
        .build()?
        .try_deserialize()
}

impl DatabaseSettings {
    /// postgres://username:password@host:port/db_name 형식의 문자열을 얻는다.
    pub fn connection_string(&self) -> SecretString {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port,
            self.database_name,
        )
        .into()
    }

    /// postgres://username:password@host:port 형식의 문자열을 얻는다.
    pub fn connection_string_without_db(&self) -> SecretString {
        format!(
            "postgres://{}:{}@{}:{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port,
        )
        .into()
    }
}
