use crate::{
    database::Z2PADBError, domain::InvalidNewSubscriber, email_client::EmailClientError,
    utils::error_chain_fmt,
};

// 오류 관리를 편하게 하기 위한 래퍼 타입
// 오류 타입을 일치시킬 필요성이 있을 때 사용한다.
// 지역적으로 사용하는 에러는 사용하는 곳에 정의한다.
#[derive(thiserror::Error)]
pub enum Z2PAError {
    #[error("Name Error: {0}")]
    InvalidNewSubscriber(#[from] InvalidNewSubscriber),

    #[error("Reqwest Error")]
    ReqwestError(#[from] reqwest::Error),

    #[error("Email Client Error")]
    EmailClientError(#[from] EmailClientError),

    #[error("IO Error")]
    IOError(#[from] std::io::Error),

    #[error("Database Error")]
    DatabaseError(#[from] Z2PADBError),

    #[error("Config Error")]
    ConfigError(#[from] config::ConfigError),

    #[error("Url Parse Error")]
    UrlParseError(#[from] url::ParseError),
}

impl std::fmt::Debug for Z2PAError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}
