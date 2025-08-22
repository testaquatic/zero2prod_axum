use std::sync::Arc;

use anyhow::Context;
use axum::{
    extract::{Query, Request, State},
    http::header,
    response::{IntoResponse, Response},
};
use hmac::{Hmac, Mac};
use reqwest::StatusCode;
use secrecy::ExposeSecret;

use crate::{routes::serve_index_file, startup::HmacSecret};

/// hmac 태그 검증을 위한 파라미터
#[derive(serde::Deserialize)]
pub struct QueryParams {
    error: Option<String>,
    tag: Option<String>,
}

/// GET /login을 담당하는 핸들러
#[axum::debug_handler]
pub async fn login_form(
    State(secret): State<Arc<HmacSecret>>,
    Query(query): Query<QueryParams>,
    request: Request,
) -> Response {
    match query {
        QueryParams {
            error: None,
            tag: None,
        } => (),
        QueryParams {
            error: Some(_),
            tag: Some(_),
        } => {
            if let Err(e) = query.verify(&secret) {
                tracing::warn!(
                    error.message = %e,
                    error.cause_chain = ?e,
                    "Failed to verify query parameters using the HMAC tab."
                );

                return (StatusCode::SEE_OTHER, [(header::LOCATION, "/login")]).into_response();
            }
        }
        // QueryParams.error와 QueryParams.tag는 동시에 있거나 없어야 한다.
        // 들어오면 안되는 요청이므로 리다이렉트 하지 않는다.
        QueryParams { error, tag } => {
            tracing::warn!(
                "Empty query parameters.\n\terror: {}\n\ttag: {}\n",
                error.unwrap_or_default(),
                tag.unwrap_or_default()
            );
            return (StatusCode::SEE_OTHER, [(header::LOCATION, "/login")]).into_response();
        }
    }

    serve_index_file(request)
        .await
        .map_err(|e| {
            tracing::error!("Failed to serve file.\n\tError: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })
        .into_response()
}

impl QueryParams {
    /// 메시지 인증 코드가 메시지와 일치하는지 확인한다.
    /// 일치할 경우에 오류 문자열을 Result::Ok에 감싸서 반환한다.
    fn verify(self, secret: &HmacSecret) -> Result<String, anyhow::Error> {
        let tag = hex::decode(self.tag.context("Empty tag.")?)?;
        let query_string = format!(
            "error={}",
            urlencoding::Encoded::new(self.error.as_ref().context("Empty error.")?)
        );
        let mut mac = Hmac::<sha3::Sha3_256>::new_from_slice(secret.0.expose_secret().as_bytes())
            .context("Failed to create Mac.")?;
        mac.update(query_string.as_bytes());
        mac.verify_slice(&tag)?;

        Ok(self.error.expect("Emapty error."))
    }
}
