use std::sync::Arc;

use axum::{
    Form,
    extract::State,
    http::header,
    response::{IntoResponse, Response},
};
use reqwest::StatusCode;
use sea_orm::DatabaseConnection;

use crate::{
    authentication::{AuthError, Credentials, validate_credentials},
    routes::error_chain_fmt,
};

/// Form의 정보를 저장한다.
#[derive(serde::Deserialize)]
pub struct FormData {
    pub username: String,
    pub password: String,
}

/// POST /login 을 처리하는 핸들러
#[tracing::instrument(skip_all, fields(username = tracing::field::Empty, user_id = tracing::field::Empty))]
pub async fn login(
    State(pool): State<Arc<DatabaseConnection>>,
    Form(form): Form<FormData>,
) -> Result<Response, LoginError> {
    let credentials = Credentials {
        username: form.username,
        password: form.password.into(),
    };
    tracing::Span::current().record("username", &tracing::field::display(&credentials.username));
    validate_credentials(credentials, &pool)
        .await
        .map(|user_id| {
            tracing::Span::current().record("user_id", &tracing::field::display(&user_id));
            // https://docs.rs/axum/latest/axum/response/index.html 이 문서를 참고로 했다.
            (StatusCode::SEE_OTHER, [(header::LOCATION, "/")]).into_response()
        })
        .map_err(|e| match e {
            AuthError::UnexpectedError(_) => LoginError::UnexpectedError(e.into()),
            AuthError::InvalidCredentials(_) => LoginError::AuthError(e.into()),
        })
}

/// login 핸들러에서 반환하는 오류이다.
#[derive(thiserror::Error)]
pub enum LoginError {
    /// 401 Unauthorized를 반환한다.
    #[error("Authentication failed")]
    AuthError(#[source] anyhow::Error),
    /// 500 Internal Server Error 를 반환한다.
    #[error("Something went wrong")]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for LoginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl IntoResponse for LoginError {
    fn into_response(self) -> Response {
        match self {
            LoginError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            LoginError::AuthError(_) => StatusCode::UNAUTHORIZED.into_response(),
        }
    }
}
