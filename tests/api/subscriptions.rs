use crate::helpers::spawn_app;
use reqwest::{StatusCode, header};

/// 유효한 폼을 전송하면 200 OK를 반환해야 한다.
#[tokio::test(flavor = "multi_thread")]
async fn subscribe_returns_a_200_for_valid_form_data() -> Result<(), anyhow::Error> {
    // 준비
    let spawn_app = spawn_app().await?;
    let app = spawn_app;
    let client = reqwest::Client::new();

    // 실행
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(&format!("{}/subscriptions", app.address))
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await?;

    // 확인
    assert_eq!(StatusCode::OK, response.status());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_one(app.z_pgpool.pg_pool.as_ref())
        .await?;
    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");

    Ok(())
}

/// 폼중에 누락된 부분이 있다면 422 UNPROCESSABLE_ENTITY를 반환해야 한다.
/// actix와 axum의 차이이다.
#[tokio::test(flavor = "multi_thread")]
async fn subscribe_returns_a_422_when_data_is_missing() -> Result<(), anyhow::Error> {
    // 준비
    let app_address = spawn_app().await?.address;
    let url = format!("{app_address}/subscriptions");
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        // 실행
        let response = client
            .post(&url)
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await?;

        // 확인
        assert_eq!(
            StatusCode::UNPROCESSABLE_ENTITY,
            response.status(),
            "The API did not fail with 400 Bad Request when the payload was {error_message}.",
        );
    }

    Ok(())
}

/// 유효하지 않은 데이터를 입력한 폼에 대해서는 400 BAD_REQUEST를 반환해야 한다.
#[tokio::test(flavor = "multi_thread")]
async fn subscribe_returns_a_400_when_fields_are_present_but_invalid() -> Result<(), anyhow::Error>
{
    // 준비
    let app = spawn_app().await?;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=&email=ursula_le_guin%40gmail.com", "empty name"),
        ("name=Ursula&email=", "empty email"),
        ("name=Ursula&email=definitely-not-an-email", "invalid_email"),
    ];

    for (body, description) in test_cases {
        // 실행
        let response = client
            .post(&format!("{}/subscriptions", app.address))
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await?;

        // 확인
        assert_eq!(
            StatusCode::BAD_REQUEST,
            response.status(),
            "The API did not return a 200 OK when the payload was {description}",
        );
    }

    Ok(())
}
