use fake::{
    Fake,
    faker::{internet::en::SafeEmail, name::en::Name},
};
use reqwest::{Method, StatusCode};
use wiremock::{Mock, ResponseTemplate, matchers};

use crate::helpers::test_app::{ConfirmationLinks, TestApp};

/// 테스트 대상 애플리케이션의 퍼블릭 API를 사용해서 확인되지 않은 구독자를 생성한다.
pub async fn create_unconfirmed_subscriber(
    app: &TestApp,
) -> Result<ConfirmationLinks, anyhow::Error> {
    let body = serde_json::json!({
        "name": Name().fake::<String>(),
        "email": SafeEmail().fake::<String>(),
    });

    let _mock_guard = Mock::given(matchers::path("/email"))
        .and(matchers::method(Method::POST))
        .respond_with(ResponseTemplate::new(StatusCode::OK))
        .named("Create unconfirmed subscriber")
        .expect(1)
        .mount_as_scoped(&app.email_server)
        .await;

    app.post_subscriptions(&body).await.error_for_status()?;

    let email_request = &app
        .email_server
        .received_requests()
        .await
        .unwrap()
        .pop()
        .unwrap();
    let confirmation_links = app.get_confirmation_links(email_request);

    Ok(confirmation_links)
}

pub async fn create_confirmed_subscriber(app: &TestApp) -> Result<(), anyhow::Error> {
    let confirmation_link = create_unconfirmed_subscriber(app).await?;
    app.api_client
        .get(confirmation_link.html)
        .send()
        .await?
        .error_for_status()?;

    Ok(())
}
