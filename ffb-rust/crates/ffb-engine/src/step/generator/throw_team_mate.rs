/// Root-level abstract base for the ThrowTeamMate step sequence generator.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.ThrowTeamMate`.
use ffb_model::types::FieldCoordinate;

#[derive(Debug, Clone, Default)]
pub struct ThrowTeamMateParams {
    pub thrown_player_id: Option<String>,
    pub target_coordinate: Option<FieldCoordinate>,
    pub kicked: bool,
}

pub struct ThrowTeamMate;

impl ThrowTeamMate {
    pub fn new() -> Self { Self }
}

impl Default for ThrowTeamMate {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn throw_team_mate_params_default_no_player() {
        let p = ThrowTeamMateParams::default();
        assert!(p.thrown_player_id.is_none());
    }

    #[test]
    fn throw_team_mate_params_default_not_kicked() {
        let p = ThrowTeamMateParams::default();
        assert!(!p.kicked);
    }

    #[test]
    fn throw_team_mate_params_default_no_target() {
        let p = ThrowTeamMateParams::default();
        assert!(p.target_coordinate.is_none());
    }
}
