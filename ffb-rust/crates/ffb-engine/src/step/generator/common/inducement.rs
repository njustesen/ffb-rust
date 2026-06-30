/// Builds the generic inducement activation step sequence.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.common.Inducement`.
use ffb_model::enums::{ApothecaryMode, InducementPhase};
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::sequence::{Sequence, SequenceStep};

/// Parameters for the Inducement sequence — mirrors Java `Inducement.SequenceParams`.
#[derive(Debug, Clone)]
pub struct InducementParams {
    pub inducement_phase: InducementPhase,
    pub home_team: bool,
    pub check_forgo: bool,
}

pub struct Inducement;

impl Inducement {
    pub fn new() -> Self { Self }

    /// Build the inducement step sequence (Java `pushSequence`).
    pub fn build_sequence(params: &InducementParams) -> Vec<SequenceStep> {
        let mut seq = Sequence::new();

        // 1 INIT_INDUCEMENT
        seq.add(StepId::InitInducement, vec![
            StepParameter::InducementPhase(params.inducement_phase),
            StepParameter::HomeTeam(params.home_team),
        ]);
        // 2 APOTHECARY (DEFENDER)
        seq.add(StepId::Apothecary, vec![
            StepParameter::ApothecaryMode(ApothecaryMode::Defender),
        ]);
        // 3 APOTHECARY (ATTACKER)
        seq.add(StepId::Apothecary, vec![
            StepParameter::ApothecaryMode(ApothecaryMode::Attacker),
        ]);
        // 4 CATCH_SCATTER_THROW_IN
        seq.add(StepId::CatchScatterThrowIn, vec![]);
        // 5 END_INDUCEMENT
        seq.add(StepId::EndInducement, vec![
            StepParameter::CheckForgo(params.check_forgo),
        ]);

        seq.build()
    }
}

impl Default for Inducement {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inducement_sequence_has_five_steps() {
        let params = InducementParams {
            inducement_phase: InducementPhase::BeforeSetup,
            home_team: true,
            check_forgo: false,
        };
        let steps = Inducement::build_sequence(&params);
        assert_eq!(steps.len(), 5);
    }

    #[test]
    fn inducement_sequence_starts_with_init_inducement() {
        let params = InducementParams {
            inducement_phase: InducementPhase::BeforeSetup,
            home_team: false,
            check_forgo: false,
        };
        let steps = Inducement::build_sequence(&params);
        assert_eq!(steps[0].step_id, StepId::InitInducement);
    }

    #[test]
    fn inducement_sequence_ends_with_end_inducement() {
        let params = InducementParams {
            inducement_phase: InducementPhase::StartOfOwnTurn,
            home_team: true,
            check_forgo: true,
        };
        let steps = Inducement::build_sequence(&params);
        assert_eq!(steps[4].step_id, StepId::EndInducement);
    }
}
