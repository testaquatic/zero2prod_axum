use axum::{extract::Request, http::StatusCode, response::IntoResponse};
use tower_http::services::ServeFile;

pub async fn home(request: Request) -> impl IntoResponse {
    serve_index_file(request).await.map_err(|e| {
        tracing::error!("Failed to serve file.\n\tError: {e}");
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    })
}

pub async fn serve_index_file(request: Request) -> Result<impl IntoResponse, std::io::Error> {
    ServeFile::new("web/dist/index.html")
        .try_call(request)
        .await
}
