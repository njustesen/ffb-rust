/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandUseHatred`.
/// Sent when a player uses Hatred to make an extra block.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandUseHatred {
    /// Java: `targetId`
    pub target_id: Option<String>,
}

impl ClientCommandUseHatred {
    pub fn new() -> Self { Self::default() }
    pub fn with_target(target_id: impl Into<String>) -> Self {
        Self { target_id: Some(target_id.into()) }
    }
    pub fn get_target_id(&self) -> Option<&str> { self.target_id.as_deref() }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn target_stored() {
        let cmd = ClientCommandUseHatred::with_target("p1");
        assert_eq!(cmd.get_target_id(), Some("p1"));
    }
    #[test]
    fn default_none() {
        assert!(ClientCommandUseHatred::new().target_id.is_none());
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ClientCommandUseHatred::default()).is_empty());
    }


    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandUseHatred::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ClientCommandUseHatred::default());
        assert!(s.contains("ClientCommandUseHatred"));
    }
}
