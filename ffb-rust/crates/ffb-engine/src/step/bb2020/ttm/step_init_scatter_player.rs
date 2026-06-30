/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2020.ttm.StepInitScatterPlayer`.
///
/// Step in the TTM scatter sequence. Calculates where the thrown/kicked player lands:
/// - If in-bounds with a player there: injury the hit player, continue scatter loop.
/// - If in-bounds empty (and no crash-landing): place player, end loop.
/// - If crash-landing: drop player without landing roll.
/// - If out-of-bounds: crowd-injury.
///
/// BB2020 differences vs BB2016:
///  - Adds `deviate` flag (WILDLY_INACCURATE path — deviates from thrower coordinate).
///  - Adds `crash_landing` flag.
///  - Adds `swoop_direction` for the Swoop skill path.
///  - If player state is PICKED_UP at start, changes it to IN_THE_AIR.
///
/// Init params (mandatory): THROWN_PLAYER_ID, THROWN_PLAYER_STATE,
///   THROWN_PLAYER_HAS_BALL, THROWN_PLAYER_COORDINATE, THROW_SCATTER.
/// Optional init: IS_KICKED_PLAYER, PASS_DEVIATES, CRASH_LANDING, DIRECTION.
///
/// TODO(InitScatterPlayer-scatter): UtilThrowTeamMateSequence.scatterPlayer/kickPlayer deferred.
/// TODO(InitScatterPlayer-deviate): deviate() / swoop() paths deferred.
/// TODO(InitScatterPlayer-injury): UtilServerInjury.handleInjury deferred.
/// TODO(InitScatterPlayer-animation): Animation/syncGameModel deferred.
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use ffb_model::enums::{PlayerState, PS_PICKED_UP, PS_IN_THE_AIR};
use ffb_model::types::FieldCoordinate;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};

/// Java: `StepInitScatterPlayer` (bb2020/ttm).
pub struct StepInitScatterPlayer {
    /// Java: `thrownPlayerId` — mandatory init param.
    thrown_player_id: Option<String>,
    /// Java: `thrownPlayerState` — mandatory init param.
    thrown_player_state: Option<PlayerState>,
    /// Java: `thrownPlayerHasBall` — mandatory init param.
    thrown_player_has_ball: bool,
    /// Java: `thrownPlayerCoordinate` — mandatory init param.
    thrown_player_coordinate: Option<FieldCoordinate>,
    /// Java: `throwScatter` — mandatory init param.
    throw_scatter: bool,
    /// Java: `isKickedPlayer` — optional.
    is_kicked_player: bool,
    /// Java: `deviate` (PASS_DEVIATES) — BB2020 addition: wildly-inaccurate path.
    deviate: bool,
    /// Java: `crashLanding` — BB2020 addition.
    crash_landing: bool,
    /// Java: `swoopDirection` (DIRECTION) — BB2020 addition (Swoop skill).
    swoop_direction: Option<ffb_model::enums::Direction>,
}

impl StepInitScatterPlayer {
    pub fn new() -> Self {
        Self {
            thrown_player_id: None,
            thrown_player_state: None,
            thrown_player_has_ball: false,
            thrown_player_coordinate: None,
            throw_scatter: false,
            is_kicked_player: false,
            deviate: false,
            crash_landing: false,
            swoop_direction: None,
        }
    }

    fn execute_step(&self, game: &mut Game) -> StepOutcome {
        // BB2020: if player state is PICKED_UP → change to IN_THE_AIR.
        if let (Some(id), Some(state)) = (&self.thrown_player_id, self.thrown_player_state) {
            if state.base() == PS_PICKED_UP && game.player(id).is_some() {
                let new_state = state.change_base(PS_IN_THE_AIR);
                game.field_model.set_player_state(id, new_state);
            }
        }

        // Guard: no player or coordinate → skip.
        if self.thrown_player_id.is_none() || self.thrown_player_coordinate.is_none() {
            return StepOutcome::next();
        }

        // TODO(InitScatterPlayer-deviate): if deviate → call deviate() (direction+distance roll from thrower).
        // TODO(InitScatterPlayer-swoop): if swoopDirection.is_some() → call swoop().
        // TODO(InitScatterPlayer-scatter): call UtilThrowTeamMateSequence::scatterPlayer / kickPlayer
        //   to get the landing FieldCoordinate.
        // TODO(InitScatterPlayer-inBounds+hit): if in-bounds + player → injury + continue loop +
        //   set DROP_THROWN_PLAYER, crashLanding=false.
        // TODO(InitScatterPlayer-crashLanding): if in-bounds + empty + crash_landing → drop player.
        // TODO(InitScatterPlayer-empty): if in-bounds empty + no crash → place player + end loop.
        // TODO(InitScatterPlayer-outOfBounds): crowd injury (TtmToCrowdHandler).

        // Always publish the carried parameters so downstream steps can consume them.
        StepOutcome::next()
            .publish(StepParameter::ThrownPlayerId(self.thrown_player_id.clone()))
            .publish(StepParameter::ThrownPlayerState(self.thrown_player_state.unwrap_or_default()))
            .publish(StepParameter::ThrownPlayerHasBall(self.thrown_player_has_ball))
            .publish(StepParameter::IsKickedPlayer(self.is_kicked_player))
    }
}

impl Default for StepInitScatterPlayer {
    fn default() -> Self { Self::new() }
}

impl Step for StepInitScatterPlayer {
    fn id(&self) -> StepId { StepId::InitScatterPlayer }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::ThrownPlayerId(v)          => { self.thrown_player_id = v.clone(); true }
            StepParameter::ThrownPlayerState(v)       => { self.thrown_player_state = Some(*v); true }
            StepParameter::ThrownPlayerHasBall(v)     => { self.thrown_player_has_ball = *v; true }
            StepParameter::ThrownPlayerCoordinate(v)  => { self.thrown_player_coordinate = *v; true }
            StepParameter::ThrowScatter(v)            => { self.throw_scatter = *v; true }
            StepParameter::IsKickedPlayer(v)          => { self.is_kicked_player = *v; true }
            StepParameter::CrashLanding(v)            => { self.crash_landing = *v; true }
            // Kicked-player aliases (same step handles both TTM and KTM scatter).
            StepParameter::KickedPlayerId(v)          => { self.thrown_player_id = v.clone(); true }
            StepParameter::KickedPlayerState(v)       => { self.thrown_player_state = Some(*v); true }
            StepParameter::KickedPlayerHasBall(v)     => { self.thrown_player_has_ball = *v; true }
            StepParameter::KickedPlayerCoordinate(v)  => { self.thrown_player_coordinate = Some(*v); true }
            _ => false,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::enums::{Rules, PS_STANDING, PlayerState};

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2020)
    }

    #[test]
    fn id_is_init_scatter_player() {
        assert_eq!(StepInitScatterPlayer::new().id(), StepId::InitScatterPlayer);
    }

    #[test]
    fn no_player_returns_next() {
        let mut game = make_game();
        let out = StepInitScatterPlayer::new().start(&mut game, &mut GameRng::new(0));
        assert!(matches!(out.action, StepAction::NextStep));
    }

    #[test]
    fn publishes_thrown_player_id_when_set() {
        let mut game = make_game();
        let mut step = StepInitScatterPlayer::new();
        step.thrown_player_id = Some("p1".into());
        step.thrown_player_coordinate = Some(FieldCoordinate::new(5, 5));
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::ThrownPlayerId(Some(_)))));
    }

    #[test]
    fn publishes_is_kicked_player() {
        let mut game = make_game();
        let mut step = StepInitScatterPlayer::new();
        step.thrown_player_id = Some("p1".into());
        step.thrown_player_coordinate = Some(FieldCoordinate::new(5, 5));
        step.is_kicked_player = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::IsKickedPlayer(true))));
    }

    #[test]
    fn set_parameter_throw_scatter() {
        let mut step = StepInitScatterPlayer::new();
        assert!(step.set_parameter(&StepParameter::ThrowScatter(true)));
        assert!(step.throw_scatter);
    }

    #[test]
    fn set_parameter_crash_landing() {
        let mut step = StepInitScatterPlayer::new();
        assert!(step.set_parameter(&StepParameter::CrashLanding(true)));
        assert!(step.crash_landing);
    }

    #[test]
    fn picked_up_player_changes_state_to_in_the_air() {
        use ffb_model::model::player::Player;
        use ffb_model::enums::{PlayerType, PlayerGender};
        use std::collections::HashSet;

        let mut game = make_game();
        game.team_home.players.push(Player {
            id: "p1".into(), name: "p1".into(), nr: 1,
            position_id: "lineman".into(), player_type: PlayerType::Regular,
            gender: PlayerGender::Male, movement: 6, strength: 3, agility: 3,
            passing: 4, armour: 8, starting_skills: vec![], extra_skills: vec![],
            temporary_skills: vec![], used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0,
            career_spps: 0, race: None,
        });
        game.field_model.set_player_coordinate("p1", FieldCoordinate::new(5, 5));
        let picked_up = PlayerState::new(PS_PICKED_UP);
        game.field_model.set_player_state("p1", picked_up);

        let mut step = StepInitScatterPlayer::new();
        step.thrown_player_id = Some("p1".into());
        step.thrown_player_state = Some(picked_up);
        step.thrown_player_coordinate = Some(FieldCoordinate::new(5, 5));
        step.start(&mut game, &mut GameRng::new(0));

        let state = game.field_model.player_state("p1").unwrap();
        assert_eq!(state.base(), PS_IN_THE_AIR);
    }
}
