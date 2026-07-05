/// 1:1 translation of com.fumbbl.ffb.net.commands.ClientCommandKickOffResultChoice.
use ffb_model::enums::KickoffResult;

#[derive(Debug, Clone, Default)]
pub struct ClientCommandKickOffResultChoice {
    /// Java: `kickoffResult`
    pub kickoff_result: Option<KickoffResult>,
}

impl ClientCommandKickOffResultChoice {
    pub fn new() -> Self {
        Self::default()
    }

    /// Java: `getKickoffResult()`
    pub fn get_kickoff_result(&self) -> Option<KickoffResult> {
        self.kickoff_result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_has_no_kickoff_result() {
        let cmd = ClientCommandKickOffResultChoice::new();
        assert!(cmd.get_kickoff_result().is_none());
    }

    #[test]
    fn stores_kickoff_result() {
        let cmd = ClientCommandKickOffResultChoice {
            kickoff_result: Some(KickoffResult::QuickSnap),
        };
        assert_eq!(cmd.get_kickoff_result(), Some(KickoffResult::QuickSnap));
    }
}
