use ffb_model::enums::{Direction, PlayerState};
use ffb_model::types::FieldCoordinate;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepAction, StepId, StepParameter};

/// Handles the Swoop skill: deflects a scattered TTM player toward a chosen target square.
///
/// Java executeStep logic:
///   thrownPlayer = game.getPlayerById(thrownPlayerId)
///   if thrownPlayer==null || thrownPlayerCoordinate==null -> NEXT_STEP; return
///
///   if usingSwoop==null:
///     showDialog(DialogSkillUseParameter(thrownPlayer, swoopSkill))
///     return (wait for CLIENT_USE_SKILL)
///
///   if !usingSwoop:
///     publish USING_SWOOP=false; NEXT_STEP; return
///
///   passCoordinate = game.passCoordinate
///   if throwScatter:
///     game.fieldModel.setRangeRuler(null); clearMoveSquares
///     render animation(thrownPlayerCoordinate -> passCoordinate)
///     syncGameModel
///     setPlayerCoordinate(thrownPlayer, passCoordinate)
///     changeActingPlayer(thrownPlayerId, SWOOP)
///     if blitzTurnState: blitzTurnState.changeActingPlayer()
///     if thrownPlayerHasBall: setBallCoordinate(passCoordinate)
///     setCurrentMove(thrownPlayer.movementWithModifiers - 3)
///     publish THROWN_PLAYER_ID, THROWN_PLAYER_STATE, THROWN_PLAYER_HAS_BALL
///     syncGameModel
///
///   if coordinateTo==null:
///     UtilServerPlayerSwoop.updateSwoopSquares(thrownPlayer)
///     publish USING_SWOOP=true
///   // else: coordinateTo was set by CLIENT_SWOOP -> executeSwoop hook runs
///
/// handleCommand additionally handles:
///   CLIENT_USE_SKILL -> usingSwoop = isSkillUsed -> executeStep
///   CLIENT_SWOOP -> coordinateTo = swoopCommand.targetCoordinate -> executeSwoop()
///   CLIENT_USE_RE_ROLL -> reRollSource / reRolledAction -> executeSwoop()
///
/// executeSwoop delegates to: getGameState().executeStepHooks(this, state)
///
/// Unported utilities:
///   TODO: UtilServerDialog.showDialog (Swoop skill dialog)
///   TODO: UtilActingPlayer.changeActingPlayer(game, thrownPlayerId, SWOOP)
///   TODO: UtilServerPlayerSwoop.updateSwoopSquares
///   TODO: game.setPassCoordinate / getPassCoordinate
///   TODO: fieldModel animation, clearMoveSquares
///   TODO: executeStepHooks (Swoop scatter/deflection hook)
///   TODO: game.blitzTurnState.changeActingPlayer()
///
/// Mirrors Java `com.fumbbl.ffb.server.step.bb2025.ttm.StepSwoop`.
pub struct StepSwoop {
    /// Java: state.status (ActionStatus)
    pub status: Option<String>,
    /// Java: state.thrownPlayerId (mandatory init param)
    pub thrown_player_id: Option<String>,
    /// Java: state.thrownPlayerState (mandatory init param)
    pub thrown_player_state: Option<PlayerState>,
    /// Java: state.thrownPlayerHasBall (init param)
    pub thrown_player_has_ball: bool,
    /// Java: state.thrownPlayerCoordinate (mandatory init param)
    pub thrown_player_coordinate: Option<FieldCoordinate>,
    /// Java: state.throwScatter (init param)
    pub throw_scatter: bool,
    /// Java: state.coordinateFrom (set by COORDINATE_FROM parameter)
    pub coordinate_from: Option<FieldCoordinate>,
    /// Java: state.coordinateTo (set by CLIENT_SWOOP command)
    pub coordinate_to: Option<FieldCoordinate>,
    /// Java: state.goToLabelOnFallDown (mandatory init param)
    pub goto_label_on_fall_down: String,
    /// Java: state.usingSwoop (Boolean tristate — None=not yet asked)
    pub using_swoop: Option<bool>,
    /// Java: state.swoopDirection (Direction enum)
    pub swoop_direction: Option<Direction>,
    /// Java: state.reRolledAction
    pub re_rolled_action: Option<String>,
    /// Java: state.reRollSource
    pub re_roll_source: Option<String>,
}

impl StepSwoop {
    pub fn new(goto_label_on_fall_down: String) -> Self {
        Self {
            status: None,
            thrown_player_id: None,
            thrown_player_state: None,
            thrown_player_has_ball: false,
            thrown_player_coordinate: None,
            throw_scatter: false,
            coordinate_from: None,
            coordinate_to: None,
            goto_label_on_fall_down,
            using_swoop: None,
            swoop_direction: None,
            re_rolled_action: None,
            re_roll_source: None,
        }
    }
}

impl Default for StepSwoop {
    fn default() -> Self { Self::new(String::new()) }
}

impl Step for StepSwoop {
    fn id(&self) -> StepId { StepId::Swoop }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        match action {
            Action::UseSkill { use_skill, .. } => {
                // CLIENT_USE_SKILL -> usingSwoop = isSkillUsed
                self.using_swoop = Some(*use_skill);
            }
            Action::Pass { coord } => {
                // CLIENT_SWOOP -> coordinateTo (Swoop target square selection)
                // TODO: transform coordinate for away team
                self.coordinate_to = Some(*coord);
                // TODO: executeSwoop() (hooks)
                return StepOutcome::next();
            }
            Action::UseReRoll { use_reroll: _ } => {
                // CLIENT_USE_RE_ROLL -> update reRollSource/reRolledAction -> executeSwoop
                // TODO: extract ReRollSource/ReRolledAction from command
                // TODO: executeSwoop() (hooks)
                return StepOutcome::next();
            }
            _ => {}
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::CoordinateFrom(v) => { self.coordinate_from = Some(*v); true }
            StepParameter::CoordinateTo(v) => { self.coordinate_to = Some(*v); true }
            _ => false,
        }
    }
}

impl StepSwoop {
    fn execute_step(&self, _game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // Guard: thrown player and coordinate must be present.
        if self.thrown_player_id.is_none() || self.thrown_player_coordinate.is_none() {
            return StepOutcome::next();
        }

        // TODO: if using_swoop == None: show DialogSkillUseParameter -> wait (Continue)
        if self.using_swoop.is_none() {
            return StepOutcome::cont();
        }

        if !self.using_swoop.unwrap_or(false) {
            // TODO: publish USING_SWOOP=false
            return StepOutcome::next();
        }

        // usingSwoop==true
        // TODO: if throwScatter: animate + move player + update movement points
        // TODO: if coordinateTo==None: updateSwoopSquares + publish USING_SWOOP=true -> wait
        if self.coordinate_to.is_none() {
            // TODO: UtilServerPlayerSwoop.updateSwoopSquares; publish USING_SWOOP=true
            return StepOutcome::cont();
        }

        // coordinateTo known -> executeSwoop hook handles the rest
        // TODO: executeStepHooks(this, state)
        StepOutcome::next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::StepAction;
    use ffb_model::enums::Rules;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    #[test]
    fn start_no_player_returns_next_step() {
        let mut game = make_game();
        let mut step = StepSwoop::new("fall".into());
        // thrown_player_id is None -> guard -> NEXT_STEP
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn start_with_player_but_no_swoop_decision_waits() {
        let mut game = make_game();
        let mut step = StepSwoop::new("fall".into());
        step.thrown_player_id = Some("p1".into());
        step.thrown_player_coordinate = Some(FieldCoordinate { x: 3, y: 3 });
        // using_swoop is None -> must show dialog -> Continue
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::Continue);
    }

    #[test]
    fn use_skill_false_stores_decision() {
        let mut game = make_game();
        let mut step = StepSwoop::new("fall".into());
        step.thrown_player_id = Some("p1".into());
        step.thrown_player_coordinate = Some(FieldCoordinate { x: 3, y: 3 });
        use ffb_mechanics::skills::SkillId;
        step.handle_command(&Action::UseSkill { skill_id: SkillId::Swoop, use_skill: false }, &mut game, &mut GameRng::new(0));
        assert_eq!(step.using_swoop, Some(false));
    }

    #[test]
    fn set_coordinate_from_accepted() {
        let mut step = StepSwoop::default();
        let c = FieldCoordinate { x: 1, y: 2 };
        assert!(step.set_parameter(&StepParameter::CoordinateFrom(c)));
        assert_eq!(step.coordinate_from, Some(c));
    }

    #[test]
    fn set_coordinate_to_accepted() {
        let mut step = StepSwoop::default();
        let c = FieldCoordinate { x: 3, y: 4 };
        assert!(step.set_parameter(&StepParameter::CoordinateTo(c)));
        assert_eq!(step.coordinate_to, Some(c));
    }
}
