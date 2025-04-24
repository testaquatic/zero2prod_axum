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
    pub password: String,
    pub port: u16,
    pub host: String,
    pub database_name: String,
}

/// 애플리케이션 설정을 읽는다.
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
    pub fn connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database_name,
        )
    }

    pub fn connection_string_without_db(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}",
            self.username, self.password, self.host, self.port,
        )
    }
}
