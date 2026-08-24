/// 1:1 translation of `com.fumbbl.ffb.server.step.mixed.blitz.StepSelectBlitzTargetEnd`.
///
/// Handles the end of blitz-target selection. Depending on the state of the
/// `TargetSelectionState`, pushes EndPlayerAction, Move or Select sequences.
///
/// Runtime params: END_TURN, BLOOD_LUST_ACTION.
///
/// Stub: sequence generator pushes (EndPlayerAction, Move, Select) are not fully
/// translated here — the parameter handling and state logic are ported.
/// Generator dispatch is represented as publishing EndPlayerAction or clearing state.
use ffb_model::enums::PlayerAction;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::generator::bb2025::select::{Select, SelectParams};
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};

/// Java: `StepSelectBlitzTargetEnd` (mixed/blitz, BB2020 + BB2025).
#[derive(Debug, Default)]
pub struct StepSelectBlitzTargetEnd {
    /// Java: `endTurn`
    end_turn: bool,
    /// Java: `bloodlustAction`
    bloodlust_action: Option<PlayerAction>,
}

impl StepSelectBlitzTargetEnd {
    pub fn new() -> Self { Self::default() }

    fn execute_step(&self, game: &mut Game) -> StepOutcome {
        // Java logic:
        // if endTurn → push EndPlayerAction sequence
        // else if targetSelectionState != null:
        //   if canceled → reset stalling/skills, push Select
        //   if selected && bloodlust → push Move sequence
        //   if selected → push Select with BLITZ_MOVE action, set blitzUsed=true
        //   if skipped → push Select with BLITZ_MOVE action, set blitzUsed=true
        //   if failed → push END_MOVING(end_player_action=true)

        let tss = game.field_model.target_selection_state.clone();
        if std::env::var("FFB_BZ_PROBE").is_ok() {
            eprintln!("BZPROBE SBTEnd end_turn={} tss={:?} defender={:?}",
                self.end_turn, tss.as_ref().map(|t| t.get_status()), game.defender_id);
        }

        if self.end_turn {
            // Java: EndPlayerAction.pushSequence(true, true, endTurn)
            // Publish end player action and proceed — full generator push not yet ported
            game.defender_id = None; // Java: game.setDefenderId(null)
            return StepOutcome::next().publish(StepParameter::EndPlayerAction(true));
        }

        if let Some(ref ts) = tss {
            if ts.is_canceled() {
                // Java: if (actingPlayer.hasActed()) mark skills used; else resetStalling()
                // Java: removeSkillEnhancements, changePlayerAction(null), setTargetSelectionState(null), push Select
                game.field_model.target_selection_state = None;
                return StepOutcome::next();
            } else if ts.is_selected() {
                // Java: changePlayerAction(actingPlayer, BLITZ_MOVE, false) then
                //       Select.pushSequence(new Select.SequenceParams(gameState, false)),
                //       then game.getTurnData().setBlitzUsed(true).
                // The Select push is the LOOP-BACK: re-entering StepInitSelecting with a
                // non-null targetSelectionState takes the ordinary BLITZ_MOVE path and runs the
                // real move + block. Without it this step was the terminus of the whole blitz
                // sequence and the driver stalled (lineman bb2025 0/20, rust_total 0.07s) - the
                // same publish-only shape as StepEndThrowKeg and StepEndThenIStartedBlastin.
                let before = game.acting_player.player_id.clone();
                if let Some(pid) = game.acting_player.player_id.clone() {
                    crate::step::util_server_steps::change_player_action(
                        game, &pid, PlayerAction::BlitzMove, false);
                }
                if std::env::var("FFB_BZ_PROBE").is_ok() {
                    eprintln!("BZPROBE SBTEnd SELECTED pid_before={:?} pid_after={:?} pa_after={:?}",
                        before, game.acting_player.player_id, game.acting_player.player_action);
                }
                game.turn_data_mut().blitz_used = true;
                let seq = Select::build_sequence(&SelectParams {
                    update_persistence: false,
                    is_blitz_move: false,
                    ..Default::default()
                });
                if std::env::var("FFB_BZ_PROBE").is_ok() {
                    eprintln!("BZPROBE SBTEnd PUSHSEQ {:?}",
                        seq.iter().map(|s| format!("{:?}", s.step_id)).collect::<Vec<_>>());
                }
                return StepOutcome::next().push_seq(seq);
            } else if ts.is_skipped() {
                // Java (:109-115) does FOUR things here, and Rust used to do only the last two -
                // the comment already said "changePlayerAction(BLITZ_MOVE), push Select" but
                // neither was implemented. Without the push this step is the terminus of the
                // whole blitz sequence, exactly the publish-only stall shape the SELECTED branch
                // above was fixed for: on lineman bb2025 seed 14 the game simply ENDED at the
                // first no-target blitz (10 steps, rust_total 0.004s, Java ran 874).
                //
                // The skip path is reached when the declared blitzer has no adjacent blockable
                // opponent (Java harness prints BLITZ_TARGET_NONE). The blitz is still SPENT:
                // blitz_used and has_moved are set, and re-entering Select with a non-null (but
                // skipped) targetSelectionState takes the ordinary BLITZ_MOVE dispatch, so the
                // player still gets their move.
                if let Some(pid) = game.acting_player.player_id.clone() {
                    crate::step::util_server_steps::change_player_action(
                        game, &pid, PlayerAction::BlitzMove, false);
                }
                let seq = Select::build_sequence(&SelectParams {
                    update_persistence: false,
                    is_blitz_move: false,
                    ..Default::default()
                });
                game.turn_data_mut().blitz_used = true;
                game.acting_player.has_moved = true;
                return StepOutcome::next().push_seq(seq);
            } else if ts.is_failed() {
                // Java: push END_MOVING(end_player_action=true)
                return StepOutcome::next().publish(StepParameter::EndPlayerAction(true));
            }
        }

        StepOutcome::next()
    }
}

impl Step for StepSelectBlitzTargetEnd {
    fn id(&self) -> StepId { StepId::SelectBlitzTargetEnd }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::EndTurn(v) => { self.end_turn = *v; false }
            StepParameter::BloodLustAction(v) => { self.bloodlust_action = *v; false }
            _ => false,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::enums::Rules;
    use ffb_model::model::target_selection_state::TargetSelectionState;
    use ffb_model::util::rng::GameRng;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    #[test]
    fn returns_next_with_no_state() {
        let mut step = StepSelectBlitzTargetEnd::new();
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn end_turn_publishes_end_player_action() {
        let mut step = StepSelectBlitzTargetEnd::new();
        step.end_turn = true;
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        assert_eq!(out.action, StepAction::NextStep);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndPlayerAction(true))));
    }

    #[test]
    fn selected_state_sets_blitz_used() {
        let mut step = StepSelectBlitzTargetEnd::new();
        let mut game = make_game();
        let mut rng = GameRng::new(0);

        let mut tss = TargetSelectionState::default();
        tss.select();
        game.field_model.target_selection_state = Some(tss);

        step.start(&mut game, &mut rng);

        assert!(game.turn_data().blitz_used);
    }

    #[test]
    fn canceled_state_clears_selection_state() {
        let mut step = StepSelectBlitzTargetEnd::new();
        let mut game = make_game();
        let mut rng = GameRng::new(0);

        let mut tss = TargetSelectionState::default();
        tss.cancel();
        game.field_model.target_selection_state = Some(tss);

        step.start(&mut game, &mut rng);

        assert!(game.field_model.target_selection_state.is_none());
    }

    #[test]
    fn set_parameter_end_turn() {
        let mut step = StepSelectBlitzTargetEnd::new();
        step.set_parameter(&StepParameter::EndTurn(true));
        assert!(step.end_turn);
    }

    #[test]
    fn skipped_state_sets_blitz_used_and_has_moved() {
        let mut step = StepSelectBlitzTargetEnd::new();
        let mut game = make_game();
        let mut rng = GameRng::new(0);

        let mut tss = TargetSelectionState::default();
        tss.skip();
        game.field_model.target_selection_state = Some(tss);

        step.start(&mut game, &mut rng);

        assert!(game.turn_data().blitz_used);
        assert!(game.acting_player.has_moved);
    }
}
