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
    session_state::TypedSession,
    startup::{HmacSecret, IndexHtml},
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
    State(html): State<Arc<IndexHtml>>,
    State(secret): State<Arc<HmacSecret>>,
    cookie_jar: CookieJar,
    session: TypedSession,
    Form(form): Form<FormData>,
) -> axum::response::Result<Response, ErrorResponse> {
    // 로그인을 처리한다.
    let credentials = Credentials {
        username: form.username,
        password: form.password.into(),
    };
    tracing::Span::current().record("username", tracing::field::display(&credentials.username));
    match validate_credentials(credentials, &pool).await {
        Ok(user_id) => {
            tracing::Span::current().record("user_id", tracing::field::display(&user_id));
            session.renew().await.map_err(|e| {
                tracing::error!("Failed to cycle session id: {e}");
                login_failed(
                    LoginError::UnexpectedError(e.into()),
                    &html.pub_html,
                    secret.as_ref(),
                    cookie_jar.clone(),
                )
            })?;
            // https://docs.rs/axum/latest/axum/response/index.html 이 문서를 참고로 했다.
            session.insert_user_id(user_id).await.map_err(|e| {
                tracing::error!("Failed to insert session: {e}");
                login_failed(
                    LoginError::UnexpectedError(e.into()),
                    &html.pub_html,
                    secret.as_ref(),
                    cookie_jar,
                )
            })?;

            let response = (
                StatusCode::SEE_OTHER,
                [(header::LOCATION, "/admin/dashboard")],
            )
                .into_response();

            Ok(response)
        }

        Err(e) => {
            let e = match e {
                AuthError::UnexpectedError(_) => LoginError::UnexpectedError(e.into()),
                AuthError::InvalidCredentials(_) => LoginError::AuthError(e.into()),
            };
            tracing::error!(?e);
            Err(login_failed(e, &html.pub_html, secret.as_ref(), cookie_jar))
        }
    }
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

/// 오류 메시지와 함께 login 페이지로 리다이렉트 한다.
fn login_failed(
    e: LoginError,
    html: &str,
    secret: &HmacSecret,
    cookie_jar: CookieJar,
) -> ErrorResponse {
    // 쿠키를 설정한다.
    // https://docs.rs/axum-extra/latest/axum_extra/extract/cookie/struct.Cookie.html 이 문서를 참고로 했다.
    let message = Cookie::new("_flash", e.to_string());
    let hmac = match generate_hmac(secret, message.value()) {
        Ok(hmac) => hmac,
        // 이 분기로 들어올 가능성은 거의 없어 보이나 혹시 모르니 처리
        Err(e) => {
            tracing::error!("Failed to generate hmac: {e}");
            return ErrorResponse::from(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let cookie_jar = cookie_jar
        .add(message)
        .add(Cookie::new("_flash_hmac", hmac));

    let response = (
        StatusCode::UNAUTHORIZED,
        AppendHeaders([(header::CONTENT_TYPE, "text/html; charset=utf-8")]),
        cookie_jar,
        html.to_string(),
    )
        .into_response();

    ErrorResponse::from(response)
}
