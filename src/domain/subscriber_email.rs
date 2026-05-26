use validator::ValidateEmail;

#[derive(Debug)]
pub struct SubscriberEmail(String);

impl SubscriberEmail {
    pub fn parse(s: String) -> Result<SubscriberEmail, String> {
        if s.validate_email() {
            Ok(SubscriberEmail(s))
        } else {
            Err(format!("{} is not a valid subscriber email", s))
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

#[cfg(test)]
mod tests {
    use fake::{Fake, faker::internet::en::SafeEmail};

    use crate::domain::subscriber_email::SubscriberEmail;

    #[derive(Debug, Clone)]
    struct ValidEmailFixture(pub String);

    impl quickcheck::Arbitrary for ValidEmailFixture {
        fn arbitrary(_: &mut quickcheck::Gen) -> Self {
            let email = SafeEmail().fake();
            Self(email)
        }
    }

    #[test]
    fn empty_string_is_rejected() {
        let email = "".to_string();
        assert!(
            SubscriberEmail::parse(email.clone()).is_err(),
            "empty email should be rejected: {}",
            email
        );
    }

    #[test]
    fn email_missing_at_symbol_is_rejected() {
        let email = "ursulaguin.com".to_string();
        assert!(
            SubscriberEmail::parse(email.clone()).is_err(),
            "email missing @: {}",
            email
        );
    }

    #[test]
    fn email_missing_subject_is_rejected() {
        let email = "@gmail.com".to_string();
        assert!(
            SubscriberEmail::parse(email.clone()).is_err(),
            "email missing subject: {}",
            email
        );
    }

    #[test]
    fn valid_email_is_parsed_successfully() {
        let email = SafeEmail().fake::<String>();
        assert!(
            SubscriberEmail::parse(email.clone()).is_ok(),
            "valid email should be parsed successfully: {}",
            email
        );
    }

    #[quickcheck_macros::quickcheck]
    fn valid_emails_are_parsed_successfully(valid_email: ValidEmailFixture) -> bool {
        SubscriberEmail::parse(valid_email.0).is_ok()
    }
}
