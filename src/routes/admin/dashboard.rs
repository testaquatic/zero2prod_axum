use std::sync::Arc;

use axum::{
    extract::State,
    response::{IntoResponse, Response},
};
use tower_sessions::Session;
use uuid::Uuid;

use crate::{database::Z2PADB, settings::DefaultDBPool, utils::error_chain_fmt};

#[derive(thiserror::Error)]
#[error(transparent)]
pub struct AppError500(anyhow::Error);

impl std::fmt::Debug for AppError500 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl IntoResponse for AppError500 {
    fn into_response(self) -> Response {
        tracing::Span::current()
            .record("error", tracing::field::display(&self))
            .record("error_detail", tracing::field::debug(self));

        http::StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}

pub async fn admin_dashboard(
    session: Session,
    State(pool): State<Arc<DefaultDBPool>>,
) -> Result<impl IntoResponse, AppError500> {
    let username = if let Some(user_id) = session
        .get::<Uuid>("user_id")
        .await
        .map_err(|e| AppError500(e.into()))?
    {
        pool.as_ref()
            .get_username(user_id)
            .await
            .map_err(|e| AppError500(e.into()))?
    } else {
        todo!()
    };

    Ok((
        http::StatusCode::OK,
        [(http::header::CONTENT_TYPE, "text/html")],
        format!(include_str!("dashboard.html"), username = username),
    ))
}
