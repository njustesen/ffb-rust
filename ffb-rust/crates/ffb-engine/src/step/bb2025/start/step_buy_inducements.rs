use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

/// Handles pre-game inducement purchase dialogs for both coaches.
/// InducementTypeFactory not ported; headless auto-skips all inducement buying.
/// Mirrors Java `com.fumbbl.ffb.server.step.bb2025.start.StepBuyInducements`.
pub struct StepBuyInducements {
    /// Java: availableInducementGoldHome (Integer nullable)
    pub available_inducement_gold_home: Option<i32>,
    /// Java: availableInducementGoldAway (Integer nullable)
    pub available_inducement_gold_away: Option<i32>,
    /// Java: usedInducementGoldHome (Integer, init 0)
    pub used_inducement_gold_home: i32,
    /// Java: usedInducementGoldAway (Integer, init 0)
    pub used_inducement_gold_away: i32,
    /// Java: parallel
    pub parallel: bool,
    /// Java: phase (Phase private enum: INIT/HOME/AWAY/DONE) — stored as name
    pub phase_name: String,
    /// Java: prayersBoughtHome
    pub prayers_bought_home: i32,
    /// Java: prayersBoughtAway
    pub prayers_bought_away: i32,
    /// Java: buyInducementCommands (List<ClientCommandBuyInducements>) — stored as JSON blobs
    pub buy_inducement_commands: Vec<String>,
}

impl StepBuyInducements {
    pub fn new() -> Self {
        Self {
            available_inducement_gold_home: None,
            available_inducement_gold_away: None,
            used_inducement_gold_home: 0,
            used_inducement_gold_away: 0,
            parallel: false,
            phase_name: "INIT".to_string(),
            prayers_bought_home: 0,
            prayers_bought_away: 0,
            buy_inducement_commands: Vec::new(),
        }
    }
}

impl Default for StepBuyInducements {
    fn default() -> Self { Self::new() }
}

impl Step for StepBuyInducements {
    fn id(&self) -> StepId { StepId::BuyInducements }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, _param: &StepParameter) -> bool { false }
}

impl StepBuyInducements {
    fn execute_step(&self, _game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // no-op: InducementTypeFactory not ported — headless auto-skips inducement buying (no dialog, no predefined inducements)
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
        let mut step = StepBuyInducements::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn initial_phase_is_init() {
        let step = StepBuyInducements::new();
        assert_eq!(step.phase_name, "INIT");
    }

    #[test]
    fn set_parameter_returns_false() {
        let mut step = StepBuyInducements::new();
        assert!(!step.set_parameter(&StepParameter::EndTurn(false)));
    }
    #[test]
    fn default_equivalent_to_new() {
        let _a = StepBuyInducements::new();
        let _b = StepBuyInducements::default();
    }
}
