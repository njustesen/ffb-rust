use ffb_model::enums::{Direction, PlayerState};
use ffb_model::types::FieldCoordinate;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepAction, StepId, StepParameter};

/// Scatters the thrown/kicked player to their landing square.
///
/// Java executeStep logic:
///   thrownPlayer = game.getPlayerById(thrownPlayerId)
///   swooping = swoopDirection != null && usingSwoop
///
///   // Change base state PICKED_UP -> IN_THE_AIR (unless swooping)
///   if !swooping && thrownPlayerState.base == PICKED_UP:
///     thrownPlayerState = thrownPlayerState.changeBase(IN_THE_AIR)
///     fieldModel.setPlayerState(thrownPlayer, thrownPlayerState)
///
///   if thrownPlayer==null || thrownPlayerCoordinate==null -> NEXT_STEP; return
///
///   if usingBullseye:
///     clearRangeRuler; clearMoveSquares
///     endCoordinate = game.passCoordinate
///     setAnimation(startCoord -> endCoord)
///     handleLanding(thrownPlayer, endCoordinate); return
///
///   startCoordinate = thrownPlayerCoordinate
///   if throwScatter: startCoordinate = game.passCoordinate; clearRangeRuler; clearMoveSquares
///
///   if swooping:
///     doRoll = reRolledAction != SWOOP_DISTANCE || (reRollSource && useReRoll)
///     if doRoll: scatterResult = swoop(startCoordinate, swoopDirection, distanceOption)
///     thrownPlayerState = thrownPlayerState.changeBase(IN_THE_AIR)
///   else:
///     scatterResult = UtilThrowTeamMateSequence.scatterPlayer(...)
///
///   endCoordinate = scatterResult.lastValidCoordinate
///   setAnimation(startCoord -> endCoord)
///   syncGameModel
///   if scatterResult.inBounds: handleLanding(thrownPlayer, endCoordinate)
///   else: TtmToCrowdHandler.handle(...); publish THROWN_PLAYER_ID/STATE/HAS_BALL/IS_KICKED; NEXT_STEP
///
/// handleLanding logic (abbreviated):
///   playerLandedUpon = fieldModel.getPlayer(endCoordinate) (not the thrown player itself)
///   if playerLandedUpon:
///     publish DROP_THROWN_PLAYER=true
///     handleInjury(InjuryTypeTTMHitPlayer / InjuryTypeTTMHitPlayerForSpp, ...)
///     publish SteadyFootingContext
///     publish THROWN_PLAYER_COORDINATE=endCoordinate (loop continues)
///   else:
///     setPlayerCoordinate(thrownPlayer, endCoordinate)
///     setPlayerState(thrownPlayer, thrownPlayerState)
///     publish THROWN_PLAYER_COORDINATE=null (stop loop)
///   publish THROWN_PLAYER_ID/STATE/HAS_BALL/IS_KICKED/USING_SWOOP/OLD_DEFENDER_STATE
///   NEXT_STEP
///
/// Unported utilities:
///   TODO: UtilThrowTeamMateSequence.scatterPlayer (scatter direction roll)
///   TODO: swoop() helper (distance roll + UtilServerCatchScatterThrowIn.findScatterCoordinate)
///   TODO: UtilServerInjury.handleInjury (InjuryTypeTTMHitPlayer / ForSpp)
///   TODO: TtmToCrowdHandler.handle (crowd-push path)
///   TODO: fieldModel operations (getPlayer, setPlayerCoordinate, setPlayerState, etc.)
///   TODO: SteadyFootingContext / DeferredCommand (HitPlayerTurnOverCommand, DropPlayerCommand)
///   TODO: game.passCoordinate, blitzTurnState, SppMechanic
///   TODO: ReRolledActions.SWOOP_DISTANCE re-roll check
///
/// Mirrors Java `com.fumbbl.ffb.server.step.bb2025.ttm.StepInitScatterPlayer`.
pub struct StepInitScatterPlayer {
    /// Java: thrownPlayerId (mandatory init param)
    pub thrown_player_id: Option<String>,
    /// Java: thrownPlayerState (mandatory init param)
    pub thrown_player_state: Option<PlayerState>,
    /// Java: oldPlayerState
    pub old_player_state: Option<PlayerState>,
    /// Java: thrownPlayerCoordinate (mandatory init param)
    pub thrown_player_coordinate: Option<FieldCoordinate>,
    /// Java: thrownPlayerHasBall
    pub thrown_player_has_ball: bool,
    /// Java: throwScatter
    pub throw_scatter: bool,
    /// Java: isKickedPlayer
    pub is_kicked_player: bool,
    /// Java: usingBullseye
    pub using_bullseye: bool,
    /// Java: usingSwoop
    pub using_swoop: bool,
    /// Java: swoopDirection
    pub swoop_direction: Option<Direction>,
    /// Java: scatterResult (UtilThrowTeamMateSequence.ScatterResult)
    pub scatter_result_name: Option<String>,
    // AbstractStepWithReRoll stubs
    pub re_rolled_action: Option<String>,
    pub re_roll_source: Option<String>,
}

impl StepInitScatterPlayer {
    pub fn new() -> Self {
        Self {
            thrown_player_id: None,
            thrown_player_state: None,
            old_player_state: None,
            thrown_player_coordinate: None,
            thrown_player_has_ball: false,
            throw_scatter: false,
            is_kicked_player: false,
            using_bullseye: false,
            using_swoop: false,
            swoop_direction: None,
            scatter_result_name: None,
            re_rolled_action: None,
            re_roll_source: None,
        }
    }
}

impl Default for StepInitScatterPlayer {
    fn default() -> Self { Self::new() }
}

impl Step for StepInitScatterPlayer {
    fn id(&self) -> StepId { StepId::InitScatterPlayer }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // AbstractStepWithReRoll: re-roll responses may trigger executeStep
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::IsKickedPlayer(v) => { self.is_kicked_player = *v; true }
            StepParameter::Direction(v) => { self.swoop_direction = Some(*v); true }
            StepParameter::UsingBullseye(v) => { self.using_bullseye = *v; true }
            StepParameter::UsingSwoop(v) => { self.using_swoop = *v; true }
            StepParameter::OldDefenderState(v) => { self.old_player_state = Some(*v); true }
            _ => false,
        }
    }
}

impl StepInitScatterPlayer {
    fn execute_step(&self, _game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // Guard: thrownPlayer and coordinate must be present.
        if self.thrown_player_id.is_none() || self.thrown_player_coordinate.is_none() {
            return StepOutcome::next();
        }

        // DEFERRED: let swooping = swoop_direction.is_some() && using_swoop;
        //
        // DEFERRED: if !swooping && thrownPlayerState.base == PICKED_UP:
        //   thrownPlayerState = changeBase(IN_THE_AIR)
        //   fieldModel.setPlayerState(thrownPlayer, thrownPlayerState)
        //
        // DEFERRED: if using_bullseye:
        //   endCoordinate = game.passCoordinate
        //   animation; handleLanding(thrownPlayer, endCoordinate); return
        //
        // DEFERRED: startCoordinate = if throwScatter { game.passCoordinate } else { thrownPlayerCoordinate }
        //
        // DEFERRED: if swooping:
        //   doRoll check; scatterResult = swoop(startCoord, swoopDirection, distanceOption)
        //   thrownPlayerState = changeBase(IN_THE_AIR)
        // else:
        //   scatterResult = UtilThrowTeamMateSequence.scatterPlayer(...)
        //
        // DEFERRED: if scatterResult.inBounds: handleLanding
        //   else: TtmToCrowdHandler; publish params; NEXT_STEP

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
    fn no_thrown_player_returns_next_step() {
        let mut game = make_game();
        let mut step = StepInitScatterPlayer::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn set_is_kicked_player_accepted() {
        let mut step = StepInitScatterPlayer::default();
        assert!(step.set_parameter(&StepParameter::IsKickedPlayer(true)));
        assert!(step.is_kicked_player);
    }

    #[test]
    fn set_using_bullseye_accepted() {
        let mut step = StepInitScatterPlayer::default();
        assert!(step.set_parameter(&StepParameter::UsingBullseye(true)));
        assert!(step.using_bullseye);
    }

    #[test]
    fn set_using_swoop_accepted() {
        let mut step = StepInitScatterPlayer::default();
        assert!(step.set_parameter(&StepParameter::UsingSwoop(true)));
        assert!(step.using_swoop);
    }

    #[test]
    fn set_direction_accepted() {
        let mut step = StepInitScatterPlayer::default();
        assert!(step.set_parameter(&StepParameter::Direction(Direction::North)));
        assert_eq!(step.swoop_direction, Some(Direction::North));
    }
}
