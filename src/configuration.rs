use secrecy::{ExposeSecret, SecretString};

/// 애플리케이션 설정을 저장하는 구조체이다.
#[derive(serde::Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application_port: u16,
}

/// 데이터베이스 연결 정보를 포함한다.
#[derive(serde::Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: SecretString,
    pub host: String,
    pub port: u16,
    pub database_name: String,
}

/// 구성 파일(configuration.json5)을 읽어 Settings 구조체로 변환한다.
pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let settings = config::Config::builder()
        .add_source(config::File::new(
            "configuration.json5",
            config::FileFormat::Json5,
        ))
        .build()?;

    settings.try_deserialize()
}

impl DatabaseSettings {
    /// 데이터베이스 연결 문자열을 반환한다.
    /// 예: "postgres://username:password@host:port/database_name"
    pub fn connection_string(&self) -> SecretString {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port,
            self.database_name
        )
        .into()
    }

    /// 데이터베이스 연결 문자열에서 데이터베이스 이름을 제외한 부분을 반환한다.
    /// 예: "postgres://username:password@host:port"
    pub fn connection_string_without_db(&self) -> SecretString {
        format!(
            "postgres://{}:{}@{}:{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port
        )
        .into()
    }
}
