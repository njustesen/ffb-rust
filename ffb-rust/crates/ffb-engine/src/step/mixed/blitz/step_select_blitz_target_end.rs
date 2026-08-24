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

        if self.end_turn {
            // Java: `game.setDefenderId(null); endGenerator.pushSequence(
            //         new EndPlayerAction.SequenceParams(gameState, true, true, endTurn))`
            // - feedingAllowed=true, endPlayerAction=true, endTurn.
            //
            // Rust only PUBLISHED the parameter and pushed nothing, and the comment here said so
            // ("full generator push not yet ported"). That is the publish-only stall shape: with
            // an empty step stack the driver has nothing left to run and the GAME ENDS. It only
            // became reachable once the chain started routing blitzes through this step - skaven
            // seed 73 stopped at step 148 where Java played 262, right after a failed Animal
            // Savagery set end_turn on the way to END_BLITZING.
            game.defender_id = None; // Java: game.setDefenderId(null)
            let seq = crate::step::generator::bb2025::end_player_action::EndPlayerAction::build_sequence(
                &crate::step::generator::bb2025::end_player_action::EndPlayerActionParams {
                    feeding_allowed: true,
                    end_player_action: true,
                    end_turn: self.end_turn,
                    check_forgo: false,
                    rules: game.rules,
                });
            return StepOutcome::next().push_seq(seq);
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
                if let Some(pid) = game.acting_player.player_id.clone() {
                    crate::step::util_server_steps::change_player_action(
                        game, &pid, PlayerAction::BlitzMove, false);
                }
                // Java `StepSelectBlitzTargetEnd:94`: a blitzer SUFFERING BLOOD LUST with a
                // bloodlustAction does NOT continue the blitz - it drops the blitz target and is
                // re-dispatched into a plain MOVE sequence to go and feed. Rust only had the else
                // arm, so a blood-lust blitzer was pushed back through Select, took the
                // USE_ALTERNATE_LABEL jump straight to END_SELECTING and ended its activation
                // without ever blocking: vampire seed 1 i=101 spent 1 die where Java spends 5, and
                // the defender it should have knocked down stayed Standing.
                if game.acting_player.suffering_blood_lust && self.bloodlust_action.is_some() {
                    if let Some(sel) = game.field_model.target_selection_state.as_ref()
                        .and_then(|ts| ts.get_selected_player_id().cloned())
                    {
                        if let Some(ps) = game.field_model.player_state(&sel) {
                            game.field_model.set_player_state(&sel, ps.remove_selected_blitz_target());
                        }
                    }
                    game.defender_id = None;
                    if let Some(pid) = game.acting_player.player_id.clone() {
                        if let Some(bl) = self.bloodlust_action {
                            crate::step::util_server_steps::change_player_action(game, &pid, bl, false);
                        }
                    }
                    game.turn_data_mut().blitz_used = true;
                    let seq = crate::step::generator::bb2025::move_::Move::build_sequence(
                        &crate::step::generator::bb2025::move_::MoveParams {
                            bloodlust_action: self.bloodlust_action,
                            rules: game.rules,
                            ..Default::default()
                        });
                    return StepOutcome::next().push_seq(seq);
                }
                game.turn_data_mut().blitz_used = true;
                let seq = Select::build_sequence(&SelectParams {
                    update_persistence: false,
                    is_blitz_move: false,
                    ..Default::default()
                });
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
                // Java: `sequence.add(END_MOVING, END_PLAYER_ACTION=true); stepStack.push(...)`.
                // Rust only PUBLISHED the parameter and pushed nothing, which is the publish-only
                // stall shape (same as StepEndThrowKeg / EndThenIStartedBlastin): with nothing on
                // the stack to consume it the drive falls through and the game ends early.
                //
                // The bug was invisible until now because this branch was UNREACHABLE - nothing
                // ever set the state to FAILED. Once the negatrait behaviours started marking it
                // (Java does this in every negatrait FAILURE branch) the dead branch went live and
                // goblin collapsed to 1/20, which is how it was found.
                let seq = vec![crate::step::framework::SequenceStep {
                    step_id: StepId::EndMoving,
                    label: None,
                    params: vec![StepParameter::EndPlayerAction(true)],
                }];
                return StepOutcome::next().push_seq(seq);
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

    /// Java pushes an EndPlayerAction SEQUENCE here (feedingAllowed=true, endPlayerAction=true,
    /// endTurn) and clears the defender. This test used to assert only that the parameter was
    /// PUBLISHED, which is exactly what let the publish-only stall survive: with nothing pushed
    /// the driver's stack empties and the game ends (skaven seed 73 stopped at step 148 against
    /// Java's 262). Pin the push.
    #[test]
    fn end_turn_pushes_end_player_action_sequence() {
        let mut step = StepSelectBlitzTargetEnd::new();
        step.end_turn = true;
        let mut game = make_game();
        game.defender_id = Some("away_01".into());
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        assert_eq!(out.action, StepAction::NextStep);
        assert!(!out.pushes.is_empty(), "end_turn must push the EndPlayerAction sequence");
        assert!(game.defender_id.is_none(), "Java: game.setDefenderId(null)");
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

    /// Java :109-115 does FOUR things on the skip path, and this test used to assert only the
    /// last two - so it stayed green while the branch pushed nothing and the step silently
    /// became the terminus of the blitz sequence (lineman bb2025 seed 14: the whole game ended
    /// at the first no-target blitz, 10 steps against Java's 874). Pin the push itself.
    #[test]
    fn skipped_state_pushes_select_and_restores_blitz_move() {
        let mut step = StepSelectBlitzTargetEnd::new();
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        game.acting_player.player_id = Some("home_1".into());

        let mut tss = TargetSelectionState::default();
        tss.skip();
        game.field_model.target_selection_state = Some(tss);

        let out = step.start(&mut game, &mut rng);

        assert_eq!(out.action, StepAction::NextStep);
        assert!(!out.pushes.is_empty(), "skip path must push the Select sequence");
        assert_eq!(
            out.pushes[0].first().map(|s| s.step_id),
            Some(StepId::InitSelecting),
            "the pushed sequence is Select, which begins with InitSelecting",
        );
        assert_eq!(game.acting_player.player_action, Some(PlayerAction::BlitzMove));
    }

    /// The SELECTED path has the same push requirement, for the same reason.
    #[test]
    fn selected_state_pushes_select_sequence() {
        let mut step = StepSelectBlitzTargetEnd::new();
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        game.acting_player.player_id = Some("home_1".into());

        let mut tss = TargetSelectionState::default();
        tss.select();
        game.field_model.target_selection_state = Some(tss);

        let out = step.start(&mut game, &mut rng);

        assert!(!out.pushes.is_empty(), "selected path must push the Select sequence");
        assert_eq!(
            out.pushes[0].first().map(|s| s.step_id),
            Some(StepId::InitSelecting),
        );
        assert_eq!(game.acting_player.player_action, Some(PlayerAction::BlitzMove));
    }
}
