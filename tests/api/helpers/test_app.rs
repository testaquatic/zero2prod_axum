use chrono::Utc;
use secrecy::{ExposeSecret, SecretString};
use uuid::Uuid;
use wiremock::MockServer;
use zero2prod_axum::{
    configuration::Settings, domain::credential::Claims,
    service::auth_token_service::AuthTokenService,
    startup::get_private_and_public_key_from_configuration,
};

use crate::helpers::{startup::post_login, test_user::TestUser};

pub struct TestApp {
    pub configuration: Settings,
    /// 이메일 서버를 모사한다.
    pub email_server: MockServer,
    pub test_user: TestUser,
    pub auth_token: String,
    pub api_client: reqwest::Client,
    pub _api_server_handle: tokio::task::JoinHandle<Result<(), std::io::Error>>,
}

impl TestApp {
    pub fn app_address(&self) -> String {
        format!(
            "http://{}:{}",
            self.configuration.application.host, self.configuration.application.port
        )
    }

    pub async fn post_subscriptions(&self, body: &impl serde::Serialize) -> reqwest::Response {
        self.api_client
            .post(&format!("{}/subscriptions", self.app_address()))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub fn get_confirmation_links(&self, email_request: &wiremock::Request) -> ConfirmationLinks {
        let body: serde_json::Value =
            serde_json::from_slice(&email_request.body).expect("failed to get json form request");

        let get_link = |s: &str| {
            let links: Vec<_> = linkify::LinkFinder::new()
                .links(s)
                .filter(|l| *l.kind() == linkify::LinkKind::Url)
                .collect();
            assert_eq!(
                links.len(),
                1,
                "Found {} links in email body but expected one: {}",
                links.len(),
                s
            );
            let raw_link = links[0].as_str();

            let mut confirmation_link = reqwest::Url::parse(raw_link).unwrap();
            assert_eq!(
                confirmation_link.host_str(),
                Some("127.0.0.1"),
                "email link does not point to localhost: {:?}",
                confirmation_link
            );
            confirmation_link
                .set_port(Some(self.configuration.application.port))
                .expect("Failed to set port");
            confirmation_link
        };

        let html = get_link(&body["HtmlBody"].as_str().unwrap());
        let plain_text = get_link(&body["TextBody"].as_str().unwrap());
        ConfirmationLinks { html, plain_text }
    }

    pub async fn post_newsletters(
        &self,
        body: serde_json::Value,
        token: &str,
    ) -> reqwest::Response {
        self.api_client
            .post(&format!("{}/newsletter", self.app_address()))
            .bearer_auth(token)
            .json(&body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_login<Body: serde::Serialize>(&self, body: &Body) -> reqwest::Response {
        post_login(&self.configuration, body).await
    }

    pub async fn get_admin_dashboard(&self, token: &str) -> reqwest::Response {
        self.api_client
            .get(&format!("{}/admin/dashboard", self.app_address()))
            .bearer_auth(token)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_change_password(
        &self,
        body: &impl serde::Serialize,
        token: &str,
    ) -> reqwest::Response {
        self.api_client
            .post(&format!("{}/admin/password", self.app_address()))
            .bearer_auth(token)
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn invalid_token(&self) -> SecretString {
        let claims = Claims {
            auth_id: Uuid::new_v4(),
            exp: (Utc::now() + chrono::Duration::hours(1)).timestamp(),
            iat: Utc::now().timestamp(),
        };

        let (private_key, public_key) =
            get_private_and_public_key_from_configuration(&self.configuration)
                .expect("failed to read key file");

        let invalid_token = AuthTokenService {
            token_secret_private_key: private_key,
            token_secret_public_key: public_key,
        }
        .generate_token(&claims)
        .await
        .expect("Failed to generate token");

        assert_ne!(invalid_token.expose_secret(), self.auth_token);

        invalid_token
    }

    pub async fn post_logout(&self, token: &str) -> reqwest::Response {
        self.api_client
            .post(&format!("{}/admin/logout", self.app_address()))
            .bearer_auth(token)
            .send()
            .await
            .expect("Failed to execute request.")
    }
}

/// 이메일 API에 대한 요청에 포함된 확인 링크
#[derive(Debug)]
pub struct ConfirmationLinks {
    pub html: reqwest::Url,
    pub plain_text: reqwest::Url,
}
