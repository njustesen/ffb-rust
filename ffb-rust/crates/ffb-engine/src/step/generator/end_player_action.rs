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
