use ffb_model::enums::InducementPhase;
use ffb_model::types::FieldCoordinate;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::sequences::inducement_sequence;

/// 1:1 translation of `com.fumbbl.ffb.server.step.mixed.kickoff.StepKickoff`.
///
/// Waits for `CLIENT_KICKOFF` (via `Action::Kickoff`) carrying the ball
/// placement coordinate. Once received, publishes
/// `StepParameter::KickoffStartCoordinate` and pushes inducement sequences
/// for the BEFORE_KICKOFF_SCATTER phase for both teams.
///
/// Sets stepParameter `KICKOFF_START_COORDINATE` for all steps on the stack.
///
/// BB2016 / BB2020.
pub struct StepKickoff {
    /// Java: fKickoffStartCoordinate
    pub kickoff_start_coordinate: Option<FieldCoordinate>,
}

impl StepKickoff {
    pub fn new() -> Self {
        Self { kickoff_start_coordinate: None }
    }
}

impl Default for StepKickoff {
    fn default() -> Self { Self::new() }
}

impl Step for StepKickoff {
    fn id(&self) -> StepId { StepId::Kickoff }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        if let Action::KickBall { coord } = action {
            // Java: CLIENT_KICKOFF → store coordinate (transform if away-kicking for field symmetry)
            self.kickoff_start_coordinate = Some(if game.home_playing {
                *coord
            } else {
                coord.transform()
            });
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, _param: &StepParameter) -> bool { false }
}

impl StepKickoff {
    fn execute_step(&self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        if let Some(coord) = self.kickoff_start_coordinate {
            let home = game.home_playing;
            // Java: push Inducement BEFORE_KICKOFF_SCATTER for both teams (home first, then away)
            return StepOutcome::next()
                .publish(StepParameter::KickoffStartCoordinate(coord))
                .push_seq(inducement_sequence(InducementPhase::BeforeKickoffScatter, home))
                .push_seq(inducement_sequence(InducementPhase::BeforeKickoffScatter, !home));
        }
        // No coordinate yet — wait for CLIENT_KICKOFF
        StepOutcome::cont()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::model::game::Game;
    use ffb_model::enums::Rules;
    use ffb_model::util::rng::GameRng;
    use ffb_model::types::FieldCoordinate;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2020)
    }

    #[test]
    fn id_is_kickoff() {
        assert_eq!(StepKickoff::new().id(), StepId::Kickoff);
    }

    #[test]
    fn start_waits_without_coordinate() {
        let mut step = StepKickoff::new();
        let mut game = make_game();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::Continue);
    }

    #[test]
    fn kickoff_action_stores_coordinate_home_playing() {
        let mut step = StepKickoff::new();
        let mut game = make_game();
        game.home_playing = true;
        let coord = FieldCoordinate::new(7, 5);
        let out = step.handle_command(&Action::KickBall { coord }, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(step.kickoff_start_coordinate, Some(coord));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::KickoffStartCoordinate(_))));
    }

    #[test]
    fn kickoff_action_transforms_coordinate_away_playing() {
        let mut step = StepKickoff::new();
        let mut game = make_game();
        game.home_playing = false;
        let coord = FieldCoordinate::new(7, 5);
        step.handle_command(&Action::KickBall { coord }, &mut game, &mut GameRng::new(0));
        assert_eq!(step.kickoff_start_coordinate, Some(coord.transform()));
    }

    #[test]
    fn set_parameter_returns_false() {
        let mut step = StepKickoff::new();
        assert!(!step.set_parameter(&StepParameter::EndTurn(false)));
    }

    #[test]
    fn kickoff_pushes_inducement_sequences_for_both_teams() {
        let mut step = StepKickoff::new();
        let mut game = make_game();
        game.home_playing = true;
        let coord = FieldCoordinate::new(7, 5);
        let out = step.handle_command(&Action::KickBall { coord }, &mut game, &mut GameRng::new(0));
        // Two inducement sequences pushed (one per team)
        assert_eq!(out.pushes.len(), 2);
    }
}
