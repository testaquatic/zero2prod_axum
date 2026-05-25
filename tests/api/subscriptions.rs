use reqwest::{Method, StatusCode};
use wiremock::{Mock, ResponseTemplate, matchers};
use zero2prod_axum::startup::get_connection_pool;

use crate::helpers::startup::spawn_app;

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    Mock::given(matchers::path("/email"))
        .and(matchers::method(Method::POST))
        .respond_with(ResponseTemplate::new(StatusCode::OK))
        .expect(1)
        .mount(&app.email_server)
        .await;

    let response = app.post_subscriptions(body.into()).await;

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn subscribe_persists_the_new_subscriber() {
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    Mock::given(matchers::path("/email"))
        .and(matchers::method(Method::POST))
        .respond_with(ResponseTemplate::new(StatusCode::OK))
        .mount(&app.email_server)
        .await;

    app.post_subscriptions(body.into()).await;

    let db_pool = zero2prod_axum::startup::get_connection_pool(&app.configuration);
    let saved = sqlx::query!("SELECT email, name, status FROM subscriptions;")
        .fetch_one(&db_pool)
        .await
        .expect("failed to fetch saved subscription");

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
        saved.status, "pending_confirmation",
        "did not store status correctly in database: {:?}",
        saved
    );
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    let app = spawn_app().await;

    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = app.post_subscriptions(invalid_body.into()).await;

        assert_eq!(
            response.status(),
            StatusCode::UNPROCESSABLE_ENTITY,
            "The API did not fail with 422 Unprocessable Entity when the payload was {}: {:?}",
            error_message,
            response
        );
    }
}

#[tokio::test]
async fn subscribe_returns_a_400_when_fields_are_present_but_invalid() {
    let app = spawn_app().await;
    let test_cases = vec![
        ("name=&email=ursula_le_guin%40gmail.com", "empty name"),
        ("name=le%20guin&email=", "empty email"),
        (
            "name=le%20guin&email=definitedly_not_an_email",
            "invalid email",
        ),
    ];

    for (body, description) in test_cases {
        let response = app.post_subscriptions(body.into()).await;

        assert_eq!(
            response.status(),
            StatusCode::BAD_REQUEST,
            "The API did not return a 400 Bad Request when the payload was {}: {:?}",
            description,
            response
        );
    }
}

#[tokio::test]
async fn subscribe_sends_a_confirmation_email_for_valid_data() {
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    Mock::given(matchers::path("/email"))
        .and(matchers::method("POST"))
        .respond_with(ResponseTemplate::new(StatusCode::OK))
        .expect(1)
        .mount(&app.email_server)
        .await;

    app.post_subscriptions(body.into()).await;
}

#[tokio::test]
async fn subscribe_sends_a_confirmation_email_with_a_link() {
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    Mock::given(matchers::path("/email"))
        .and(matchers::method(Method::POST))
        .respond_with(ResponseTemplate::new(StatusCode::OK))
        .mount(&app.email_server)
        .await;

    app.post_subscriptions(body.into()).await;

    let email_request = &app.email_server.received_requests().await.unwrap()[0];

    let confirmation_links = app.get_confirmation_links(email_request);

    assert_eq!(
        confirmation_links.html, confirmation_links.plain_text,
        "html link and text link are different: {:?}",
        confirmation_links
    );
}

#[tokio::test]
async fn subscribe_fails_if_there_is_a_fatal_database_error() -> Result<(), sqlx::Error> {
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    sqlx::query!("ALTER TABLE subscription_tokens DROP COLUMN subscription_token;",)
        .execute(&get_connection_pool(&app.configuration))
        .await
        .unwrap();

    let response = app.post_subscriptions(body.into()).await;
    assert_eq!(
        response.status(),
        StatusCode::INTERNAL_SERVER_ERROR,
        "expected 500 Internal Server Error but got: {:?} instead",
        response
    );

    Ok(())
}
