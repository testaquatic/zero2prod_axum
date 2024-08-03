use std::sync::Arc;

use axum::{
    extract::State,
    response::{self, IntoResponse},
    Form,
};
use axum_flash::Flash;
use secrecy::{ExposeSecret, Secret};
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    authentication::{AuthError, Credentials},
    database::Z2PADB,
    session_state::TypedSession,
    settings::DefaultDBPool,
    utils::AppError500,
};

#[derive(serde::Deserialize)]
pub struct FormData {
    current_password: Secret<String>,
    new_password: Secret<String>,
    new_password_check: Secret<String>,
}

pub async fn change_password(
    session: TypedSession,
    flash: Flash,
    State(pool): State<Arc<DefaultDBPool>>,
    Form(form): Form<FormData>,
) -> axum::response::Result<impl IntoResponse> {
    let user_id = match session.get_user_id().await.map_err(AppError500::new)? {
        None => return Ok(response::Redirect::to("/login").into_response()),
        Some(user_id) => user_id,
    };

    // `Secret<String>`은 `Eq`를 구현하지 않으므로 그 내부의 `String`을 비교해야 한다.
    if form.new_password.expose_secret() != form.new_password_check.expose_secret() {
        return Ok((
            flash.error("새로운 비밀번호가 일치하지 않습니다."),
            response::Redirect::to("/admin/password"),
        )
            .into_response());
    }

    let new_password_len = form.new_password.expose_secret().graphemes(true).count();
    if new_password_len < 12 {
        return Ok((
            flash.error("비밀번호는 12자 이상이어야 합니다."),
            response::Redirect::to("/admin/password"),
        )
            .into_response());
    } else if new_password_len > 128 {
        return Ok((
            flash.error("비밀번호는 128자 이하이어야 합니다."),
            response::Redirect::to("/admin/password"),
        )
            .into_response());
    }

    let username = pool
        .as_ref()
        .get_username(user_id)
        .await
        .map_err(AppError500::new)?;
    let credentials = Credentials {
        username,
        password: form.current_password,
    };
    if let Err(e) = credentials.validate_credentials(pool.as_ref()).await {
        match e {
            AuthError::InvalidCredentials(_) => {
                tracing::warn!(error = %e, error.details = ?e);
                return Err((
                    flash.error("비밀번호를 잘못 입력했습니다."),
                    response::Redirect::to("/admin/password"),
                )
                    .into());
            }
            AuthError::UnexpectedError(_) => return Err((AppError500::new(e)).into()),
        };
    }
    todo!()
}
