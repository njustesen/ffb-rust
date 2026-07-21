/// Root-level abstract base for the EndTurn step sequence generator.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.EndTurn`.

#[derive(Debug, Clone, Default)]
pub struct EndTurnParams {
    pub check_forgo: bool,
}

pub struct EndTurn;

impl EndTurn {
    pub fn new() -> Self { Self }
}

impl Default for EndTurn {
    fn default() -> Self { Self::new() }
}
