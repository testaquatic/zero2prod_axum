use crate::domain::{DatabaseSettings, Settings};

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let settings = config::Config::builder()
        .add_source(config::File::new(
            "configuration.yaml",
            config::FileFormat::Yaml,
        ))
        .build()?;

    settings.try_deserialize()
}

pub trait DatabaseSettingsExt {
    fn database_name(&self) -> &str;
    fn connection_string(&self) -> String {
        format!(
            "{}/{}",
            self.connection_string_without_db(),
            self.database_name()
        )
    }
    fn connection_string_without_db(&self) -> String;
}

impl DatabaseSettingsExt for DatabaseSettings {
    fn database_name(&self) -> &str {
        &self.database_name
    }

    fn connection_string_without_db(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}",
            self.username, self.password, self.host, self.port,
        )
    }
}
