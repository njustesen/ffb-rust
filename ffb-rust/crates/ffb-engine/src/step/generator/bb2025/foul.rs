/// BB2025 foul action step sequence.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.bb2025.Foul`.
use ffb_model::enums::ApothecaryMode;
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::sequence::{Sequence, SequenceStep, labels};
use super::activation_sequence_builder::ActivationSequenceBuilder;

/// Parameters — mirrors Java `Foul.SequenceParams`.
#[derive(Debug, Clone, Default)]
pub struct FoulParams {
    pub fouled_defender_id: Option<String>,
    pub using_chainsaw: bool,
}

pub struct Foul;

impl Foul {
    pub fn new() -> Self { Self }

    /// Build the foul step sequence (Java `pushSequence`).
    pub fn build_sequence(params: &FoulParams) -> Vec<SequenceStep> {
        let mut seq = Sequence::new();
        let fl = labels::END_FOULING;
        let apothecary_attacker = labels::APOTHECARY_ATTACKER;

        // 1 INIT_FOULING
        let mut init_params = vec![
            StepParameter::GotoLabelOnEnd(fl.into()),
            StepParameter::UsingChainsaw(params.using_chainsaw),
        ];
        if let Some(ref id) = params.fouled_defender_id {
            init_params.push(StepParameter::FoulDefenderId(id.clone()));
        }
        seq.add(StepId::InitFouling, init_params);

        // 2 [ACTIVATION(END_FOULING; SET_DEFENDER)]
        ActivationSequenceBuilder::new()
            .with_failure_label(fl)
            .with_eventual_defender(params.fouled_defender_id.clone())
            .add_to(&mut seq);

        // 3 FOUL_CHAINSAW (goto APOTHECARY_ATTACKER on failure)
        seq.add(StepId::FoulChainsaw, vec![
            StepParameter::GotoLabelOnFailure(apothecary_attacker.into()),
        ]);
        // 4 HANDLE_DROP_PLAYER_CONTEXT
        seq.add(StepId::HandleDropPlayerContext, vec![]);
        // 5 FOUL
        seq.add(StepId::Foul, vec![]);
        // 6 HANDLE_DROP_PLAYER_CONTEXT
        seq.add(StepId::HandleDropPlayerContext, vec![]);
        // 7 APOTHECARY (defender)
        seq.add(StepId::Apothecary, vec![
            StepParameter::ApothecaryMode(ApothecaryMode::Defender),
        ]);
        // 8 REFEREE
        seq.add(StepId::Referee, vec![
            StepParameter::GotoLabelOnEnd(fl.into()),
        ]);
        // 9 BRIBES
        seq.add(StepId::Bribes, vec![
            StepParameter::GotoLabelOnEnd(fl.into()),
        ]);
        // 10 EJECT_PLAYER
        seq.add(StepId::EjectPlayer, vec![
            StepParameter::GotoLabelOnEnd(fl.into()),
        ]);
        // 11 GOTO_LABEL → END_FOULING
        seq.jump(fl);
        // 12 APOTHECARY [APOTHECARY_ATTACKER] (attacker)
        seq.add_labelled(StepId::Apothecary, apothecary_attacker, vec![
            StepParameter::ApothecaryMode(ApothecaryMode::Attacker),
        ]);
        // 13 CATCH_SCATTER_THROW_IN [END_FOULING]
        seq.add_labelled(StepId::CatchScatterThrowIn, fl, vec![]);
        // 14 END_FOULING
        seq.add(StepId::EndFouling, vec![]);

        seq.build()
    }
}

impl Default for Foul {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn foul_sequence_starts_with_init_fouling() {
        let steps = Foul::build_sequence(&FoulParams::default());
        assert_eq!(steps[0].step_id, StepId::InitFouling);
    }

    #[test]
    fn foul_sequence_ends_with_end_fouling() {
        let steps = Foul::build_sequence(&FoulParams::default());
        assert_eq!(steps.last().unwrap().step_id, StepId::EndFouling);
    }

    #[test]
    fn foul_catch_scatter_is_labelled_end_fouling() {
        let steps = Foul::build_sequence(&FoulParams::default());
        // Find CATCH_SCATTER_THROW_IN that has the END_FOULING label
        let cst = steps.iter().find(|s| {
            s.step_id == StepId::CatchScatterThrowIn && s.label.as_deref() == Some(labels::END_FOULING)
        });
        assert!(cst.is_some());
    }

    #[test]
    fn foul_apothecary_attacker_is_labelled() {
        let steps = Foul::build_sequence(&FoulParams::default());
        let apo = steps.iter().find(|s| s.label.as_deref() == Some(labels::APOTHECARY_ATTACKER)).unwrap();
        assert_eq!(apo.step_id, StepId::Apothecary);
    }
}
