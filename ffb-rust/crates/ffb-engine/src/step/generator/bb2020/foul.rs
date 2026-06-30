/// BB2020 foul action step sequence.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.bb2020.Foul`.
use ffb_model::enums::ApothecaryMode;
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::sequence::{Sequence, SequenceStep, labels};

/// Parameters for the BB2020 Foul sequence.
#[derive(Debug, Clone, Default)]
pub struct FoulParams {
    pub foul_defender_id: Option<String>,
    pub using_chainsaw: bool,
}

pub struct Foul;

impl Foul {
    pub fn new() -> Self { Self }

    pub fn build_sequence(params: &FoulParams) -> Vec<SequenceStep> {
        let mut seq = Sequence::new();
        let fl = labels::END_FOULING;
        let apothecary_attacker = labels::APOTHECARY_ATTACKER;

        // 1 INIT_FOULING
        let mut init_params = vec![
            StepParameter::GotoLabelOnEnd(fl.into()),
            StepParameter::UsingChainsaw(params.using_chainsaw),
        ];
        if let Some(ref id) = params.foul_defender_id {
            init_params.push(StepParameter::FoulDefenderId(id.clone()));
        }
        seq.add(StepId::InitFouling, init_params);

        // 2-13 ACTIVATION BLOCK (with GotoLabel, SetDefender)
        let fl_s: String = fl.into();
        seq.add(StepId::InitActivation, vec![]);
        seq.add(StepId::AnimalSavagery, vec![StepParameter::GotoLabelOnFailure(fl_s.clone())]);
        seq.add(StepId::HandleDropPlayerContext, vec![]);
        seq.add(StepId::PlaceBall, vec![]);
        seq.add(StepId::Apothecary, vec![StepParameter::ApothecaryMode(ApothecaryMode::AnimalSavagery)]);
        seq.add(StepId::CatchScatterThrowIn, vec![]);
        // SetDefender with block_defender_id (unconditional, matches Java which passes null if none)
        seq.add(StepId::SetDefender, vec![StepParameter::BlockDefenderId(
            params.foul_defender_id.clone().unwrap_or_default(),
        )]);
        seq.add(StepId::GotoLabel, vec![
            StepParameter::GotoLabel(labels::NEXT.into()),
            StepParameter::AlternateGotoLabel(fl_s.clone()),
        ]);
        seq.add_labelled(StepId::BoneHead, labels::NEXT, vec![StepParameter::GotoLabelOnFailure(fl_s.clone())]);
        seq.add(StepId::ReallyStupid, vec![StepParameter::GotoLabelOnFailure(fl_s.clone())]);
        seq.add(StepId::TakeRoot, vec![]);
        seq.add(StepId::UnchannelledFury, vec![StepParameter::GotoLabelOnFailure(fl_s.clone())]);
        seq.add(StepId::BloodLust, vec![StepParameter::GotoLabelOnFailure(fl_s.clone())]);

        // FOUL_CHAINSAW
        seq.add(StepId::FoulChainsaw, vec![StepParameter::GotoLabelOnFailure(apothecary_attacker.into())]);
        // HANDLE_DROP_PLAYER_CONTEXT
        seq.add(StepId::HandleDropPlayerContext, vec![]);
        // FOUL
        seq.add(StepId::Foul, vec![]);
        // HANDLE_DROP_PLAYER_CONTEXT
        seq.add(StepId::HandleDropPlayerContext, vec![]);
        // APOTHECARY (defender)
        seq.add(StepId::Apothecary, vec![StepParameter::ApothecaryMode(ApothecaryMode::Defender)]);
        // REFEREE
        seq.add(StepId::Referee, vec![StepParameter::GotoLabelOnEnd(fl.into())]);
        // BRIBES
        seq.add(StepId::Bribes, vec![StepParameter::GotoLabelOnEnd(fl.into())]);
        // EJECT_PLAYER
        seq.add(StepId::EjectPlayer, vec![StepParameter::GotoLabelOnEnd(fl.into())]);
        // GOTO → END_FOULING
        seq.jump(fl);
        // APOTHECARY [APOTHECARY_ATTACKER] (attacker)
        seq.add_labelled(StepId::Apothecary, apothecary_attacker, vec![
            StepParameter::ApothecaryMode(ApothecaryMode::Attacker),
        ]);
        // CATCH_SCATTER_THROW_IN [END_FOULING]
        seq.add_labelled(StepId::CatchScatterThrowIn, fl, vec![]);
        // END_FOULING
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
    fn foul_has_activation_block() {
        let steps = Foul::build_sequence(&FoulParams::default());
        assert!(steps.iter().any(|s| s.step_id == StepId::InitActivation));
    }

    #[test]
    fn foul_apothecary_attacker_is_labelled() {
        let steps = Foul::build_sequence(&FoulParams::default());
        let apo = steps.iter().find(|s| s.label.as_deref() == Some(labels::APOTHECARY_ATTACKER)).unwrap();
        assert_eq!(apo.step_id, StepId::Apothecary);
    }

    #[test]
    fn foul_catch_scatter_labelled_end_fouling() {
        let steps = Foul::build_sequence(&FoulParams::default());
        let cst = steps.iter().find(|s| {
            s.step_id == StepId::CatchScatterThrowIn && s.label.as_deref() == Some(labels::END_FOULING)
        });
        assert!(cst.is_some());
    }
}
