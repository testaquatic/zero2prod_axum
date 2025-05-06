use http::StatusCode;
use reqwest::Url;

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

    let get_link = |s: &str| {
        let links = linkify::LinkFinder::new()
            .links(s)
            .filter(|l| *l.kind() == linkify::LinkKind::Url)
            .collect::<Vec<_>>();
        assert_eq!(links.len(), 1);
        links.get(0).unwrap().as_str().to_owned()
    };
    let raw_confirmation_link = &get_link(&email_request.html_body);
    let mut confirmation_link = Url::parse(&raw_confirmation_link)?;
    // 외부에 API를 호출하지 않는다.
    assert_eq!(
        confirmation_link
            .host_str()
            .expect("confirmation_link host_str is none!"),
        "127.0.0.1"
    );
    confirmation_link
        .set_port(app.address.port())
        .expect("cannot set port");
    let response = reqwest::get(confirmation_link).await?;

    // 확인
    assert_eq!(response.status(), StatusCode::OK);

    Ok(())
}
