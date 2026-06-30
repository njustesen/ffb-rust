/// Builds the Pile Driver foul step sequence (BB2020/BB2025).
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.mixed.PileDriver`.
use ffb_model::enums::ApothecaryMode;
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::sequence::{Sequence, SequenceStep, labels};

/// Parameters for the mixed PileDriver sequence.
#[derive(Debug, Clone, Default)]
pub struct PileDriverParams {
    /// ID of the player being targeted by Pile Driver.
    pub target_player_id: Option<String>,
}

pub struct PileDriver;

impl PileDriver {
    pub fn new() -> Self { Self }

    /// Build the mixed pile-driver foul step sequence (Java `pushSequence`).
    pub fn build_sequence(params: &PileDriverParams) -> Vec<SequenceStep> {
        let mut seq = Sequence::new();

        // 1 PILE_DRIVER → SKIP_PILE_DRIVER on end
        let mut pd_params = vec![
            StepParameter::GotoLabelOnEnd(labels::SKIP_PILE_DRIVER.into()),
        ];
        if let Some(ref id) = params.target_player_id {
            pd_params.push(StepParameter::PlayerId(id.clone()));
        }
        seq.add(StepId::PileDriver, pd_params);

        // 2 FOUL_CHAINSAW → APOTHECARY_ATTACKER on failure
        seq.add(StepId::FoulChainsaw, vec![
            StepParameter::GotoLabelOnFailure(labels::APOTHECARY_ATTACKER.into()),
        ]);
        // 3 HANDLE_DROP_PLAYER_CONTEXT
        seq.add(StepId::HandleDropPlayerContext, vec![]);
        // 4 FOUL
        seq.add(StepId::Foul, vec![]);
        // 5 HANDLE_DROP_PLAYER_CONTEXT
        seq.add(StepId::HandleDropPlayerContext, vec![]);
        // 6 APOTHECARY (DEFENDER)
        seq.add(StepId::Apothecary, vec![StepParameter::ApothecaryMode(ApothecaryMode::Defender)]);
        // 7 REFEREE → END_FOULING on end
        seq.add(StepId::Referee, vec![StepParameter::GotoLabelOnEnd(labels::END_FOULING.into())]);
        // 8 BRIBES → END_FOULING on end
        seq.add(StepId::Bribes, vec![StepParameter::GotoLabelOnEnd(labels::END_FOULING.into())]);
        // 9 EJECT_PLAYER → END_FOULING on end
        seq.add(StepId::EjectPlayer, vec![StepParameter::GotoLabelOnEnd(labels::END_FOULING.into())]);
        // 10 GOTO_LABEL → END_FOULING
        seq.jump(labels::END_FOULING);
        // 11 [APOTHECARY_ATTACKER] APOTHECARY (ATTACKER)
        seq.add_labelled(StepId::Apothecary, labels::APOTHECARY_ATTACKER, vec![
            StepParameter::ApothecaryMode(ApothecaryMode::Attacker),
        ]);
        // 12 [END_FOULING] DROP_ACTING_PLAYER
        seq.add_labelled(StepId::DropActingPlayer, labels::END_FOULING, vec![]);
        // 13 [APOTHECARY_ATTACKER] APOTHECARY (DROPPED_BY_OWN_PLAYER)
        seq.add_labelled(StepId::Apothecary, labels::APOTHECARY_ATTACKER, vec![
            StepParameter::ApothecaryMode(ApothecaryMode::DroppedByOwnPlayer),
        ]);
        // 14 PLACE_BALL
        seq.add(StepId::PlaceBall, vec![]);
        // 15 [SKIP_PILE_DRIVER] CATCH_SCATTER_THROW_IN
        seq.add_labelled(StepId::CatchScatterThrowIn, labels::SKIP_PILE_DRIVER, vec![]);
        // 16 END_FOULING
        seq.add(StepId::EndFouling, vec![]);

        seq.build()
    }
}

impl Default for PileDriver {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pile_driver_starts_with_pile_driver() {
        let steps = PileDriver::build_sequence(&PileDriverParams::default());
        assert_eq!(steps[0].step_id, StepId::PileDriver);
    }

    #[test]
    fn pile_driver_ends_with_end_fouling() {
        let steps = PileDriver::build_sequence(&PileDriverParams::default());
        assert_eq!(steps.last().unwrap().step_id, StepId::EndFouling);
    }

    #[test]
    fn pile_driver_skip_pile_driver_labels_catch_scatter() {
        let steps = PileDriver::build_sequence(&PileDriverParams::default());
        let s = steps.iter().find(|s| s.label.as_deref() == Some(labels::SKIP_PILE_DRIVER)).unwrap();
        assert_eq!(s.step_id, StepId::CatchScatterThrowIn);
    }

    #[test]
    fn pile_driver_apothecary_attacker_labels_first_apothecary_attacker() {
        let steps = PileDriver::build_sequence(&PileDriverParams::default());
        let s = steps.iter().find(|s| s.label.as_deref() == Some(labels::APOTHECARY_ATTACKER)).unwrap();
        assert_eq!(s.step_id, StepId::Apothecary);
    }

    #[test]
    fn pile_driver_end_fouling_labels_drop_acting_player() {
        let steps = PileDriver::build_sequence(&PileDriverParams::default());
        let s = steps.iter().find(|s| s.label.as_deref() == Some(labels::END_FOULING)).unwrap();
        assert_eq!(s.step_id, StepId::DropActingPlayer);
    }

    #[test]
    fn pile_driver_has_two_handle_drop_player_context_steps() {
        let steps = PileDriver::build_sequence(&PileDriverParams::default());
        let count = steps.iter().filter(|s| s.step_id == StepId::HandleDropPlayerContext).count();
        assert_eq!(count, 2);
    }
}
