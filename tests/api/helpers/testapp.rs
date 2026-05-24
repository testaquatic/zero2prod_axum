use reqwest::header;
use zero2prod_axum::configuration::Settings;

pub struct TestApp {
    pub configuration: Settings,
}

impl TestApp {
    pub fn app_address(&self) -> String {
        format!(
            "http://{}:{}",
            self.configuration.application.host, self.configuration.application.port
        )
    }

    pub async fn post_subscriptions(&self, body: String) -> reqwest::Response {
        reqwest::Client::new()
            .post(&format!("{}/subscriptions", self.app_address()))
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }
}
