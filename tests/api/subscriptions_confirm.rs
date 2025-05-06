use http::StatusCode;

use crate::helpers::spawn_app;

#[tokio::test(flavor = "multi_thread")]
async fn confirmations_without_tokens_are_rejected_with_a_400() -> Result<(), anyhow::Error> {
    // 준비
    let app = spawn_app().await?;

    // 실행
    let response = reqwest::get(app.address.join("subscriptions/confirm")?).await?;

    // 확인
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn the_line_returned_by_sbscribe_returns_a_200_if_called() -> Result<(), anyhow::Error> {
    // 준비
    let app = spawn_app().await?;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    // 실행
    app.post_subscriptions(body.to_string()).await?;
    let email_requests = app.email_server.recieved_reqeusts().await?;
    let email_request = &email_requests[0];
    let confirmation_link = app.get_confirmation_links(email_request)?;

    let response = reqwest::get(confirmation_link.html).await?;

    // 확인
    assert_eq!(response.status(), StatusCode::OK);

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn clicking_on_the_confirmation_link_confirms_a_subscriber() -> Result<(), anyhow::Error> {
    // 준비
    let app = spawn_app().await?;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    app.post_subscriptions(body.to_string()).await?;
    let email_requests = app.email_server.recieved_reqeusts().await?;
    let email_request = &email_requests[0];
    let confirmation_link = app.get_confirmation_links(&*email_request)?;

    // 실행
    reqwest::get(confirmation_link.html)
        .await?
        .error_for_status()?;

    // 확인
    let saved = sqlx::query!("SELECT email, name, status FROM subscriptions;")
        .fetch_one(app.z_pgpool.as_ref())
        .await?;

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
    assert_eq!(saved.status, "confirmed");

    Ok(())
}
