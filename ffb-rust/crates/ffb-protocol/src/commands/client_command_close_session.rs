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

#[test]    fn default_same_as_new() {        let _ = ClientCommandCloseSession::default();    }
}
