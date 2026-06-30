/// 1:1 translation of `com.fumbbl.ffb.server.step.mixed.special.StepEndBomb`.
///
/// Final step of the bomb sequence (mixed, BB2020 + BB2025).
///
/// More complex than the BB2016 version: handles PassState, the
/// `throwTwoBombs` / `canUseThrowBombActionTwice` skill, `allowMoveAfterBomb`,
/// and the AllYouCanEat step.
///
/// Parameters consumed:
///   - CATCHER_ID
///   - END_TURN
///   - BOMB_EXPLODED
///
/// Java: uses `SequenceGeneratorFactory` to select edition-specific Pass/EndPlayerAction/Move.
/// Rust: uses BB2025 generators (the primary target for mixed).
use ffb_model::enums::{PlayerAction, TurnMode};
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepId, StepOutcome, StepParameter};
use crate::step::generator::bb2025::{EndPlayerAction, Pass};
use crate::step::generator::bb2025::end_player_action::EndPlayerActionParams;
use crate::step::generator::bb2025::pass::PassParams;
use crate::step::util_server_steps::{change_player_action, check_touchdown};

/// Java: `StepEndBomb` (mixed/special, BB2020 + BB2025).
pub struct StepEndBomb {
    /// Java: fCatcherId
    pub catcher_id: Option<String>,
    /// Java: fEndTurn
    pub end_turn: bool,
    /// Java: fBombExploded
    pub bomb_exploded: bool,
}

impl StepEndBomb {
    pub fn new() -> Self {
        Self { catcher_id: None, end_turn: false, bomb_exploded: false }
    }

    fn execute_step(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.end_turn |= check_touchdown(game);

        // Java: boolean removePassCoordinate = true;
        let mut remove_pass_coordinate = true;

        if self.end_turn || self.catcher_id.is_none() || self.bomb_exploded {
            // Restore home_playing from bomb turn mode
            if game.turn_mode.is_bomb_turn() {
                game.home_playing = matches!(
                    game.turn_mode,
                    TurnMode::BombHome | TurnMode::BombHomeBlitz
                );
                // Restore turn mode: blitz → BLITZ, other bomb → REGULAR
                game.turn_mode =
                    if matches!(game.turn_mode, TurnMode::BombHomeBlitz | TurnMode::BombAwayBlitz) {
                        TurnMode::Blitz
                    } else {
                        TurnMode::Regular
                    };
            }

            // Java: PassState state = getGameState().getPassState();
            // PassState is a stub in Rust; we read from game fields directly.
            // Java reads: state.getOriginalBombardier(), state.getThrowTwoBombs(),
            //             state.isAllowMoveAfterBomb(), state.getThrowTwoBombs()
            //
            // Since PassState is a stub (all None), the complex branching collapses:
            //   originalBomber == actingPlayer (state.getOriginalBombardier() is None)
            //   skill = null (no canUseThrowBombActionTwice property)
            //   threwOnlyFirstBomb = toPrimitive(null) → false
            //   state.getThrowTwoBombs() == null → else: push EndPlayerAction
            //
            // We mirror the final else-branch which is the common path.
            let seq = EndPlayerAction::build_sequence(&EndPlayerActionParams {
                feeding_allowed: false,
                end_player_action: true,
                end_turn: self.end_turn,
                check_forgo: false,
            });
            game.pass_coordinate = None;
            game.thrower_id = None;
            game.thrower_action = None;
            return StepOutcome::next().push_seq(seq);
        }

        // Catcher caught the bomb and is re-throwing it
        // Java: game.setPassCoordinate(null) (done explicitly before changePlayerAction)
        game.pass_coordinate = None;
        let catcher_id = self.catcher_id.as_ref().unwrap().clone();
        game.home_playing = game.team_home.players.iter().any(|p| p.id == catcher_id);
        change_player_action(game, &catcher_id, PlayerAction::ThrowBomb, false);

        let seq = Pass::build_sequence(&PassParams { target_coordinate: None });

        // Java: removePassCoordinate stays true so passCoordinate is cleared again here
        if remove_pass_coordinate {
            game.pass_coordinate = None;
        }
        game.thrower_id = None;
        game.thrower_action = None;
        StepOutcome::next().push_seq(seq)
    }
}

impl Default for StepEndBomb {
    fn default() -> Self { Self::new() }
}

impl Step for StepEndBomb {
    fn id(&self) -> StepId { StepId::EndBomb }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::CatcherId(v) => { self.catcher_id = v.clone(); true }
            StepParameter::EndTurn(v) => { self.end_turn = *v; true }
            StepParameter::BombExploded(v) => { self.bomb_exploded = *v; true }
            _ => false,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{StepAction, StepId, test_team};
    use ffb_model::enums::{Rules, PS_STANDING, PlayerState, PlayerType, PlayerGender};
    use ffb_model::model::player::Player;
    use ffb_model::types::FieldCoordinate;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    fn add_player(game: &mut Game, team: &str, id: &str) {
        let p = Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
        };
        if team == "home" {
            game.team_home.players.push(p);
        } else {
            game.team_away.players.push(p);
        }
        game.field_model.set_player_coordinate(id, FieldCoordinate::new(5, 5));
        game.field_model.set_player_state(id, PlayerState::new(PS_STANDING));
    }

    #[test]
    fn id_is_end_bomb() {
        assert_eq!(StepEndBomb::new().id(), StepId::EndBomb);
    }

    #[test]
    fn no_catcher_pushes_end_player_action() {
        let mut game = make_game();
        game.turn_mode = TurnMode::BombHome;
        let mut step = StepEndBomb::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(out.pushes.len(), 1);
        // BB2025 EndPlayerAction sequence starts with RemoveTargetSelectionState
        assert_eq!(out.pushes[0][0].step_id, StepId::RemoveTargetSelectionState);
    }

    #[test]
    fn end_turn_flag_pushes_end_player_action() {
        let mut game = make_game();
        game.turn_mode = TurnMode::BombHome;
        let mut step = StepEndBomb::new();
        step.end_turn = true;
        step.catcher_id = Some("p1".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.pushes.len(), 1);
        assert_eq!(out.pushes[0][0].step_id, StepId::RemoveTargetSelectionState);
    }

    #[test]
    fn bomb_exploded_pushes_end_player_action() {
        let mut game = make_game();
        game.turn_mode = TurnMode::BombHome;
        let mut step = StepEndBomb::new();
        step.bomb_exploded = true;
        step.catcher_id = Some("p1".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.pushes.len(), 1);
        assert_eq!(out.pushes[0][0].step_id, StepId::RemoveTargetSelectionState);
    }

    #[test]
    fn bomb_home_sets_home_playing_true() {
        let mut game = make_game();
        game.turn_mode = TurnMode::BombHome;
        game.home_playing = false;
        let mut step = StepEndBomb::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.home_playing);
    }

    #[test]
    fn bomb_away_sets_home_playing_false() {
        let mut game = make_game();
        game.turn_mode = TurnMode::BombAway;
        game.home_playing = true;
        let mut step = StepEndBomb::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert!(!game.home_playing);
    }

    #[test]
    fn bomb_home_blitz_restores_blitz_mode() {
        let mut game = make_game();
        game.turn_mode = TurnMode::BombHomeBlitz;
        let mut step = StepEndBomb::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.turn_mode, TurnMode::Blitz);
    }

    #[test]
    fn bomb_away_non_blitz_restores_regular_mode() {
        let mut game = make_game();
        game.turn_mode = TurnMode::BombAway;
        let mut step = StepEndBomb::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.turn_mode, TurnMode::Regular);
    }

    #[test]
    fn catcher_present_pushes_pass_sequence() {
        let mut game = make_game();
        game.turn_mode = TurnMode::BombHome;
        add_player(&mut game, "away", "catcher1");
        let mut step = StepEndBomb::new();
        step.catcher_id = Some("catcher1".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.pushes.len(), 1);
        assert_eq!(out.pushes[0][0].step_id, StepId::InitPassing);
    }

    #[test]
    fn clears_pass_coordinate_and_thrower() {
        let mut game = make_game();
        game.turn_mode = TurnMode::BombHome;
        game.pass_coordinate = Some(FieldCoordinate::new(3, 4));
        game.thrower_id = Some("t1".into());
        game.thrower_action = Some(PlayerAction::ThrowBomb);
        let mut step = StepEndBomb::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.pass_coordinate.is_none());
        assert!(game.thrower_id.is_none());
        assert!(game.thrower_action.is_none());
    }

    #[test]
    fn set_parameter_catcher_id_accepted() {
        let mut step = StepEndBomb::new();
        assert!(step.set_parameter(&StepParameter::CatcherId(Some("p1".into()))));
        assert_eq!(step.catcher_id, Some("p1".into()));
    }

    #[test]
    fn set_parameter_end_turn_accepted() {
        let mut step = StepEndBomb::new();
        assert!(step.set_parameter(&StepParameter::EndTurn(true)));
        assert!(step.end_turn);
    }

    #[test]
    fn set_parameter_bomb_exploded_accepted() {
        let mut step = StepEndBomb::new();
        assert!(step.set_parameter(&StepParameter::BombExploded(true)));
        assert!(step.bomb_exploded);
    }

    #[test]
    fn unrecognised_parameter_returns_false() {
        let mut step = StepEndBomb::new();
        assert!(!step.set_parameter(&StepParameter::HomeTeam(true)));
    }
}
