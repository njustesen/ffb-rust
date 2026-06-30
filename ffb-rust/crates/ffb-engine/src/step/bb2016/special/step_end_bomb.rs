use ffb_model::enums::{PlayerAction, TurnMode};
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::bb2016::{EndPlayerAction, Pass};
use crate::step::generator::bb2016::end_player_action::EndPlayerActionParams;
use crate::step::generator::bb2016::pass::PassParams;
use crate::step::util_server_steps::{change_player_action, check_touchdown};

/// Final step of the bomb sequence (BB2016).
///
/// If the turn is ending, the bomb exploded, or there is no catcher:
///   - Set home_playing from turn mode
///   - Restore turn mode (BLITZ → BLITZ, other bomb → REGULAR)
///   - Push EndPlayerAction(feeding=false, endPlayerAction=true, endTurn)
/// Otherwise (catcher caught the bomb):
///   - Set home_playing from catcher's team
///   - Set acting player = catcher with THROW_BOMB action
///   - Push Pass sequence (re-throw)
/// Always clears pass_coordinate, thrower_id, thrower_action.
///
/// 1:1 translation of com.fumbbl.ffb.server.step.bb2016.special.StepEndBomb.
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
        // Java: game.setPassCoordinate(null) — done early in BB2016 version
        game.pass_coordinate = None;

        self.end_turn |= check_touchdown(game);

        if self.end_turn || self.catcher_id.is_none() || self.bomb_exploded {
            // Restore home_playing from the bomb turn mode
            game.home_playing = matches!(game.turn_mode, TurnMode::BombHome | TurnMode::BombHomeBlitz);
            // Restore turn_mode
            game.turn_mode = if matches!(game.turn_mode, TurnMode::BombHomeBlitz | TurnMode::BombAwayBlitz) {
                TurnMode::Blitz
            } else {
                TurnMode::Regular
            };

            let seq = EndPlayerAction::build_sequence(&EndPlayerActionParams {
                feeding_allowed: false,
                end_player_action: true,
                end_turn: self.end_turn,
            });
            game.thrower_id = None;
            game.thrower_action = None;
            StepOutcome::next().push_seq(seq)
        } else {
            // Catcher re-throws the bomb
            let catcher_id = self.catcher_id.as_ref().unwrap().clone();
            game.home_playing = game.team_home.players.iter().any(|p| p.id == catcher_id);
            change_player_action(game, &catcher_id, PlayerAction::ThrowBomb, false);

            let seq = Pass::build_sequence(&PassParams { target_coordinate: None });
            // Java: second setPassCoordinate(null) + clear thrower
            game.pass_coordinate = None;
            game.thrower_id = None;
            game.thrower_action = None;
            StepOutcome::next().push_seq(seq)
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{StepAction, StepId, test_team};
    use ffb_model::enums::{Rules, PS_STANDING, PlayerState, PlayerType, PlayerGender};
    use ffb_model::model::player::Player;
    use ffb_model::types::FieldCoordinate;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2016)
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
    fn no_catcher_pushes_end_player_action() {
        let mut game = make_game();
        game.turn_mode = TurnMode::BombHome;
        let mut step = StepEndBomb::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(out.pushes.len(), 1);
        assert_eq!(out.pushes[0][0].step_id, StepId::InitFeeding);
    }

    #[test]
    fn end_turn_pushes_end_player_action() {
        let mut game = make_game();
        game.turn_mode = TurnMode::BombHome;
        let mut step = StepEndBomb::new();
        step.end_turn = true;
        step.catcher_id = Some("foo".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.pushes.len(), 1);
        assert_eq!(out.pushes[0][0].step_id, StepId::InitFeeding);
    }

    #[test]
    fn bomb_exploded_pushes_end_player_action() {
        let mut game = make_game();
        game.turn_mode = TurnMode::BombHome;
        let mut step = StepEndBomb::new();
        step.bomb_exploded = true;
        step.catcher_id = Some("foo".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.pushes.len(), 1);
        assert_eq!(out.pushes[0][0].step_id, StepId::InitFeeding);
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
    fn bomb_away_blitz_restores_blitz_mode() {
        let mut game = make_game();
        game.turn_mode = TurnMode::BombAwayBlitz;
        let mut step = StepEndBomb::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.turn_mode, TurnMode::Blitz);
    }

    #[test]
    fn bomb_home_non_blitz_restores_regular_mode() {
        let mut game = make_game();
        game.turn_mode = TurnMode::BombHome;
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
    fn away_catcher_sets_home_playing_false() {
        let mut game = make_game();
        game.turn_mode = TurnMode::BombHome;
        add_player(&mut game, "away", "awaycatcher");
        game.home_playing = true;
        let mut step = StepEndBomb::new();
        step.catcher_id = Some("awaycatcher".into());
        step.start(&mut game, &mut GameRng::new(0));
        assert!(!game.home_playing);
    }

    #[test]
    fn home_catcher_sets_home_playing_true() {
        let mut game = make_game();
        game.turn_mode = TurnMode::BombAway;
        add_player(&mut game, "home", "homecatcher");
        game.home_playing = false;
        let mut step = StepEndBomb::new();
        step.catcher_id = Some("homecatcher".into());
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.home_playing);
    }

    #[test]
    fn clears_pass_coordinate() {
        let mut game = make_game();
        game.turn_mode = TurnMode::BombHome;
        game.pass_coordinate = Some(FieldCoordinate::new(5, 5));
        let mut step = StepEndBomb::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.pass_coordinate.is_none());
    }

    #[test]
    fn clears_thrower_id_and_action() {
        let mut game = make_game();
        game.turn_mode = TurnMode::BombHome;
        game.thrower_id = Some("thrower".into());
        game.thrower_action = Some(PlayerAction::ThrowBomb);
        let mut step = StepEndBomb::new();
        step.start(&mut game, &mut GameRng::new(0));
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
