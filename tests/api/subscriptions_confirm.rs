use reqwest::{Method, StatusCode};
use wiremock::{Mock, ResponseTemplate, matchers};
use zero2prod_axum::startup::get_connection_pool;

use crate::helpers::startup::spawn_app;

#[tokio::test]
async fn confirmation_without_token_are_rejected_with_a_400() -> Result<(), anyhow::Error> {
    let app = spawn_app().await;
    let response = reqwest::get(&format!("{}/subscriptions/confirm", app.app_address())).await?;

    assert_eq!(
        response.status(),
        StatusCode::BAD_REQUEST,
        "expected 400 Bad Request: {:?}",
        response
    );

    Ok(())
}

#[tokio::test]
async fn the_link_returned_by_subscribe_returns_a_200_if_called() -> Result<(), anyhow::Error> {
    let app = spawn_app().await;
    let body = serde_json::json!({
        "name": "le guin",
        "email": "ursula_le_guin@gmail.com"
    });

    Mock::given(matchers::path("/email"))
        .and(matchers::method(Method::POST))
        .respond_with(ResponseTemplate::new(StatusCode::OK))
        .mount(&app.email_server)
        .await;

    app.post_subscriptions(&body).await;

    let email_requst = &app.email_server.received_requests().await.unwrap()[0];

    let confirmation_links = app.get_confirmation_links(email_requst);

    let response = reqwest::get(confirmation_links.html).await?;
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "expected 200 OK: {:?}",
        response
    );

    Ok(())
}

#[tokio::test]
async fn clicking_on_the_confirmation_link_confirms_a_subscriber() -> Result<(), anyhow::Error> {
    let app = spawn_app().await;
    let body = serde_json::json!({
        "name": "le guin",
        "email": "ursula_le_guin@gmail.com"
    });
    Mock::given(matchers::path("/email"))
        .and(matchers::method(Method::POST))
        .respond_with(ResponseTemplate::new(StatusCode::OK))
        .mount(&app.email_server)
        .await;

    app.post_subscriptions(&body).await;
    let email_requst = &app.email_server.received_requests().await.unwrap()[0];
    let confirmation_links = app.get_confirmation_links(email_requst);
    reqwest::get(confirmation_links.html)
        .await
        .expect("failed to execute request");

    let saved = sqlx::query!("SELECT email, name, status FROM subscriptions")
        .fetch_one(&get_connection_pool(&app.configuration))
        .await?;

    assert_eq!(
        saved.email, "ursula_le_guin@gmail.com",
        "did not store email correctly in database: {:?}",
        saved
    );
    assert_eq!(
        saved.name, "le guin",
        "did not store name correctly in database: {:?}",
        saved
    );
    assert_eq!(
        saved.status, "confirmed",
        "did not store status correctly in database: {:?}",
        saved
    );

    Ok(())
}
