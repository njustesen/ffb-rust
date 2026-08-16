/// BB2025 end-player-action step sequence.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.bb2025.EndPlayerAction`.
use ffb_model::enums::ApothecaryMode;
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::sequence::{Sequence, SequenceStep, labels};

/// Parameters — mirrors Java `EndPlayerAction.SequenceParams`.
#[derive(Debug, Clone)]
pub struct EndPlayerActionParams {
    pub feeding_allowed: bool,
    pub end_player_action: bool,
    pub end_turn: bool,
    pub check_forgo: bool,
    /// Edition this sequence is being built for. The driver runs this BB2025 generator for BB2020
    /// games, but the two Java sequences differ (see `build_sequence`), so BB2020 has to be routed
    /// to its own. Defaults to BB2025 — every live call site should pass `game.rules`.
    pub rules: ffb_model::enums::Rules,
}

impl Default for EndPlayerActionParams {
    fn default() -> Self {
        Self {
            feeding_allowed: false,
            end_player_action: false,
            end_turn: false,
            check_forgo: false,
            rules: ffb_model::enums::Rules::Bb2025,
        }
    }
}

pub struct EndPlayerAction;

impl EndPlayerAction {
    pub fn new() -> Self { Self }

    /// Build the end-player-action step sequence (Java `pushSequence`).
    ///
    /// BB2020 gets a genuinely different sequence, not a variation on this one. Java
    /// `generator/bb2020/EndPlayerAction.java:25-36` is 7 steps to this file's 11:
    ///
    ///   bb2020  … CATCH_SCATTER_THROW_IN, CHECK_STALLING[END_FEEDING](IGNORE_ACTED_FLAG=false),
    ///           END_FEEDING
    ///   bb2025  … CATCH_SCATTER_THROW_IN, STALLING_PLAYER[END_FEEDING], STEADY_FOOTING(HIT_PLAYER),
    ///           PLACE_BALL, APOTHECARY(HIT_PLAYER), CATCH_SCATTER_THROW_IN, END_FEEDING(CHECK_FORGO)
    ///
    /// The two editions do not even agree on what `STALLING_PLAYER` means: BB2025's is the stalling
    /// CHECK (`bb2025/shared/StepStallingPlayer.java`), while BB2020's is the rock throw AT a
    /// staller (`bb2020/StepStallingPlayer.java`) and its check lives in `CHECK_STALLING`. Running
    /// this sequence for BB2020 therefore ran BB2025's stalling check in BB2020 games and skipped
    /// `CHECK_STALLING` entirely. BB2020 also passes RESET_FOR_FAILED_BLOCK=false to
    /// RESET_FUMBLEROOSKIE and takes no CHECK_FORGO on END_FEEDING.
    pub fn build_sequence(params: &EndPlayerActionParams) -> Vec<SequenceStep> {
        if params.rules == ffb_model::enums::Rules::Bb2020 {
            use crate::step::generator::bb2020::end_player_action as bb2020;
            return bb2020::EndPlayerAction::build_sequence(&bb2020::EndPlayerActionParams {
                feeding_allowed: params.feeding_allowed,
                end_player_action: params.end_player_action,
                end_turn: params.end_turn,
            });
        }

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

    /// BB2020's EndPlayerAction is a different sequence, not a variant of BB2025's
    /// (`generator/bb2020/EndPlayerAction.java:25-36`). Critically the two editions disagree on
    /// what STALLING_PLAYER *is*: BB2025's is the stalling check, BB2020's is the rock throw at a
    /// staller, and BB2020 does its check in CHECK_STALLING. Building this sequence for a BB2020
    /// game therefore ran BB2025's stalling check and never ran CHECK_STALLING at all.
    #[test]
    fn bb2020_end_player_action_uses_check_stalling_not_stalling_player() {
        use ffb_model::enums::Rules;
        let bb2020 = EndPlayerAction::build_sequence(&EndPlayerActionParams {
            rules: Rules::Bb2020, ..Default::default() });

        assert!(!bb2020.iter().any(|s| s.step_id == StepId::StallingPlayer),
            "BB2020 has no STALLING_PLAYER in this sequence");
        let cs = bb2020.iter().find(|s| s.step_id == StepId::CheckStalling)
            .expect("BB2020 resolves stalling with CHECK_STALLING");
        assert_eq!(cs.label.as_deref(), Some(labels::END_FEEDING));
        assert!(cs.params.iter().any(|p| matches!(p, StepParameter::IgnoreActedFlag(false))),
            "Java passes IGNORE_ACTED_FLAG=false");

        // BB2020 stops at END_FEEDING: no HIT_PLAYER tail, and no CHECK_FORGO.
        assert_eq!(bb2020.len(), 7, "Java bb2020 builds exactly 7 steps");
        assert!(!bb2020.iter().any(|s| s.step_id == StepId::SteadyFooting));
        assert!(!bb2020.iter().any(|s| s.step_id == StepId::PlaceBall));
        let ef = bb2020.iter().find(|s| s.step_id == StepId::EndFeeding).unwrap();
        assert!(!ef.params.iter().any(|p| matches!(p, StepParameter::CheckForgo(_))));
        // ...and it passes RESET_FOR_FAILED_BLOCK=false, which BB2025 omits.
        let rf = bb2020.iter().find(|s| s.step_id == StepId::ResetFumblerooskie).unwrap();
        assert!(rf.params.iter().any(|p| matches!(p, StepParameter::ResetForFailedBlock(false))));

        // BB2025 keeps its own shape.
        let bb2025 = EndPlayerAction::build_sequence(&EndPlayerActionParams::default());
        assert_eq!(bb2025.len(), 11);
        assert!(bb2025.iter().any(|s| s.step_id == StepId::StallingPlayer));
        assert!(!bb2025.iter().any(|s| s.step_id == StepId::CheckStalling));
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
