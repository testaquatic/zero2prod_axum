use axum::{
    extract::Request,
    middleware::Next,
    response::{self, IntoResponse, Response},
};
use uuid::Uuid;

use crate::{session_state::TypedSession, utils::AppError500};

#[derive(Clone)]
pub struct UserId(pub Uuid);

pub async fn reject_anonymous_users(
    typed_session: TypedSession,
    mut request: Request,
    next: Next,
) -> response::Result<Response> {
    match typed_session
        .get_user_id()
        .await
        .map_err(AppError500::new)?
    {
        Some(user_id) => {
            request.extensions_mut().insert(UserId(user_id));
            Ok(next.run(request).await)
        }
        None => {
            tracing::warn!("The user has not logged in");
            Ok(response::Redirect::to("/login").into_response())
        }
    }
}
