/// BB2020 Furious Outburst step sequence.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.bb2020.FuriousOutburst`.
use ffb_model::enums::ApothecaryMode;
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::sequence::{Sequence, SequenceStep, labels};

pub struct FuriousOutburst;

impl FuriousOutburst {
    pub fn new() -> Self { Self }

    pub fn build_sequence() -> Vec<SequenceStep> {
        let mut seq = Sequence::new();
        let fl = labels::END;

        // 1 INIT_FURIOUS_OUTBURST
        seq.add(StepId::InitFuriousOutburst, vec![StepParameter::GotoLabelOnEnd(fl.into())]);

        // ACTIVATION BLOCK (with GotoLabel, BloodLust no failure)
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
        // DUMP_OFF
        seq.add(StepId::DumpOff, vec![]);
        // FIRST_MOVE_FURIOUS_OUTBURST
        seq.add(StepId::FirstMoveFuriousOutburst, vec![StepParameter::GotoLabelOnEnd(fl.into())]);
        // CATCH_SCATTER_THROW_IN
        seq.add(StepId::CatchScatterThrowIn, vec![]);
        // BLOCK_STATISTICS [NEXT]
        seq.add_labelled(StepId::BlockStatistics, labels::NEXT, vec![]);
        // STAB → NEXT (skip DROP_FALLING_PLAYERS on success)
        seq.add(StepId::Stab, vec![StepParameter::GotoLabelOnSuccess(labels::NEXT.into())]);
        // DROP_FALLING_PLAYERS
        seq.add(StepId::DropFallingPlayers, vec![]);
        // HANDLE_DROP_PLAYER_CONTEXT
        seq.add(StepId::HandleDropPlayerContext, vec![]);
        // PLACE_BALL [NEXT]
        seq.add_labelled(StepId::PlaceBall, labels::NEXT, vec![]);
        // APOTHECARY (defender)
        seq.add(StepId::Apothecary, vec![StepParameter::ApothecaryMode(ApothecaryMode::Defender)]);
        // CATCH_SCATTER_THROW_IN
        seq.add(StepId::CatchScatterThrowIn, vec![]);
        // SECOND_MOVE_FURIOUS_OUTBURST
        seq.add(StepId::SecondMoveFuriousOutburst, vec![StepParameter::GotoLabelOnEnd(fl.into())]);
        // CATCH_SCATTER_THROW_IN
        seq.add(StepId::CatchScatterThrowIn, vec![]);
        // END_FURIOUS_OUTBURST [END]
        seq.add_labelled(StepId::EndFuriousOutburst, labels::END, vec![]);

        seq.build()
    }
}

impl Default for FuriousOutburst {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn furious_outburst_ends_with_end_furious_outburst_labelled_end() {
        let steps = FuriousOutburst::build_sequence();
        let last = steps.last().unwrap();
        assert_eq!(last.step_id, StepId::EndFuriousOutburst);
        assert_eq!(last.label.as_deref(), Some(labels::END));
    }

    #[test]
    fn furious_outburst_has_activation_block() {
        let steps = FuriousOutburst::build_sequence();
        assert!(steps.iter().any(|s| s.step_id == StepId::InitActivation));
    }

    #[test]
    fn furious_outburst_place_ball_is_labelled_next() {
        let steps = FuriousOutburst::build_sequence();
        let pb = steps.iter().find(|s| s.step_id == StepId::PlaceBall && s.label.as_deref() == Some(labels::NEXT));
        assert!(pb.is_some());
    }

    #[test]
    fn furious_outburst_blood_lust_has_no_failure_label() {
        let steps = FuriousOutburst::build_sequence();
        let bl = steps.iter().find(|s| s.step_id == StepId::BloodLust).unwrap();
        assert!(!bl.params.iter().any(|p| matches!(p, StepParameter::GotoLabelOnFailure(_))));
    }

    #[test]
    fn furious_outburst_starts_with_init_furious_outburst() {
        let steps = FuriousOutburst::build_sequence();
        assert_eq!(steps[0].step_id, StepId::InitFuriousOutburst);
    }

    #[test]
    fn furious_outburst_has_second_move() {
        let steps = FuriousOutburst::build_sequence();
        assert!(steps.iter().any(|s| s.step_id == StepId::SecondMoveFuriousOutburst));
    }
}
