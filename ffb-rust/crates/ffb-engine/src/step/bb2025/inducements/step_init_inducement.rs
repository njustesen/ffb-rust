use ffb_model::enums::InducementPhase;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

/// Initializes the inducement sequence: checks which inducements can be used at the given
/// phase, shows dialog, then routes to the appropriate sub-sequence generator.
/// DEFERRED: InducementType handling and sequence generator routing not yet ported.
/// Mirrors Java `com.fumbbl.ffb.server.step.bb2025.inducements.StepInitInducement`.
pub struct StepInitInducement {
    /// Java: fInducementPhase (init param)
    pub inducement_phase: Option<InducementPhase>,
    /// Java: fHomeTeam (init param)
    pub home_team: bool,
    /// Java: fInducementType (InducementType) — stored as name until InducementType is ported
    pub inducement_type: Option<String>,
    /// Java: fEndInducementPhase (transient)
    pub end_inducement_phase: bool,
    /// Java: fTouchdownOrEndOfHalf (transient)
    pub touchdown_or_end_of_half: bool,
}

impl StepInitInducement {
    pub fn new(inducement_phase: InducementPhase, home_team: bool) -> Self {
        Self {
            inducement_phase: Some(inducement_phase),
            home_team,
            inducement_type: None,
            end_inducement_phase: false,
            touchdown_or_end_of_half: false,
        }
    }
}

impl Default for StepInitInducement {
    fn default() -> Self {
        Self {
            inducement_phase: None,
            home_team: false,
            inducement_type: None,
            end_inducement_phase: false,
            touchdown_or_end_of_half: false,
        }
    }
}

impl Step for StepInitInducement {
    fn id(&self) -> StepId { StepId::InitInducement }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // DEFERRED: handle CLIENT_USE_INDUCEMENT command once Action::UseInducement is ported.
        self.end_inducement_phase = true;
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::InducementPhase(v) => { self.inducement_phase = Some(*v); true }
            StepParameter::HomeTeam(v) => { self.home_team = *v; true }
            _ => false,
        }
    }
}

impl StepInitInducement {
    fn execute_step(&self, _game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // DEFERRED: InducementType routing, sequence generators (Wizard/ThrowARock/WeatherMage).
        let phase = match self.inducement_phase {
            Some(p) => p,
            None => return StepOutcome::next(),
        };
        StepOutcome::next()
            .publish(StepParameter::EndInducementPhase(true))
            .publish(StepParameter::HomeTeam(self.home_team))
            .publish(StepParameter::InducementPhase(phase))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::StepAction;
    use ffb_model::enums::{Rules, InducementPhase};

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2025)
    }

    #[test]
    fn start_returns_next_and_publishes_phase() {
        let mut game = make_game();
        let mut step = StepInitInducement::new(InducementPhase::EndOfOwnTurn, true);
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::InducementPhase(_))));
    }

    #[test]
    fn publishes_home_team_flag() {
        let mut game = make_game();
        let mut step = StepInitInducement::new(InducementPhase::StartOfOwnTurn, false);
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::HomeTeam(false))));
    }

    #[test]
    fn handle_command_sets_end_inducement_phase() {
        let mut game = make_game();
        let mut step = StepInitInducement::new(InducementPhase::EndOfOwnTurn, true);
        let out = step.handle_command(&Action::Acknowledge, &mut game, &mut GameRng::new(0));
        assert!(step.end_inducement_phase);
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn publishes_end_inducement_phase_true() {
        let mut game = make_game();
        let mut step = StepInitInducement::new(InducementPhase::BeforeSetup, true);
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndInducementPhase(true))));
    }

    #[test]
    fn set_parameter_inducement_phase_accepted() {
        let mut step = StepInitInducement::default();
        assert!(step.set_parameter(&StepParameter::InducementPhase(InducementPhase::StartOfOwnTurn)));
        assert_eq!(step.inducement_phase, Some(InducementPhase::StartOfOwnTurn));
    }

    #[test]
    fn set_parameter_home_team_accepted() {
        let mut step = StepInitInducement::default();
        assert!(step.set_parameter(&StepParameter::HomeTeam(true)));
        assert!(step.home_team);
    }

    #[test]
    fn default_with_set_parameter_publishes_correctly() {
        let mut game = make_game();
        let mut step = StepInitInducement::default();
        step.set_parameter(&StepParameter::InducementPhase(InducementPhase::StartOfOwnTurn));
        step.set_parameter(&StepParameter::HomeTeam(false));
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndInducementPhase(true))));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::InducementPhase(InducementPhase::StartOfOwnTurn))));
    }
}
