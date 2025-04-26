use secrecy::{ExposeSecret, SecretString};

/// 애플리케이션 설정
#[derive(serde::Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    /// 포트
    pub application: ApplicationSettings,
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

// 애플리케이션 설정
#[derive(serde::Deserialize)]
pub struct ApplicationSettings {
    /// 포트
    pub port: u16,
    pub host: String,
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
        .build()?
        .try_deserialize()
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
