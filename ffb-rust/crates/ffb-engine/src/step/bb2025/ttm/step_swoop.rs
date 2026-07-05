use ffb_model::enums::{Direction, PlayerState};
use ffb_model::types::FieldCoordinate;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepAction, StepId, StepParameter};
use crate::util::util_server_player_swoop::UtilServerPlayerSwoop;

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
///     // DEFERRED: render animation(thrownPlayerCoordinate -> passCoordinate)
///     // DEFERRED: syncGameModel
///     setPlayerCoordinate(thrownPlayer, passCoordinate)
///     // DEFERRED: changeActingPlayer(thrownPlayerId, SWOOP)
///     // DEFERRED: if blitzTurnState: blitzTurnState.changeActingPlayer()
///     if thrownPlayerHasBall: setBallCoordinate(passCoordinate)
///     setCurrentMove(thrownPlayer.movementWithModifiers - 3)
///     publish THROWN_PLAYER_ID, THROWN_PLAYER_STATE, THROWN_PLAYER_HAS_BALL
///     // DEFERRED: syncGameModel
///
///   if coordinateTo==null:
///     UtilServerPlayerSwoop.updateSwoopSquares(thrownPlayer)
///     publish USING_SWOOP=true
///     return (wait for CLIENT_SWOOP)
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
///   TODO: fieldModel animation, syncGameModel
///   TODO: game.blitzTurnState.changeActingPlayer()
///   TODO: executeStepHooks (Swoop scatter/deflection hook)
///   TODO: coordinate transform for away-team CLIENT_SWOOP command
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
                // Java: CLIENT_USE_SKILL -> usingSwoop = isSkillUsed -> EXECUTE_STEP
                self.using_swoop = Some(*use_skill);
            }
            Action::Pass { coord } => {
                // Java: CLIENT_SWOOP -> coordinateTo = swoopCommand.getTargetCoordinate()
                // Java: if !checkCommandIsFromHomePlayer: coordinateTo = coordinateTo.transform()
                // DEFERRED: away-team coordinate transform
                self.coordinate_to = Some(*coord);
                // Java: executeSwoop() = executeStepHooks(this, state)
                // DEFERRED: executeSwoop hook
                return StepOutcome::next();
            }
            Action::UseReRoll { .. } => {
                // Java: CLIENT_USE_RE_ROLL -> state.reRollSource = command.reRollSource
                //                             state.reRolledAction = command.reRolledAction
                //                             executeSwoop()
                // DEFERRED: extract ReRollSource/ReRolledAction from command; executeSwoop
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
    fn execute_step(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // Java: thrownPlayer = game.getPlayerById(state.thrownPlayerId)
        // Java: if (thrownPlayer == null || state.thrownPlayerCoordinate == null) -> NEXT_STEP
        let player_id = match self.thrown_player_id.as_deref() {
            Some(id) if game.player(id).is_some() => id.to_string(),
            _ => return StepOutcome::next(),
        };
        if self.thrown_player_coordinate.is_none() {
            return StepOutcome::next();
        }

        // Java: if (state.usingSwoop == null):
        //   UtilServerDialog.showDialog(gameState, DialogSkillUseParameter(...))
        //   return  // wait for CLIENT_USE_SKILL
        // DEFERRED: UtilServerDialog.showDialog — return Continue to wait
        if self.using_swoop.is_none() {
            return StepOutcome::cont();
        }

        // Java: if (!state.usingSwoop):
        //   publishParameter(USING_SWOOP, false); NEXT_STEP; return
        if !self.using_swoop.unwrap_or(false) {
            return StepOutcome::next().publish(StepParameter::UsingSwoop(false));
        }

        // usingSwoop == true
        // Java: if (state.throwScatter):
        //   fieldModel.setRangeRuler(null); fieldModel.clearMoveSquares()
        //   passCoordinate = game.getPassCoordinate()
        //   [animation — DEFERRED]
        //   [syncGameModel — DEFERRED]
        //   fieldModel.setPlayerCoordinate(thrownPlayer, passCoordinate)
        //   [UtilActingPlayer.changeActingPlayer — DEFERRED]
        //   [blitzTurnState.changeActingPlayer — DEFERRED]
        //   if thrownPlayerHasBall: setBallCoordinate(passCoordinate)
        //   actingPlayer.setCurrentMove(thrownPlayer.movementWithModifiers - 3)
        //   publish THROWN_PLAYER_ID, THROWN_PLAYER_STATE, THROWN_PLAYER_HAS_BALL
        //   [syncGameModel — DEFERRED]
        let mut outcome = StepOutcome::cont();
        if self.throw_scatter {
            game.field_model.range_ruler = None;
            game.field_model.clear_move_squares();
            if let Some(pass_coord) = game.pass_coordinate {
                // Move player to the pass coordinate
                game.field_model.set_player_coordinate(&player_id, pass_coord);
                // DEFERRED: UtilActingPlayer.changeActingPlayer(game, thrownPlayerId, SWOOP, false)
                // DEFERRED: blitzTurnState.changeActingPlayer()
                if self.thrown_player_has_ball {
                    game.field_model.ball_coordinate = Some(pass_coord);
                }
                // setCurrentMove(thrownPlayer.movementWithModifiers - 3)
                let movement = game.player(&player_id)
                    .map(|p| p.movement_with_modifiers())
                    .unwrap_or(0);
                game.acting_player.current_move = movement - 3;
            }
            // publish THROWN_PLAYER_ID, THROWN_PLAYER_STATE, THROWN_PLAYER_HAS_BALL
            outcome = outcome
                .publish(StepParameter::ThrownPlayerId(self.thrown_player_id.clone()))
                .publish(StepParameter::ThrownPlayerState(self.thrown_player_state.unwrap_or_default()))
                .publish(StepParameter::ThrownPlayerHasBall(self.thrown_player_has_ball));
        }

        // Java: if (state.coordinateTo == null):
        //   UtilServerPlayerSwoop.updateSwoopSquares(gameState, thrownPlayer)
        //   publishParameter(USING_SWOOP, true)
        //   [implicit wait for CLIENT_SWOOP]
        if self.coordinate_to.is_none() {
            UtilServerPlayerSwoop::update_swoop_squares(game, &player_id);
            return outcome.publish(StepParameter::UsingSwoop(true));
        }

        // coordinateTo is known -> executeSwoop hook handles the rest
        // DEFERRED: executeStepHooks(this, state)
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

    fn add_player(game: &mut Game, id: &str) {
        use ffb_model::model::player::Player;
        use ffb_model::enums::{PlayerType, PlayerGender};
        use std::collections::HashSet;
        game.team_home.players.push(Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            ..Default::default()
        });
        game.field_model.set_player_coordinate(id, FieldCoordinate { x: 3, y: 3 });
    }

    #[test]
    fn start_with_player_but_no_swoop_decision_waits() {
        let mut game = make_game();
        add_player(&mut game, "p1");
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
        add_player(&mut game, "p1");
        let mut step = StepSwoop::new("fall".into());
        step.thrown_player_id = Some("p1".into());
        step.thrown_player_coordinate = Some(FieldCoordinate { x: 3, y: 3 });
        use ffb_mechanics::skills::SkillId;
        step.handle_command(&Action::UseSkill { skill_id: SkillId::Swoop, use_skill: false }, &mut game, &mut GameRng::new(0));
        assert_eq!(step.using_swoop, Some(false));
    }

    #[test]
    fn use_skill_false_publishes_using_swoop_false() {
        let mut game = make_game();
        add_player(&mut game, "p1");
        let mut step = StepSwoop::new("fall".into());
        step.thrown_player_id = Some("p1".into());
        step.thrown_player_coordinate = Some(FieldCoordinate { x: 3, y: 3 });
        use ffb_mechanics::skills::SkillId;
        let out = step.handle_command(
            &Action::UseSkill { skill_id: SkillId::Swoop, use_skill: false },
            &mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::UsingSwoop(false))));
        assert_eq!(out.action, StepAction::NextStep);
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

    #[test]
    fn using_swoop_true_with_no_coord_to_publishes_using_swoop_true() {
        use ffb_model::model::player::Player;
        use ffb_model::enums::{PlayerType, PlayerGender};
        use std::collections::HashSet;
        let mut game = make_game();
        let p = Player {
            id: "p1".into(), name: "p1".into(), nr: 1, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            ..Default::default()
        };
        game.team_home.players.push(p);
        game.field_model.set_player_coordinate("p1", FieldCoordinate { x: 5, y: 5 });
        let mut step = StepSwoop::new("fall".into());
        step.thrown_player_id = Some("p1".into());
        step.thrown_player_coordinate = Some(FieldCoordinate { x: 5, y: 5 });
        step.using_swoop = Some(true);
        // coordinate_to is None -> should publish UsingSwoop(true) and wait
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::UsingSwoop(true))));
    }
}
