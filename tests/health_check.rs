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

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    // 테스트 데이터
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    // 준비
    let app_address = spawn_app().await.expect("Failed to spawn our app.");
    let client = reqwest::Client::new();

    // 실행
    let response = client
        .post(format!("{}/subscriptions", &app_address))
        .header(
            reqwest::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    // 확인
    assert_eq!(response.status(), reqwest::StatusCode::OK);
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    // 테스트 데이터
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];
    // 준비
    let app_address = spawn_app().await.unwrap();
    let client = reqwest::Client::new();

    for (invalid_body, error_messages) in test_cases {
        // 실행
        let response = client
            .post(format!("{}/subscriptions", &app_address))
            .header(
                reqwest::header::CONTENT_TYPE,
                "application/x-www-form-urlencoded",
            )
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request.");

        // 확인
        assert_eq!(
            response.status(),
            reqwest::StatusCode::UNPROCESSABLE_ENTITY,
            // 테스트 실패시 출력할 커스터마이즈된 추가 오류 메세지
            "The API did not fail with 400 Bad Request when the payload was {},",
            error_messages,
        );
    }
}

/// 애플리케이션 인스턴스를 새로 실행하고 그 주소를 반환한다.
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
