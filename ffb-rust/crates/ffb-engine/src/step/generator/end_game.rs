/// Root-level abstract base for the EndGame step sequence generator.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.EndGame`.

#[derive(Debug, Clone, Default)]
pub struct EndGameParams {
    pub admin_mode: bool,
}

pub struct EndGame;

impl EndGame {
    pub fn new() -> Self { Self }
}

impl Default for EndGame {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn end_game_params_default_not_admin_mode() {
        let p = EndGameParams::default();
        assert!(!p.admin_mode);
    }

    #[test]
    fn end_game_params_can_set_admin_mode() {
        let p = EndGameParams { admin_mode: true };
        assert!(p.admin_mode);
    }

    #[test]
    fn end_game_struct_is_default() {
        let _ = EndGame::default();
    }
}
