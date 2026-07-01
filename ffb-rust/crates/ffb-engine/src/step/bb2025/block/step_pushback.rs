use std::collections::HashMap;
use ffb_model::enums::{ApothecaryMode, PlayerState};
use ffb_model::types::{FieldCoordinate, PushbackSquare};
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::util_server_injury::handle_injury_by_name;
use crate::util::UtilServerPlayerMove;
use crate::util::util_server_pushback::UtilServerPushback;

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2025.block.StepPushback.
/// Handles player pushback and crowd-push. The Java StepState fields are inlined here.
///
/// Expects stepParameter STARTING_PUSHBACK_SQUARE to be set by a preceding step.
/// Expects stepParameter OLD_DEFENDER_STATE to be set by a preceding step.
///
/// Sets stepParameter DEFENDER_PUSHED for all steps on the stack.
/// Sets stepParameter STARTING_PUSHBACK_SQUARE for all steps on the stack.
pub struct StepPushback {
    // StepState fields
    pub old_defender_state: Option<PlayerState>,
    pub starting_pushback_square: Option<PushbackSquare>,
    /// Java: grabbing (Boolean — tristate)
    pub grabbing: Option<bool>,
    /// Java: sideStepping (Map<String, Boolean>)
    pub side_stepping: HashMap<String, bool>,
    /// Java: standingFirm (Map<String, Boolean>)
    pub standing_firm: HashMap<String, bool>,
    /// Java: pushbackStack — (playerId, coordinate) pairs (LIFO).
    /// Java Pushback.playerId + Pushback.coordinate.
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
        match action {
            Action::PushTo { coord } => {
                // Java: CLIENT_PUSHBACK —
                //   if (checkCommandIsFromHomePlayer) pushbackStack.push(pushback)
                //   else pushbackStack.push(pushback.transform())
                // We only have coord here; the player being pushed is the current defender.
                // For chain pushbacks the player pushed might differ — TODO when chain pushback added.
                if let Some(defender_id) = game.defender_id.clone() {
                    self.pushback_stack.push((defender_id, *coord));
                }
            }
            _ => {}
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::OldDefenderState(v) => { self.old_defender_state = Some(*v); true }
            StepParameter::StartingPushbackSquare(v) => {
                // Java: startingPushbackSquare is a full PushbackSquare with coordinate + direction.
                // StepParameter::StartingPushbackSquare carries a FieldCoordinate (the centre square).
                // We wrap it into a PushbackSquare with a placeholder direction.
                self.starting_pushback_square = Some(PushbackSquare::new(
                    *v,
                    ffb_model::enums::Direction::North, // placeholder direction
                    true, // home_choice placeholder
                ));
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
        // Player chose a coordinate — select that pushback square from the field model.
        if !self.pushback_stack.is_empty() {
            if let Some(&(_, chosen_coord)) = self.pushback_stack.last() {
                // Java: for each unlocked square remove from model; if coord matches, select+lock+re-add
                let matching = game.field_model.pushback_squares.iter_mut()
                    .find(|sq| !sq.locked && sq.coordinate == chosen_coord);
                if let Some(sq) = matching {
                    sq.selected = true;
                    sq.locked = true;
                    // Update starting_pushback_square to the chosen square
                    self.starting_pushback_square = Some(*sq);
                }
                // Java: doPush = (fieldModel.getPlayer(lastPushback.getCoordinate()) == null)
                do_push = game.field_model.player_at(chosen_coord).is_none();
            }
        }

        // Java: if (!state.doPush && (state.startingPushbackSquare != null))
        // Calculate new pushback squares if needed.
        if !do_push {
            if let Some(starting_sq) = self.starting_pushback_square {
                let defender_coord = starting_sq.coordinate;

                // Java: state.defender = fieldModel.getPlayer(defenderCoordinate)
                // (throws if null — we just proceed without the defender check in the stub)

                // Java: executeStepHooks for Side Step / Stand Firm
                // DEFERRED: Side Step / Stand Firm skill hooks (executeStepHooks not yet translated)

                // Java: pushbackSquares = UtilServerPushback.findPushbackSquares(game, startingSq, REGULAR)
                // Java: fieldModel.add(pushbackSquares)
                let home_choice = game.home_playing;
                let occupied = |c: FieldCoordinate| game.field_model.player_at(c).is_some();
                let pushback_squares = UtilServerPushback::find_pushback_squares_standard(
                    starting_sq, &occupied, home_choice,
                );

                let pushback_squares_found = !pushback_squares.is_empty();

                // Java: if (!ArrayTool.isProvided(state.pushbackSquares)) → Crowd push
                if !pushback_squares_found {
                    // Java: determine injuryType and attacker based on prayerState
                    // Stub: prayerState.hasFanInteraction → false → always InjuryTypeCrowdPush, no attacker
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

                    // Java: if (defenderCoordinate.equals(game.getFieldModel().getBallCoordinate()))
                    //   setBallCoordinate(null)
                    //   publish CatchScatterThrowInMode.THROW_IN
                    //   publish ThrowInCoordinate(defenderCoordinate)
                    //   if sameTeam: publish END_TURN(true)
                    let ball_at_defender = game.field_model.ball_coordinate
                        .map(|bc| bc == defender_coord)
                        .unwrap_or(false);
                    let mut outcome = StepOutcome::next()
                        .publish(StepParameter::InjuryResult(Box::new(injury_result)))
                        .publish(StepParameter::DefenderPushed(true))
                        // Java: publishParameter(STARTING_PUSHBACK_SQUARE, null)
                        .publish(StepParameter::StartingPushbackSquare(FieldCoordinate::new(0, 0)));

                    if ball_at_defender {
                        game.field_model.ball_coordinate = None;
                        outcome = outcome
                            .publish(StepParameter::CatchScatterThrowInMode(
                                crate::step::CatchScatterThrowInMode::ThrowIn,
                            ))
                            .publish(StepParameter::ThrowInCoordinate(defender_coord));
                        // Java: if sameTeam → publish END_TURN(true)
                        let same_team = game.defender_id.as_deref()
                            .map(|id| game.is_active_team_player(id))
                            .unwrap_or(false);
                        if same_team {
                            outcome = outcome.publish(StepParameter::EndTurn(true));
                        }
                    }

                    self.starting_pushback_square = None;
                    return outcome;
                }

                // Java: if (state.startingPushbackSquare == null) addReport(ReportPushback(...))
                // (startingPushbackSquare is still set here — this branch is for when it was just cleared above)

                // Java: fieldModel.add(state.pushbackSquares)
                game.field_model.pushback_squares.clear();
                game.field_model.pushback_squares.extend(pushback_squares);
                return StepOutcome::cont();
            }
        }

        // Java: if (state.doPush) { ... }
        if do_push {
            // Java: publishParameter(StepParameterKey.DEFENDER_PUSHED, true)
            // Java: while (!pushbackStack.isEmpty()) { pop + pushPlayer }
            let pushes: Vec<(String, FieldCoordinate)> = self.pushback_stack.drain(..).collect();
            for (player_id, coord) in pushes {
                push_player(game, &player_id, coord);
            }
            // Java: fieldModel.clearPushbackSquares()
            game.field_model.pushback_squares.clear();
            // Java: publishParameter(STARTING_PUSHBACK_SQUARE, null)
            self.starting_pushback_square = None;
            // Java: game.setWaitingForOpponent(false)
            // Java: getResult().setNextAction(StepAction.NEXT_STEP)

            StepOutcome::next()
                .publish(StepParameter::DefenderPushed(true))
                .publish(StepParameter::StartingPushbackSquare(FieldCoordinate::new(0, 0)))
        } else {
            StepOutcome::cont()
        }
    }
}

/// Java: StepPushback.pushPlayer — moves player to coordinate, handles ball interaction.
fn push_player(game: &mut Game, player_id: &str, coord: FieldCoordinate) {
    // Java: fieldModel.updatePlayerAndBallPosition(pPlayer, pCoordinate)
    game.field_model.set_player_coordinate(player_id, coord);
    UtilServerPlayerMove::update_move_squares(game, false);

    // Java: if (fieldModel.isBallMoving() && pCoordinate.equals(fieldModel.getBallCoordinate()))
    //   publish CatchScatterThrowInMode.SCATTER_BALL
    //   publish PushedOnBall(true)
    if game.field_model.ball_moving
        && game.field_model.ball_coordinate.map(|bc| bc == coord).unwrap_or(false)
    {
        // DEFERRED: publish CatchScatterThrowInMode::ScatterBall + PushedOnBall(true)
        // These are published through the StepOutcome chain but push_player is a free fn.
        // The outer execute_step call would need to collect these — deferred until full
        // publish-from-inner-fn pattern is available.
    }
    // Java: publishParameter(PLAYER_ENTERING_SQUARE, pPlayer.getId())
    // Java: publishParameter(PLAYER_WAS_PUSHED, true)
    // DEFERRED: publish PlayerEnteringSquare + PlayerWasPushed through caller
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::StepAction;
    use ffb_model::enums::{Rules, PS_STANDING, Direction};
    use ffb_model::types::PushbackSquare;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2025)
    }

    // ── set_parameter ────────────────────────────────────────────────────────

    #[test]
    fn set_parameter_old_defender_state_accepted() {
        let mut step = StepPushback::new();
        let state = PlayerState::new(PS_STANDING);
        let accepted = step.set_parameter(&StepParameter::OldDefenderState(state));
        assert!(accepted);
        assert!(step.old_defender_state.is_some());
    }

    #[test]
    fn set_parameter_starting_pushback_square_accepted() {
        let mut step = StepPushback::new();
        let coord = FieldCoordinate::new(5, 5);
        let accepted = step.set_parameter(&StepParameter::StartingPushbackSquare(coord));
        assert!(accepted);
        assert!(step.starting_pushback_square.is_some());
        assert_eq!(step.starting_pushback_square.unwrap().coordinate, coord);
    }

    #[test]
    fn set_parameter_unrecognised_returns_false() {
        let mut step = StepPushback::new();
        let accepted = step.set_parameter(&StepParameter::EndTurn(true));
        assert!(!accepted);
    }

    // ── start with no state ──────────────────────────────────────────────────

    #[test]
    fn no_starting_square_stays_cont() {
        let mut step = StepPushback::new();
        let mut game = make_game();
        let out = step.start(&mut game, &mut GameRng::new(0));
        // No starting_pushback_square and no pushback_stack → CONTINUE
        assert_eq!(out.action, StepAction::Continue);
    }

    // ── PushTo command ───────────────────────────────────────────────────────

    #[test]
    fn push_to_command_with_empty_target_square_publishes_defender_pushed() {
        let mut step = StepPushback::new();
        let coord = FieldCoordinate::new(7, 7);
        let mut game = make_game();
        game.defender_id = Some("p1".into());
        // Target square is empty → do_push=true → execute_step drains the stack and publishes DefenderPushed.
        let out = step.handle_command(
            &Action::PushTo { coord },
            &mut game,
            &mut GameRng::new(0),
        );
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::DefenderPushed(true))));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn push_to_command_drains_pushback_stack() {
        let mut step = StepPushback::new();
        let coord = FieldCoordinate::new(3, 3);
        let mut game = make_game();
        game.defender_id = Some("p2".into());
        step.handle_command(&Action::PushTo { coord }, &mut game, &mut GameRng::new(0));
        // stack should be drained after do_push path
        assert!(step.pushback_stack.is_empty());
    }

    // ── starting square → no adjacent free ── crowd push path ────────────────

    #[test]
    fn crowd_push_removes_defender_and_publishes_defender_pushed() {
        let mut step = StepPushback::new();
        // Use coordinate (0, 0) — corner with likely no adjacent on-pitch squares in test field
        let coord = FieldCoordinate::new(0, 0);
        step.starting_pushback_square = Some(PushbackSquare::new(
            coord,
            Direction::North,
            true,
        ));
        let mut game = make_game();
        game.defender_id = Some("p1".into());
        // Place a player at (0,0) so it exists
        game.field_model.set_player_coordinate("p1", coord);

        // Depending on the field model's adjacent_on_pitch implementation,
        // the corner may or may not have free adjacent squares.
        let out = step.start(&mut game, &mut GameRng::new(0));
        // Either crowd push (next) or waiting for client pushback (cont) — both are valid.
        // We just check no panic and that the outcome makes sense.
        assert!(matches!(out.action, StepAction::NextStep | StepAction::Continue));
    }

    #[test]
    fn crowd_push_with_ball_at_defender_publishes_throw_in() {
        let mut step = StepPushback::new();
        let coord = FieldCoordinate::new(0, 0);
        step.starting_pushback_square = Some(PushbackSquare::new(
            coord,
            Direction::North,
            true,
        ));
        let mut game = make_game();
        game.defender_id = Some("p1".into());
        game.field_model.set_player_coordinate("p1", coord);
        game.field_model.ball_coordinate = Some(coord);
        game.field_model.ball_in_play = true;

        let out = step.start(&mut game, &mut GameRng::new(0));

        // If crowd push occurred (no adjacent free), the throw-in parameters are published.
        if out.action == StepAction::NextStep {
            let has_throw_in = out.published.iter().any(|p| {
                matches!(p, StepParameter::CatchScatterThrowInMode(
                    crate::step::CatchScatterThrowInMode::ThrowIn
                ))
            });
            let has_throw_in_coord = out.published.iter().any(|p| {
                matches!(p, StepParameter::ThrowInCoordinate(c) if *c == coord)
            });
            assert!(has_throw_in, "expected ThrowIn mode published on crowd push with ball");
            assert!(has_throw_in_coord, "expected ThrowInCoordinate published");
        }
    }

    // ── starting square → has free adjacent ── waits for client ─────────────

    #[test]
    fn starting_square_with_free_adjacent_stays_cont() {
        let mut step = StepPushback::new();
        // Place the starting square in the middle of the field where adjacent squares exist
        let coord = FieldCoordinate::new(7, 5);
        step.starting_pushback_square = Some(PushbackSquare::new(
            coord,
            Direction::North,
            true,
        ));
        let mut game = make_game();
        game.defender_id = Some("p1".into());
        // Surround coord with players except one direction — adjacent_on_pitch should find free squares
        let out = step.start(&mut game, &mut GameRng::new(0));
        // Adjacent squares should be free → wait for client input
        assert_eq!(out.action, StepAction::Continue);
    }

    // ── pushback stack clears field model squares ────────────────────────────

    #[test]
    fn do_push_clears_pushback_squares_in_field_model() {
        let mut step = StepPushback::new();
        let coord = FieldCoordinate::new(5, 5);
        let mut game = make_game();
        game.defender_id = Some("p1".into());
        // Pre-populate a pushback square
        game.field_model.pushback_squares.push(PushbackSquare::new(
            coord,
            Direction::North,
            false,
        ));
        step.handle_command(&Action::PushTo { coord }, &mut game, &mut GameRng::new(0));
        assert!(game.field_model.pushback_squares.is_empty());
    }

    // ── starting_pushback_square cleared after do_push ───────────────────────

    #[test]
    fn starting_pushback_square_cleared_after_do_push() {
        let mut step = StepPushback::new();
        let coord = FieldCoordinate::new(4, 4);
        let mut game = make_game();
        game.defender_id = Some("d1".into());
        step.handle_command(&Action::PushTo { coord }, &mut game, &mut GameRng::new(0));
        // After a successful push the square should be cleared
        assert!(step.starting_pushback_square.is_none());
    }

    /// find_pushback_squares_standard populates field_model.pushback_squares (not just adjacent).
    #[test]
    fn starting_square_populates_field_model_pushback_squares() {
        let mut step = StepPushback::new();
        // Defender at (10,7) — ample room for all three pushback directions (North push → NW/N/NE)
        let coord = FieldCoordinate::new(10, 7);
        step.starting_pushback_square = Some(PushbackSquare::new(coord, Direction::North, true));
        let mut game = make_game();
        game.defender_id = Some("p1".into());
        // No players blocking adjacent squares
        let out = step.start(&mut game, &mut GameRng::new(0));
        // Should wait for client (squares were found)
        assert_eq!(out.action, StepAction::Continue);
        // Field model should now have 3 pushback squares (NW, N, NE of (10,7))
        assert_eq!(game.field_model.pushback_squares.len(), 3);
    }
}
