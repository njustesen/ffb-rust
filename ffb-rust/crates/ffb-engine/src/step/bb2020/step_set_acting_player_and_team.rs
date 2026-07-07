use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

/// Sets the acting player (by id) and toggles home_playing if the player belongs
/// to the other team (BB2020).
///
/// 1:1 translation of com.fumbbl.ffb.server.step.bb2020.StepSetActingPlayerAndTeam.
pub struct StepSetActingPlayerAndTeam {
    /// Java: `playerId` (init parameter PLAYER_ID).
    player_id: Option<String>,
}

impl StepSetActingPlayerAndTeam {
    pub fn new() -> Self { Self { player_id: None } }
}

impl Default for StepSetActingPlayerAndTeam {
    fn default() -> Self { Self::new() }
}

impl Step for StepSetActingPlayerAndTeam {
    fn id(&self) -> StepId { StepId::SetActingPlayerAndTeam }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::PlayerId(id) => { self.player_id = Some(id.clone()); true }
            _ => false,
        }
    }
}

impl StepSetActingPlayerAndTeam {
    fn execute_step(&self, game: &mut Game) -> StepOutcome {
        let player_id = match &self.player_id {
            Some(id) => id.clone(),
            None => return StepOutcome::next(),
        };

        // Java: game.getActingPlayer().setPlayer(player) — just sets the active player id
        game.acting_player.player_id = Some(player_id.clone());

        // Java: if (player.getTeam() != game.getActingTeam()) { game.setHomePlaying(!game.isHomePlaying()); }
        let player_is_home = game.team_home.players.iter().any(|p| p.id == player_id);
        if player_is_home != game.home_playing {
            game.home_playing = !game.home_playing;
        }

        StepOutcome::next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::enums::{Rules, PS_STANDING, PlayerType, PlayerGender};
    use ffb_model::model::player::Player;
    use ffb_model::types::FieldCoordinate;
    use ffb_model::util::rng::GameRng;

    fn make_game() -> Game {
        let mut game = Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2020);
        game.team_home.players.push(Player {
            id: "h1".into(), name: "h1".into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        });
        game.team_away.players.push(Player {
            id: "a1".into(), name: "a1".into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        });
        game.field_model.set_player_coordinate("h1", FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("h1", ffb_model::enums::PlayerState::new(PS_STANDING));
        game.field_model.set_player_coordinate("a1", FieldCoordinate::new(6, 6));
        game.field_model.set_player_state("a1", ffb_model::enums::PlayerState::new(PS_STANDING));
        game
    }

    #[test]
    fn no_player_id_returns_next() {
        let mut game = make_game();
        let mut step = StepSetActingPlayerAndTeam::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn sets_acting_player_id() {
        let mut game = make_game();
        let mut step = StepSetActingPlayerAndTeam::new();
        step.player_id = Some("h1".into());
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.acting_player.player_id.as_deref(), Some("h1"));
    }

    #[test]
    fn home_player_when_away_playing_toggles_home_playing() {
        let mut game = make_game();
        game.home_playing = false; // away is currently acting
        let mut step = StepSetActingPlayerAndTeam::new();
        step.player_id = Some("h1".into()); // home player
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.home_playing, "should switch to home playing when home player activated");
    }

    #[test]
    fn home_player_when_home_playing_no_toggle() {
        let mut game = make_game();
        game.home_playing = true;
        let mut step = StepSetActingPlayerAndTeam::new();
        step.player_id = Some("h1".into());
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.home_playing);
    }

    #[test]
    fn away_player_when_home_playing_toggles() {
        let mut game = make_game();
        game.home_playing = true;
        let mut step = StepSetActingPlayerAndTeam::new();
        step.player_id = Some("a1".into());
        step.start(&mut game, &mut GameRng::new(0));
        assert!(!game.home_playing);
    }

    #[test]
    fn returns_next_step_action() {
        let mut game = make_game();
        let mut step = StepSetActingPlayerAndTeam::new();
        step.player_id = Some("h1".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn set_parameter_player_id_accepted() {
        let mut step = StepSetActingPlayerAndTeam::new();
        assert!(step.set_parameter(&StepParameter::PlayerId("p1".into())));
        assert_eq!(step.player_id.as_deref(), Some("p1"));
    }

    #[test]
    fn set_parameter_unknown_returns_false() {
        let mut step = StepSetActingPlayerAndTeam::new();
        assert!(!step.set_parameter(&StepParameter::EndTurn(true)));
    }
}
