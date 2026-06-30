/// 1:1 translation of `com.fumbbl.ffb.server.step.mixed.move.StepTrapDoor`.
///
/// When a player enters a Trap Door square, roll 1d6: on a 2+ the player escapes;
/// on a 1 the player falls through the trap door (removed from the pitch, injury
/// applied).  A re-roll may be offered.
///
/// Java `@RulesCollection(BB2020, BB2025)`.
///
/// Incoming parameters: PLAYER_ENTERING_SQUARE (consumed), THROWN_PLAYER_HAS_BALL,
///                       PLAYER_WAS_PUSHED (consumed).
use ffb_model::model::game::Game;
use ffb_model::model::re_rolled_action::ReRolledAction;
use ffb_model::types::FieldCoordinate;
use ffb_model::util::rng::GameRng;
use ffb_model::enums::ApothecaryMode;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};
use crate::step::abstract_step_with_re_roll::ReRollState;

/// Java: `ReRolledActions.TRAP_DOOR` equivalent.
const RE_ROLLED_ACTION: &str = "TRAP_DOOR";

/// Java: `StepTrapDoor` (mixed/move, BB2020 + BB2025).
/// Extends AbstractStepWithReRoll.
#[derive(Debug, Default)]
pub struct StepTrapDoor {
    /// Java: `playerId` — player on the trap door (consumed from PLAYER_ENTERING_SQUARE).
    pub player_id: Option<String>,
    /// Java: `thrownPlayerHasBall` — Some(bool) only in TTM context.
    pub thrown_player_has_ball: Option<bool>,
    /// Java: `playerWasPushed` — whether the push came from a block (consumed).
    pub player_was_pushed: bool,
    /// Re-roll tracking (AbstractStepWithReRoll).
    pub re_roll_state: ReRollState,
}

impl StepTrapDoor {
    pub fn new() -> Self { Self::default() }

    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let player_id = match self.player_id.clone() {
            Some(id) => id,
            None => return StepOutcome::next(),
        };

        // Java: FieldCoordinate playerCoordinate = fieldModel.getPlayerCoordinate(player)
        let player_coord = match game.field_model.player_coordinate(&player_id) {
            Some(c) => c,
            None => return StepOutcome::next(),
        };

        // Java: if (!isOnTrapDoor(fieldModel, playerCoordinate)) { nextStep; return; }
        if !game.field_model.has_trap_door(player_coord) {
            return StepOutcome::next();
        }

        // Java: if (getReRolledAction() == RE_ROLLED_ACTION) { use reroll or fall; }
        if let Some(ref action) = self.re_roll_state.re_rolled_action.clone() {
            if action.get_name() == RE_ROLLED_ACTION {
                // re-roll was asked — check if source is set
                let did_reroll = if let Some(ref src) = self.re_roll_state.re_roll_source.clone() {
                    crate::step::util_server_re_roll::use_reroll(game, src, &player_id)
                } else {
                    false
                };
                if !did_reroll {
                    return self.trap_door_triggered(game, rng, player_id, player_coord);
                }
                // fall through to roll again
            }
        }

        // Java: int roll = getDiceRoller().rollDice(6)
        let roll = rng.d6();
        let escaped = roll != 1;

        // Emit TrapDoor event (ReportTrapDoor)
        let outcome_base = StepOutcome::next()
            .with_event(ffb_model::events::GameEvent::TrapDoor {
                player_id: player_id.clone(),
                roll,
                escaped,
            });

        if escaped {
            return outcome_base;
        }

        // Java: else if (getReRolledAction() != null || !UtilServerReRoll.askForReRollIfAvailable(...))
        if self.re_roll_state.re_rolled_action.is_some() {
            // already re-rolled once — fall through the trap door
            return outcome_base
                .with_events(self.trap_door_triggered(game, rng, player_id, player_coord).events)
                .with_published(self.trap_door_triggered_params(game, player_coord));
        }

        // Offer a re-roll if one is available
        if let Some(prompt) = crate::step::util_server_re_roll::ask_for_reroll_if_available(
            game, RE_ROLLED_ACTION, 2, false,
        ) {
            self.re_roll_state.re_rolled_action = Some(ReRolledAction::new(RE_ROLLED_ACTION));
            return outcome_base.with_prompt(prompt);
        }

        // No re-roll available — fall through
        let mut outcome = outcome_base;
        for p in self.trap_door_triggered_params(game, player_coord) {
            outcome = outcome.publish(p);
        }
        game.field_model.remove_player(&player_id);
        outcome
    }

    /// Java: `trapDoorTriggered` — apply injury, remove player, scatter ball if needed.
    fn trap_door_triggered(&mut self, game: &mut Game, rng: &mut GameRng, player_id: String, coord: FieldCoordinate) -> StepOutcome {
        let mut outcome = StepOutcome::next();
        for p in self.trap_door_triggered_params(game, coord) {
            outcome = outcome.publish(p);
        }
        // Java: game.getFieldModel().remove(player)
        game.field_model.remove_player(&player_id);
        // TODO: call UtilServerInjury.handleInjury with TrapDoorFall / TrapDoorFallForSpp
        // when InjuryType registry is complete.
        outcome
    }

    /// Build the parameters to publish when the trap door triggers.
    fn trap_door_triggered_params(&self, game: &Game, coord: FieldCoordinate) -> Vec<StepParameter> {
        let mut params = Vec::new();
        let player_id = match self.player_id.clone() {
            Some(id) => id,
            None => return params,
        };

        let has_ball = self.thrown_player_has_ball.unwrap_or_else(|| {
            // Java: UtilPlayer.hasBall(game, player)
            game.field_model.ball_coordinate == game.field_model.player_coordinate(&player_id)
        });

        if has_ball {
            params.push(StepParameter::CatchScatterThrowInMode(
                ffb_model::model::catch_scatter_throw_in_mode::CatchScatterThrowInMode::ScatterBall,
            ));
            // Java: if (game.getActingTeam().hasPlayer(player)) publishParameter(END_TURN, true)
            let acting_team_has_player = if game.home_playing {
                game.team_home.players.iter().any(|p| p.id == player_id)
            } else {
                game.team_away.players.iter().any(|p| p.id == player_id)
            };
            if acting_team_has_player {
                params.push(StepParameter::EndTurn(true));
            }
        }

        // Java: if (thrownPlayerHasBall != null) { TTM context cleanup }
        if self.thrown_player_has_ball.is_some() {
            params.push(StepParameter::ThrownPlayerCoordinate(None));
        }

        params
    }
}

// Extension to add published parameters to a StepOutcome
trait WithPublished {
    fn with_published(self, params: Vec<StepParameter>) -> Self;
}

impl WithPublished for StepOutcome {
    fn with_published(mut self, params: Vec<StepParameter>) -> Self {
        self.published.extend(params);
        self
    }
}

impl Step for StepTrapDoor {
    fn id(&self) -> StepId { StepId::TrapDoor }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::PlayerEnteringSquare(id) => {
                // Java: consume and set player_id if that square has a trap door
                // Without the consume mechanism here, just store the id.
                self.player_id = Some(id.clone());
                true
            }
            StepParameter::ThrownPlayerHasBall(v) => { self.thrown_player_has_ball = Some(*v); true }
            _ => false,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{test_team, StepAction};
    use ffb_model::enums::Rules;
    use ffb_model::model::game::Game;
    use ffb_model::util::rng::GameRng;
    use ffb_model::types::FieldCoordinate;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    #[test]
    fn id_is_trap_door() {
        assert_eq!(StepTrapDoor::new().id(), StepId::TrapDoor);
    }

    #[test]
    fn no_player_id_returns_next() {
        let mut step = StepTrapDoor::new();
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn player_not_on_trap_door_returns_next() {
        let mut step = StepTrapDoor::new();
        step.player_id = Some("p1".into());
        let coord = FieldCoordinate::new(5, 5);
        let mut game = make_game();
        // Place player but no trap door
        game.field_model.set_player_coordinate("p1", coord);
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn player_on_trap_door_rolls_and_emits_event() {
        let mut step = StepTrapDoor::new();
        step.player_id = Some("p1".into());
        let coord = FieldCoordinate::new(5, 5);
        let mut game = make_game();
        game.field_model.set_player_coordinate("p1", coord);
        game.field_model.trap_doors.push(coord);
        // Seed 5 → roll_d6 ≥ 2 → escaped = true
        let mut rng = GameRng::new(5);
        let out = step.start(&mut game, &mut rng);
        let has_trap_door_event = out.events.iter().any(|e| matches!(e, ffb_model::events::GameEvent::TrapDoor { .. }));
        assert!(has_trap_door_event);
    }

    #[test]
    fn set_parameter_thrown_player_has_ball() {
        let mut step = StepTrapDoor::new();
        step.set_parameter(&StepParameter::ThrownPlayerHasBall(true));
        assert_eq!(step.thrown_player_has_ball, Some(true));
    }
}
