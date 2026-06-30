/// BB2016 foul action step sequence.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.bb2016.Foul`.
use ffb_model::enums::ApothecaryMode;
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::sequence::{Sequence, SequenceStep, labels};

/// Parameters for the BB2016 Foul sequence.
#[derive(Debug, Clone, Default)]
pub struct FoulParams {
    pub fouled_defender_id: Option<String>,
    pub using_chainsaw: bool,
}

pub struct Foul;

impl Foul {
    pub fn new() -> Self { Self }

    /// Build the BB2016 foul step sequence (Java `pushSequence`).
    pub fn build_sequence(params: &FoulParams) -> Vec<SequenceStep> {
        let mut seq = Sequence::new();
        let fl = labels::END_FOULING;

        // 1 INIT_FOULING
        let mut init_params = vec![
            StepParameter::GotoLabelOnEnd(fl.into()),
            StepParameter::UsingChainsaw(params.using_chainsaw),
        ];
        if let Some(ref id) = params.fouled_defender_id {
            init_params.push(StepParameter::FoulDefenderId(id.clone()));
        }
        seq.add(StepId::InitFouling, init_params);

        // 2 BONE_HEAD
        seq.add(StepId::BoneHead, vec![StepParameter::GotoLabelOnFailure(fl.into())]);
        // 3 REALLY_STUPID
        seq.add(StepId::ReallyStupid, vec![StepParameter::GotoLabelOnFailure(fl.into())]);
        // 4 TAKE_ROOT
        seq.add(StepId::TakeRoot, vec![]);
        // 5 WILD_ANIMAL
        seq.add(StepId::WildAnimal, vec![StepParameter::GotoLabelOnFailure(fl.into())]);
        // 6 BLOOD_LUST (no failure label — feeding happens regardless)
        seq.add(StepId::BloodLust, vec![]);
        // 7 FOUL_CHAINSAW
        seq.add(StepId::FoulChainsaw, vec![
            StepParameter::GotoLabelOnFailure(labels::APOTHECARY_ATTACKER.into()),
        ]);
        // 8 FOUL
        seq.add(StepId::Foul, vec![]);
        // 9 APOTHECARY (defender)
        seq.add(StepId::Apothecary, vec![StepParameter::ApothecaryMode(ApothecaryMode::Defender)]);
        // 10 REFEREE
        seq.add(StepId::Referee, vec![StepParameter::GotoLabelOnEnd(fl.into())]);
        // 11 BRIBES
        seq.add(StepId::Bribes, vec![StepParameter::GotoLabelOnEnd(fl.into())]);
        // 12 EJECT_PLAYER
        seq.add(StepId::EjectPlayer, vec![StepParameter::GotoLabelOnEnd(fl.into())]);
        // 13 GOTO_LABEL → END_FOULING
        seq.jump(fl);
        // 14 APOTHECARY [APOTHECARY_ATTACKER] (attacker)
        seq.add_labelled(StepId::Apothecary, labels::APOTHECARY_ATTACKER, vec![
            StepParameter::ApothecaryMode(ApothecaryMode::Attacker),
        ]);
        // 15 CATCH_SCATTER_THROW_IN [END_FOULING]
        seq.add_labelled(StepId::CatchScatterThrowIn, fl, vec![]);
        // 16 END_FOULING
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
    fn foul_starts_with_init_fouling() {
        let steps = Foul::build_sequence(&FoulParams::default());
        assert_eq!(steps[0].step_id, StepId::InitFouling);
    }

    #[test]
    fn foul_ends_with_end_fouling() {
        let steps = Foul::build_sequence(&FoulParams::default());
        assert_eq!(steps.last().unwrap().step_id, StepId::EndFouling);
    }

    #[test]
    fn foul_has_bone_head() {
        let steps = Foul::build_sequence(&FoulParams::default());
        assert!(steps.iter().any(|s| s.step_id == StepId::BoneHead));
    }

    #[test]
    fn foul_catch_scatter_is_labelled_end_fouling() {
        let steps = Foul::build_sequence(&FoulParams::default());
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
