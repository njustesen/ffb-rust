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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kick_team_mate_params_default_zero_dice() {
        let p = KickTeamMateParams::default();
        assert_eq!(p.num_dice, 0);
    }

    #[test]
    fn kick_team_mate_params_default_no_kicked_player() {
        let p = KickTeamMateParams::default();
        assert!(p.kicked_player_id.is_none());
    }

    #[test]
    fn kick_team_mate_struct_is_default() {
        let _ = KickTeamMate::default();
    }

    #[test]
    fn params_with_fields_set() {
        let p = KickTeamMateParams {
            num_dice: 2,
            kicked_player_id: Some("p1".into()),
        };
        assert_eq!(p.num_dice, 2);
        assert_eq!(p.kicked_player_id.as_deref(), Some("p1"));
    }

    #[test]
    fn params_clone() {
        let p = KickTeamMateParams { kicked_player_id: Some("x".into()), ..Default::default() };
        let q = p.clone();
        assert_eq!(q.kicked_player_id.as_deref(), Some("x"));
    }
}
