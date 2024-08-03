use axum::response::{self, ErrorResponse, IntoResponse, Response};
use axum_flash::IncomingFlashes;

use crate::{session_state::TypedSession, utils::AppError500};

pub async fn change_password_form(
    session: TypedSession,
    flash_messages: IncomingFlashes,
) -> Result<Response, ErrorResponse> {
    if session
        .get_user_id()
        .await
        .map_err(AppError500::new)?
        .is_none()
    {
        return Ok(response::Redirect::to("/login").into_response());
    }

    let msg_html = flash_messages
        .iter()
        .map(|s| format!("<p><i>{}</i></p>\n", s.1))
        .collect::<String>();

    Ok((
        flash_messages,
        response::Html(format!(include_str!("password.html"), msg_html = msg_html)),
    )
        .into_response())
}
