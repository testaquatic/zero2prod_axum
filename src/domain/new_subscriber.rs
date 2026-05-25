use crate::{
    domain::{subscriber_email::SubscriberEmail, subscriber_name::SubscriberName},
    handler::subscriptions::SubscribeFormData,
};

/// 유효성을 검사한 가입을 요청한 사용자의 정보
pub struct NewSubscriber {
    pub email: SubscriberEmail,
    pub name: SubscriberName,
}

impl TryFrom<SubscribeFormData> for NewSubscriber {
    type Error = String;
    fn try_from(value: SubscribeFormData) -> Result<Self, Self::Error> {
        Ok(NewSubscriber {
            email: SubscriberEmail::parse(value.email)?,
            name: SubscriberName::parse(value.name)?,
        })
    }
}
