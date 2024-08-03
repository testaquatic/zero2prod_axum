use std::sync::Arc;

use axum::{
    extract::State,
    response::{self, ErrorResponse, IntoResponse},
    Form,
};
use axum_flash::Flash;
use secrecy::Secret;

use crate::{
    authentication::{AuthError, Credentials},
    session_state::TypedSession,
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
    session: TypedSession,
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
            session.cycle_id().await.map_err(|e| {
                login_redirect(LoginError::UnexpectedError(e.into()), flash.clone())
            })?;
            session
                .insert_user_id(user_id)
                .await
                .map_err(|e| login_redirect(LoginError::UnexpectedError(e.into()), flash))?;

            Ok(response::Redirect::to("/admin/dashboard"))
        }
        Err(e) => {
            let e = match e {
                AuthError::InvalidCredentials(_) => LoginError::AuthError(e.into()),
                AuthError::UnexpectedError(_) => LoginError::UnexpectedError(e.into()),
            };

            Err(login_redirect(e, flash))
        }
    }
}

// 오류 메시지와 함께 login 페이지로 리다이렉트 한다.
fn login_redirect(e: LoginError, flash: Flash) -> ErrorResponse {
    tracing::warn!(error.login = %e, error.login.details = ?e);
    (flash, response::Redirect::to("/login")).into()
}
