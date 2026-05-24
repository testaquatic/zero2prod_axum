use unicode_segmentation::UnicodeSegmentation;

/// 가입을 요청한 사용자의 정보
pub struct NewSubscriber {
    pub email: String,
    pub name: SubscriberName,
}

pub struct SubscriberName(String);

impl SubscriberName {
    /// 문자열을 검사하고 `SubscriberName`을 반환한다.
    pub fn parse(s: String) -> Result<SubscriberName, String> {
        // 빈 문자열인지 검사
        let is_empty_or_whitespace = s.trim().is_empty();
        // 256자를 초과했는지 검사
        let is_too_long = s.graphemes(true).count() > 256;
        // 금지된 문자가 포함되었는지 검사
        let fobidden_characters = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
        let contains_forbidden_characters = s.contains(fobidden_characters);

        if is_empty_or_whitespace || is_too_long || contains_forbidden_characters {
            Err(format!("{} is not a valid subscriber name", s))
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
mod tests {
    use crate::domain::subscriber::SubscriberName;

    #[test]
    fn a_256_grapheme_long_name_is_valid() {
        let name = "쀍".repeat(256);
        assert!(SubscriberName::parse(name).is_ok());
    }

    #[test]
    fn a_name_longer_than_256_graphemes_is_rejected() {
        let name = "a".repeat(257);
        assert!(SubscriberName::parse(name).is_err());
    }

    #[test]
    fn whitespace_only_names_are_rejected() {
        let name = " ".to_string();
        assert!(SubscriberName::parse(name).is_err());
    }

    #[test]
    fn empty_string_is_rejected() {
        let name = "".to_string();
        assert!(SubscriberName::parse(name).is_err());
    }

    #[test]
    fn names_containing_an_invalid_character_are_rejected() {
        for name in &['/', '(', ')', '"', '<', '>', '\\', '{', '}'] {
            let name = name.to_string();
            assert!(SubscriberName::parse(name).is_err());
        }
    }

    #[test]
    fn a_valid_name_is_parsed_successfully() {
        let name = "Ursula Le Guin".to_string();
        assert!(SubscriberName::parse(name).is_ok());
    }
}
