/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandBlock`.
/// Sent when a player initiates a block action.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandBlock {
    /// Java: `fActingPlayerId`
    pub acting_player_id: Option<String>,
    /// Java: `fDefenderId`
    pub defender_id: Option<String>,
    /// Java: `fUsingStab`
    pub using_stab: bool,
    /// Java: `usingChainsaw`
    pub using_chainsaw: bool,
    /// Java: `usingVomit`
    pub using_vomit: bool,
    /// Java: `usingBreatheFire`
    pub using_breathe_fire: bool,
    /// Java: `usingChomp`
    pub using_chomp: bool,
}

impl ClientCommandBlock {
    pub fn new(
        acting_player_id: impl Into<String>,
        defender_id: impl Into<String>,
        using_stab: bool,
        using_chainsaw: bool,
        using_vomit: bool,
        using_breathe_fire: bool,
        using_chomp: bool,
    ) -> Self {
        Self {
            acting_player_id: Some(acting_player_id.into()),
            defender_id: Some(defender_id.into()),
            using_stab,
            using_chainsaw,
            using_vomit,
            using_breathe_fire,
            using_chomp,
        }
    }

    pub fn get_acting_player_id(&self) -> Option<&str> { self.acting_player_id.as_deref() }
    pub fn get_defender_id(&self) -> Option<&str> { self.defender_id.as_deref() }
    pub fn is_using_stab(&self) -> bool { self.using_stab }
    pub fn is_using_chainsaw(&self) -> bool { self.using_chainsaw }
    pub fn is_using_vomit(&self) -> bool { self.using_vomit }
    pub fn is_using_breathe_fire(&self) -> bool { self.using_breathe_fire }
    pub fn is_using_chomp(&self) -> bool { self.using_chomp }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields_stored_correctly() {
        let cmd = ClientCommandBlock::new("atk1", "def1", true, false, false, false, false);
        assert_eq!(cmd.get_acting_player_id(), Some("atk1"));
        assert_eq!(cmd.get_defender_id(), Some("def1"));
        assert!(cmd.is_using_stab());
        assert!(!cmd.is_using_chainsaw());
    }

    #[test]
    fn chainsaw_flag() {
        let cmd = ClientCommandBlock::new("a", "b", false, true, false, false, false);
        assert!(cmd.is_using_chainsaw());
        assert!(!cmd.is_using_stab());
    }

    #[test]
    fn default_all_false() {
        let cmd = ClientCommandBlock::default();
        assert!(!cmd.using_stab);
        assert!(!cmd.using_chainsaw);
        assert!(!cmd.using_vomit);
        assert!(!cmd.using_breathe_fire);
        assert!(!cmd.using_chomp);
    }
}
