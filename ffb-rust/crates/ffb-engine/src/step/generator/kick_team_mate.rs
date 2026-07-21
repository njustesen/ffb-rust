/// Root-level abstract base for the KickTeamMate step sequence generator.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.KickTeamMate`.

#[derive(Debug, Clone, Default)]
pub struct KickTeamMateParams {
    pub num_dice: i32,
    pub kicked_player_id: Option<String>,
}

pub struct KickTeamMate;

impl KickTeamMate {
    pub fn new() -> Self { Self }
}

impl Default for KickTeamMate {
    fn default() -> Self { Self::new() }
}
