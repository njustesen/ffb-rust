/// 1:1 translation of com.fumbbl.ffb.client.util.UtilClientChat (Java class).
///
/// `apply_insertion` in Java also drives a `JTextComponent` (set text + move
/// caret) — that half is Swing UI glue with no headless equivalent, so only
/// the pure text-manipulation logic (`replace_range`) is ported.
pub struct UtilClientChat;

impl UtilClientChat {
    pub fn replace_range(text: &str, start: usize, end: usize, insertion: &str) -> String {
        let prefix = &text[..start];
        let suffix = &text[end..];
        format!("{prefix}{insertion}{suffix}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replace_range_middle() {
        assert_eq!(UtilClientChat::replace_range("hello world", 6, 11, "there"), "hello there");
    }

    #[test]
    fn replace_range_insert_at_end() {
        assert_eq!(UtilClientChat::replace_range("hello", 5, 5, " world"), "hello world");
    }

    #[test]
    fn replace_range_insert_at_start() {
        assert_eq!(UtilClientChat::replace_range("world", 0, 0, "hello "), "hello world");
    }

    #[test]
    fn replace_range_empty_insertion_deletes() {
        assert_eq!(UtilClientChat::replace_range("hello world", 5, 11, ""), "hello");
    }
}
