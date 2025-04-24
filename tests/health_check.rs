use std::io;

/// 백그라운드에서 애플리케이션을 구동한다.
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

    // 조작
    let response = client.get(format!("{address}/health_check")).send().await?;

    // 확인
    assert!(response.status().is_success());
    pretty_assertions::assert_eq!(Some(0), response.content_length());

    Ok(())
}
