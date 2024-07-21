// `String`과 `&str`에 `graphemes` 메서드를 제공하기 위한 확장 트레이트
use unicode_segmentation::UnicodeSegmentation;

pub struct NewSubscriber {
    pub email: String,
    pub name: SubscriberName,
}

pub struct SubscriberName(String);

impl SubscriberName {
    /// 입력이 subscriber 이름에 대한 검증 조건을 모두 만족하면
    /// `SubscriberName` 인스턴스를 반환한다.
    /// 그렇지 않으면 패닉에 빠진다.
    pub fn parse(s: String) -> Self {
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
        let conatains_forbidden_characters = s.contains(&forbidden_characters);

        // 어느 한 조건이라도 위반하면 `false`를 반환한다.
        if is_empty_or_whitespace || is_too_long || conatains_forbidden_characters {
            panic!("{} is not a valid user name.", s)
        } else {
            Self(s)
        }
    }
}

impl AsRef<str> for SubscriberName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
