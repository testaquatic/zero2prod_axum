use super::{SubscriberEmail, SubscriberName};

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
