use unicode_segmentation::UnicodeSegmentation;

pub struct NewSubscriber {
    pub email: String,
    pub name: SubscriberName,
}

pub struct SubscriberName(String);

impl SubscriberName {
    pub fn parse(s: String) -> Result<SubscriberName, String> {
        // 删除空格
        let is_empty_or_whitespace = s.trim().is_empty();

        // graphemes(true)返回一个Grapheme迭代器，该迭代器将字符串拆分为Unicode字符
        let is_too_long = s.graphemes(true).count() > 256;

        // 检查禁止的字符
        let forbidden_characters = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
        let contains_forbidden_characters = s.chars().any(|g| forbidden_characters.contains(&g));

        // 如果违反了上述条件，则返回false
        if is_empty_or_whitespace || is_too_long || contains_forbidden_characters {
            // panic被用来处理不可恢复的错误。
            panic!("{} is not a valid subscriber name.", s)
        } else {
            Ok(Self(s))
        }
    }
}

// 实现AsRef trait，用于将SubscriberName转换为&str
impl AsRef<str> for SubscriberName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
