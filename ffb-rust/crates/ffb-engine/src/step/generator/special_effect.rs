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
