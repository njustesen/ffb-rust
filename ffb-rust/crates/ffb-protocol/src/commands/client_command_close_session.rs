/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandCloseSession`.
/// Sent when a client disconnects or closes their session (no payload).
#[derive(Debug, Clone, Default)]
pub struct ClientCommandCloseSession;

impl ClientCommandCloseSession {
    pub fn new() -> Self { Self }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn can_construct() { let _ = ClientCommandCloseSession::new(); }

    #[test]
    fn default_same_as_new() { let _ = ClientCommandCloseSession::default(); }

    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ClientCommandCloseSession::new()).is_empty());
    }

    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandCloseSession::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ClientCommandCloseSession::default());
        assert!(s.contains("ClientCommandCloseSession"));
    }
}
