use crate::helper::spawn_app;

/// /health_check가 작동하는지 확인한다.
#[tokio::test]
async fn health_check_works() {
    let _server_handle = spawn_app().await.expect("failed to spawn app");

    let client = reqwest::Client::new();
    let response = client
        .get("http://127.0.0.1:8000/health_check")
        .send()
        .await
        .expect("failed to execute request");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}
