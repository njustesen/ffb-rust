/// Root-level abstract base for the EndPlayerAction step sequence generator.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.EndPlayerAction`.

#[derive(Debug, Clone, Default)]
pub struct EndPlayerActionParams {
    pub feeding_allowed: bool,
    pub end_player_action: bool,
    pub end_turn: bool,
    pub check_forgo: bool,
}

pub struct EndPlayerAction;

impl EndPlayerAction {
    pub fn new() -> Self { Self }
}

impl Default for EndPlayerAction {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn end_player_action_params_default_all_false() {
        let p = EndPlayerActionParams::default();
        assert!(!p.feeding_allowed);
        assert!(!p.end_player_action);
        assert!(!p.end_turn);
        assert!(!p.check_forgo);
    }

    #[test]
    fn end_player_action_params_can_set_end_turn() {
        let p = EndPlayerActionParams { end_turn: true, ..Default::default() };
        assert!(p.end_turn);
    }

    #[test]
    fn end_player_action_struct_is_default() {
        let _ = EndPlayerAction::default();
    }
}
