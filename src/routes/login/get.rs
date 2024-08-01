use axum::response::IntoResponse;
use axum_extra::extract::{cookie::Cookie, CookieJar};

pub async fn login_form(jar: CookieJar) -> impl IntoResponse {
    let error_html = match jar.get("_flash") {
        None => "".into(),
        Some(cookie) => {
            format!("<p><i>{}</i></p>", cookie.value())
        }
    };

    let cookie = Cookie::build("_flash").removal().build();
    let jar = CookieJar::new().add(cookie);
    (
        http::StatusCode::OK,
        [(http::header::CONTENT_TYPE, "text/html")],
        jar,
        format!(include_str!("login.html"), error_html = error_html),
    )
}
