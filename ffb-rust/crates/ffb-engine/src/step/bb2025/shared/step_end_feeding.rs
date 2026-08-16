use ffb_model::enums::{TurnMode};
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::util_server_steps::check_touchdown;
use crate::step::generator::bb2025::{EndTurn, Pass, Select};
use crate::step::generator::bb2025::end_turn::EndTurnParams;
use crate::step::generator::bb2025::pass::PassParams;
use crate::step::generator::bb2025::select::SelectParams;
use crate::step::sequences::{inducement_sequence, inducement_sequence_with_check_forgo};
use ffb_model::enums::InducementPhase;

/// Final step of the feed sequence. Consumes EndPlayerAction/EndTurn/CheckForgo.
/// 1:1 translation of com.fumbbl.ffb.server.step.bb2025.shared.StepEndFeeding.
pub struct StepEndFeeding {
    pub end_player_action: bool,
    pub end_turn: bool,
    pub check_forgo: bool,
}

impl StepEndFeeding {
    pub fn new() -> Self {
        Self { end_player_action: false, end_turn: false, check_forgo: false }
    }
}

impl Default for StepEndFeeding {
    fn default() -> Self { Self::new() }
}

impl Step for StepEndFeeding {
    fn id(&self) -> StepId { StepId::EndFeeding }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::EndPlayerAction(v) => { self.end_player_action = *v; true }
            StepParameter::EndTurn(v) => { self.end_turn = *v; true }
            StepParameter::CheckForgo(v) => { self.check_forgo = *v; true }
            _ => false,
        }
    }

    // Java: setParameter consume()s END_PLAYER_ACTION and END_TURN (CheckForgo is not consumed).
    fn consumes_parameter(&self, param: &StepParameter) -> bool {
        matches!(param, StepParameter::EndPlayerAction(_) | StepParameter::EndTurn(_))
    }
}

impl StepEndFeeding {
    fn execute_step(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // Java: fEndTurn |= UtilServerSteps.checkTouchdown(getGameState())
        self.end_turn |= check_touchdown(game);

        // Java: game.setDefenderId(null)
        game.defender_id = None;

        // Java: markSkillsTrackedOutsideOfActivationAndRemoveEffects(game) — not yet ported.

        if self.end_turn {
            match game.turn_mode {
                TurnMode::PassBlock => {
                    // Java: EndTurn.pushSequence(new EndTurn.SequenceParams(getGameState(), false))
                    let seq = EndTurn::build_sequence(&EndTurnParams { check_forgo: false });
                    return StepOutcome::next().push_seq(seq);
                }
                TurnMode::Regular => {
                    // Java: changePlayerAction(null, null, false) → clear acting player,
                    // restoring its MOVING base (STANDING+inactive when acted, PRONE when
                    // standing up) — full null-path semantics, not just the id clear.
                    crate::step::util_server_steps::change_player_action_to_none(game);
                    // Java: Inducement(EndOfOpponentTurn, !home, checkForgo)
                    // Java: Inducement(EndOfOwnTurn, home)
                    // Java: push PickMeUp step
                    let home = game.home_playing;
                    let seq_opponent = inducement_sequence_with_check_forgo(InducementPhase::EndOfOpponentTurn, !home, self.check_forgo);
                    let seq_own = inducement_sequence(InducementPhase::EndOfOwnTurn, home);
                    let pick_me_up = vec![
                        crate::step::framework::SequenceStep::new(StepId::PickMeUp),
                    ];
                    // Java push order (last pushed runs first): opponent window pushed FIRST,
                    // own window second, PickMeUp last → run order PickMeUp → END_OF_OWN_TURN
                    // window → END_OF_OPPONENT_TURN window. The opponent window must run LAST:
                    // its InitInducement hands home_playing to the opposing team and its
                    // EndInducement chains into the EndTurn sequence.
                    return StepOutcome::next()
                        .push_seq(seq_opponent)
                        .push_seq(seq_own)
                        .push_seq(pick_me_up);
                }
                TurnMode::KickoffReturn => {
                    let seq = EndTurn::build_sequence(&EndTurnParams { check_forgo: false });
                    return StepOutcome::next().push_seq(seq);
                }
                _ => {
                    // Other turn modes: just next
                    return StepOutcome::next();
                }
            }
        }

        // Java: else if (!fEndPlayerAction && throwerAction != null && throwerAction.isPassing())
        // — the pass-continuation branch for PASS_MOVE activations (move first, throw later).
        // Guarded here on thrower == acting player: game.thrower_id/thrower_action are only
        // cleared at turn end (Java Game.startTurn), so a completed pass earlier in the SAME
        // turn leaves them stale; without the guard the next player's plain Move activation
        // pushed a Pass sequence for the old thrower, whose StepEndPassing then ended the
        // game on an empty stack.
        let thrower_passing = game.thrower_action
            .map(|a| a.is_passing())
            .unwrap_or(false)
            && game.thrower_id.is_some()
            && game.thrower_id == game.acting_player.player_id;
        if !self.end_player_action && thrower_passing {
            let params = PassParams {
                target_coordinate: game.pass_coordinate,
                            rules: game.rules,
            };
            let seq = Pass::build_sequence(&params);
            return StepOutcome::next().push_seq(seq);
        }

        // Java: else if (KickoffReturn || PassBlock) → publish EndPlayerAction
        if matches!(game.turn_mode, TurnMode::KickoffReturn | TurnMode::PassBlock) {
            return StepOutcome::next()
                .publish(StepParameter::EndPlayerAction(true));
        }

        // Java: else → changePlayerAction(null, null, false); Select.pushSequence(false)
        crate::step::util_server_steps::change_player_action_to_none(game);
        let seq = Select::build_sequence(&SelectParams { update_persistence: false, is_blitz_move: false, ..Default::default() });
        StepOutcome::next().push_seq(seq)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::{StepAction, StepId};
    use ffb_model::enums::{Rules, TurnMode};

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2025)
    }

    #[test]
    fn default_pushes_select_sequence() {
        let mut game = make_game();
        game.turn_mode = TurnMode::Regular;
        let mut step = StepEndFeeding::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(out.pushes.len(), 1);
        assert_eq!(out.pushes[0][0].step_id, StepId::InitSelecting);
    }

    #[test]
    fn end_turn_regular_pushes_inducement_and_pick_me_up() {
        let mut game = make_game();
        game.turn_mode = TurnMode::Regular;
        let mut step = StepEndFeeding::new();
        step.end_turn = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        // Java push order (last pushed runs first): opponent window pushed FIRST, own window
        // second, PickMeUp last → pushes = [seq_opponent, seq_own, pick_me_up].
        assert_eq!(out.pushes.len(), 3);
        assert_eq!(out.pushes[0][0].step_id, StepId::InitInducement);
        assert_eq!(out.pushes[2][0].step_id, StepId::PickMeUp);
    }

    // Java: `new Inducement.SequenceParams(getGameState(), InducementPhase.END_OF_OPPONENT_TURN,
    //       !game.isHomePlaying(), checkForgo)` — the step's own `checkForgo` field (set via the
    // CHECK_FORGO init/step parameter) must reach the opponent-turn inducement sequence's
    // END_INDUCEMENT step, not be silently dropped as `false`.
    #[test]
    fn end_turn_regular_forwards_check_forgo_to_opponent_inducement_sequence() {
        let mut game = make_game();
        game.turn_mode = TurnMode::Regular;
        let mut step = StepEndFeeding::new();
        step.end_turn = true;
        step.check_forgo = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        // pushes: [seq_opponent, seq_own, pick_me_up] (Java push order; last pushed runs first)
        let seq_opponent = &out.pushes[0];
        let end_inducement = seq_opponent.iter().find(|s| s.step_id == StepId::EndInducement)
            .expect("expected EndInducement step in opponent-turn inducement sequence");
        assert!(
            end_inducement.params.iter().any(|p| matches!(p, StepParameter::CheckForgo(true))),
            "expected CheckForgo(true) to be forwarded to opponent-turn EndInducement, got {:?}",
            end_inducement.params
        );
    }

    #[test]
    fn end_turn_pass_block_pushes_end_turn_sequence() {
        let mut game = make_game();
        game.turn_mode = TurnMode::PassBlock;
        let mut step = StepEndFeeding::new();
        step.end_turn = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.pushes.len(), 1);
        assert_eq!(out.pushes[0].last().unwrap().step_id, StepId::EndTurn);
    }

    #[test]
    fn kickoff_return_with_end_player_action_publishes_end_player_action() {
        let mut game = make_game();
        game.turn_mode = TurnMode::KickoffReturn;
        let mut step = StepEndFeeding::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndPlayerAction(true))));
    }

    #[test]
    fn set_parameter_end_turn_accepted() {
        let mut step = StepEndFeeding::new();
        assert!(step.set_parameter(&StepParameter::EndTurn(true)));
        assert!(step.end_turn);
    }

    #[test]
    fn set_parameter_check_forgo_accepted() {
        let mut step = StepEndFeeding::new();
        assert!(step.set_parameter(&StepParameter::CheckForgo(true)));
        assert!(step.check_forgo);
    }

    #[test]
    fn set_parameter_end_player_action_accepted() {
        let mut step = StepEndFeeding::new();
        assert!(step.set_parameter(&StepParameter::EndPlayerAction(true)));
        assert!(step.end_player_action);
    }

    #[test]
    fn clears_defender_id_on_execute() {
        let mut game = make_game();
        game.defender_id = Some("def".into());
        let mut step = StepEndFeeding::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.defender_id.is_none());
    }
}
