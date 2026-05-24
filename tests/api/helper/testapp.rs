use zero2prod_axum::configuration::Settings;

pub struct TestApp {
    pub configuration: Settings,
    _server_handle: tokio::task::JoinHandle<()>,
}

impl TestApp {
    pub fn new(configuration: Settings, server_handle: tokio::task::JoinHandle<()>) -> Self {
        Self {
            configuration,
            _server_handle: server_handle,
        }
    }

    pub fn app_address(&self) -> String {
        format!(
            "http://{}:{}",
            self.configuration.application.host, self.configuration.application.port
        )
    }
}
