use ffb_model::model::ReRolledAction;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandUseSkill`.
/// Sent when a player uses or declines to use a skill.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandUseSkill {
    /// Java: `fSkill` — stored as skill name string.
    pub skill_name: Option<String>,
    /// Java: `fSkillUsed`
    pub skill_used: bool,
    /// Java: `neverUse`
    pub never_use: bool,
    /// Java: `playerId`
    pub player_id: Option<String>,
    /// Java: `reRolledAction`
    pub re_rolled_action: Option<ReRolledAction>,
}

impl ClientCommandUseSkill {
    pub fn new() -> Self { Self::default() }
    pub fn get_skill_name(&self) -> Option<&str> { self.skill_name.as_deref() }
    pub fn is_skill_used(&self) -> bool { self.skill_used }
    pub fn is_never_use(&self) -> bool { self.never_use }
    pub fn get_player_id(&self) -> Option<&str> { self.player_id.as_deref() }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn skill_used_flag() {
        let mut cmd = ClientCommandUseSkill::new();
        cmd.skill_used = true;
        assert!(cmd.is_skill_used());
    }
    #[test]
    fn default_all_false() {
        let cmd = ClientCommandUseSkill::new();
        assert!(!cmd.skill_used);
        assert!(!cmd.never_use);
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ClientCommandUseSkill::default()).is_empty());
    }

}
