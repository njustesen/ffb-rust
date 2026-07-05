/// BB2025 end-player-action step sequence.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.bb2025.EndPlayerAction`.
use ffb_model::enums::ApothecaryMode;
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::sequence::{Sequence, SequenceStep, labels};

/// Parameters — mirrors Java `EndPlayerAction.SequenceParams`.
#[derive(Debug, Clone, Default)]
pub struct EndPlayerActionParams {
    pub feeding_allowed: bool,
    pub end_player_action: bool,
    pub end_turn: bool,
    pub check_forgo: bool,
}

pub struct EndPlayerAction;

impl EndPlayerAction {
    pub fn new() -> Self { Self }

    /// Build the end-player-action step sequence (Java `pushSequence`).
    pub fn build_sequence(params: &EndPlayerActionParams) -> Vec<SequenceStep> {
        let mut seq = Sequence::new();
        let fl = labels::END_FEEDING;

        // 1 REMOVE_TARGET_SELECTION_STATE
        seq.add(StepId::RemoveTargetSelectionState, vec![]);
        // 2 RESET_FUMBLEROOSKIE (END_PLAYER_ACTION)
        seq.add(StepId::ResetFumblerooskie, vec![
            StepParameter::EndPlayerAction(true),
        ]);
        // 3 INIT_FEEDING
        seq.add(StepId::InitFeeding, vec![
            StepParameter::GotoLabelOnEnd(fl.into()),
            StepParameter::FeedingAllowed(params.feeding_allowed),
            StepParameter::EndPlayerAction(params.end_player_action),
            StepParameter::EndTurn(params.end_turn),
        ]);
        // 4 APOTHECARY (FEEDING)
        seq.add(StepId::Apothecary, vec![
            StepParameter::ApothecaryMode(ApothecaryMode::Feeding),
        ]);
        // 5 CATCH_SCATTER_THROW_IN
        seq.add(StepId::CatchScatterThrowIn, vec![]);
        // 6 STALLING_PLAYER [END_FEEDING]
        seq.add_labelled(StepId::StallingPlayer, fl, vec![]);
        // 7 STEADY_FOOTING (HIT_PLAYER)
        seq.add(StepId::SteadyFooting, vec![
            StepParameter::ApothecaryMode(ApothecaryMode::HitPlayer),
        ]);
        // 8 PLACE_BALL
        seq.add(StepId::PlaceBall, vec![]);
        // 9 APOTHECARY (HIT_PLAYER)
        seq.add(StepId::Apothecary, vec![
            StepParameter::ApothecaryMode(ApothecaryMode::HitPlayer),
        ]);
        // 10 CATCH_SCATTER_THROW_IN
        seq.add(StepId::CatchScatterThrowIn, vec![]);
        // 11 END_FEEDING (CHECK_FORGO)
        seq.add(StepId::EndFeeding, vec![
            StepParameter::CheckForgo(params.check_forgo),
        ]);

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
    fn stalling_player_is_labelled_end_feeding() {
        let steps = EndPlayerAction::build_sequence(&EndPlayerActionParams::default());
        let sp = steps.iter().find(|s| s.step_id == StepId::StallingPlayer).unwrap();
        assert_eq!(sp.label.as_deref(), Some(labels::END_FEEDING));
    }

    #[test]
    fn total_step_count_is_eleven() {
        let steps = EndPlayerAction::build_sequence(&EndPlayerActionParams::default());
        assert_eq!(steps.len(), 11);
    }

    #[test]
    fn check_forgo_param_flows_to_end_feeding() {
        let params = EndPlayerActionParams { check_forgo: true, ..Default::default() };
        let steps = EndPlayerAction::build_sequence(&params);
        let end_feeding = steps.iter().find(|s| s.step_id == StepId::EndFeeding).unwrap();
        let has_check_forgo = end_feeding.params.iter().any(|p| matches!(p, StepParameter::CheckForgo(true)));
        assert!(has_check_forgo);
    }

    #[test]
    fn feeding_allowed_param_flows_to_init_feeding() {
        let params = EndPlayerActionParams { feeding_allowed: true, ..Default::default() };
        let steps = EndPlayerAction::build_sequence(&params);
        let init_feeding = steps.iter().find(|s| s.step_id == StepId::InitFeeding).unwrap();
        let has_feeding = init_feeding.params.iter().any(|p| matches!(p, StepParameter::FeedingAllowed(true)));
        assert!(has_feeding);
    }
}
