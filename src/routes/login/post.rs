use std::sync::Arc;

use axum::{
    body::Body,
    extract::State,
    response::{IntoResponse, Response},
    Form,
};
use axum_flash::Flash;
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
        tracing::Span::current()
            .record("error", tracing::field::display(&self))
            .record("error_detail", tracing::field::debug(&self));
        match self {
            LoginError::AuthError(_) => http::StatusCode::UNAUTHORIZED,
            LoginError::UnexpectedError(_) => http::StatusCode::INTERNAL_SERVER_ERROR,
        }
        .into_response()
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
    flash: Flash,
    State(pool): State<Arc<DefaultDBPool>>,
    // `HmacSecret`은 더 이상 필요하지 않다.
    Form(form): Form<FormData>,
) -> axum::response::Result<impl IntoResponse> {
    let credentials = Credentials {
        username: form.username,
        password: form.password,
    };
    tracing::Span::current().record("username", tracing::field::display(&credentials.username));

    match credentials.validate_credentials(&pool).await {
        Ok(user_id) => {
            tracing::Span::current().record("user_id", tracing::field::display(&user_id));
            let response = (
                http::StatusCode::SEE_OTHER,
                [(http::header::LOCATION, "/")],
                Body::empty(),
            );

            Ok(response)
        }
        Err(e) => {
            let e = match e {
                AuthError::InvalidCredentials(_) => LoginError::AuthError(e.into()),
                AuthError::UnexpectedError(_) => LoginError::UnexpectedError(e.into()),
            };
            tracing::warn!(error.login = %e);

            let response = (
                http::StatusCode::SEE_OTHER,
                flash.error(e.to_string()),
                // 이제 쿠키가 없다.
                [(http::header::LOCATION, "/login")],
                Body::empty(),
            );

            Err(response.into())
        }
    }
}
