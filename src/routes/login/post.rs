use std::sync::Arc;

use axum::{
    Form,
    extract::State,
    http::{StatusCode, header},
    response::{AppendHeaders, ErrorResponse, IntoResponse, Response},
};
use axum_extra::extract::CookieJar;
use axum_extra::extract::cookie::Cookie;
use sea_orm::DatabaseConnection;

use crate::{
    authentication::{AuthError, Credentials, validate_credentials},
    routes::{error_chain_fmt, generate_hmac},
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
    State(index_html): State<Arc<String>>,
    State(secret): State<Arc<HmacSecret>>,
    cookie_jar: CookieJar,
    Form(form): Form<FormData>,
) -> axum::response::Result<Response, ErrorResponse> {
    // 로그인을 처리한다.
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

            // 쿠키를 설정한다.
            // https://docs.rs/axum-extra/latest/axum_extra/extract/cookie/struct.Cookie.html 이 문서를 참고로 했다.
            let message = Cookie::new("_flash", e.to_string());
            let Ok(hmac) = generate_hmac(secret.as_ref(), &message.value()) else {
                return ErrorResponse::from(StatusCode::INTERNAL_SERVER_ERROR.into_response());
            };
            let cookie_jar = cookie_jar
                .add(message)
                .add(Cookie::new("_flash_hmac", hmac));

            let response = (
                StatusCode::UNAUTHORIZED,
                AppendHeaders([(header::CONTENT_TYPE, "text/html; charset=utf-8")]),
                cookie_jar,
                index_html.to_string(),
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
