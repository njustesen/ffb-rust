/// 1:1 translation of com.fumbbl.ffb.server.step.phase.kickoff.StepTouchback.
///
/// Expects stepParameter TOUCHBACK to be set by a preceding step.
///
/// When touchback is true: waits for CLIENT_TOUCHBACK (Action::Touchback), then places the
/// ball at the chosen player's coordinate. Also sets TurnMode::REGULAR after placement.
///
/// Sets stepParameter CATCH_SCATTER_THROW_IN_MODE when a player without PREVENT_HOLD_BALL
/// receives the ball at a position with tackle zones.
use ffb_model::enums::TurnMode;
use ffb_model::model::game::Game;
use ffb_model::model::property::NamedProperties;
use ffb_model::prompts::AgentPrompt;
use ffb_model::types::FieldCoordinate;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{CatchScatterThrowInMode, Step, StepId, StepOutcome, StepParameter};

pub struct StepTouchback {
    /// Java: fTouchback — whether a touchback was triggered.
    touchback: bool,
    /// Java: fTouchbackCoordinate — the coordinate chosen by the coach (None until chosen).
    touchback_coordinate: Option<FieldCoordinate>,
}

impl StepTouchback {
    pub fn new() -> Self {
        Self { touchback: false, touchback_coordinate: None }
    }
}

impl Default for StepTouchback {
    fn default() -> Self { Self::new() }
}

impl Step for StepTouchback {
    fn id(&self) -> StepId { StepId::Touchback }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: case CLIENT_TOUCHBACK: fTouchbackCoordinate = touchbackCommand.getBallCoordinate()
        // In Rust, the client sends the player_id; we resolve the coordinate from field_model.
        if let Action::Touchback { player_id } = action {
            self.touchback_coordinate = game.field_model.player_coordinate(player_id);
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::Touchback(v) => {
                self.touchback = *v;
                true
            }
            _ => false,
        }
    }
}

impl StepTouchback {
    fn execute_step(&self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // Java: boolean doNextStep = true;
        let mut do_next_step = true;

        if self.touchback {
            if self.touchback_coordinate.is_none() {
                // Java: game.getFieldModel().setBallCoordinate(null)
                game.field_model.ball_coordinate = None;
                // Java: game.setTurnMode(TurnMode.TOUCHBACK)
                game.turn_mode = TurnMode::Touchback;
                // Java: game.setDialogParameter(new DialogTouchbackParameter())
                // → in Rust: build eligible-players list and return cont() with prompt
                do_next_step = false;
                let receiving_team = if game.home_playing { &game.team_away } else { &game.team_home };
                let eligible: Vec<(String, FieldCoordinate)> = receiving_team
                    .players
                    .iter()
                    .filter_map(|p| {
                        let coord = game.field_model.player_coordinate(&p.id)?;
                        if !coord.is_on_pitch() { return None; }
                        let ps = game.field_model.player_state(&p.id)?;
                        if !ps.has_tacklezones() { return None; }
                        Some((p.id.clone(), coord))
                    })
                    .collect();
                return StepOutcome::cont().with_prompt(AgentPrompt::Touchback { eligible_players: eligible });
            } else {
                let coord = self.touchback_coordinate.unwrap();
                // Java: game.getFieldModel().setOutOfBounds(false)
                game.field_model.out_of_bounds = false;
                // client-only: UtilServerDialog.hideDialog(getGameState())
                // Java: game.getFieldModel().setBallCoordinate(fTouchbackCoordinate)
                game.field_model.ball_coordinate = Some(coord);
                // Java: Player<?> player = game.getFieldModel().getPlayer(fTouchbackCoordinate)
                // Java always falls through to `game.setTurnMode(TurnMode.REGULAR)` after this
                // block, regardless of which sub-branch ran — publishParameter() is a plain
                // statement in Java, not an early return, so it must not skip the turn-mode set.
                let mut publish_catch_scatter = false;
                if let Some(player_id) = game.field_model.player_at(coord).cloned() {
                    // Java: PlayerState playerState = game.getFieldModel().getPlayerState(player)
                    let player_state = game.field_model.player_state(&player_id);
                    // Java: find the player object to call hasSkillProperty
                    let prevents_hold = game.team_home.players.iter()
                        .chain(game.team_away.players.iter())
                        .find(|p| p.id == player_id)
                        .map(|p| p.has_skill_property(NamedProperties::PREVENT_HOLD_BALL))
                        .unwrap_or(false);
                    let has_tz = player_state.map(|ps| ps.has_tacklezones()).unwrap_or(false);
                    if !prevents_hold && has_tz {
                        // Java: game.getFieldModel().setBallMoving(false)
                        game.field_model.ball_moving = false;
                        // client-only: getResult().setSound(SoundId.CATCH)
                    } else {
                        // Java: publishParameter(CATCH_SCATTER_THROW_IN_MODE, CATCH_KICKOFF)
                        publish_catch_scatter = true;
                    }
                }
                // Java: game.setTurnMode(TurnMode.REGULAR)
                game.turn_mode = TurnMode::Regular;
                if publish_catch_scatter {
                    return StepOutcome::next().publish(StepParameter::CatchScatterThrowInMode(
                        CatchScatterThrowInMode::CatchKickoff,
                    ));
                }
            }
        }

        if do_next_step {
            StepOutcome::next()
        } else {
            // Already returned cont() above
            unreachable!()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::enums::Rules;
    use ffb_model::util::rng::GameRng;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    #[test]
    fn step_id_is_touchback() {
        assert_eq!(StepTouchback::new().id(), StepId::Touchback);
    }

    #[test]
    fn no_touchback_returns_next_step_immediately() {
        let mut game = make_game();
        let mut step = StepTouchback::new();
        // touchback = false: no dialog needed, proceed
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn touchback_true_without_coordinate_returns_cont() {
        let mut game = make_game();
        let mut step = StepTouchback::new();
        step.set_parameter(&StepParameter::Touchback(true));
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::Continue, "should wait for player choice");
        // Java: game.setTurnMode(TurnMode.TOUCHBACK)
        assert_eq!(game.turn_mode, TurnMode::Touchback);
        // Java: game.getFieldModel().setBallCoordinate(null)
        assert!(game.field_model.ball_coordinate.is_none());
    }

    #[test]
    fn set_parameter_touchback_accepted() {
        let mut step = StepTouchback::new();
        assert!(step.set_parameter(&StepParameter::Touchback(true)));
        assert!(step.touchback);
    }

    #[test]
    fn set_parameter_unrecognized_returns_false() {
        let mut step = StepTouchback::new();
        assert!(!step.set_parameter(&StepParameter::EndTurn(false)));
    }

    #[test]
    fn touchback_with_coordinate_resolves_turn_mode_regular() {
        let mut game = make_game();
        let mut step = StepTouchback::new();
        step.set_parameter(&StepParameter::Touchback(true));
        // Inject a coordinate directly to simulate that the command was received
        let coord = FieldCoordinate::new(5, 8);
        step.touchback_coordinate = Some(coord);
        game.field_model.out_of_bounds = true;
        let out = step.execute_step(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        // Java: game.setTurnMode(TurnMode.REGULAR)
        assert_eq!(game.turn_mode, TurnMode::Regular);
        // Java: game.getFieldModel().setOutOfBounds(false)
        assert!(!game.field_model.out_of_bounds);
        // Java: game.getFieldModel().setBallCoordinate(fTouchbackCoordinate)
        assert_eq!(game.field_model.ball_coordinate, Some(coord));
    }

    /// Java: `publishParameter(CATCH_SCATTER_THROW_IN_MODE, ...)` is a plain statement,
    /// followed unconditionally by `game.setTurnMode(TurnMode.REGULAR)` — the turn mode
    /// must still be set to REGULAR even when the ball lands on a player without
    /// tacklezones (triggering the catch-scatter-throw-in branch instead of a catch).
    #[test]
    fn touchback_catch_scatter_branch_still_sets_turn_mode_regular() {
        use ffb_model::enums::{PS_PRONE, PlayerState};

        let mut game = make_game();
        let mut step = StepTouchback::new();
        step.set_parameter(&StepParameter::Touchback(true));

        let coord = FieldCoordinate::new(5, 8);
        step.touchback_coordinate = Some(coord);

        // Place a prone player (no tacklezones) at the touchback coordinate so the
        // "else" branch (publish CatchScatterThrowInMode) is taken instead of the catch.
        let mut player = ffb_model::model::player::Player::default();
        player.id = "p1".into();
        game.team_home.players.push(player);
        game.field_model.set_player_coordinate("p1", coord);
        game.field_model.set_player_state("p1", PlayerState::new(PS_PRONE));

        let out = step.execute_step(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        // The catch-scatter-throw-in mode should be published...
        let published_catch_scatter = out.published.iter().any(|p| {
            matches!(p, StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::CatchKickoff))
        });
        assert!(published_catch_scatter, "should publish CatchScatterThrowInMode::CatchKickoff");
        // ...but turn mode must still become REGULAR (Java always runs this afterward).
        assert_eq!(game.turn_mode, TurnMode::Regular, "turn mode must be REGULAR even on the catch-scatter branch");
    }

    #[test]
    fn default_creates_fresh_instance() {
        let step = StepTouchback::default();
        assert!(!step.touchback);
        assert!(step.touchback_coordinate.is_none());
    }
}
