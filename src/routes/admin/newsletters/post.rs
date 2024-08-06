use std::sync::Arc;

use axum::{
    extract::{rejection::FormRejection, State},
    response::{self, IntoResponse, Response},
    Extension,
};
use axum_flash::Flash;

use crate::{
    authentication::UserId,
    database::Z2PADB,
    email_client::{BodyData, EmailClient},
    idempotency::IdempotencyKey,
    settings::{DefaultDBPool, DefaultEmailClient},
};

use super::AdminPublishError;

#[derive(serde::Deserialize)]
pub struct FormData {
    title: String,
    html_content: String,
    text_content: String,
    idempotency_key: String,
}

#[tracing::instrument(name = "Publish newsletter by WEB interface.", skip_all)]
pub async fn admin_publish_newsletter(
    State(email_client): State<Arc<DefaultEmailClient>>,
    State(pool): State<Arc<DefaultDBPool>>,
    // 사용자 세션에서 추출한 사용자 id를 주입힌다.
    Extension(user_id): Extension<UserId>,
    flash: Flash,
    form_data: Result<axum::Form<FormData>, FormRejection>,
) -> Result<Response, AdminPublishError> {
    let form_data = match form_data {
        Ok(form_data) => form_data.0,
        Err(e) => {
            tracing::error!(error=%e, error_detail= ?e);
            return Ok((
                flash.error("입력을 잘못했습니다."),
                response::Redirect::to("/admin/newsletters"),
            )
                .into_response());
        }
    };
    if form_data.html_content.is_empty()
        || form_data.text_content.is_empty()
        || form_data.title.is_empty()
    {
        return Ok((
            flash.error("입력을 잘못했습니다."),
            response::Redirect::to("/admin/newsletters"),
        )
            .into_response());
    }

    // 차용 검사기가 오류를 발생하지 않도록 폼을 제거해야 한다.
    let FormData {
        title,
        html_content,
        text_content,
        idempotency_key,
    } = form_data;

    let idempotency_key =
        IdempotencyKey::try_from(idempotency_key).map_err(AdminPublishError::BadRequest)?;

    // 데이터베이스에 저장된 응답이 있다면 일찍 반환한다.
    if let Some(response) = pool
        .as_ref()
        .get_saved_response(&idempotency_key, user_id.0)
        .await
        .map_err(|e| AdminPublishError::UnexpectedError(e.into()))?
    {
        return Ok(response.into_response());
    }

    let body_data = BodyData::new(title, html_content, text_content);

    email_client
        .as_ref()
        .publish_newsletter(&pool, &body_data)
        .await
        .map_err(|e| AdminPublishError::UnexpectedError(e.into()))?;

    let response = (
        flash.info("이메일 전송을 완료했습니다."),
        response::Redirect::to("/admin/newsletters"),
    )
        .into_response();
    let response = pool
        .save_response(&idempotency_key, user_id.0, response)
        .await
        .map_err(|e| AdminPublishError::UnexpectedError(e.into()))?;
    Ok(response)
}
