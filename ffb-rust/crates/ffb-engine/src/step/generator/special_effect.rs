/// Root-level abstract base for the SpecialEffect step sequence generator.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.SpecialEffect`.
use ffb_model::model::special_effect::SpecialEffect as SpecialEffectKind;

#[derive(Debug, Clone, Default)]
pub struct SpecialEffectParams {
    /// SpecialEffect kind.
    pub special_effect: Option<SpecialEffectKind>,
    pub player_id: Option<String>,
    pub roll_for_effect: bool,
}

pub struct SpecialEffect;

impl SpecialEffect {
    pub fn new() -> Self { Self }
}

impl Default for SpecialEffect {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn special_effect_params_default_no_effect() {
        let p = SpecialEffectParams::default();
        assert!(p.special_effect.is_none());
    }

    #[test]
    fn special_effect_params_default_no_player() {
        let p = SpecialEffectParams::default();
        assert!(p.player_id.is_none());
    }

    #[test]
    fn special_effect_params_default_no_roll() {
        let p = SpecialEffectParams::default();
        assert!(!p.roll_for_effect);
    }

    #[test]
    fn special_effect_params_accepts_lightning() {
        let p = SpecialEffectParams { special_effect: Some(SpecialEffectKind::LIGHTNING), ..Default::default() };
        assert_eq!(p.special_effect, Some(SpecialEffectKind::LIGHTNING));
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", SpecialEffectParams::default()).is_empty());
    }

}
