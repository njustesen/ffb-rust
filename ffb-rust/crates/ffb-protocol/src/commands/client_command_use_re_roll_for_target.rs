/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandUseReRollForTarget`.
/// Extends UseReRoll for a specific block target.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandUseReRollForTarget {
    /// Java: `targetId` (from `ClientCommandUseReRollForTarget`)
    pub target_id: Option<String>,
    /// Java: `fReRolledAction` (inherited from `ClientCommandUseReRoll`)
    pub re_rolled_action: Option<String>,
}

impl ClientCommandUseReRollForTarget {
    pub fn new() -> Self { Self::default() }
    pub fn get_target_id(&self) -> Option<&str> { self.target_id.as_deref() }
    pub fn get_re_rolled_action(&self) -> Option<&str> { self.re_rolled_action.as_deref() }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fields_stored() {
        let mut cmd = ClientCommandUseReRollForTarget::new();
        cmd.target_id = Some("p2".into());
        assert_eq!(cmd.get_target_id(), Some("p2"));
    }
    #[test]
    fn default_none() {
        assert!(ClientCommandUseReRollForTarget::new().target_id.is_none());
    }
}
