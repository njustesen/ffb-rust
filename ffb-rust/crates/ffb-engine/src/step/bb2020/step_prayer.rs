/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2020.StepPrayer` (BB2020).
///
/// Applies a single "Prayer to Nuffle" effect during play.
///
/// Differs from BB2025 only in the report class used (BB2020 uses ReportPrayerBb2020,
/// BB2025 uses ReportPrayer). Reports are not translated, so behavior is identical.
///
/// Stub: PrayerHandlerFactory not translated → NEXT_STEP immediately.
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

pub struct StepPrayer {
    pub roll: i32,
    pub team_id: Option<String>,
    pub player_id: Option<String>,
    pub first_run: bool,
}

impl StepPrayer {
    pub fn new(roll: i32, team_id: impl Into<String>) -> Self {
        Self { roll, team_id: Some(team_id.into()), player_id: None, first_run: true }
    }
}

impl Default for StepPrayer {
    fn default() -> Self {
        Self { roll: 0, team_id: None, player_id: None, first_run: true }
    }
}

impl Step for StepPrayer {
    fn id(&self) -> StepId { StepId::Prayer }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        match action {
            Action::SelectPlayer { player_id } => {
                self.player_id = Some(player_id.clone());
            }
            Action::SelectSkill { skill_id: _ } => {}
            _ => {}
        }
        self.execute_step(game)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::PrayerRoll(v) => { self.roll = *v; true }
            StepParameter::TeamId(v) => { self.team_id = Some(v.clone()); true }
            _ => false,
        }
    }
}

impl StepPrayer {
    fn execute_step(&mut self, _game: &mut Game) -> StepOutcome {
        // Stub: PrayerHandlerFactory not translated → treat as "no handler found" path.
        self.first_run = false;
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
        Game::new(home, away, Rules::Bb2020)
    }

    #[test]
    fn start_returns_next_step() {
        let mut game = make_game();
        let mut step = StepPrayer::new(3, "home");
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn start_sets_first_run_false() {
        let mut game = make_game();
        let mut step = StepPrayer::new(3, "home");
        assert!(step.first_run);
        step.start(&mut game, &mut GameRng::new(0));
        assert!(!step.first_run);
    }

    #[test]
    fn handle_command_select_player_stores_id() {
        let mut game = make_game();
        let mut step = StepPrayer::new(3, "home");
        let action = Action::SelectPlayer { player_id: "p1".into() };
        step.handle_command(&action, &mut game, &mut GameRng::new(0));
        assert_eq!(step.player_id.as_deref(), Some("p1"));
    }

    #[test]
    fn prayer_roll_parameter_accepted() {
        let mut step = StepPrayer::default();
        step.set_parameter(&StepParameter::PrayerRoll(5));
        assert_eq!(step.roll, 5);
    }

    #[test]
    fn team_id_parameter_accepted() {
        let mut step = StepPrayer::default();
        step.set_parameter(&StepParameter::TeamId("away".into()));
        assert_eq!(step.team_id.as_deref(), Some("away"));
    }
}
