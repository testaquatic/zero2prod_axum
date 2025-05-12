use std::sync::Arc;

use axum::{
    Form,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::{database::ZPgPool, startup::RouterState};

/// 이메일 확인 링크를 클릭하면 처리한다.
#[tracing::instrument(name = "Confirm a pending subscriber", skip_all)]
pub async fn confirm(
    router_state: State<Arc<RouterState>>,
    parameters: Form<Parameters>,
) -> Response {
    if !check_subscription_token_format(&parameters.subscription_token) {
        return StatusCode::BAD_REQUEST.into_response();
    }

    let id =
        match get_subscriber_id_from_token(&router_state.z_pgpool, &parameters.subscription_token)
            .await
        {
            Ok(id) => id,
            Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        };

    match id {
        None => StatusCode::UNAUTHORIZED.into_response(),
        Some(subscriber_id) => {
            if confirm_subscriber(&router_state.z_pgpool, &subscriber_id)
                .await
                .is_err()
            {
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }

            StatusCode::OK.into_response()
        }
    }
}

#[derive(serde::Deserialize, Debug)]
pub struct Parameters {
    subscription_token: String,
}

/// 사용자의 상태를 `confirm`으로 변경한다.
#[tracing::instrument(name = "Mark subscriber as confirmed", skip_all, err)]
async fn confirm_subscriber(
    z_pgpool: &ZPgPool,
    subscriber_id: &uuid::Uuid,
) -> Result<(), sqlx::Error> {
    z_pgpool.confirm_subscriber(subscriber_id).await
}

/// `subscription_token`으로부터 `subscriber_id`를 얻는다.
#[tracing::instrument(name = "Get subscriber_id from token", skip_all, err)]
async fn get_subscriber_id_from_token(
    z_pgpool: &ZPgPool,
    subscription_token: &str,
) -> Result<Option<uuid::Uuid>, sqlx::Error> {
    let result = z_pgpool
        .get_subscriber_id_from_token(subscription_token)
        .await?;

    Ok(result)
}

/// 토큰은 25자로 된 영숫자이다.
fn check_subscription_token_format(subscription_token: &str) -> bool {
    if subscription_token.len() != 25 {
        return false;
    }

    subscription_token
        .chars()
        .all(|c| c.is_ascii_alphanumeric())
}
