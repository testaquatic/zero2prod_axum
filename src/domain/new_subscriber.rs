use super::{SubscriberEmail, SubscriberName};

#[derive(thiserror::Error, Debug)]
pub enum InvalidNewSubscriber {
    #[error("{0}")]
    InvalidSubscriberEmail(String),
    #[error("{0}")]
    InvalidSubscriberName(String),
}

pub struct NewSubscriber {
    // `String`은 더 이상 사용하지 않는다.
    pub email: SubscriberEmail,
    pub name: SubscriberName,
}

impl NewSubscriber {
    pub fn new(email: SubscriberEmail, name: SubscriberName) -> Self {
        Self { email, name }
    }
}
