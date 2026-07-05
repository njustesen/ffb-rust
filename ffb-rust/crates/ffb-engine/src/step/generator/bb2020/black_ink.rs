/// BB2020 Black Ink step sequence.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.bb2020.BlackInk`.
use ffb_model::enums::ApothecaryMode;
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::sequence::{Sequence, SequenceStep, labels};

#[derive(Debug, Clone, Default)]
pub struct BlackInkParams {
    pub failure_label: String,
    pub old_player_state: Option<ffb_model::enums::PlayerState>,
}

pub struct BlackInk;

impl BlackInk {
    pub fn new() -> Self { Self }

    pub fn build_sequence(params: &BlackInkParams) -> Vec<SequenceStep> {
        let mut seq = Sequence::new();
        let fl = labels::END;

        // ACTIVATION BLOCK (with GotoLabel, BloodLust no failure label)
        let fl_s: String = fl.into();
        seq.add(StepId::InitActivation, vec![]);
        seq.add(StepId::AnimalSavagery, vec![StepParameter::GotoLabelOnFailure(fl_s.clone())]);
        seq.add(StepId::HandleDropPlayerContext, vec![]);
        seq.add(StepId::PlaceBall, vec![]);
        seq.add(StepId::Apothecary, vec![StepParameter::ApothecaryMode(ApothecaryMode::AnimalSavagery)]);
        seq.add(StepId::CatchScatterThrowIn, vec![]);
        seq.add(StepId::GotoLabel, vec![
            StepParameter::GotoLabel(labels::NEXT.into()),
            StepParameter::AlternateGotoLabel(fl_s.clone()),
        ]);
        seq.add_labelled(StepId::BoneHead, labels::NEXT, vec![StepParameter::GotoLabelOnFailure(fl_s.clone())]);
        seq.add(StepId::ReallyStupid, vec![StepParameter::GotoLabelOnFailure(fl_s.clone())]);
        seq.add(StepId::TakeRoot, vec![]);
        seq.add(StepId::UnchannelledFury, vec![StepParameter::GotoLabelOnFailure(fl_s.clone())]);
        seq.add(StepId::BloodLust, vec![]);  // no failure label

        // FOUL_APPEARANCE → END
        seq.add(StepId::FoulAppearance, vec![StepParameter::GotoLabelOnFailure(fl.into())]);

        // BLACK_INK [END] (→ failure_label, OldPlayerState)
        let mut ink_params = vec![StepParameter::GotoLabelOnFailure(params.failure_label.clone())];
        if let Some(state) = params.old_player_state {
            ink_params.push(StepParameter::OldPlayerState(state));
        }
        seq.add_labelled(StepId::BlackInk, labels::END, ink_params);

        seq.build()
    }
}

impl Default for BlackInk {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn black_ink_has_activation_block() {
        let steps = BlackInk::build_sequence(&BlackInkParams::default());
        assert!(steps.iter().any(|s| s.step_id == StepId::InitActivation));
    }

    #[test]
    fn black_ink_ends_with_black_ink_labelled_end() {
        let steps = BlackInk::build_sequence(&BlackInkParams { failure_label: "X".into(), old_player_state: None });
        let last = steps.last().unwrap();
        assert_eq!(last.step_id, StepId::BlackInk);
        assert_eq!(last.label.as_deref(), Some(labels::END));
    }

    #[test]
    fn black_ink_has_foul_appearance() {
        let steps = BlackInk::build_sequence(&BlackInkParams::default());
        assert!(steps.iter().any(|s| s.step_id == StepId::FoulAppearance));
    }

    #[test]
    fn black_ink_blood_lust_has_no_failure_label() {
        let steps = BlackInk::build_sequence(&BlackInkParams::default());
        let bl = steps.iter().find(|s| s.step_id == StepId::BloodLust).unwrap();
        assert!(!bl.params.iter().any(|p| matches!(p, StepParameter::GotoLabelOnFailure(_))));
    }

    #[test]
    fn failure_label_passed_to_black_ink_step() {
        let steps = BlackInk::build_sequence(&BlackInkParams { failure_label: "theLabel".into(), old_player_state: None });
        let ink = steps.iter().find(|s| s.step_id == StepId::BlackInk).unwrap();
        let has = ink.params.iter().any(|p| matches!(p, StepParameter::GotoLabelOnFailure(l) if l == "theLabel"));
        assert!(has);
    }

    #[test]
    fn old_player_state_added_when_some() {
        use ffb_model::enums::{PlayerState, PS_STANDING};
        let state = PlayerState::new(PS_STANDING);
        let steps = BlackInk::build_sequence(&BlackInkParams { failure_label: "X".into(), old_player_state: Some(state) });
        let ink = steps.iter().find(|s| s.step_id == StepId::BlackInk).unwrap();
        let has = ink.params.iter().any(|p| matches!(p, StepParameter::OldPlayerState(_)));
        assert!(has);
    }
}
