use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug)]
pub struct SubscriberName(String);

impl AsRef<str> for SubscriberName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for SubscriberName {
    type Error = String;

    /// 입력이 subscriber 이름에 대한 검증 조건을 모두 만족시키면 `SubscriberName` 인스턴스를 반환한다.
    /// 그렇지 않으면 패닉에 빠진다.
    ///
    /// 검증조건
    /// - 앞뒤 공백을 제거하고 1자 이상
    /// - 256자 이하
    /// - 일부 특수문자 비포함
    fn try_from(s: String) -> Result<Self, Self::Error> {
        let is_empty_or_whitespace = s.trim().is_empty();
        let is_too_long = s.graphemes(true).count() > 256;
        let forbidden_characters = ['/', '(', ')', '"', '<', '>', '\\', '{', '}', ';', ':', '|'];
        let contains_forbidden_characters = s.contains(forbidden_characters);

        if is_empty_or_whitespace || is_too_long || contains_forbidden_characters {
            return Err(format!("{s} is not a valid subscriber name"));
        }

        Ok(Self(s))
    }
}

#[cfg(test)]
mod test {
    use claim::{assert_err, assert_ok};

    use crate::domain::SubscriberName;

    #[test]
    fn a_256_grapheme_long_name_is_valid() {
        let name = "뷁".repeat(256);
        assert_ok!(SubscriberName::try_from(name));
    }

    #[test]
    fn a_name_longer_than_256_graphemes_is_rejected() {
        let name = "꿹".repeat(257);
        assert_err!(SubscriberName::try_from(name));
    }

    #[test]
    fn whitespace_only_names_are_rejected() {
        let name = " ".to_string();
        assert_err!(SubscriberName::try_from(name));
    }

    #[test]
    fn empty_string_is_rejected() {
        let name = "".to_string();
        assert_err!(SubscriberName::try_from(name));
    }

    #[test]
    fn names_containing_an_invalid_character_are_rejected() {
        for name in &['/', '(', ')', '"', '<', '>', '\\', '{', '}', ';', ':', '|'] {
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
