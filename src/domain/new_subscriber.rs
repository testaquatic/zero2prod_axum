use crate::domain::{SubscriberName, subscriber_email::SubscriberEmail};

/// 새로운 구독자 정보를 저장한다.
pub struct NewSubscriber {
    pub email: SubscriberEmail,
    pub name: SubscriberName,
}
