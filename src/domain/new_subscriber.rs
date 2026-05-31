use crate::domain::{
    form_data::SubscriptionData, subscriber_email::SubscriberEmail, subscriber_name::SubscriberName,
};

/// 유효성을 검사한 가입을 요청한 사용자의 정보
#[derive(Debug)]
pub struct NewSubscriber {
    pub email: SubscriberEmail,
    pub name: SubscriberName,
}

impl TryFrom<SubscriptionData> for NewSubscriber {
    type Error = String;
    fn try_from(value: SubscriptionData) -> Result<Self, Self::Error> {
        Ok(NewSubscriber {
            email: SubscriberEmail::parse(value.email)?,
            name: SubscriberName::parse(value.name)?,
        })
    }
}
