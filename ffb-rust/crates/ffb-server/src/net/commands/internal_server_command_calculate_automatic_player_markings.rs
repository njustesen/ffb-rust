/// 1:1 translation of com.fumbbl.ffb.server.net.commands.InternalServerCommandCalculateAutomaticPlayerMarkings.
/// AutoMarkingConfig and Game stored as opaque Strings; full types not yet available.
use super::internal_server_command::InternalServerCommand;

pub struct InternalServerCommandCalculateAutomaticPlayerMarkings {
    pub auto_marking_config: String,
    pub index: i32,
    /// Game serialized as opaque string; full Game type not yet available.
    pub game: String,
}

impl InternalServerCommandCalculateAutomaticPlayerMarkings {
    pub fn new(auto_marking_config: String, index: i32, game: String) -> Self {
        Self { auto_marking_config, index, game }
    }

    pub fn get_auto_marking_config(&self) -> &str {
        &self.auto_marking_config
    }

    pub fn get_index(&self) -> i32 {
        self.index
    }

    pub fn get_game(&self) -> &str {
        &self.game
    }
}

impl InternalServerCommand for InternalServerCommandCalculateAutomaticPlayerMarkings {
    fn get_id(&self) -> &'static str {
        "internalCalculateAutomaticPlayerMarkings"
    }

    fn get_game_id(&self) -> i64 {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct() {
        let _ = InternalServerCommandCalculateAutomaticPlayerMarkings::new(
            "cfg".to_string(), 0, "game".to_string());
    }

    #[test]
    fn get_id() {
        let c = InternalServerCommandCalculateAutomaticPlayerMarkings::new(
            "cfg".to_string(), 0, "game".to_string());
        assert_eq!(c.get_id(), "internalCalculateAutomaticPlayerMarkings");
    }

    #[test]
    fn get_index() {
        let c = InternalServerCommandCalculateAutomaticPlayerMarkings::new(
            "cfg".to_string(), 3, "game".to_string());
        assert_eq!(c.get_index(), 3);
    }

    #[test]
    fn is_internal() {
        let c = InternalServerCommandCalculateAutomaticPlayerMarkings::new(
            "cfg".to_string(), 0, "game".to_string());
        assert!(c.is_internal());
    }

    #[test]
    fn get_auto_marking_config() {
        let c = InternalServerCommandCalculateAutomaticPlayerMarkings::new(
            "myCfg".to_string(), 0, "game".to_string());
        assert_eq!(c.get_auto_marking_config(), "myCfg");
    }
}
