/// 1:1 translation of com.fumbbl.ffb.net.commands.ClientCommandSkillSelection.
///
/// Java: `skill` is a complex Skill object. We use the skill's string identifier.

#[derive(Debug, Clone, Default)]
pub struct ClientCommandSkillSelection {
    /// Java: `playerId`
    pub player_id: Option<String>,
    /// Java: `skill` — stored as the skill's string identifier (DEFERRED: full Skill object).
    pub skill_id: Option<String>,
}

impl ClientCommandSkillSelection {
    pub fn new() -> Self {
        Self::default()
    }

    /// Java: `getPlayerId()`
    pub fn get_player_id(&self) -> Option<&str> {
        self.player_id.as_deref()
    }

    /// Java: `getSkill()` — returns the skill identifier string.
    pub fn get_skill_id(&self) -> Option<&str> {
        self.skill_id.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_has_no_player_or_skill() {
        let cmd = ClientCommandSkillSelection::new();
        assert!(cmd.get_player_id().is_none());
        assert!(cmd.get_skill_id().is_none());
    }

    #[test]
    fn stores_player_id_and_skill_id() {
        let cmd = ClientCommandSkillSelection {
            player_id: Some("player_2".to_string()),
            skill_id: Some("BLOCK".to_string()),
        };
        assert_eq!(cmd.get_player_id(), Some("player_2"));
        assert_eq!(cmd.get_skill_id(), Some("BLOCK"));
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ClientCommandSkillSelection::default()).is_empty());
    }

}
