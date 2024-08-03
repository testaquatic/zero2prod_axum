use std::sync::Arc;

use axum::{
    extract::State,
    response::{self, IntoResponse, Response},
    Extension,
};

use crate::{
    authentication::UserId, database::Z2PADB, settings::DefaultDBPool, utils::AppError500,
};

pub async fn admin_dashboard(
    State(pool): State<Arc<DefaultDBPool>>,
    Extension(UserId(user_id)): Extension<UserId>,
) -> axum::response::Result<Response> {
    let username = pool
        .as_ref()
        .get_username(user_id)
        .await
        .map_err(AppError500::new)?;

    Ok(
        response::Html(format!(include_str!("dashboard.html"), username = username))
            .into_response(),
    )
}
