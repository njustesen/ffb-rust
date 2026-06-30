use ffb_model::enums::{InducementPhase, TurnMode};
use ffb_model::prompts::AgentPrompt;
use ffb_model::types::FieldCoordinate;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::common::Inducement;
use crate::step::generator::common::inducement::InducementParams;

/// Accepts the kicker's placement choice; publishes `KickoffStartCoordinate`
/// and pushes two `BeforeKickoffScatter` Inducement sequences (home + away).
/// Mirrors Java `com.fumbbl.ffb.server.step.bb2025.kickoff.StepKickoff`.
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
        // Java CLIENT_KICKOFF: store ball coordinate (possibly transformed for away team).
        if let Action::KickBall { coord } = action {
            let normalized = if game.home_playing {
                *coord
            } else {
                coord.transform()
            };
            self.kickoff_start_coordinate = Some(normalized);
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, _param: &StepParameter) -> bool { false }
}

impl StepKickoff {
    fn execute_step(&self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        match self.kickoff_start_coordinate {
            Some(coord) => {
                // Java: pushSequence(InducementPhase.BEFORE_KICKOFF_SCATTER, home_playing)
                // Java: pushSequence(InducementPhase.BEFORE_KICKOFF_SCATTER, !home_playing)
                let seq_home = Inducement::build_sequence(&InducementParams {
                    inducement_phase: InducementPhase::BeforeKickoffScatter,
                    home_team: game.home_playing,
                    check_forgo: false,
                });
                let seq_away = Inducement::build_sequence(&InducementParams {
                    inducement_phase: InducementPhase::BeforeKickoffScatter,
                    home_team: !game.home_playing,
                    check_forgo: false,
                });
                StepOutcome::next()
                    .publish(StepParameter::KickoffStartCoordinate(coord))
                    .push_seq(seq_home)
                    .push_seq(seq_away)
            }
            None => {
                // Waiting for CLIENT_KICKOFF command: set TurnMode to Kickoff so the
                // client knows it should offer a placement UI.
                game.turn_mode = TurnMode::Kickoff;
                StepOutcome::cont().with_prompt(AgentPrompt::KickBall)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::{StepAction, StepParameter};
    use ffb_model::enums::Rules;
    use ffb_model::util::rng::GameRng;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2025)
    }

    #[test]
    fn start_without_coordinate_returns_cont() {
        let mut game = make_game();
        let mut step = StepKickoff::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::Continue);
    }

    #[test]
    fn start_without_coordinate_sets_kickoff_turn_mode() {
        let mut game = make_game();
        let mut step = StepKickoff::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.turn_mode, TurnMode::Kickoff);
    }

    #[test]
    fn kick_ball_action_home_team_stores_coordinate_as_is() {
        let mut game = make_game();
        game.home_playing = true;
        let mut step = StepKickoff::new();
        let coord = FieldCoordinate::new(13, 7);
        let action = Action::KickBall { coord };
        let out = step.handle_command(&action, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(step.kickoff_start_coordinate, Some(coord));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::KickoffStartCoordinate(c) if *c == coord)));
    }

    #[test]
    fn kick_ball_action_away_team_transforms_coordinate() {
        let mut game = make_game();
        game.home_playing = false;
        let mut step = StepKickoff::new();
        let coord = FieldCoordinate::new(13, 7);
        let action = Action::KickBall { coord };
        step.handle_command(&action, &mut game, &mut GameRng::new(0));
        let expected = coord.transform();
        assert_eq!(step.kickoff_start_coordinate, Some(expected));
    }

    #[test]
    fn kickoff_with_coordinate_set_returns_next_step() {
        let mut game = make_game();
        let mut step = StepKickoff::new();
        step.kickoff_start_coordinate = Some(FieldCoordinate::new(13, 7));
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn kickoff_with_coordinate_pushes_two_inducement_sequences() {
        let mut game = make_game();
        game.home_playing = true;
        let mut step = StepKickoff::new();
        step.kickoff_start_coordinate = Some(FieldCoordinate::new(13, 7));
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.pushes.len(), 2, "should push two Inducement sequences");
        assert_eq!(out.pushes[0][0].step_id, StepId::InitInducement);
        assert_eq!(out.pushes[1][0].step_id, StepId::InitInducement);
    }
}
