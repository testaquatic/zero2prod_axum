use crate::helpers::spawn_app;
use reqwest::StatusCode;

/// 유효한 폼을 전송하면 200 OK를 반환해야 한다.
#[tokio::test(flavor = "multi_thread")]
async fn subscribe_returns_a_200_for_valid_form_data() -> Result<(), anyhow::Error> {
    // 준비
    let spawn_app = spawn_app().await?;
    let app = spawn_app;

    // 실행
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = app.post_subscriptions(body.to_string()).await?;

    // 확인
    assert_eq!(StatusCode::OK, response.status());

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn subscribe_persists_the_new_subscriber() -> Result<(), anyhow::Error> {
    // 준비
    let app = spawn_app().await?;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    // 실행
    app.post_subscriptions(body.to_string()).await?;

    // 확인
    let saved = sqlx::query!("SELECT email, name, status FROM subscriptions;")
        .fetch_one(app.z_pgpool.pg_pool.as_ref())
        .await?;
    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
    assert_eq!(saved.status, "pending_confirmation");

    Ok(())
}

/// 폼중에 누락된 부분이 있다면 422 UNPROCESSABLE_ENTITY를 반환해야 한다.
/// actix와 axum의 차이이다.
#[tokio::test(flavor = "multi_thread")]
async fn subscribe_returns_a_422_when_data_is_missing() -> Result<(), anyhow::Error> {
    // 준비
    let app = spawn_app().await?;
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        // 실행
        let response = app.post_subscriptions(invalid_body.to_string()).await?;

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
    let test_cases = vec![
        ("name=&email=ursula_le_guin%40gmail.com", "empty name"),
        ("name=Ursula&email=", "empty email"),
        ("name=Ursula&email=definitely-not-an-email", "invalid_email"),
    ];

    for (body, description) in test_cases {
        // 실행
        let response = app.post_subscriptions(body.to_string()).await?;

        // 확인
        assert_eq!(
            StatusCode::BAD_REQUEST,
            response.status(),
            "The API did not return a 200 OK when the payload was {description}",
        );
    }

    Ok(())
}

/// 이메일을 전송하는지 확인한다.
#[tokio::test(flavor = "multi_thread")]
async fn subscribe_sends_a_confirmation_email_for_valid_data() -> Result<(), anyhow::Error> {
    // 준비
    let app = spawn_app().await?;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    // 실행
    let response = app.post_subscriptions(body.to_string()).await?;
    assert_eq!(response.status(), StatusCode::OK);

    // 확인
    let email_requsts = app.email_server.get_all_requests_info().await?;
    assert_eq!(email_requsts.len(), 1);

    Ok(())
}

/// 이메일의 링크를 추출하고 확인한다.
#[tokio::test(flavor = "multi_thread")]
async fn subscribe_sends_a_confirmation_email_with_a_link() -> Result<(), anyhow::Error> {
    // 준비
    let app = spawn_app().await?;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    // 실행
    app.post_subscriptions(body.to_string()).await?;

    // 확인
    let email_requests = app.email_server.get_all_requests_info().await?;
    let email_request = email_requests.values().collect::<Vec<_>>()[0];

    let get_link = |s: &str| {
        let links = linkify::LinkFinder::new()
            .links(s)
            .filter(|l| *l.kind() == linkify::LinkKind::Url)
            .collect::<Vec<_>>();
        assert_eq!(links.len(), 1);
        links.get(0).unwrap().as_str().to_owned()
    };

    let html_link = get_link(&email_request.body.html_body);
    let plain_text_link = get_link(&email_request.body.text_body);

    assert_eq!(plain_text_link, html_link);

    Ok(())
}
