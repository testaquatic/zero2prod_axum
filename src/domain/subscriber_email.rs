use validator::ValidateEmail;

/// 사용자의 이메일 정보를 표현한다.
#[derive(Debug)]
pub struct SubscriberEmail(String);

impl SubscriberEmail {
    /// `String`이 유효한 이메일 주소 형식이라면 `SubscriberEmail`을 생성한다.
    pub fn parse(s: String) -> Result<SubscriberEmail, String> {
        if ValidateEmail::validate_email(&s) {
            Ok(Self(s))
        } else {
            Err(format!("{s} is not a valid subscriber email."))
        }
    }
}

impl AsRef<str> for SubscriberEmail {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for SubscriberEmail {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// 함수 이름으로 충분하니 자세한 주석을 생략한다.
#[cfg(test)]
mod tests {
    use crate::domain::subscriber_email::SubscriberEmail;
    use claim::assert_err;
    use fake::{Fake, faker::internet::en::SafeEmail};

    #[test]
    fn empty_string_is_rejected() {
        let email = "".to_string();
        assert_err!(SubscriberEmail::parse(email));
    }

    #[test]
    fn email_missing_at_symbol_is_rejected() {
        let email = "ursuladomain.com".to_string();
        assert_err!(SubscriberEmail::parse(email));
    }

    #[test]
    fn email_missing_subject_is_rejected() {
        let email = "@domain.com".to_string();
        assert_err!(SubscriberEmail::parse(email));
    }

    /// quickcheck에서 사용할 무작위 이메일을 생성한다.
    #[derive(Debug, Clone)]
    struct ValidEmailFixture(pub String);

    impl quickcheck::Arbitrary for ValidEmailFixture {
        fn arbitrary(_: &mut quickcheck::Gen) -> Self {
            // 책과는 다르게 fake_with_rng는 작동하지 않는다.
            let email = SafeEmail().fake();

            Self(email)
        }
    }

    /// 무작위로 생성된 이메일 주소로부터 테스트를 진행한다.
    #[quickcheck_macros::quickcheck]
    fn valid_emails_are_parsed_successfully(valid_emai: ValidEmailFixture) -> bool {
        SubscriberEmail::parse(valid_emai.0).is_ok()
    }
}
