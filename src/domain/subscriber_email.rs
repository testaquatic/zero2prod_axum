use validator::ValidateEmail;

#[derive(Debug)]
pub struct SubscriberEmail(String);

impl TryFrom<String> for SubscriberEmail {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        if s.validate_email() {
            Ok(Self(s))
        } else {
            Err(format!("{s} is not a valid subscriber email"))
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

    use claim::{assert_err, assert_ok};
    use fake::{Fake, faker::internet::en::SafeEmail};
    use proptest::proptest;

    use crate::domain::SubscriberEmail;

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

    // `fn() -> T`는 `Strategy`를 구현한다.
    // https://docs.rs/proptest/latest/proptest/strategy/trait.Strategy.html#impl-Strategy-for-fn()+-%3E+T
    fn generate_email_string() -> String {
        SafeEmail().fake()
    }

    proptest! {
        #[test]
        fn valid_emails_are_parsed_successfully(email in generate_email_string as fn()->String) {
            // println!("{email}");
            assert_ok!(SubscriberEmail::try_from(email));
        }
    }
}
