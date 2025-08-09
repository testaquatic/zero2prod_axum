use unicode_segmentation::UnicodeSegmentation;

/// 구독자의 이름을 저장한다.
/// `SubscriberName::parse`로 생성해야 한다.
#[derive(Debug)]
pub struct SubscriberName(String);

impl SubscriberName {
    /// `String`으로부터 `SubscriberName`을 생성한다.
    /// 이름은
    /// 1. 비어있거나 공백만 있으면 안된다.
    /// 2. 256자 이하이어야 한다.
    /// 3. '/', '(', ')', '"', '<', '>', '\\', '{', '}', ';'은 넣을 수 없다.
    pub fn parse(s: String) -> Result<SubscriberName, String> {
        let is_empty_or_whitespace = s.trim().is_empty();
        let is_too_log = s.graphemes(true).count() > 256;
        let forbidden_characters = ['/', '(', ')', '"', '<', '>', '\\', '{', '}', ';'];
        let contains_forbidden_characters = s.contains(forbidden_characters);

        if is_empty_or_whitespace || is_too_log || contains_forbidden_characters {
            Err(format!("{s} is not a valid subscribeer name."))
        } else {
            Ok(Self(s))
        }
    }
}

impl AsRef<str> for SubscriberName {
    /// 내부의 데이터에 대한 뷰를 생성한다.
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// `SubscriberName::parse`를 집중적으로 테스트한다.
/// 함수이름으로 테스트의 의도를 전달할 수 있으니 자세한 주석은 생략한다.
#[cfg(test)]
mod tests {
    use super::SubscriberName;
    use claim::{assert_err, assert_ok};

    #[test]
    fn a_256_grapheme_long_name_is_valid() {
        let name = "쬡".repeat(256);
        assert_ok!(SubscriberName::parse(name));
    }

    #[test]
    fn a_name_longer_than_256_graphemes_is_rejected() {
        let name = "쬡".repeat(257);
        assert_err!(SubscriberName::parse(name));
    }

    #[test]
    fn whitespace_only_names_are_rejected() {
        let name = " ".to_string();
        assert_err!(SubscriberName::parse(name));
    }

    #[test]
    fn empty_string_is_rejected() {
        let name = "".to_string();
        assert_err!(SubscriberName::parse(name));
    }

    #[test]
    fn names_containing_an_invalid_character_are_rejected() {
        ['/', '(', ')', '"', '<', '>', '\\', '{', '}', ';']
            .iter()
            .for_each(|name| {
                let name = name.to_string();
                assert_err!(SubscriberName::parse(name));
            });
    }

    #[test]
    fn a_valid_name_is_parsed_successfully() {
        let name = "Ursula Le Guin".to_string();
        assert_ok!(SubscriberName::parse(name));
    }
}
