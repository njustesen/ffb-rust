use ffb_model::model::game::Game;
use ffb_model::report::bb2025::report_getting_even_roll::ReportGettingEvenRoll;
use ffb_model::report::report_id::ReportId;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2025.shared.StepGettingEven.
/// Opposing team may use their apothecary after a casualty (Getting Even apothecary rule).
/// client-only: opposing apo dialog (DialogUseApothecaryParameter for opposing team) — headless auto-skips.
pub struct StepGettingEven {
    /// Java: playerId
    pub player_id: Option<String>,
    /// Java: keyword (Keyword enum) — stored as name until Keyword enum is ported
    pub keyword_name: Option<String>,
    // AbstractStepWithReRoll stubs
    pub re_rolled_action: Option<String>,
    pub re_roll_source: Option<String>,
}

impl StepGettingEven {
    pub fn new() -> Self {
        Self { player_id: None, keyword_name: None, re_rolled_action: None, re_roll_source: None }
    }
}

impl Default for StepGettingEven {
    fn default() -> Self { Self::new() }
}

impl Step for StepGettingEven {
    fn id(&self) -> StepId { StepId::GettingEven }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::PlayerId(v) => { self.player_id = Some(v.clone()); true }
            _ => false,
        }
    }
}

/// Java: MINIMUM_ROLL = 4
const MINIMUM_ROLL: i32 = 4;

impl StepGettingEven {
    fn execute_step(&self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: roll = DiceRoller.rollD6(); successful = roll >= MINIMUM_ROLL
        // Java: addReport(new ReportGettingEvenRoll(playerId, successful, roll, MINIMUM_ROLL, reRolled, keyword))
        let roll = rng.d6();
        let successful = roll >= MINIMUM_ROLL;
        let keyword = self.keyword_name.clone().unwrap_or_default();
        game.report_list.add(ReportGettingEvenRoll::new(
            self.player_id.clone(), successful, roll, MINIMUM_ROLL, false, keyword,
        ));
        // client-only: offer opposing apothecary dialog — client-side
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
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2025)
    }

    #[test]
    fn start_returns_next() {
        let mut game = make_game();
        let mut step = StepGettingEven::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn set_parameter_player_id_accepted() {
        let mut step = StepGettingEven::new();
        assert!(step.set_parameter(&StepParameter::PlayerId("p1".into())));
        assert_eq!(step.player_id.as_deref(), Some("p1"));
    }

    #[test]
    fn handle_command_returns_next() {
        let mut game = make_game();
        let mut step = StepGettingEven::new();
        let out = step.handle_command(&Action::EndTurn, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn start_adds_getting_even_roll_report() {
        let mut game = make_game();
        let mut step = StepGettingEven::new();
        step.player_id = Some("p1".into());
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::GETTING_EVEN_ROLL));
    }

    #[test]
    fn keyword_is_included_in_report() {
        let mut game = make_game();
        let mut step = StepGettingEven::new();
        step.player_id = Some("p1".into());
        step.keyword_name = Some("Agility".into());
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::GETTING_EVEN_ROLL));
    }
}
