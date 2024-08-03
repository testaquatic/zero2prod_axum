use axum::response::{self, ErrorResponse, IntoResponse, Response};
use axum_flash::IncomingFlashes;
use std::fmt::Write;

use crate::utils::AppError500;

pub async fn change_password_form(
    flash_messages: IncomingFlashes,
) -> Result<Response, ErrorResponse> {
    let mut msg_html = String::new();
    for (_, msg) in flash_messages.iter() {
        write!(msg_html, "<p><i>{}</i></p>", msg).map_err(AppError500::new)?;
    }

    Ok((
        flash_messages,
        response::Html(format!(include_str!("password.html"), msg_html = msg_html)),
    )
        .into_response())
}
