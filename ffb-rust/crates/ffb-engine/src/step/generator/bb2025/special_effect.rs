/// BB2025 special-effect card step sequence.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.bb2025.SpecialEffect`.
use ffb_model::enums::ApothecaryMode;
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::sequence::{Sequence, SequenceStep, labels};

#[derive(Debug, Clone, Default)]
pub struct SpecialEffectParams {
    /// Java SpecialEffectKey enum — string until the enum is fully ported.
    pub special_effect_key: String,
    pub player_id: String,
    pub roll_for_effect: bool,
}

pub struct SpecialEffect;

impl SpecialEffect {
    pub fn new() -> Self { Self }

    pub fn build_sequence(params: &SpecialEffectParams) -> Vec<SequenceStep> {
        let mut seq = Sequence::new();
        // 1 SPECIAL_EFFECT
        seq.add(StepId::SpecialEffect, vec![
            StepParameter::SpecialEffectKey(params.special_effect_key.clone()),
            StepParameter::PlayerId(params.player_id.clone()),
            StepParameter::RollForEffect(params.roll_for_effect),
            StepParameter::GotoLabelOnFailure(labels::END_SPECIAL_EFFECT.into()),
        ]);
        // 2 STEADY_FOOTING (goto END_SPECIAL_EFFECT on success)
        seq.add(StepId::SteadyFooting, vec![
            StepParameter::GotoLabelOnSuccess(labels::END_SPECIAL_EFFECT.into()),
        ]);
        // 3 PLACE_BALL
        seq.add(StepId::PlaceBall, vec![]);
        // 4 APOTHECARY (SPECIAL_EFFECT)
        seq.add(StepId::Apothecary, vec![
            StepParameter::ApothecaryMode(ApothecaryMode::SpecialEffect),
        ]);
        // 5 NEXT_STEP [END_SPECIAL_EFFECT]
        seq.add_labelled(StepId::NextStep, labels::END_SPECIAL_EFFECT, vec![]);
        seq.build()
    }
}

impl Default for SpecialEffect {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn special_effect_has_5_steps() {
        let steps = SpecialEffect::build_sequence(&SpecialEffectParams::default());
        assert_eq!(steps.len(), 5);
    }

    #[test]
    fn special_effect_next_step_is_labelled_end_special_effect() {
        let steps = SpecialEffect::build_sequence(&SpecialEffectParams::default());
        let last = steps.last().unwrap();
        assert_eq!(last.step_id, StepId::NextStep);
        assert_eq!(last.label.as_deref(), Some(labels::END_SPECIAL_EFFECT));
    }
}
