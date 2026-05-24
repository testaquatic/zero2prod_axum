use reqwest::StatusCode;
use zero2prod_axum::{domain::new_subscriber::SubscribeFormData, startup::get_connection_pool};

use crate::helpers::startup::spawn_app;

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    let app = spawn_app().await;
    let db_pool = get_connection_pool(&app.configuration);

    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = app.post_subscriptions(body.into()).await;

    assert_eq!(response.status(), StatusCode::OK);

    let saved = sqlx::query_as!(SubscribeFormData, "SELECT email, name FROM subscriptions;")
        .fetch_one(&db_pool)
        .await
        .expect("failed to fetch saved subscription");

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
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
            "The API did not fail with 422 Unprocessable Entity when the payload was {}.",
            error_message
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
            "The API did not return a 400 Bad Request when the payload was {}.",
            description
        );
    }
}
