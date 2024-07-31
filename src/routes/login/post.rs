use std::sync::Arc;

use anyhow::Context;
use axum::{
    body::Body,
    response::{IntoResponse, Response},
    Extension, Form,
};
use secrecy::Secret;

use crate::{
    authentication::{AuthError, Credentials},
    settings::DefaultDBPool,
    utils::error_chain_fmt,
};

#[derive(serde::Deserialize)]
pub struct FormData {
    username: String,
    password: Secret<String>,
}

#[derive(thiserror::Error)]
pub enum LoginError {
    // => 401
    #[error("Authentication failed.")]
    AuthError(#[source] anyhow::Error),
    // => 500
    #[error("Something went wrong.")]
    UnexpectedError(#[source] anyhow::Error),
}

impl std::fmt::Debug for LoginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl IntoResponse for LoginError {
    fn into_response(self) -> Response {
        let status_code = match self {
            LoginError::AuthError(_) => http::StatusCode::UNAUTHORIZED,
            LoginError::UnexpectedError(_) => http::StatusCode::INTERNAL_SERVER_ERROR,
        };
        let http_body = Body::new(format!(
            include_str!("login_failed.html"),
            error_message = self
        ));
        Response::builder()
            .status(status_code)
            .body(http_body)
            .unwrap_or(http::StatusCode::INTERNAL_SERVER_ERROR.into_response())
    }
}

#[tracing::instrument(
    skip_all,
    fields(
        username=tracing::field::Empty,
        user_id=tracing::field::Empty,
    )
)]
// `DefaultDBPool`을 주입해서 데이터베이스로부터 저장된 크리덴셜을 꺼낸다.
pub async fn login(
    Extension(pool): Extension<Arc<DefaultDBPool>>,
    Form(form): Form<FormData>,
) -> Result<impl IntoResponse, LoginError> {
    let credentials = Credentials {
        username: form.username,
        password: form.password,
    };
    tracing::Span::current().record("username", tracing::field::display(&credentials.username));

    let user_id = credentials
        .validate_credentials(&pool)
        .await
        .map_err(|e| match e {
            AuthError::InvalidCredentials(_) => LoginError::AuthError(e.into()),
            AuthError::UnexpectedError(_) => LoginError::UnexpectedError(e.into()),
        })?;
    tracing::Span::current().record("user_id", tracing::field::display(&user_id));

    Ok(Response::builder()
        .header(http::header::LOCATION, "/")
        .body(Body::empty())
        .context("Failed to build Response.")
        .map_err(LoginError::UnexpectedError)?)
}
