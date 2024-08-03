use axum::response::{self, IntoResponse, Response};
use axum_flash::Flash;

use crate::{session_state::TypedSession, utils::AppError500};

pub async fn log_out(session: TypedSession, flash: Flash) -> axum::response::Result<Response> {
    session.log_out().await.map_err(AppError500::new)?;
    Ok((
        flash.info("로그아웃 했습니다."),
        response::Redirect::to("/login"),
    )
        .into_response())
}
