use anyhow::Context;
// 해야할 것
// 완료 - 1. 핸들러 뼈대 만들기
// 완료 - 2. 로그인한 사용자만 접근 가능하게 하기
// 완료 - 3. 폼 HTML 제작
// 4. 전송 완료를 표시하기
//      ! 문제
// 완료     1. 전송 완료를 마쳤을 때 완료를 표시해야 하는지 - 사용자가 문제를 인식하기 좋다.
//          2. 전송 완료 예약을 표시해야 하는지 - 사용자에게 전송 완료를 알려야 한다.
//          => 일단은 간단한 1로 접근하고, 나중에 2의 코드를 작성하기로 한다.
// 완료 5. /newsletter와 /admin/newsletter는 많은 코드가 중복될 것으로 예상된다.
//      코드를 EmailClient나 DBPool에 메서드로 붙여야 하는지 아니면 독립함수인지
//      => 일단 독립 함수로 작성하고 필요한 때 구조체에 붙인다?
use axum::response::{self, IntoResponse, Response};
use axum_flash::IncomingFlashes;
use std::fmt::Write;
use uuid::Uuid;

use super::AdminPublishError;

pub async fn admin_publish_newsletter_form(
    incoming_flashes: IncomingFlashes,
) -> Result<Response, AdminPublishError> {
    let mut flash_messages = String::new();
    for (_, message) in incoming_flashes.into_iter() {
        write!(flash_messages, "<p><i>{}</i></p>", message)
            .context("Flash to write to string.")
            .map_err(AdminPublishError::UnexpectedError)?;
    }
    let idempotency_key = Uuid::new_v4();
    Ok((
        incoming_flashes,
        response::Html(format!(
            include_str!("newsletters.html"),
            flash_messages = flash_messages,
            idempotency_key = idempotency_key,
        )),
    )
        .into_response())
}
