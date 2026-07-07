/// 1:1 translation of ClientCommandTalk (Java field: fTalk).
#[derive(Debug, Clone, Default)]
pub struct ClientCommandTalk {
    pub talk: Option<String>,
}

impl ClientCommandTalk {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_talk(talk: impl Into<String>) -> Self {
        Self { talk: Some(talk.into()) }
    }

    pub fn get_talk(&self) -> Option<&str> {
        self.talk.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_has_no_talk() {
        let cmd = ClientCommandTalk::new();
        assert!(cmd.get_talk().is_none());
    }

    #[test]
    fn with_talk_stores_value() {
        let cmd = ClientCommandTalk::with_talk("hello");
        assert_eq!(cmd.get_talk(), Some("hello"));
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ClientCommandTalk::default()).is_empty());
    }


    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandTalk::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ClientCommandTalk::default());
        assert!(s.contains("ClientCommandTalk"));
    }
}
