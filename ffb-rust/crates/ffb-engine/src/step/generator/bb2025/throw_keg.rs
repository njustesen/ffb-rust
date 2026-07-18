/// BB2025 Throw Keg step sequence.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.bb2025.ThrowKeg`.
use ffb_model::enums::ApothecaryMode;
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::sequence::{Sequence, SequenceStep, labels};
use super::activation_sequence_builder::ActivationSequenceBuilder;

#[derive(Debug, Clone, Default)]
pub struct ThrowKegParams {
    pub player_id: Option<String>,
}

pub struct ThrowKeg;

impl ThrowKeg {
    pub fn new() -> Self { Self }

    pub fn build_sequence(params: &ThrowKegParams) -> Vec<SequenceStep> {
        let mut seq = Sequence::new();

        // 1-13 [ACTIVATION(END)]
        ActivationSequenceBuilder::new()
            .with_failure_label(labels::END)
            .add_to(&mut seq);

        // 14 THROW_KEG
        let mut p = vec![];
        if let Some(ref id) = params.player_id {
            p.push(StepParameter::TargetPlayerId(Some(id.clone())));
        }
        seq.add(StepId::ThrowKeg, p);
        // 15 STEADY_FOOTING (goto END on success)
        seq.add(StepId::SteadyFooting, vec![
            StepParameter::GotoLabelOnSuccess(labels::END.into()),
        ]);
        // 16 HANDLE_DROP_PLAYER_CONTEXT
        seq.add(StepId::HandleDropPlayerContext, vec![]);
        // 17 APOTHECARY (DEFENDER)
        seq.add(StepId::Apothecary, vec![
            StepParameter::ApothecaryMode(ApothecaryMode::Defender),
        ]);
        // 18 CATCH_SCATTER_THROW_IN
        seq.add(StepId::CatchScatterThrowIn, vec![]);
        // 19 END_THROW_KEG [END]
        seq.add_labelled(StepId::EndThrowKeg, labels::END, vec![]);
        seq.build()
    }
}

impl Default for ThrowKeg {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn throw_keg_has_19_steps_with_activation() {
        // Java pushSequence: ActivationSequenceBuilder.create()...addTo(sequence) (13) + 6 own steps = 19.
        let steps = ThrowKeg::build_sequence(&ThrowKegParams::default());
        assert_eq!(steps.len(), 19);
        assert_eq!(steps[0].step_id, StepId::InitActivation);
    }

    #[test]
    fn throw_keg_ends_with_end_throw_keg_labelled_end() {
        let steps = ThrowKeg::build_sequence(&ThrowKegParams::default());
        let last = steps.last().unwrap();
        assert_eq!(last.step_id, StepId::EndThrowKeg);
        assert_eq!(last.label.as_deref(), Some(labels::END));
    }

    #[test]
    fn throw_keg_step_follows_activation_sub_sequence() {
        let steps = ThrowKeg::build_sequence(&ThrowKegParams::default());
        assert_eq!(steps[13].step_id, StepId::ThrowKeg);
    }

    #[test]
    fn player_id_param_wired_when_provided() {
        let steps = ThrowKeg::build_sequence(&ThrowKegParams { player_id: Some("p1".into()) });
        let throw_keg_step = &steps[13];
        assert!(throw_keg_step.params.iter().any(|p| matches!(p, StepParameter::TargetPlayerId(Some(id)) if id == "p1")));
    }

    #[test]
    fn no_player_id_produces_empty_throw_keg_params() {
        let steps = ThrowKeg::build_sequence(&ThrowKegParams::default());
        assert!(steps[13].params.is_empty());
    }

    #[test]
    fn contains_apothecary_step() {
        let steps = ThrowKeg::build_sequence(&ThrowKegParams::default());
        assert!(steps.iter().any(|s| s.step_id == StepId::Apothecary));
    }
}
