/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandPing`.
/// Heartbeat ping from client to server.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandPing {
    /// Java: `fTimestamp` — client-side timestamp at ping send time.
    pub timestamp: i64,
}

impl ClientCommandPing {
    pub fn new(timestamp: i64) -> Self { Self { timestamp } }
    pub fn get_timestamp(&self) -> i64 { self.timestamp }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn timestamp_stored() {
        let cmd = ClientCommandPing::new(12345);
        assert_eq!(cmd.get_timestamp(), 12345);
    }
    #[test]
    fn default_zero() {
        let cmd = ClientCommandPing::default();
        assert_eq!(cmd.timestamp, 0);
    }
#[test]    fn debug_format_nonempty() {        let v = ClientCommandPing::default();        assert!(!format!("{:?}", v).is_empty());    }

    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandPing::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ClientCommandPing::default());
        assert!(s.contains("ClientCommandPing"));
    }
}
