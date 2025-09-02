use std::sync::Arc;

use anyhow::Context;
use axum::{
    extract::State,
    http::{StatusCode, header},
    response::{AppendHeaders, ErrorResponse, IntoResponse, Response},
};
use entities::users;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use tower_cookies::Cookies;
use uuid::Uuid;

use crate::{
    cookie::{CookieFeeder, HmacSecret},
    session_state::TypedSession,
    startup::IndexHtml,
};

/// GET /admin/dashboard을 처리하는 핸들러
pub async fn admin_dashbaord(
    State(pool): State<Arc<DatabaseConnection>>,
    State(index_html): State<Arc<IndexHtml>>,
    State(secret): State<Arc<HmacSecret>>,
    cookies: Cookies,
    session: TypedSession,
) -> Result<Response, ErrorResponse> {
    let user_name = match session
        .get_user_id()
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

    let manage_cookie = CookieFeeder::set_hmac(&secret, None, Some(user_name), Some(cookies))
        .map_err(|_| ErrorResponse::from(StatusCode::INTERNAL_SERVER_ERROR.into_response()))?;

    Ok((
        StatusCode::OK,
        (
            AppendHeaders([(header::CONTENT_TYPE, "text/html; charset=utf-8")]),
            manage_cookie,
        ),
        index_html.admin_html.clone(),
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
    users::Entity::find()
        .filter(users::Column::UserId.eq(user_id))
        .one(pool)
        .await?
        .map(|model| model.username)
        .context("232")
}
