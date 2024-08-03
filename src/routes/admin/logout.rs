use axum::response::{self, IntoResponse, Response};
use axum_flash::{Flash, IncomingFlashes};
use futures_util::TryFutureExt;

use crate::{session_state::TypedSession, utils::AppError500};

pub async fn log_out(session: TypedSession, flash: Flash) -> axum::response::Result<Response> {
    match session.get_user_id().await.map_err(AppError500::new)? {
        Some(_) => {
            session.log_out().await.map_err(AppError500::new)?;
            Ok((
                flash.info("로그아웃 했습니다."),
                response::Redirect::to("/login"),
            )
                .into_response())
        }
        None => Ok(response::Redirect::to("/login").into_response()),
    }
}
