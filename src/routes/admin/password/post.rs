use axum::{
    response::{self, IntoResponse},
    Form,
};
use axum_flash::Flash;
use secrecy::{ExposeSecret, Secret};

use crate::{session_state::TypedSession, utils::AppError500};

#[derive(serde::Deserialize)]
pub struct FormData {
    current_password: Secret<String>,
    new_password: Secret<String>,
    new_password_check: Secret<String>,
}

pub async fn change_password(
    session: TypedSession,
    flash: Flash,
    Form(form): Form<FormData>,
) -> axum::response::Result<impl IntoResponse> {
    if session
        .get_user_id()
        .await
        .map_err(AppError500::new)?
        .is_none()
    {
        return Ok(response::Redirect::to("/login").into_response());
    };

    // `Secret<String>`은 `Eq`를 구현하지 않으므로 그 내부의 `String`을 비교해야 한다.
    if form.new_password.expose_secret() != form.new_password_check.expose_secret() {
        let flash = flash.error("새로운 비밀번호가 일치하지 않습니다.");
        return Ok((flash, response::Redirect::to("/admin/password")).into_response());
    }
    todo!()
}
