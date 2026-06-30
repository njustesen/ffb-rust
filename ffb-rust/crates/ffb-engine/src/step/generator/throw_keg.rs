/// Root-level abstract base for the ThrowKeg step sequence generator.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.ThrowKeg`.

#[derive(Debug, Clone, Default)]
pub struct ThrowKegParams {
    pub player_id: Option<String>,
}

pub struct ThrowKeg;

impl ThrowKeg {
    pub fn new() -> Self { Self }
}

impl Default for ThrowKeg {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn throw_keg_params_default_no_player() {
        let p = ThrowKegParams::default();
        assert!(p.player_id.is_none());
    }

    #[test]
    fn throw_keg_params_can_set_player() {
        let p = ThrowKegParams { player_id: Some("player-1".to_string()) };
        assert_eq!(p.player_id.as_deref(), Some("player-1"));
    }

    #[test]
    fn throw_keg_struct_is_default() {
        let _ = ThrowKeg::default();
    }
}
