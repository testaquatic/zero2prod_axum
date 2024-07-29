use crate::helpers::TestApp;

// `tokio::test`는 테스팅에 있어서 `tokio::main`과 동등하다.
// `#[test]` 속성을 지정하는 수고를 덜 수 있다.
//
// `cargo expand --test health_check`를 사용해서 코드가 무엇을 생성하는지 확인할 수 있다.
#[tokio::test]
async fn health_check_works() -> Result<(), anyhow::Error> {
    // 준비
    let test_app = TestApp::spawn_app().await?;
    // `reqwest`를 사용해서 애플리케이션에 대한 HTTP 요청을 수행한다.
    let client = reqwest::Client::new();

    //실행
    let response = client
        .get(test_app.uri()?.join("health_check")?)
        .send()
        .await?;
    // 확인
    // 응답 상태 코드가 OK인지 확인한다.
    assert_eq!(response.status(), http::StatusCode::OK);
    // 응답 본문의 길이가 0인지 확인한다.
    assert_eq!(Some(0), response.content_length());

    Ok(())
}
