/// 1:1 translation of com.fumbbl.ffb.server.net.commands.InternalServerCommandApplyAutomatedPlayerMarkings.
/// AutoMarkingConfig stored as opaque String; full type not yet available.
use super::internal_server_command::InternalServerCommand;

pub struct InternalServerCommandApplyAutomatedPlayerMarkings {
    pub game_id: i64,
    pub auto_marking_config: String,
}

impl InternalServerCommandApplyAutomatedPlayerMarkings {
    pub fn new(game_id: i64, auto_marking_config: String) -> Self {
        Self { game_id, auto_marking_config }
    }

    pub fn get_auto_marking_config(&self) -> &str {
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

    #[test]
    fn construct() {
        let _ = InternalServerCommandApplyAutomatedPlayerMarkings::new(1, "cfg".to_string());
    }

    #[test]
    fn get_id() {
        let c = InternalServerCommandApplyAutomatedPlayerMarkings::new(1, "cfg".to_string());
        assert_eq!(c.get_id(), "internalApplyAutomaticPlayerMarkings");
    }

    #[test]
    fn get_game_id() {
        let c = InternalServerCommandApplyAutomatedPlayerMarkings::new(7, "cfg".to_string());
        assert_eq!(c.get_game_id(), 7);
    }

    #[test]
    fn is_internal() {
        let c = InternalServerCommandApplyAutomatedPlayerMarkings::new(1, "cfg".to_string());
        assert!(c.is_internal());
    }

    #[test]
    fn get_auto_marking_config() {
        let c = InternalServerCommandApplyAutomatedPlayerMarkings::new(1, "myConfig".to_string());
        assert_eq!(c.get_auto_marking_config(), "myConfig");
    }
}
