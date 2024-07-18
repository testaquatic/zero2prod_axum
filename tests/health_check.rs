use tokio::net::TcpListener;

// `tokio::test`는 테스팅에 있어서 `tokio::main`과 동등하다.
// `#[test]` 속성을 지정하는 수고를 덜 수 있다.
//
// `cargo expand --test health_check`를 사용해서 코드가 무엇을 생성하는지 확인할 수 있다.
#[tokio::test]
async fn health_check_works() {
    // 준비
    let address = spawn_app().await.expect("Failed to spawn our app.");
    // `reqwest`를 사용해서 애플리케이션에 대한 HTTP 요청을 수행한다.
    let client = reqwest::Client::new();

    //실행
    let response = client
        .get(format!("{}/health_check", &address))
        .send()
        .await
        .expect("Failed to execute request.");
    // 확인
    // 응답 상태 코드가 OK인지 확인한다.
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    // 응답 본문의 길이가 0인지 확인한다.
    assert_eq!(Some(0), response.content_length());
}

// 백그라운드에서 애플리케이션을 구동한다.
async fn spawn_app() -> Result<String, std::io::Error> {
    let tcp_listener = TcpListener::bind("127.0.0.1:0").await?;
    // OS가 할당한 포트 번호를 추출한다.
    let port = tcp_listener.local_addr().unwrap().port();
    let server = zero2prod_axum::run(tcp_listener);
    // 서버를 백그라운드로 구동한다.
    // tokio::spawn은 생성된 퓨처에 대한 핸들을 반환한다.
    // 하지만 여기에서는 사용하지 않으므로 let을 바인딩하지 않는다.
    let _ = tokio::spawn(server);
    // 애플리케이션 주소를 호출자에게 반환한다.
    Ok(format!("http://127.0.0.1:{}", port))
}
