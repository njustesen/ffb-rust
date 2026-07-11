/// 1:1 translation of `ParagraphStyle.java`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum ParagraphStyle {
    INDENT_0,
    INDENT_1,
    INDENT_2,
    INDENT_3,
    INDENT_4,
    INDENT_5,
    INDENT_6,
    SPACE_ABOVE,
    SPACE_BELOW,
    SPACE_ABOVE_BELOW,
    CHAT_BODY,
}

impl ParagraphStyle {
    pub fn get_name(&self) -> &'static str {
        match self {
            ParagraphStyle::INDENT_0 => "indent0",
            ParagraphStyle::INDENT_1 => "indent1",
            ParagraphStyle::INDENT_2 => "indent2",
            ParagraphStyle::INDENT_3 => "indent3",
            ParagraphStyle::INDENT_4 => "indent4",
            ParagraphStyle::INDENT_5 => "indent5",
            ParagraphStyle::INDENT_6 => "indent6",
            ParagraphStyle::SPACE_ABOVE => "spaceAbove",
            ParagraphStyle::SPACE_BELOW => "spaceBelow",
            ParagraphStyle::SPACE_ABOVE_BELOW => "spaceAboveBelow",
            ParagraphStyle::CHAT_BODY => "chatBody",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_name_indent_0() {
        assert_eq!(ParagraphStyle::INDENT_0.get_name(), "indent0");
    }

    #[test]
    fn get_name_indent_6() {
        assert_eq!(ParagraphStyle::INDENT_6.get_name(), "indent6");
    }

    #[test]
    fn get_name_space_above_below() {
        assert_eq!(ParagraphStyle::SPACE_ABOVE_BELOW.get_name(), "spaceAboveBelow");
    }

    #[test]
    fn get_name_chat_body() {
        assert_eq!(ParagraphStyle::CHAT_BODY.get_name(), "chatBody");
    }
}
