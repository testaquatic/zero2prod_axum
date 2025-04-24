use std::io;

use reqwest::{StatusCode, header};

/// 백그라운드에서 애플리케이션을 구동한다.  
///
/// 반환  
///     `String`은 서버의 주소이다.
async fn spawn_app() -> Result<String, io::Error> {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let port = listener.local_addr()?.port();
    let server = zero2prod_axum::run(listener);
    tokio::spawn(async move {
        server.await.expect("Failed to start server.");
    });

    Ok(format!("http://127.0.0.1:{port}"))
}

/// 멀티스레드 런타임을 사용하도록 설정해야 테스트를 통과한다.
/// [이 페이지](https://docs.rs/tokio/latest/tokio/attr.test.html)를 참고했다.
#[tokio::test(flavor = "multi_thread")]
async fn health_check_works() -> Result<(), anyhow::Error> {
    // 준비
    let address = spawn_app().await?;
    let client = reqwest::Client::new();

    // 실행
    let response = client.get(format!("{address}/health_check")).send().await?;

    // 확인
    assert!(response.status().is_success());
    pretty_assertions::assert_eq!(Some(0), response.content_length());

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn subscribe_returns_a_200_for_valid_form_data() -> Result<(), anyhow::Error> {
    // 준비
    let app_address = spawn_app().await?;
    let client = reqwest::Client::new();

    // 실행
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(&format!("{app_address}/subscriptions"))
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await?;

    // 확인
    pretty_assertions::assert_eq!(StatusCode::OK, response.status());

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn subscribe_returns_a_400_when_data_is_missing() -> Result<(), anyhow::Error> {
    // 준비
    let app_address = spawn_app().await?;
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
        pretty_assertions::assert_eq!(
            StatusCode::UNPROCESSABLE_ENTITY,
            response.status(),
            "The API did not fail with 400 Bad Request when the payload was {error_message}.",
        );
    }

    Ok(())
}
