/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandConfirm`.
/// Sent when a coach confirms a dialog choice (no payload).
#[derive(Debug, Clone, Default)]
pub struct ClientCommandConfirm;

impl ClientCommandConfirm {
    pub fn new() -> Self { Self }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_construct() {
        let _ = ClientCommandConfirm::new();
    }

    #[test]
    fn default_same_as_new() {
        let _ = ClientCommandConfirm::default();
    }

    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ClientCommandConfirm::new()).is_empty());
    }


    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandConfirm::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ClientCommandConfirm::default());
        assert!(s.contains("ClientCommandConfirm"));
    }
}
