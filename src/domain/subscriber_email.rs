use validator::ValidateEmail;

use super::new_subscriber::InvalidNewSubscriber;

#[derive(Debug)]
pub struct SubscriberEmail(String);

impl TryFrom<String> for SubscriberEmail {
    type Error = InvalidNewSubscriber;
    fn try_from(s: String) -> Result<SubscriberEmail, Self::Error> {
        if s.validate_email() {
            Ok(Self(s))
        } else {
            Err(InvalidNewSubscriber::InvalidSubscriberEmail(format!(
                "{} is not a valid subscriber email.",
                s
            )))
        }
    }
}

impl AsRef<str> for SubscriberEmail {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::SubscriberEmail;
    use claim::assert_err;
    // `SafeEmail` 페이커를 임포트한다.
    // 또한 `Fake`트레이트르 사용해서 `SafeEmail`의 `.fake` 메서드에 접근한다.
    use fake::{faker::internet::en::SafeEmail, Fake};

    #[test]
    fn empty_string_is_rejected() {
        let email = "".to_string();
        assert_err!(SubscriberEmail::try_from(email));
    }

    #[test]
    fn email_missing_at_symbol_is_rejected() {
        let email = "ursuladomain.com".to_string();
        assert_err!(SubscriberEmail::try_from(email));
    }

    #[test]
    fn email_missing_subject_is_rejected() {
        let email = "@domain.com".to_string();
        assert_err!(SubscriberEmail::try_from(email));
    }

    // `quickcheck`는 `Clone`과 'Debug'가 필요하다.
    #[derive(Clone, Debug)]
    struct ValidEmailFixture(pub String);

    impl quickcheck::Arbitrary for ValidEmailFixture {
        fn arbitrary(_: &mut quickcheck::Gen) -> ValidEmailFixture {
            let email = SafeEmail().fake();
            Self(email)
        }
    }

    #[quickcheck_macros::quickcheck]
    fn valid_emails_are_converted_successfully(valid_email: ValidEmailFixture) -> bool {
        SubscriberEmail::try_from(valid_email.0).is_ok()
    }
}
