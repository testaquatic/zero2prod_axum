use sqlx::FromRow;
use tokio::net::TcpListener;
use zero2prod_axum::{
    database::basic::Zero2ProdAxumDatabase,
    settings::{DefaultDBPool, Settings},
};

pub struct TestApp {
    pub settings: Settings,
}

/// 애플리케이션 인스턴스를 새로 실행하고 그 주소를 반환한다.
// 백그라운드에서 애플리케이션을 구동한다.
async fn spawn_app() -> TestApp {
    let mut settings = Settings::get_settings().expect("Failed to read settings");
    let tcp_listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind address to listener.");
    // OS가 할당한 포트 번호를 추출한다.
    // 임의의 포트가 할당되므로 설정을 변경한다.
    settings.application_port = tcp_listener.local_addr().unwrap().port();
    let pool = DefaultDBPool::connect(&settings.database).expect("Failed to connect to database.");
    let server = zero2prod_axum::startup::run(tcp_listener, pool);
    // 서버를 백그라운드로 구동한다.
    // tokio::spawn은 생성된 퓨처에 대한 핸들을 반환한다.
    // 하지만 여기에서는 사용하지 않으므로 let을 바인딩하지 않는다.
    let _ = tokio::spawn(server);
    // 애플리케이션 주소를 호출자에게 반환한다.
    TestApp { settings }
}

impl TestApp {
    fn address(&self) -> String {
        format!("127.0.0.1:{}", self.settings.application_port)
    }
    fn subscriptions_uri(&self) -> String {
        format!("http://{}/subscriptions", self.address())
    }
}

// `tokio::test`는 테스팅에 있어서 `tokio::main`과 동등하다.
// `#[test]` 속성을 지정하는 수고를 덜 수 있다.
//
// `cargo expand --test health_check`를 사용해서 코드가 무엇을 생성하는지 확인할 수 있다.
#[tokio::test]
async fn health_check_works() {
    // 준비
    let test_app = spawn_app().await;
    // `reqwest`를 사용해서 애플리케이션에 대한 HTTP 요청을 수행한다.
    let client = reqwest::Client::new();

    //실행
    let response = client
        .get(format!("http://{}/health_check", test_app.address()))
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
    let test_app = spawn_app().await;
    let pool = DefaultDBPool::connect(&test_app.settings.database)
        .expect("Failed to connect to Postgres.");
    let client = reqwest::Client::new();

    // 실행
    let response = client
        .post(test_app.subscriptions_uri())
        .header(
            reqwest::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");
    let row = pool
        .fetch_one("SELECT email, name FROM subscriptions;")
        .await
        .expect("Failed to fetch saved subscriptions.");

    // 확인
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    #[derive(sqlx::FromRow)]
    struct Saved {
        email: String,
        name: String,
    }
    let saved = Saved::from_row(&row).expect("Failed to get data from Database.");
    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin")
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
    let test_app = spawn_app().await;
    let client = reqwest::Client::new();

    for (invalid_body, error_messages) in test_cases {
        // 실행
        let response = client
            .post(test_app.subscriptions_uri())
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
