use axum::response::IntoResponse;
use axum_flash::IncomingFlashes;

pub async fn login_form(flashes: IncomingFlashes) -> impl IntoResponse {
    let error_html = flashes
        .iter()
        .filter_map(|(l, e)| {
            if l == axum_flash::Level::Error {
                Some(format!("<p><i>{}</i></p>", e))
            } else {
                None
            }
        })
        .collect::<String>();

    (
        http::StatusCode::OK,
        [(http::header::CONTENT_TYPE, "text/html")],
        flashes,
        format!(include_str!("login.html"), error_html = error_html),
    )
}
