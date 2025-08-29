use std::sync::Arc;

use anyhow::Context;
use axum::{
    body::Body,
    extract::State,
    http::{HeaderName, StatusCode, header},
    response::{AppendHeaders, ErrorResponse, IntoResponse, Response},
};
use entities::users;
use sea_orm::{DatabaseConnection, EntityTrait, QuerySelect};
use tower_sessions::Session;
use uuid::Uuid;

use crate::startup::IndexHtml;

/// GET /admin/dashboard을 처리하는 핸들러
pub async fn admin_dashbaord(
    State(pool): State<Arc<DatabaseConnection>>,
    State(index_html): State<Arc<IndexHtml>>,
    session: Session,
) -> Result<Response, ErrorResponse> {
    let username = match session
        .get::<Uuid>("user_id")
        .await
        .map_err(e500_internal_server_error)?
    {
        Some(user_id) => get_username(user_id, &pool)
            .await
            .map_err(e500_internal_server_error)?,
        None => {
            return Ok((
                StatusCode::SEE_OTHER,
                AppendHeaders([(header::LOCATION, "/login")]),
            )
                .into_response());
        }
    };

    Ok((
        StatusCode::OK,
        AppendHeaders([
            (header::CONTENT_TYPE, "text/html; charset=utf-8"),
            (HeaderName::from_static("X-username"), &username),
        ]),
        Body::new(index_html.admin_html.clone()),
    )
        .into_response())
}

/// 로깅을 위해 오류의 근본 원인은 유지하면서 불투명한 500 Internal Server Error를 반환한다.
fn e500_internal_server_error<T>(e: T) -> ErrorResponse
where
    T: std::fmt::Display + std::fmt::Debug,
{
    tracing::error!("{e:?}");
    ErrorResponse::from(StatusCode::INTERNAL_SERVER_ERROR)
}

#[tracing::instrument(name = "Get username", skip_all, fields(user_id = %user_id))]
async fn get_username(user_id: Uuid, pool: &DatabaseConnection) -> Result<String, anyhow::Error> {
    users::Entity::find_by_id(user_id)
        .select_only()
        .column(users::Column::Username)
        .one(pool)
        .await?
        .map(|model| model.username)
        .context("User not found.")
}
