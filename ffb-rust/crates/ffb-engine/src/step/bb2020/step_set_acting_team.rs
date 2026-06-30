use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

/// Sets the acting team (by team ID) and toggles home_playing if needed (BB2020).
/// Clears the acting player (Java: `game.getActingPlayer().setPlayer(null)`).
///
/// 1:1 translation of com.fumbbl.ffb.server.step.bb2020.StepSetActingTeam.
pub struct StepSetActingTeam {
    /// Java: `teamId` (init parameter TEAM_ID).
    team_id: Option<String>,
}

impl StepSetActingTeam {
    pub fn new() -> Self { Self { team_id: None } }
}

impl Default for StepSetActingTeam {
    fn default() -> Self { Self::new() }
}

impl Step for StepSetActingTeam {
    fn id(&self) -> StepId { StepId::SetActingTeam }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::TeamId(id) => { self.team_id = Some(id.clone()); true }
            _ => false,
        }
    }
}

impl StepSetActingTeam {
    fn execute_step(&self, game: &mut Game) -> StepOutcome {
        // Java: game.getActingPlayer().setPlayer(null) — clear acting player
        game.acting_player.player_id = None;

        if let Some(team_id) = &self.team_id {
            // Java: if (team != game.getActingTeam()) { game.setHomePlaying(!game.isHomePlaying()); }
            let team_is_home = game.team_home.id == *team_id;
            if team_is_home != game.home_playing {
                game.home_playing = !game.home_playing;
            }
        }

        StepOutcome::next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::enums::Rules;
    use ffb_model::util::rng::GameRng;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2020)
    }

    #[test]
    fn clears_acting_player() {
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        let mut step = StepSetActingTeam::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.acting_player.player_id.is_none());
    }

    #[test]
    fn no_team_id_clears_player_and_returns_next() {
        let mut game = make_game();
        let mut step = StepSetActingTeam::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(game.acting_player.player_id.is_none());
    }

    #[test]
    fn home_team_when_away_playing_toggles() {
        let mut game = make_game();
        game.home_playing = false;
        let mut step = StepSetActingTeam::new();
        step.team_id = Some(game.team_home.id.clone());
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.home_playing);
    }

    #[test]
    fn home_team_when_home_playing_no_toggle() {
        let mut game = make_game();
        game.home_playing = true;
        let mut step = StepSetActingTeam::new();
        step.team_id = Some(game.team_home.id.clone());
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.home_playing);
    }

    #[test]
    fn away_team_when_home_playing_toggles() {
        let mut game = make_game();
        game.home_playing = true;
        let mut step = StepSetActingTeam::new();
        step.team_id = Some(game.team_away.id.clone());
        step.start(&mut game, &mut GameRng::new(0));
        assert!(!game.home_playing);
    }

    #[test]
    fn returns_next_step_action() {
        let mut game = make_game();
        let mut step = StepSetActingTeam::new();
        step.team_id = Some(game.team_home.id.clone());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn set_parameter_team_id_accepted() {
        let mut step = StepSetActingTeam::new();
        assert!(step.set_parameter(&StepParameter::TeamId("team1".into())));
        assert_eq!(step.team_id.as_deref(), Some("team1"));
    }

    #[test]
    fn set_parameter_unknown_returns_false() {
        let mut step = StepSetActingTeam::new();
        assert!(!step.set_parameter(&StepParameter::EndTurn(true)));
    }
}
