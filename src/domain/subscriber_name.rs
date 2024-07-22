use unicode_segmentation::UnicodeSegmentation;

use crate::error::DomainError;

#[derive(Debug)]
pub struct SubscriberName(String);

impl TryFrom<String> for SubscriberName {
    type Error = DomainError;
    /// 입력이 subscriber 이름에 대한 검증 조건을 모두 만족하면
    /// `Ok(SubscriberName)`을 반환한다.
    /// 그렇지 않으면 'Err(String)'을 반환한다.
    fn try_from(s: String) -> Result<Self, Self::Error> {
        // `trim()`은 입력 `s`에 대해 뒤로 계속되는 공백 문자가 없는 뷰를 반환한다.
        // `is_empty()`는 해당 뷰가 문자를 포함하고 있는지 확인한다.
        let is_empty_or_whitespace = s.trim().is_empty();

        // grapheme은 사용자가 인지할 수 있는 문자로서 유니코드 표준에 의해 정의된다.
        // `grapheme` 입력 `s`안의 graphemes에 대한 이터레이터를 반환한다.
        // `true`는 우리가 확장한 grapheme 정의 셋, 즉 권장되는 정의 셋을 사용하기 원함을 의미한다.
        let is_too_long = s.graphemes(true).count() > 256;

        // 어느 입력 `s`의 모든 문자들에 대해 반복하면서 forbidden 배열 안에 있는 문자 중, 어느 하나와 일치하는 문자가 있는지 확인한다.
        let forbidden_characters = [
            '/', '(', ')', '"', '<', '>', '\\', '{', '}', '$', ';', '%', '&', '|',
        ];
        let conatains_forbidden_characters = s.contains(forbidden_characters);

        // 어느 한 조건이라도 위반하면 `false`를 반환한다.
        if is_empty_or_whitespace || is_too_long || conatains_forbidden_characters {
            // `panic`을 `Err(e)`으로 치환한다.

            Err(DomainError::SubscriberNameError(format!(
                "{} is not a valid user name.",
                s
            )))
        } else {
            Ok(Self(s))
        }
    }
}

impl AsRef<str> for SubscriberName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod test {
    use claim::{assert_err, assert_ok};

    use crate::domain::SubscriberName;

    #[test]
    fn a_256_grapheme_long_name_is_valid() {
        let name = "쀍".repeat(256);
        assert_ok!(SubscriberName::try_from(name));
    }

    #[test]
    fn a_name_longer_than_256_graphemes_is_rejected() {
        let name = "쀍".repeat(257);
        assert_err!(SubscriberName::try_from(name));
    }

    #[test]
    fn a_whitespace_only_names_are_rejected() {
        let name = " ".to_string();
        assert_err!(SubscriberName::try_from(name));
    }

    #[test]
    fn empty_name_is_rejected() {
        let name = "".to_string();
        assert_err!(SubscriberName::try_from(name));
    }

    #[test]
    fn names_containing_an_invalid_character_are_rejected() {
        for name in &[
            '/', '(', ')', '"', '<', '>', '\\', '{', '}', '$', ';', '%', '&', '|',
        ] {
            let name = name.to_string();
            assert_err!(SubscriberName::try_from(name));
        }
    }

    #[test]
    fn a_valid_name_is_parsed_successfully() {
        let name = "Ursula Le Guin".to_string();
        assert_ok!(SubscriberName::try_from(name));
    }
}
