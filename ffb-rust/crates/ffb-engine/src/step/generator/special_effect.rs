/// Root-level abstract base for the SpecialEffect step sequence generator.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.SpecialEffect`.
// TODO: replace String placeholder with ffb_model::model::SpecialEffect once available

#[derive(Debug, Clone, Default)]
pub struct SpecialEffectParams {
    /// SpecialEffect kind — using String as placeholder until SpecialEffect is re-exported
    /// from ffb_model. TODO: use proper SpecialEffect enum type.
    pub special_effect: Option<String>,
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
}
