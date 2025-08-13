use crate::helpers::spawn_app;

/// /health_check에 GET요청을 보내면 200 OK를 반환해야 한다.
/// 응답에는 바디가 없다.
#[tokio::test]
async fn health_check_works() {
    // 준비
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    // 실행
    let response = client
        .get(&format!("{}/health_check", app.address.as_str()))
        .send()
        .await
        .expect("Failed to send request.");

    // 확인
    // 응답 상태 코드가 200 OK인지 확인
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    // 응답 바디가 비어 있는지 확인
    assert_eq!(response.content_length(), Some(0));
}
