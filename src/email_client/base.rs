use crate::{database::Z2PADBError, domain::InvalidNewSubscriber, utils::error_chain_fmt};

#[derive(thiserror::Error)]
pub enum EmailClientError {
    #[error("EmailClient: Url Error")]
    UrlParseError(#[from] url::ParseError),
    #[error("EmailClient: Reqwest Error")]
    ReqwestError(#[from] reqwest::Error),
    #[error("EmailClient: SubscriberEmail Error")]
    SubscriberEmailError(#[from] InvalidNewSubscriber),
    #[error("EmailClient: Z")]
    Z2PADBError(#[from] Z2PADBError),
}

impl std::fmt::Debug for EmailClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

#[derive(serde::Deserialize)]
pub struct BodyData {
    pub title: String,
    pub content: Content,
}

#[derive(serde::Deserialize)]
pub struct Content {
    pub html: String,
    pub text: String,
}

impl BodyData {
    pub fn new(title: String, html: String, text: String) -> BodyData {
        BodyData {
            title,
            content: Content { html, text },
        }
    }
}
