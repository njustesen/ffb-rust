/// BB2016 end-player-action step sequence.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.bb2016.EndPlayerAction`.
use ffb_model::enums::ApothecaryMode;
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::sequence::{Sequence, SequenceStep, labels};

/// Parameters for the BB2016 EndPlayerAction sequence.
#[derive(Debug, Clone, Default)]
pub struct EndPlayerActionParams {
    pub feeding_allowed: bool,
    pub end_player_action: bool,
    pub end_turn: bool,
}

pub struct EndPlayerAction;

impl EndPlayerAction {
    pub fn new() -> Self { Self }

    /// Build the BB2016 end-player-action step sequence (Java `pushSequence`).
    pub fn build_sequence(params: &EndPlayerActionParams) -> Vec<SequenceStep> {
        let mut seq = Sequence::new();

        // 1 INIT_FEEDING
        seq.add(StepId::InitFeeding, vec![
            StepParameter::GotoLabelOnEnd(labels::END_FEEDING.into()),
            StepParameter::FeedingAllowed(params.feeding_allowed),
            StepParameter::EndPlayerAction(params.end_player_action),
            StepParameter::EndTurn(params.end_turn),
        ]);
        // 2 APOTHECARY (FEEDING)
        seq.add(StepId::Apothecary, vec![
            StepParameter::ApothecaryMode(ApothecaryMode::Feeding),
        ]);
        // 3 CATCH_SCATTER_THROW_IN
        seq.add(StepId::CatchScatterThrowIn, vec![]);
        // 4 END_FEEDING [END_FEEDING]
        seq.add_labelled(StepId::EndFeeding, labels::END_FEEDING, vec![]);

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
    fn end_player_action_starts_with_init_feeding() {
        let steps = EndPlayerAction::build_sequence(&EndPlayerActionParams::default());
        assert_eq!(steps[0].step_id, StepId::InitFeeding);
    }

    #[test]
    fn end_player_action_ends_with_end_feeding_labelled() {
        let steps = EndPlayerAction::build_sequence(&EndPlayerActionParams::default());
        let last = steps.last().unwrap();
        assert_eq!(last.step_id, StepId::EndFeeding);
        assert_eq!(last.label.as_deref(), Some(labels::END_FEEDING));
    }

    #[test]
    fn end_player_action_has_4_steps() {
        let steps = EndPlayerAction::build_sequence(&EndPlayerActionParams::default());
        assert_eq!(steps.len(), 4);
    }

    #[test]
    fn end_player_action_apothecary_has_feeding_mode() {
        let steps = EndPlayerAction::build_sequence(&EndPlayerActionParams::default());
        let apo = steps.iter().find(|s| s.step_id == StepId::Apothecary).unwrap();
        assert!(apo.params.iter().any(|p| matches!(p, StepParameter::ApothecaryMode(ffb_model::enums::ApothecaryMode::Feeding))));
    }
}
