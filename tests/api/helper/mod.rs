/// 앱을 백그라운드에서 실행한다.
/// 반환값
/// 주소의 형식은 http://127.0.0.1:무작위포트 이다.
pub async fn spawn_app() -> Result<(tokio::task::JoinHandle<()>, String), std::io::Error> {
    let router = zero2prod_axum::get_app_router();

    let listner = tokio::net::TcpListener::bind("127.0.0.1:0").await?;

    let port = listner.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    let server_handle = tokio::spawn(async move {
        axum::serve(listner, router)
            .await
            .expect("failed to start server")
    });

    Ok((server_handle, address))
}
