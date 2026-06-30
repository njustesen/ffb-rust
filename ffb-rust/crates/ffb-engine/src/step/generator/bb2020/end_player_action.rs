/// BB2020 end-player-action step sequence.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.bb2020.EndPlayerAction`.
use ffb_model::enums::ApothecaryMode;
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::sequence::{Sequence, SequenceStep, labels};

/// Parameters for the BB2020 EndPlayerAction sequence.
#[derive(Debug, Clone, Default)]
pub struct EndPlayerActionParams {
    pub feeding_allowed: bool,
    pub end_player_action: bool,
    pub end_turn: bool,
}

pub struct EndPlayerAction;

impl EndPlayerAction {
    pub fn new() -> Self { Self }

    pub fn build_sequence(params: &EndPlayerActionParams) -> Vec<SequenceStep> {
        let mut seq = Sequence::new();
        let fl = labels::END_FEEDING;

        // 1 REMOVE_TARGET_SELECTION_STATE
        seq.add(StepId::RemoveTargetSelectionState, vec![]);
        // 2 RESET_FUMBLEROOSKIE (end player action, reset_for_failed_block=false)
        seq.add(StepId::ResetFumblerooskie, vec![
            StepParameter::ResetForFailedBlock(false),
            StepParameter::EndPlayerAction(true),
        ]);
        // 3 INIT_FEEDING
        seq.add(StepId::InitFeeding, vec![
            StepParameter::GotoLabelOnEnd(fl.into()),
            StepParameter::FeedingAllowed(params.feeding_allowed),
            StepParameter::EndPlayerAction(params.end_player_action),
            StepParameter::EndTurn(params.end_turn),
        ]);
        // 4 APOTHECARY (feeding)
        seq.add(StepId::Apothecary, vec![StepParameter::ApothecaryMode(ApothecaryMode::Feeding)]);
        // 5 CATCH_SCATTER_THROW_IN
        seq.add(StepId::CatchScatterThrowIn, vec![]);
        // 6 CHECK_STALLING [END_FEEDING] (BB2020-specific, replaces StallingPlayer)
        seq.add_labelled(StepId::CheckStalling, fl, vec![StepParameter::IgnoreActedFlag(false)]);
        // 7 END_FEEDING
        seq.add(StepId::EndFeeding, vec![]);

        seq.build()
    }
}

impl Default for EndPlayerAction {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn end_player_action_starts_with_remove_target_selection_state() {
        let steps = EndPlayerAction::build_sequence(&EndPlayerActionParams::default());
        assert_eq!(steps[0].step_id, StepId::RemoveTargetSelectionState);
    }

    #[test]
    fn end_player_action_ends_with_end_feeding() {
        let steps = EndPlayerAction::build_sequence(&EndPlayerActionParams::default());
        assert_eq!(steps.last().unwrap().step_id, StepId::EndFeeding);
    }

    #[test]
    fn check_stalling_is_labelled_end_feeding() {
        let steps = EndPlayerAction::build_sequence(&EndPlayerActionParams::default());
        let cs = steps.iter().find(|s| s.step_id == StepId::CheckStalling).unwrap();
        assert_eq!(cs.label.as_deref(), Some(labels::END_FEEDING));
    }

    #[test]
    fn end_player_action_has_no_steady_footing() {
        let steps = EndPlayerAction::build_sequence(&EndPlayerActionParams::default());
        assert!(!steps.iter().any(|s| s.step_id == StepId::SteadyFooting));
    }
}
