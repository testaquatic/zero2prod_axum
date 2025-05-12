use crate::helpers::spawn_app;
use http::StatusCode;
use pretty_assertions::assert_eq;

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
    let email_requsts = app.email_server.get_all_request_infos().await?;
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
    let email_requests = app.email_server.recieved_reqeusts().await?;
    let email_request = &email_requests[0].requests.body;

    let confirmation_links = app.get_confirmation_links(email_request)?;
    assert_eq!(confirmation_links.html, confirmation_links.plain_text);

    Ok(())
}

/// 사용자가 구독을 두번 했을 때를 처리한다.
/// 매번 다른 토큰을 전송해야 한다.
#[tokio::test(flavor = "multi_thread")]
async fn subscribe_twice_will_receive_two_confirmation_emails() -> Result<(), anyhow::Error> {
    // 준비
    let app = spawn_app().await?;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    // 실행
    let responses = app.post_subscriptions_n(body.to_string(), 2).await?;

    // 확인
    assert_eq!(responses.len(), 2);
    assert_eq!(responses[0].status(), StatusCode::OK);
    assert_eq!(responses[1].status(), StatusCode::OK);

    let email_requests = app.email_server.recieved_reqeusts().await?;
    assert_eq!(email_requests.len(), 2);
    assert_ne!(
        email_requests[0].requests.body.html_body,
        email_requests[1].requests.body.html_body
    );

    Ok(())
}

/// 같은 이메일로 구독을 두번 요청 받았을 때 마지막 토큰만 데이터베이스에 저장되어야 한다.
#[tokio::test(flavor = "multi_thread")]
async fn receiving_two_subcription_from_same_email_only_last_token_stored()
-> Result<(), anyhow::Error> {
    // 준비
    let app = spawn_app().await?;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    // 실행
    app.post_subscriptions_n(body.to_string(), 2).await?;

    let last_send_body = app
        .email_server
        .get_all_request_infos()
        .await?
        .pop()
        .ok_or(anyhow::anyhow!("No email request found"))?
        .requests
        .body;
    let last_token = app
        .get_confirmation_links(&last_send_body)?
        .html
        .query_pairs()
        .find(|(q, _)| q == "subscription_token")
        .ok_or(anyhow::anyhow!("No token found"))?
        .1
        .to_string();
    let last_user_uuid = app
        .z_pgpool
        .get_subscriber_id_from_token(&last_token)
        .await?
        .ok_or(anyhow::anyhow!("No token found"))?;
    let uuid = sqlx::query!(
        "SELECT id FROM subscriptions WHERE email = $1;",
        "ursula_le_guin@gmail.com"
    )
    .fetch_one(app.z_pgpool.pg_pool.as_ref())
    .await?
    .id;

    // 확인
    assert_eq!(uuid, last_user_uuid);

    Ok(())
}
