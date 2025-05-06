use std::sync::LazyLock;

use pm_mock_server::{PMBody, PMMockServer};
use reqwest::{Url, header};
use sqlx::{Connection, Executor, PgConnection};
use tracing::Subscriber;
use zero2prod_axum::{
    configuration::{DatabaseSettings, get_configuration},
    database::ZPgPool,
    startup::{Application, get_z_pgpool},
    telemetry::{get_subscriber, init_subscriber},
};

// `LazyLock`을 사용해서 한번만 실행된다.
static TRACING: LazyLock<()> = LazyLock::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();

    // 트레이트 객체를 사용했다.
    // 실제 빌드에서는 사용하지 않을 것 같다.
    let subscriber: Box<dyn Subscriber + Send + Sync> = if std::env::var("TEST_LOG").is_ok() {
        Box::new(get_subscriber(
            subscriber_name,
            default_filter_level.into(),
            std::io::stdout,
        ))
    } else {
        Box::new(get_subscriber(
            subscriber_name,
            default_filter_level.into(),
            std::io::sink,
        ))
    };
    init_subscriber(subscriber);
});

/// 테스트 정보를 저장한다.
pub struct TestApp {
    /// 인스턴스 주소
    pub address: Url,
    /// 커넥션 풀
    pub z_pgpool: ZPgPool,
    /// 목서버
    pub email_server: PMMockServer,
}

/// 이메일 API에 대한 요청에 포함된 확인 링크
pub struct ConfirmationLinks {
    pub html: url::Url,
    pub plain_text: url::Url,
}

impl TestApp {
    pub async fn post_subscriptions(
        &self,
        body: String,
    ) -> Result<reqwest::Response, anyhow::Error> {
        let response = reqwest::Client::new()
            .post(self.address.join("subscriptions")?)
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await?;

        Ok(response)
    }

    /// 이메일 API에 대한 요청에 포함된 확인 링크를 추출한다.
    pub fn get_confirmation_links(
        &self,
        email_request: &PMBody,
    ) -> Result<ConfirmationLinks, anyhow::Error> {
        let get_link = |s: &str| {
            let links = linkify::LinkFinder::new()
                .links(s)
                .filter(|l| *l.kind() == linkify::LinkKind::Url)
                .collect::<Vec<_>>();
            assert_eq!(links.len(), 1);
            let raw_link = links[0].as_str();
            let mut confirmation_link = url::Url::parse(raw_link)?;
            // 외부로 요청이 나가지 않도록 확인한다.
            assert_eq!(confirmation_link.host_str(), Some("127.0.0.1"));
            confirmation_link
                .set_port(self.address.port())
                .map_err(|_| anyhow::anyhow!("Failed to set port"))?;

            Result::<Url, anyhow::Error>::Ok(confirmation_link)
        };

        let html = get_link(&email_request.html_body)?;
        let plain_text = get_link(&email_request.text_body)?;
        let confirmation_links = ConfirmationLinks { html, plain_text };

        Ok(confirmation_links)
    }
}

/// 백그라운드에서 애플리케이션을 구동한다.  
///
/// 반환  
///     `TestApp`
#[cfg(test)]
pub async fn spawn_app() -> Result<TestApp, anyhow::Error> {
    use std::str::FromStr;

    use pm_mock_server::PMMockServer;

    LazyLock::force(&TRACING);

    let mut configuration = get_configuration()?;
    configuration.database.database_name = uuid::Uuid::new_v4().to_string();
    configuration.application.port = 0;
    configure_database(&configuration.database).await?;

    // 이메일 서버 설정을 수정한다.
    let email_server = PMMockServer::start_server().await?;
    configuration.email_client.base_url = email_server.url().to_string();

    let application = Application::build(configuration.clone()).await?;
    let application_port = application.port()?;

    // 서버 인스턴스를 백그라운드에서 실행한다.
    let _ = tokio::spawn(application.run_until_stopped());
    let address = Url::from_str(&format!(
        "http://{}:{}",
        configuration.application.host, application_port
    ))?;

    Ok(TestApp {
        address,
        z_pgpool: get_z_pgpool(&configuration.database).await,
        email_server,
    })
}

/// 테스트용 데이터베이스를 생성하고 마이그레이션한다.
async fn configure_database(database_settings: &DatabaseSettings) -> Result<(), sqlx::Error> {
    // 데이터베이스 생성
    let mut connection = PgConnection::connect_with(&database_settings.without_db()).await?;
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, database_settings.database_name).as_str())
        .await?;

    // 데이터베이스 마이그레이션
    let z_pgpool = get_z_pgpool(database_settings).await;
    sqlx::migrate!("./migrations")
        .run(z_pgpool.as_ref())
        .await?;

    Ok(())
}
