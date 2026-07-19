/// 1:1 translation of com.fumbbl.ffb.server.step.bb2016.StepPushback.
///
/// BB2016 pushback is structurally similar to BB2025 pushback but the Java sources
/// diverge in a few concrete ways (verified via `diff` against
/// `ffb-server/.../step/bb2025/block/StepPushback.java`):
///   - BB2016 has no fan-interaction / SPP-for-crowd-push variant (always plain
///     InjuryTypeCrowdPush with no attacker).
///   - BB2016 does NOT publish END_TURN when a same-team player is crowd-pushed off
///     (that `if (sameTeam) publishParameter(END_TURN, true)` block is BB2025-only).
///   - BB2016's `pushPlayer` does NOT publish PLAYER_WAS_PUSHED or PUSHED_ON_BALL —
///     it only publishes CATCH_SCATTER_THROW_IN_MODE (on ball scatter) and
///     PLAYER_ENTERING_SQUARE.
///
/// This file used to simply re-export the BB2025 implementation, which incorrectly
/// carried the BB2025-only END_TURN/PLAYER_WAS_PUSHED behavior into BB2016 games.
///
/// Expects: STARTING_PUSHBACK_SQUARE, OLD_DEFENDER_STATE.
/// Sets: CATCH_SCATTER_THROW_IN_MODE, DEFENDER_PUSHED, FOLLOWUP_CHOICE,
///       STARTING_PUSHBACK_SQUARE, INJURY_RESULT, PLAYER_ENTERING_SQUARE.
use std::collections::HashMap;
use ffb_model::enums::{ApothecaryMode, PlayerState};
use ffb_model::model::pushback_mode::PushbackMode;
use ffb_model::report::report_pushback::ReportPushback;
use ffb_model::types::{FieldCoordinate, PushbackSquare};
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::util_server_injury::handle_injury_by_name;
use crate::util::UtilServerPlayerMove;
use crate::util::util_server_pushback::UtilServerPushback;
use crate::skill_behaviour::dispatch;
use crate::step::bb2025::block::step_pushback::StepPushbackHookState;

// ── StepPushback ───────────────────────────────────────────────────────────────

/// Java: StepPushback.StepState fields are inlined here.
pub struct StepPushback {
    pub old_defender_state: Option<PlayerState>,
    pub starting_pushback_square: Option<PushbackSquare>,
    /// Java: grabbing (Boolean — tristate)
    pub grabbing: Option<bool>,
    /// Java: sideStepping (Map<String, Boolean>)
    pub side_stepping: HashMap<String, bool>,
    /// Java: standingFirm (Map<String, Boolean>)
    pub standing_firm: HashMap<String, bool>,
    /// Java: pushbackStack — (playerId, coordinate) pairs (LIFO).
    pub pushback_stack: Vec<(String, FieldCoordinate)>,
}

impl StepPushback {
    pub fn new() -> Self {
        Self {
            old_defender_state: None,
            starting_pushback_square: None,
            grabbing: None,
            side_stepping: HashMap::new(),
            standing_firm: HashMap::new(),
            pushback_stack: Vec::new(),
        }
    }
}

impl Default for StepPushback {
    fn default() -> Self { Self::new() }
}

impl Step for StepPushback {
    fn id(&self) -> StepId { StepId::Pushback }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        if let Action::PushTo { coord } = action {
            // Java: CLIENT_PUSHBACK — pushbackStack.push(pushback [.transform()])
            if let Some(defender_id) = game.defender_id.clone() {
                self.pushback_stack.push((defender_id, *coord));
            }
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::OldDefenderState(v) => { self.old_defender_state = Some(*v); true }
            StepParameter::StartingPushbackSquare(v) => {
                self.starting_pushback_square = *v;
                true
            }
            _ => false,
        }
    }
}

impl StepPushback {
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let mut do_push = false;

        // Java: if (!state.pushbackStack.isEmpty())
        if !self.pushback_stack.is_empty() {
            if let Some(&(_, chosen_coord)) = self.pushback_stack.last() {
                let matching = game.field_model.pushback_squares.iter_mut()
                    .find(|sq| !sq.locked && sq.coordinate == chosen_coord);
                if let Some(sq) = matching {
                    sq.selected = true;
                    sq.locked = true;
                    self.starting_pushback_square = Some(*sq);
                }
                // Java: doPush = (fieldModel.getPlayer(lastPushback.getCoordinate()) == null)
                do_push = game.field_model.player_at(chosen_coord).is_none();
            }
        }

        // Java: if (!state.doPush && (state.startingPushbackSquare != null))
        if !do_push {
            if let Some(starting_sq) = self.starting_pushback_square {
                let defender_coord = starting_sq.coordinate;
                let defender_id = game.defender_id.clone().unwrap_or_default();

                let home_choice = game.home_playing;
                let occupied = |c: FieldCoordinate| game.field_model.player_at(c).is_some();
                let pushback_squares = UtilServerPushback::find_pushback_squares_standard(
                    starting_sq, &occupied, home_choice,
                );

                let adjacent_squares = game.field_model.adjacent_on_pitch(defender_coord);
                let free_square_around_defender = adjacent_squares.iter()
                    .any(|c| game.field_model.player_at(*c).is_none());

                // ── Skill hooks via dispatch (StandFirm, Grab, SideStep) ──
                let mut hook_state = StepPushbackHookState::new(
                    defender_id.clone(),
                    self.old_defender_state,
                    self.starting_pushback_square,
                    self.pushback_stack.len(),
                    free_square_around_defender,
                    pushback_squares,
                    self.side_stepping.clone(),
                    self.standing_firm.clone(),
                    self.grabbing,
                );

                let stop_processing = dispatch::execute_step_hooks(
                    game, rng, StepId::Pushback, &mut hook_state,
                );

                self.side_stepping = hook_state.side_stepping;
                self.standing_firm = hook_state.standing_firm;
                self.grabbing = hook_state.grabbing;
                self.starting_pushback_square = hook_state.starting_pushback_square;
                do_push = hook_state.do_push;
                let final_pushback_squares = hook_state.pushback_squares;

                let pushback_squares_found = !final_pushback_squares.is_empty() || stop_processing;

                // Java: if (!ArrayTool.isProvided(state.pushbackSquares)) → Crowd push
                if !pushback_squares_found && !stop_processing {
                    // Java BB2016: always InjuryTypeCrowdPush, no attacker (no fan-interaction/SPP variant)
                    let crowd_push_coord = self.starting_pushback_square
                        .as_ref()
                        .map(|sq| sq.coordinate)
                        .unwrap_or(defender_coord);
                    let injury_result = handle_injury_by_name(
                        game, rng, "InjuryTypeCrowdPush",
                        None,
                        game.defender_id.as_deref().unwrap_or(""),
                        crowd_push_coord,
                        None, None, ApothecaryMode::CrowdPush,
                    );

                    // Java: game.getFieldModel().remove(state.defender)
                    if let Some(defender_id) = game.defender_id.clone() {
                        game.field_model.remove_player(&defender_id);
                    }

                    // Java BB2016: if ball at defender square → clear ball, publish THROW_IN +
                    // ThrowInCoordinate. There is NO sameTeam/END_TURN publish in BB2016.
                    let ball_at_defender = game.field_model.ball_coordinate
                        .map(|bc| bc == defender_coord)
                        .unwrap_or(false);
                    let mut outcome = StepOutcome::next()
                        .publish(StepParameter::InjuryResult(Box::new(injury_result)))
                        .publish(StepParameter::DefenderPushed(true))
                        .publish(StepParameter::StartingPushbackSquare(None));

                    if ball_at_defender {
                        game.field_model.ball_coordinate = None;
                        outcome = outcome
                            .publish(StepParameter::CatchScatterThrowInMode(
                                crate::step::CatchScatterThrowInMode::ThrowIn,
                            ))
                            .publish(StepParameter::ThrowInCoordinate(defender_coord));
                    }

                    self.starting_pushback_square = None;
                    return outcome;
                }

                // Java: if (state.startingPushbackSquare == null) addReport(ReportPushback(...))
                if self.starting_pushback_square.is_none() {
                    game.report_list.add(ReportPushback::new(
                        defender_id.clone(),
                        PushbackMode::REGULAR,
                    ));
                }

                // Java: fieldModel.add(state.pushbackSquares)
                game.field_model.pushback_squares.clear();
                game.field_model.pushback_squares.extend(final_pushback_squares);
                return StepOutcome::cont();
            }
        }

        // Java: if (state.doPush) { ... }
        if do_push {
            let pushes: Vec<(String, FieldCoordinate)> = self.pushback_stack.drain(..).collect();
            let mut extra: Vec<StepParameter> = Vec::new();
            for (player_id, coord) in pushes {
                extra.extend(push_player(game, &player_id, coord));
            }
            game.field_model.pushback_squares.clear();
            self.starting_pushback_square = None;

            let mut outcome = StepOutcome::next()
                .publish(StepParameter::DefenderPushed(true))
                .publish(StepParameter::StartingPushbackSquare(None));
            for p in extra { outcome = outcome.publish(p); }
            outcome
        } else {
            StepOutcome::cont()
        }
    }
}

/// Java: StepPushback.pushPlayer — moves player to coordinate, handles ball interaction.
/// BB2016 only publishes CATCH_SCATTER_THROW_IN_MODE (on ball scatter) and
/// PLAYER_ENTERING_SQUARE — unlike BB2025 it does NOT publish PLAYER_WAS_PUSHED.
fn push_player(game: &mut Game, player_id: &str, coord: FieldCoordinate) -> Vec<StepParameter> {
    // Java: fieldModel.updatePlayerAndBallPosition(pPlayer, pCoordinate)
    game.field_model.set_player_coordinate(player_id, coord);
    UtilServerPlayerMove::update_move_squares(game, false);

    let mut params: Vec<StepParameter> = Vec::new();
    // Java: if (fieldModel.isBallMoving() && pCoordinate.equals(fieldModel.getBallCoordinate()))
    //   publish CatchScatterThrowInMode.SCATTER_BALL
    if game.field_model.ball_moving
        && game.field_model.ball_coordinate.map(|bc| bc == coord).unwrap_or(false)
    {
        params.push(StepParameter::CatchScatterThrowInMode(
            crate::step::CatchScatterThrowInMode::ScatterBall,
        ));
    }
    // Java: publishParameter(PLAYER_ENTERING_SQUARE, pPlayer.getId())
    params.push(StepParameter::PlayerEnteringSquare(player_id.to_owned()));
    params
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::{StepAction, Step};
    use ffb_model::enums::{Rules, PS_STANDING, Direction};
    use ffb_model::model::game::Game;
    use ffb_model::types::FieldCoordinate;
    use ffb_model::util::rng::GameRng;
    use ffb_model::types::PushbackSquare;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2016)
    }

    #[test]
    fn no_starting_square_stays_cont() {
        let mut step = StepPushback::new();
        let mut game = make_game();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::Continue);
    }

    #[test]
    fn old_defender_state_parameter_accepted() {
        let mut step = StepPushback::new();
        let state = PlayerState::new(PS_STANDING);
        let accepted = step.set_parameter(&StepParameter::OldDefenderState(state));
        assert!(accepted);
        assert!(step.old_defender_state.is_some());
    }

    #[test]
    fn starting_pushback_square_parameter_accepted() {
        let mut step = StepPushback::new();
        let coord = FieldCoordinate::new(7, 5);
        let sq = PushbackSquare::new(coord, Direction::North, true);
        let accepted = step.set_parameter(&StepParameter::StartingPushbackSquare(Some(sq)));
        assert!(accepted);
        assert!(step.starting_pushback_square.is_some());
        assert_eq!(step.starting_pushback_square.unwrap().coordinate, coord);
    }

    #[test]
    fn push_to_command_on_empty_square_publishes_defender_pushed() {
        let mut step = StepPushback::new();
        let coord = FieldCoordinate::new(5, 5);
        let mut game = make_game();
        game.defender_id = Some("p1".into());
        let out = step.handle_command(
            &crate::action::Action::PushTo { coord },
            &mut game,
            &mut GameRng::new(0),
        );
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::DefenderPushed(true))));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn unrecognised_parameter_returns_false() {
        let mut step = StepPushback::new();
        let accepted = step.set_parameter(&StepParameter::EndTurn(true));
        assert!(!accepted);
    }

    /// Java bb2016 StepPushback.pushPlayer only publishes CATCH_SCATTER_THROW_IN_MODE and
    /// PLAYER_ENTERING_SQUARE — never PLAYER_WAS_PUSHED (that's BB2025-only). This would have
    /// failed before the fix (re-exported bb2025 impl which does publish PlayerWasPushed).
    #[test]
    fn push_to_command_does_not_publish_player_was_pushed_in_bb2016() {
        let mut step = StepPushback::new();
        let coord = FieldCoordinate::new(5, 5);
        let mut game = make_game();
        game.defender_id = Some("p1".into());
        let out = step.handle_command(
            &crate::action::Action::PushTo { coord },
            &mut game,
            &mut GameRng::new(0),
        );
        assert!(
            !out.published.iter().any(|p| matches!(p, StepParameter::PlayerWasPushed(_))),
            "BB2016 pushPlayer must not publish PLAYER_WAS_PUSHED"
        );
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::PlayerEnteringSquare(_))));
    }

    /// Java bb2016 crowd-push with ball at defender square publishes THROW_IN mode + coordinate,
    /// but never END_TURN — the "if (sameTeam) publishParameter(END_TURN, true)" block only
    /// exists in BB2025's StepPushback.java. This would have failed before the fix.
    #[test]
    fn crowd_push_with_ball_does_not_publish_end_turn_in_bb2016() {
        let mut step = StepPushback::new();
        let coord = FieldCoordinate::new(0, 0);
        step.starting_pushback_square = Some(PushbackSquare::new(coord, Direction::North, true));
        let mut game = make_game();
        game.defender_id = Some("p1".into());
        game.field_model.set_player_coordinate("p1", coord);
        game.field_model.ball_coordinate = Some(coord);
        game.field_model.ball_in_play = true;

        let out = step.start(&mut game, &mut GameRng::new(0));

        if out.action == StepAction::NextStep {
            assert!(
                !out.published.iter().any(|p| matches!(p, StepParameter::EndTurn(_))),
                "BB2016 crowd-push must never publish END_TURN (BB2025-only same-team behavior)"
            );
        }
    }
}
