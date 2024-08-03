use std::sync::Arc;

use axum::{
    extract::State,
    response::{self, IntoResponse, Response},
};

use crate::{
    database::Z2PADB, session_state::TypedSession, settings::DefaultDBPool, utils::AppError500,
};

pub async fn admin_dashboard(
    session: TypedSession,
    State(pool): State<Arc<DefaultDBPool>>,
) -> axum::response::Result<Response> {
    let username = if let Some(user_id) = session.get_user_id().await.map_err(AppError500::new)? {
        pool.as_ref()
            .get_username(user_id)
            .await
            .map_err(AppError500::new)?
    } else {
        return Ok(response::Redirect::to("/login").into_response());
    };

    Ok(
        response::Html(format!(include_str!("dashboard.html"), username = username))
            .into_response(),
    )
}
