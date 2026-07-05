/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandUseReRoll`.
/// Sent to consume a team re-roll or skill re-roll.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandUseReRoll {
    /// Java: `fReRolledAction` — name of the action being re-rolled.
    pub re_rolled_action: Option<String>,
    /// Java: `fReRollSource` — name of the re-roll source (TRR, PRO, etc.).
    pub re_roll_source: Option<String>,
}

impl ClientCommandUseReRoll {
    pub fn new(re_rolled_action: impl Into<String>, re_roll_source: impl Into<String>) -> Self {
        Self {
            re_rolled_action: Some(re_rolled_action.into()),
            re_roll_source: Some(re_roll_source.into()),
        }
    }

    pub fn get_re_rolled_action(&self) -> Option<&str> { self.re_rolled_action.as_deref() }
    pub fn get_re_roll_source(&self) -> Option<&str> { self.re_roll_source.as_deref() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields_stored() {
        let cmd = ClientCommandUseReRoll::new("DODGE", "TRR");
        assert_eq!(cmd.get_re_rolled_action(), Some("DODGE"));
        assert_eq!(cmd.get_re_roll_source(), Some("TRR"));
    }

    #[test]
    fn default_is_empty() {
        let cmd = ClientCommandUseReRoll::default();
        assert!(cmd.re_rolled_action.is_none());
        assert!(cmd.re_roll_source.is_none());
    }
}
