/// 1:1 translation of com.fumbbl.ffb.server.step.bb2025.StepPrayer (BB2025).
///
/// Applies a single "Prayer to Nuffle" effect during play.
///
/// Init params (via set_parameter or new()): PRAYER_ROLL, TEAM_ID.
/// Command params: SelectPlayer (CLIENT_PLAYER_CHOICE) or SelectSkill (CLIENT_PRAYER_SELECTION).
///
/// Stub: PrayerHandlerFactory / PrayerHandler not yet translated.
/// Behavior without handler: NEXT_STEP immediately (matches Java "no handler found" path).
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use ffb_model::report::bb2025::report_prayer_roll::ReportPrayerRoll;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

pub struct StepPrayer {
    /// Java: roll — from init params PRAYER_ROLL.
    pub roll: i32,
    /// Java: teamId — from init params TEAM_ID.
    pub team_id: Option<String>,
    /// Java: playerId — set from CLIENT_PLAYER_CHOICE or CLIENT_PRAYER_SELECTION command.
    pub player_id: Option<String>,
    /// Java: firstRun — tracks whether start() is being called for the first time.
    pub first_run: bool,
}

impl StepPrayer {
    pub fn new(roll: i32, team_id: impl Into<String>) -> Self {
        Self {
            roll,
            team_id: Some(team_id.into()),
            player_id: None,
            first_run: true,
        }
    }
}

impl Default for StepPrayer {
    fn default() -> Self {
        Self {
            roll: 0,
            team_id: None,
            player_id: None,
            first_run: true,
        }
    }
}

impl Step for StepPrayer {
    fn id(&self) -> StepId { StepId::Prayer }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // Java: CLIENT_PLAYER_CHOICE → set playerId → EXECUTE_STEP
        // Java: CLIENT_PRAYER_SELECTION → set playerId + skill → EXECUTE_STEP
        match action {
            Action::SelectPlayer {player_id } => {
                self.player_id = Some(player_id.clone());
            }
            Action::SelectSkill { skill_id: _ } => {
                // Java: clientCommandPrayerSelection.getPlayerId() + .getSkill()
                // In Rust Action::SelectSkill carries only skill_id; player_id not included.
                // This path is rare for the random agent (it declines dialogs).
            }
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
    fn execute_step(&mut self, game: &mut Game) -> StepOutcome {
        // Java: UtilServerDialog.hideDialog(gameState)
        // Java: lookup PrayerHandler from factories (not yet translated)
        //
        // Stub: PrayerHandlerFactory not translated → treat as "no handler found" path.
        // Java "no handler" path: getResult().setNextAction(NEXT_STEP)
        //
        // Java firstRun path when handler present: handler.initEffect() → may CONT for dialog.
        // Java secondRun path: handler.applySelection() → NEXT_STEP.
        // Both paths collapse to NEXT_STEP when the handler infrastructure is absent.
        if self.first_run {
            self.first_run = false;
            // Java: getResult().addReport(new ReportPrayerRoll(teamName, roll, isHomeTeam))
            let team_id = self.team_id.clone().unwrap_or_default();
            let team_name = game.team_by_id(&team_id)
                .map(|t| t.name.clone())
                .unwrap_or_default();
            let home_team = game.team_home.id == team_id;
            game.report_list.add(ReportPrayerRoll::new(team_name, self.roll, home_team));
        } else {
            self.first_run = false;
        }
        StepOutcome::next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::{StepAction, StepParameter};
    use ffb_mechanics::skills::SkillId;
    use ffb_model::enums::Rules;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2025)
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
    fn handle_command_select_player_stores_player_id() {
        let mut game = make_game();
        let mut step = StepPrayer::new(3, "home");
        let action = Action::SelectPlayer {player_id: "p1".into() };
        step.handle_command(&action, &mut game, &mut GameRng::new(0));
        assert_eq!(step.player_id.as_deref(), Some("p1"));
    }

    #[test]
    fn handle_command_returns_next_step() {
        let mut game = make_game();
        let mut step = StepPrayer::new(3, "home");
        let action = Action::SelectPlayer {player_id: "p1".into() };
        let out = step.handle_command(&action, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
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

    #[test]
    fn start_adds_prayer_roll_report_on_first_run() {
        let mut game = make_game();
        let mut step = StepPrayer::new(3, "home");
        step.start(&mut game, &mut GameRng::new(0));
        assert!(
            game.report_list.has_report(ffb_model::report::report_id::ReportId::PRAYER_ROLL),
            "ReportPrayerRoll must be added on first run"
        );
    }

    #[test]
    fn start_does_not_add_prayer_roll_report_on_second_run() {
        let mut game = make_game();
        let mut step = StepPrayer::new(3, "home");
        step.first_run = false; // simulate second run
        step.start(&mut game, &mut GameRng::new(0));
        assert!(
            !game.report_list.has_report(ffb_model::report::report_id::ReportId::PRAYER_ROLL),
            "ReportPrayerRoll must NOT be added on subsequent runs"
        );
    }
}
