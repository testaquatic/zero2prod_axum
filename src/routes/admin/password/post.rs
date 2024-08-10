use std::sync::Arc;

use axum::{
    extract::State,
    response::{self, IntoResponse},
    Extension, Form,
};
use axum_flash::Flash;
use secrecy::{ExposeSecret, Secret};
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    authentication::{AuthError, Credentials, UserId},
    database::postgres::PostgresPool,
    utils::AppError500,
};

#[derive(serde::Deserialize)]
pub struct FormData {
    current_password: Secret<String>,
    new_password: Secret<String>,
    new_password_check: Secret<String>,
}

pub async fn change_password(
    // TypedSession을 더 이상 주입하지 않는다.
    flash: Flash,
    State(pool): State<Arc<PostgresPool>>,
    Extension(UserId(user_id)): Extension<UserId>,
    Form(form): Form<FormData>,
) -> axum::response::Result<impl IntoResponse> {
    // `Secret<String>`은 `Eq`를 구현하지 않으므로 그 내부의 `String`을 비교해야 한다.
    if form.new_password.expose_secret() != form.new_password_check.expose_secret() {
        return Ok((
            flash.error("새로운 비밀번호가 일치하지 않습니다."),
            response::Redirect::to("/admin/password"),
        )
            .into_response());
    }

    // 아스키 문자 이외의 문자도 받을 수 있다.
    // 혼동을 방지하지 위해서 `char::is_ascii_graphic`을 통과하는 문자만 받는 것이 나은 선택일 것 같기도 하다.
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

    crate::authentication::change_password(user_id, form.new_password, &pool)
        .await
        .map_err(AppError500::new)?;

    Ok((
        flash.error("비밀번호를 변경했습니다."),
        response::Redirect::to("/admin/password"),
    )
        .into_response())
}
