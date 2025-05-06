use crate::helpers::spawn_app;

/// 멀티스레드 런타임을 사용하도록 설정해야 테스트를 통과한다.  
/// [이 페이지](https://docs.rs/tokio/latest/tokio/attr.test.html)를 참고했다.
/// /health_check 엔드포인트를 테스트한다.
/// 요청은 항상 성공해야 하고, 바디는 비어 있어야 한다.
#[tokio::test(flavor = "multi_thread")]
async fn health_check_works() -> Result<(), anyhow::Error> {
    // 준비
    let address = spawn_app().await?.address.join("health_check")?;
    let client = reqwest::Client::new();

    // 실행
    let response = client.get(address).send().await?;

    // 확인
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());

    Ok(())
}
