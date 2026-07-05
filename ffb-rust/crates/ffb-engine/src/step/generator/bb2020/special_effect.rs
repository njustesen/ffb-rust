/// BB2020 special-effect card step sequence.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.bb2020.SpecialEffect`.
use ffb_model::enums::ApothecaryMode;
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::sequence::{Sequence, SequenceStep, labels};

#[derive(Debug, Clone, Default)]
pub struct SpecialEffectParams {
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
        // 2 PLACE_BALL (no SteadyFooting in BB2020)
        seq.add(StepId::PlaceBall, vec![]);
        // 3 APOTHECARY (SpecialEffect)
        seq.add(StepId::Apothecary, vec![
            StepParameter::ApothecaryMode(ApothecaryMode::SpecialEffect),
        ]);
        // 4 NEXT_STEP [END_SPECIAL_EFFECT]
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
    fn special_effect_has_4_steps() {
        let steps = SpecialEffect::build_sequence(&SpecialEffectParams::default());
        assert_eq!(steps.len(), 4);
    }

    #[test]
    fn special_effect_next_step_is_labelled_end_special_effect() {
        let steps = SpecialEffect::build_sequence(&SpecialEffectParams::default());
        let last = steps.last().unwrap();
        assert_eq!(last.step_id, StepId::NextStep);
        assert_eq!(last.label.as_deref(), Some(labels::END_SPECIAL_EFFECT));
    }

    #[test]
    fn special_effect_has_no_steady_footing() {
        let steps = SpecialEffect::build_sequence(&SpecialEffectParams::default());
        assert!(!steps.iter().any(|s| s.step_id == StepId::SteadyFooting));
    }

    #[test]
    fn special_effect_starts_with_special_effect_step() {
        let steps = SpecialEffect::build_sequence(&SpecialEffectParams::default());
        assert_eq!(steps[0].step_id, StepId::SpecialEffect);
    }

    #[test]
    fn special_effect_key_passed_to_first_step() {
        let params = SpecialEffectParams { special_effect_key: "LIGHTNING".into(), player_id: "p1".into(), ..Default::default() };
        let steps = SpecialEffect::build_sequence(&params);
        let has = steps[0].params.iter().any(|p| matches!(p, StepParameter::SpecialEffectKey(k) if k == "LIGHTNING"));
        assert!(has);
    }

    #[test]
    fn player_id_passed_to_first_step() {
        let params = SpecialEffectParams { special_effect_key: "ZAP".into(), player_id: "player42".into(), ..Default::default() };
        let steps = SpecialEffect::build_sequence(&params);
        let has = steps[0].params.iter().any(|p| matches!(p, StepParameter::PlayerId(id) if id == "player42"));
        assert!(has);
    }
}
