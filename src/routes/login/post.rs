use std::sync::Arc;

use anyhow::Context;
use axum::{
    body::Body,
    response::{IntoResponse, Response},
    Extension, Form,
};
use hmac::{Hmac, Mac};
use secrecy::{ExposeSecret, Secret};

use crate::{
    authentication::{AuthError, Credentials},
    settings::DefaultDBPool,
    startup::HmacSecret,
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
    Extension(pool): Extension<Arc<DefaultDBPool>>,
    Extension(hmac_secret): Extension<HmacSecret>,
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
            let response = Response::builder()
                .status(http::StatusCode::SEE_OTHER)
                .header(http::header::LOCATION, "/")
                .body(Body::empty())
                .context("Failed to create response.")
                .map_err(LoginError::UnexpectedError)?;
            Ok(response)
        }
        Err(e) => {
            let e = match e {
                AuthError::InvalidCredentials(_) => LoginError::AuthError(e.into()),
                AuthError::UnexpectedError(_) => LoginError::UnexpectedError(e.into()),
            };
            let encoded_error = urlencoding::Encoded::new(e.to_string());
            let query_string = format!("error={}", encoded_error);
            let hmac_tag = {
                let mut mac = Hmac::<sha3::Sha3_256>::new_from_slice(
                    hmac_secret.0.expose_secret().as_bytes(),
                )
                .context("Failed to create Hmac.")
                .map_err(LoginError::UnexpectedError)?;
                mac.update(query_string.as_bytes());
                mac.finalize().into_bytes()
            };
            let response = Response::builder()
                .status(http::StatusCode::SEE_OTHER)
                .header(
                    http::header::LOCATION,
                    format!("/login?{}&tag={:x}", query_string, hmac_tag),
                )
                .body(Body::empty())
                .context("Failed to create Response.")
                .map_err(LoginError::UnexpectedError)?;

            Err(response.into())
        }
    }
}
