/// BB2016 special-effect card step sequence.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.bb2016.SpecialEffect`.
use ffb_model::enums::ApothecaryMode;
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::sequence::{Sequence, SequenceStep, labels};

/// Parameters for the BB2016 SpecialEffect sequence.
#[derive(Debug, Clone, Default)]
pub struct SpecialEffectParams {
    pub special_effect: Option<String>,
    pub player_id: Option<String>,
    pub roll_for_effect: bool,
}

pub struct SpecialEffect;

impl SpecialEffect {
    pub fn new() -> Self { Self }

    /// Build the BB2016 special-effect step sequence (Java `pushSequence`).
    pub fn build_sequence(params: &SpecialEffectParams) -> Vec<SequenceStep> {
        let mut seq = Sequence::new();

        // 1 SPECIAL_EFFECT
        let mut se_params = vec![
            StepParameter::RollForEffect(params.roll_for_effect),
            StepParameter::GotoLabelOnFailure(labels::END_SPECIAL_EFFECT.into()),
        ];
        if let Some(ref key) = params.special_effect {
            se_params.push(StepParameter::SpecialEffectKey(key.clone()));
        }
        if let Some(ref id) = params.player_id {
            se_params.push(StepParameter::PlayerId(id.clone()));
        }
        seq.add(StepId::SpecialEffect, se_params);

        // 2 APOTHECARY (SPECIAL_EFFECT)
        seq.add(StepId::Apothecary, vec![
            StepParameter::ApothecaryMode(ApothecaryMode::SpecialEffect),
        ]);
        // 3 NEXT_STEP [END_SPECIAL_EFFECT]
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
    fn special_effect_starts_with_special_effect() {
        let steps = SpecialEffect::build_sequence(&SpecialEffectParams::default());
        assert_eq!(steps[0].step_id, StepId::SpecialEffect);
    }

    #[test]
    fn special_effect_ends_with_next_step_labelled() {
        let steps = SpecialEffect::build_sequence(&SpecialEffectParams::default());
        let last = steps.last().unwrap();
        assert_eq!(last.step_id, StepId::NextStep);
        assert_eq!(last.label.as_deref(), Some(labels::END_SPECIAL_EFFECT));
    }

    #[test]
    fn special_effect_has_3_steps() {
        let steps = SpecialEffect::build_sequence(&SpecialEffectParams::default());
        assert_eq!(steps.len(), 3);
    }

    #[test]
    fn special_effect_apothecary_has_special_effect_mode() {
        let steps = SpecialEffect::build_sequence(&SpecialEffectParams::default());
        let apo = steps.iter().find(|s| s.step_id == StepId::Apothecary).unwrap();
        assert!(apo.params.iter().any(|p| matches!(p, StepParameter::ApothecaryMode(ApothecaryMode::SpecialEffect))));
    }
}
