/// BB2025 Furious Outburst step sequence.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.bb2025.FuriousOutburst`.
use ffb_model::enums::ApothecaryMode;
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::sequence::{Sequence, SequenceStep, labels};

pub struct FuriousOutburst;

impl FuriousOutburst {
    pub fn new() -> Self { Self }

    pub fn build_sequence() -> Vec<SequenceStep> {
        let mut seq = Sequence::new();
        // 1 INIT_FURIOUS_OUTBURST
        seq.add(StepId::InitFuriousOutburst, vec![
            StepParameter::GotoLabelOnEnd(labels::END.into()),
        ]);
        // 2 FOUL_APPEARANCE (goto END on failure)
        seq.add(StepId::FoulAppearance, vec![
            StepParameter::GotoLabelOnFailure(labels::END.into()),
        ]);
        // 3 FIRST_MOVE_FURIOUS_OUTBURST (goto END on end)
        seq.add(StepId::FirstMoveFuriousOutburst, vec![
            StepParameter::GotoLabelOnEnd(labels::END.into()),
        ]);
        // 4 CATCH_SCATTER_THROW_IN
        seq.add(StepId::CatchScatterThrowIn, vec![]);
        // 5 DUMP_OFF
        seq.add(StepId::DumpOff, vec![]);
        // 6 BLOCK_STATISTICS [NEXT]
        seq.add_labelled(StepId::BlockStatistics, labels::NEXT, vec![]);
        // 7 STAB (goto NEXT on success — skip DROP_FALLING_PLAYERS)
        seq.add(StepId::Stab, vec![
            StepParameter::GotoLabelOnSuccess(labels::NEXT.into()),
        ]);
        // 8 DROP_FALLING_PLAYERS
        seq.add(StepId::DropFallingPlayers, vec![]);
        // 9 HANDLE_DROP_PLAYER_CONTEXT
        seq.add(StepId::HandleDropPlayerContext, vec![]);
        // 10 PLACE_BALL [NEXT]
        seq.add_labelled(StepId::PlaceBall, labels::NEXT, vec![]);
        // 11 APOTHECARY (DEFENDER)
        seq.add(StepId::Apothecary, vec![
            StepParameter::ApothecaryMode(ApothecaryMode::Defender),
        ]);
        // 12 CATCH_SCATTER_THROW_IN
        seq.add(StepId::CatchScatterThrowIn, vec![]);
        // 13 SECOND_MOVE_FURIOUS_OUTBURST (goto END on end)
        seq.add(StepId::SecondMoveFuriousOutburst, vec![
            StepParameter::GotoLabelOnEnd(labels::END.into()),
        ]);
        // 14 CATCH_SCATTER_THROW_IN
        seq.add(StepId::CatchScatterThrowIn, vec![]);
        // 15 END_FURIOUS_OUTBURST [END]
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
    fn furious_outburst_has_15_steps() {
        let steps = FuriousOutburst::build_sequence();
        assert_eq!(steps.len(), 15);
    }

    #[test]
    fn furious_outburst_ends_with_end_furious_outburst_labelled_end() {
        let steps = FuriousOutburst::build_sequence();
        let last = steps.last().unwrap();
        assert_eq!(last.step_id, StepId::EndFuriousOutburst);
        assert_eq!(last.label.as_deref(), Some(labels::END));
    }

    #[test]
    fn furious_outburst_place_ball_is_labelled_next() {
        let steps = FuriousOutburst::build_sequence();
        let pb = steps.iter().find(|s| {
            s.step_id == StepId::PlaceBall && s.label.as_deref() == Some(labels::NEXT)
        });
        assert!(pb.is_some());
    }
}
