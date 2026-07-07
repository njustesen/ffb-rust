/// 1:1 translation of `com.fumbbl.ffb.net.commands.ServerCommandPong`.
/// Server-to-client heartbeat pong response.
#[derive(Debug, Clone, Default)]
pub struct ServerCommandPong {
    /// Java: `fTimestamp` — echoed client timestamp.
    pub timestamp: i64,
}

impl ServerCommandPong {
    pub fn new(timestamp: i64) -> Self { Self { timestamp } }
    pub fn get_timestamp(&self) -> i64 { self.timestamp }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timestamp_stored() {
        let cmd = ServerCommandPong::new(99999);
        assert_eq!(cmd.get_timestamp(), 99999);
    }

    #[test]
    fn default_zero() {
        let cmd = ServerCommandPong::default();
        assert_eq!(cmd.timestamp, 0);
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ServerCommandPong::default()).is_empty());
    }

}
