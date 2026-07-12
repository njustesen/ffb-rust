/// 1:1 translation of com.fumbbl.ffb.server.net.commands.InternalServerCommandApplyAutomatedPlayerMarkings.
use ffb_engine::marking::auto_marking_config::AutoMarkingConfig;
use super::internal_server_command::InternalServerCommand;

pub struct InternalServerCommandApplyAutomatedPlayerMarkings {
    /// Java: `autoMarkingConfig` — a real, already-parsed config (built server-side in
    /// `FumbblRequestLoadPlayerMarkings` before this command is dispatched).
    pub auto_marking_config: AutoMarkingConfig,
    pub game_id: i64,
}

impl InternalServerCommandApplyAutomatedPlayerMarkings {
    /// Java constructor order: `(autoMarkingConfig, gameId)`.
    pub fn new(auto_marking_config: AutoMarkingConfig, game_id: i64) -> Self {
        Self { auto_marking_config, game_id }
    }

    /// Java: `getAutoMarkingConfig()`.
    pub fn get_auto_marking_config(&self) -> &AutoMarkingConfig {
        &self.auto_marking_config
    }
}

impl InternalServerCommand for InternalServerCommandApplyAutomatedPlayerMarkings {
    fn get_id(&self) -> &'static str {
        "internalApplyAutomaticPlayerMarkings"
    }

    fn get_game_id(&self) -> i64 {
        self.game_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> AutoMarkingConfig {
        let mut c = AutoMarkingConfig::new();
        c.set_separator("/");
        c
    }

    #[test]
    fn construct() {
        let _ = InternalServerCommandApplyAutomatedPlayerMarkings::new(config(), 1);
    }

    #[test]
    fn get_id() {
        let c = InternalServerCommandApplyAutomatedPlayerMarkings::new(config(), 1);
        assert_eq!(c.get_id(), "internalApplyAutomaticPlayerMarkings");
    }

    #[test]
    fn get_game_id() {
        let c = InternalServerCommandApplyAutomatedPlayerMarkings::new(config(), 7);
        assert_eq!(c.get_game_id(), 7);
    }

    #[test]
    fn is_internal() {
        let c = InternalServerCommandApplyAutomatedPlayerMarkings::new(config(), 1);
        assert!(c.is_internal());
    }

    #[test]
    fn get_auto_marking_config() {
        let c = InternalServerCommandApplyAutomatedPlayerMarkings::new(config(), 1);
        assert_eq!(c.get_auto_marking_config().get_separator(), "/");
    }
}
