use rand::{distributions::Alphanumeric, thread_rng, Rng};

pub struct SubscriptionToken {
    subscription_token: String,
}

/// 대소문자를 구분하는 무작위 25문자로 구성된 구독 토큰을 생성한다.
pub fn generate_subscription_token() -> SubscriptionToken {
    let rng = thread_rng();

    let subscription_token = rng
        .sample_iter(Alphanumeric)
        .map(char::from)
        .take(25)
        .collect();

    SubscriptionToken { subscription_token }
}

impl AsRef<str> for SubscriptionToken {
    fn as_ref(&self) -> &str {
        &self.subscription_token
    }
}
