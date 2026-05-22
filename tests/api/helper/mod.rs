/// 앱을 백그라운드에서 실행한다.
pub async fn spawn_app() -> Result<tokio::task::JoinHandle<()>, std::io::Error> {
    let router = zero2prod_axum::get_app_router();

    let listner = tokio::net::TcpListener::bind("127.0.0.1:8000").await?;

    let server_handle = tokio::spawn(async move {
        axum::serve(listner, router)
            .await
            .expect("failed to start server")
    });

    Ok(server_handle)
}
