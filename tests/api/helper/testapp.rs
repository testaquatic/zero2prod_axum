use zero2prod_axum::domain::Settings;

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
}
