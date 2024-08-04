// 해야할 것
// 완료 - 1. 핸들러 뼈대 만들기
// 완료 - 2. 로그인한 사용자만 접근 가능하게 하기
// 완료 - 3. 폼 HTML 제작
// 4. 전송 완료를 표시하기
//      ! 문제
//          1. 전송 완료를 마쳤을 때 완료를 표시해야 하는지 - 사용자가 문제를 인식하기 좋다.
//          2. 전송 완료 예약을 표시해야 하는지 - 사용자에게 전송 완료를 알려야 한다.
//          => 일단은 간단한 1로 접근하고, 나중에 2의 코드를 작성하기로 한다.
// 5. /newsletter와 /admin/newsletter는 많은 코드가 중복될 것으로 예상된다.
//      코드를 EmailClient나 DBPool에 메서드로 붙여야 하는지 아니면 독립함수인지
//      => 일단 독립 함수로 작성하고 필요한 때 둘 모두에 붙인다?
use axum::response::{self, IntoResponse, Response};

use crate::utils::{error_chain_fmt, AppError500};

#[derive(thiserror::Error)]
pub enum AdminPublishError {
    #[error("AdminPublishError: UnexpectedError")]
    UnexpectedError(#[source] anyhow::Error),
}

impl std::fmt::Debug for AdminPublishError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl IntoResponse for AdminPublishError {
    fn into_response(self) -> Response {
        match self {
            _ => AppError500::new(self).into_response(),
        }
    }
}

pub async fn admin_publish_newsletter_form() -> Result<Response, AdminPublishError> {
    Ok(response::Html(include_str!("newsletters.html")).into_response())
}
