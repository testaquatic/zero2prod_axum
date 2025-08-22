use std::sync::Arc;

use axum::{
    Form,
    extract::State,
    http::{StatusCode, header},
    response::{ErrorResponse, IntoResponse, Response},
};
use hmac::{Hmac, Mac};
use sea_orm::DatabaseConnection;
use secrecy::ExposeSecret;
use sha3::Sha3_256;

use crate::{
    authentication::{AuthError, Credentials, validate_credentials},
    routes::error_chain_fmt,
    startup::HmacSecret,
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
    State(secret): State<Arc<HmacSecret>>,
    Form(form): Form<FormData>,
) -> axum::response::Result<Response, ErrorResponse> {
    let credentials = Credentials {
        username: form.username,
        password: form.password.into(),
    };
    tracing::Span::current().record("username", tracing::field::display(&credentials.username));
    validate_credentials(credentials, &pool)
        .await
        .map(|user_id| {
            tracing::Span::current().record("user_id", tracing::field::display(&user_id));
            // https://docs.rs/axum/latest/axum/response/index.html 이 문서를 참고로 했다.
            (StatusCode::SEE_OTHER, [(header::LOCATION, "/")]).into_response()
        })
        .map_err(|e| {
            let e = match e {
                AuthError::UnexpectedError(_) => LoginError::UnexpectedError(e.into()),
                AuthError::InvalidCredentials(_) => LoginError::AuthError(e.into()),
            };
            tracing::error!(?e);

            let query_string = format!(
                "error={}",
                urlencoding::Encoded::new(htmlescape::encode_minimal(&e.to_string()))
            );
            let hmac_tag = {
                let mut mac =
                    Hmac::<Sha3_256>::new_from_slice(secret.0.expose_secret().as_bytes()).unwrap();
                mac.update(query_string.as_bytes());
                mac.finalize().into_bytes()
            };

            let response = (
                StatusCode::SEE_OTHER,
                [(
                    header::LOCATION,
                    format!("/login?{query_string}&tag={hmac_tag:x}"),
                )],
            )
                .into_response();

            ErrorResponse::from(response)
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
