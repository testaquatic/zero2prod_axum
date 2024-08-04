use std::sync::Arc;

use axum::{
    extract::{self, State},
    response::{self, IntoResponse, Response},
};
use axum_flash::Flash;

use crate::{
    email_client::{BodyData, EmailClient},
    settings::{DefaultDBPool, DefaultEmailClient},
};

use super::AdminPublishError;

#[derive(serde::Deserialize)]
pub struct FormData {
    title: String,
    html_content: String,
    text_content: String,
}

impl From<FormData> for BodyData {
    fn from(value: FormData) -> Self {
        BodyData::new(value.title, value.html_content, value.text_content)
    }
}

pub async fn admin_publish_newsletter(
    State(email_client): State<Arc<DefaultEmailClient>>,
    State(pool): State<Arc<DefaultDBPool>>,
    flash: Flash,
    form_data: Option<extract::Form<FormData>>,
) -> Result<Response, AdminPublishError> {
    let extract::Form(form_data) = match form_data {
        Some(form_data) => form_data,
        None => {
            return Ok((
                flash.error("내용을 모두 입력해야 합니다."),
                response::Redirect::to("/admin/newsletters"),
            )
                .into_response())
        }
    };
    if form_data.html_content.is_empty()
        || form_data.text_content.is_empty()
        || form_data.title.is_empty()
    {
        return Ok((
            flash.error("내용을 모두 입력해야 합니다."),
            response::Redirect::to("/admin/newsletters"),
        )
            .into_response());
    }

    email_client
        .as_ref()
        .publish_newsletter(&pool, &form_data.into())
        .await
        .map_err(|e| AdminPublishError::UnexpectedError(e.into()))?;

    Ok((
        flash.info("이메일 전송을 완료했습니다."),
        response::Redirect::to("/admin/newsletters"),
    )
        .into_response())
}
