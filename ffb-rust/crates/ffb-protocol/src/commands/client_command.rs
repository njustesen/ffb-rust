/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommand`.
/// Base class for all client commands — carries optional entropy byte for anti-replay.
#[derive(Debug, Clone, Default)]
pub struct ClientCommand {
    /// Java: `fEntropy` — optional entropy byte for anti-replay protection.
    pub entropy: Option<u8>,
}

impl ClientCommand {
    pub fn new() -> Self { Self::default() }
    pub fn get_entropy(&self) -> Option<u8> { self.entropy }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default_no_entropy() {
        assert!(ClientCommand::new().entropy.is_none());
    }
    #[test]
    fn entropy_stored() {
        let cmd = ClientCommand { entropy: Some(42) };
        assert_eq!(cmd.get_entropy(), Some(42));
    }
}
