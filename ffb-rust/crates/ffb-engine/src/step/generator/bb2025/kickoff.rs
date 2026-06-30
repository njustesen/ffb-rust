/// BB2025 kickoff step sequence.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.bb2025.Kickoff`.
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::sequence::{Sequence, SequenceStep, labels};

/// Parameters — mirrors Java `Kickoff.SequenceParams`.
#[derive(Debug, Clone, Default)]
pub struct KickoffParams {
    /// Whether to include the COIN_CHOICE and RECEIVE_CHOICE steps (opening kickoff only).
    pub with_coin_choice: bool,
    /// Whether to use the ASK_AFTER variant of the scatter roll (game option).
    pub ask_after_roll: bool,
}

pub struct Kickoff;

impl Kickoff {
    pub fn new() -> Self { Self }

    /// Build the kickoff step sequence (Java `pushSequence`).
    pub fn build_sequence(params: &KickoffParams) -> Vec<SequenceStep> {
        let mut seq = Sequence::new();
        let end_kickoff = labels::END_KICKOFF;
        let blitz_turn = labels::BLITZ_TURN;
        let kickoff_animation = labels::KICKOFF_ANIMATION;

        // 1 COIN_CHOICE* (only if withCoinChoice)
        if params.with_coin_choice {
            seq.add(StepId::CoinChoice, vec![]);
            // 2 RECEIVE_CHOICE*
            seq.add(StepId::ReceiveChoice, vec![]);
        }
        // 3 INIT_KICKOFF
        seq.add(StepId::InitKickoff, vec![]);
        // 4 SETUP (kicking)
        seq.add(StepId::Setup, vec![]);
        // 5 SETUP (receiving)
        seq.add(StepId::Setup, vec![]);
        // 6 SWARMING (recv=false)
        seq.add(StepId::Swarming, vec![
            StepParameter::HandleReceivingTeam(false),
        ]);
        // 7 SWARMING (recv=true)
        seq.add(StepId::Swarming, vec![
            StepParameter::HandleReceivingTeam(true),
        ]);
        // 8 MASTER_CHEF
        seq.add(StepId::MasterChef, vec![]);
        // 9 KICKOFF
        seq.add(StepId::Kickoff, vec![]);
        // 10 KICKOFF_SCATTER_ROLL (or ASK_AFTER variant based on game option)
        if params.ask_after_roll {
            seq.add(StepId::KickoffScatterRollAskAfter, vec![]);
        } else {
            seq.add(StepId::KickoffScatterRoll, vec![]);
        }
        // 11 KICKOFF_RETURN
        seq.add(StepId::KickoffReturn, vec![]);
        // 12 KICKOFF_RESULT_ROLL
        seq.add(StepId::KickoffResultRoll, vec![]);
        // 13 APPLY_KICKOFF_RESULT
        seq.add(StepId::ApplyKickoffResult, vec![
            StepParameter::GotoLabelOnEnd(end_kickoff.into()),
            StepParameter::GotoLabelOnBlitz(blitz_turn.into()),
        ]);
        // 14 GOTO_LABEL → KICKOFF_ANIMATION
        seq.jump(kickoff_animation);
        // 15 BLITZ_TURN [BLITZ_TURN]
        seq.add_labelled(StepId::BlitzTurn, blitz_turn, vec![]);
        // 16 KICKOFF_ANIMATION [KICKOFF_ANIMATION]
        seq.add_labelled(StepId::KickoffAnimation, kickoff_animation, vec![]);
        // 17 CATCH_SCATTER_THROW_IN
        seq.add(StepId::CatchScatterThrowIn, vec![]);
        // 18 TOUCHBACK
        seq.add(StepId::Touchback, vec![]);
        // 19 CATCH_SCATTER_THROW_IN
        seq.add(StepId::CatchScatterThrowIn, vec![]);
        // 20 END_KICKOFF [END_KICKOFF]
        seq.add_labelled(StepId::EndKickoff, end_kickoff, vec![]);

        seq.build()
    }
}

impl Default for Kickoff {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kickoff_without_coin_choice_starts_with_init_kickoff() {
        let steps = Kickoff::build_sequence(&KickoffParams::default());
        assert_eq!(steps[0].step_id, StepId::InitKickoff);
    }

    #[test]
    fn kickoff_with_coin_choice_starts_with_coin_choice() {
        let steps = Kickoff::build_sequence(&KickoffParams { with_coin_choice: true, ..Default::default() });
        assert_eq!(steps[0].step_id, StepId::CoinChoice);
        assert_eq!(steps[1].step_id, StepId::ReceiveChoice);
        assert_eq!(steps[2].step_id, StepId::InitKickoff);
    }

    #[test]
    fn kickoff_ends_with_end_kickoff_labelled() {
        let steps = Kickoff::build_sequence(&KickoffParams::default());
        let last = steps.last().unwrap();
        assert_eq!(last.step_id, StepId::EndKickoff);
        assert_eq!(last.label.as_deref(), Some(labels::END_KICKOFF));
    }

    #[test]
    fn kickoff_has_two_swarming_steps() {
        let steps = Kickoff::build_sequence(&KickoffParams::default());
        assert_eq!(steps.iter().filter(|s| s.step_id == StepId::Swarming).count(), 2);
    }

    #[test]
    fn kickoff_blitz_turn_is_labelled() {
        let steps = Kickoff::build_sequence(&KickoffParams::default());
        let bt = steps.iter().find(|s| s.step_id == StepId::BlitzTurn).unwrap();
        assert_eq!(bt.label.as_deref(), Some(labels::BLITZ_TURN));
    }
}
