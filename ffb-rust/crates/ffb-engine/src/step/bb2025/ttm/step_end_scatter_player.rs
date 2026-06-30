use ffb_model::enums::{PlayerState, PS_STANDING};
use ffb_model::types::FieldCoordinate;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepAction, StepId, StepParameter};
use crate::step::generator::bb2025::ScatterPlayer;
use crate::step::generator::bb2025::scatter_player::ScatterPlayerParams;

/// Finalises the scattered-player position. If the player still has a non-null coordinate
/// (landing was blocked by an obstacle), pushes a new ScatterPlayer sequence to continue
/// the scatter loop.
///
/// Java executeStep logic:
///   thrownPlayer = game.getPlayerById(fThrownPlayerId)
///   if thrownPlayer != null && fThrownPlayerState != null && fThrownPlayerCoordinate != null:
///     push ScatterPlayer sequence(fThrownPlayerId, fThrownPlayerState, fThrownPlayerHasBall,
///       fThrownPlayerCoordinate, false, false, false, false)
///     if fIsKickedPlayer: publish IS_KICKED_PLAYER=true
///     publish OLD_DEFENDER_STATE=oldPlayerState
///   NEXT_STEP
///
/// Parameter notes:
///   fThrownPlayerCoordinate is NOT consumed (note: Java doesn't call consume() on it).
///   All other parameters are consumed.
///
/// Unported utilities:
///   TODO: ScatterPlayer sequence generator (SequenceGenerator.ScatterPlayer.pushSequence)
///   TODO: game.getPlayerById (player lookup)
///
/// Mirrors Java `com.fumbbl.ffb.server.step.bb2025.ttm.StepEndScatterPlayer`.
pub struct StepEndScatterPlayer {
    /// Java: fThrownPlayerId (consumed)
    pub thrown_player_id: Option<String>,
    /// Java: fThrownPlayerHasBall (consumed)
    pub thrown_player_has_ball: bool,
    /// Java: fThrownPlayerState (consumed)
    pub thrown_player_state: Option<PlayerState>,
    /// Java: oldPlayerState
    pub old_player_state: Option<PlayerState>,
    /// Java: fThrownPlayerCoordinate (NOT consumed — null means stop loop)
    pub thrown_player_coordinate: Option<FieldCoordinate>,
    /// Java: fIsKickedPlayer
    pub is_kicked_player: bool,
}

impl StepEndScatterPlayer {
    pub fn new() -> Self {
        Self {
            thrown_player_id: None,
            thrown_player_has_ball: false,
            thrown_player_state: None,
            old_player_state: None,
            thrown_player_coordinate: None,
            is_kicked_player: false,
        }
    }
}

impl Default for StepEndScatterPlayer {
    fn default() -> Self { Self::new() }
}

impl Step for StepEndScatterPlayer {
    fn id(&self) -> StepId { StepId::EndScatterPlayer }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::ThrownPlayerId(v) => { self.thrown_player_id = v.clone(); true }
            StepParameter::ThrownPlayerHasBall(v) => { self.thrown_player_has_ball = *v; true }
            StepParameter::ThrownPlayerState(v) => { self.thrown_player_state = Some(*v); true }
            StepParameter::OldDefenderState(v) => { self.old_player_state = Some(*v); true }
            StepParameter::ThrownPlayerCoordinate(v) => { self.thrown_player_coordinate = *v; true }
            StepParameter::IsKickedPlayer(v) => { self.is_kicked_player = *v; true }
            _ => false,
        }
    }
}

impl StepEndScatterPlayer {
    fn execute_step(&self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // Java: thrownPlayer = game.getPlayerById(fThrownPlayerId)
        // Java: if thrownPlayer != null && fThrownPlayerState != null && fThrownPlayerCoordinate != null:
        let thrown_player_exists = self.thrown_player_id.as_deref()
            .map(|id| game.player(id).is_some())
            .unwrap_or(false);
        if thrown_player_exists
            && self.thrown_player_state.is_some()
            && self.thrown_player_coordinate.is_some()
        {
            let seq = ScatterPlayer::build_sequence(&ScatterPlayerParams {
                thrown_player_id: self.thrown_player_id.clone(),
                thrown_player_state: self.thrown_player_state,
                thrown_player_has_ball: self.thrown_player_has_ball,
                thrown_player_coordinate: self.thrown_player_coordinate,
                throw_scatter: false,
                has_swoop: false,
            });
            let mut outcome = StepOutcome::next().push_seq(seq);
            if self.is_kicked_player {
                outcome = outcome.publish(StepParameter::IsKickedPlayer(true));
            }
            outcome = outcome.publish(StepParameter::OldDefenderState(
                self.old_player_state.unwrap_or(PlayerState(0)),
            ));
            return outcome;
        }
        StepOutcome::next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::StepAction;
    use ffb_model::enums::{PlayerState, Rules};

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    #[test]
    fn start_returns_next_step() {
        let mut game = make_game();
        let mut step = StepEndScatterPlayer::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn with_all_fields_pushes_scatter_player_sequence() {
        use ffb_model::model::player::Player;
        let mut game = make_game();
        let mut p = Player::default();
        p.id = "p1".into();
        game.team_home.players.push(p);
        let mut step = StepEndScatterPlayer::new();
        step.thrown_player_id = Some("p1".into());
        step.thrown_player_coordinate = Some(FieldCoordinate { x: 5, y: 5 });
        step.thrown_player_state = Some(PlayerState::new(PS_STANDING));
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(!out.pushes.is_empty(), "should push ScatterPlayer sequence");
        assert_eq!(out.pushes[0][0].step_id, StepId::InitScatterPlayer);
    }

    #[test]
    fn without_all_fields_returns_next_no_push() {
        let mut game = make_game();
        let mut step = StepEndScatterPlayer::new();
        // Only player_id set, no coordinate or state → no push
        step.thrown_player_id = Some("p1".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(out.pushes.is_empty());
    }

    #[test]
    fn set_thrown_player_coordinate_accepted() {
        let mut step = StepEndScatterPlayer::default();
        let coord = FieldCoordinate { x: 3, y: 4 };
        assert!(step.set_parameter(&StepParameter::ThrownPlayerCoordinate(Some(coord))));
        assert_eq!(step.thrown_player_coordinate, Some(coord));
    }

    #[test]
    fn set_is_kicked_player_accepted() {
        let mut step = StepEndScatterPlayer::default();
        assert!(step.set_parameter(&StepParameter::IsKickedPlayer(true)));
        assert!(step.is_kicked_player);
    }

    #[test]
    fn set_old_defender_state_accepted() {
        let mut step = StepEndScatterPlayer::default();
        assert!(step.set_parameter(&StepParameter::OldDefenderState(PlayerState::new(PS_STANDING))));
        assert_eq!(step.old_player_state, Some(PlayerState::new(PS_STANDING)));
    }
}
