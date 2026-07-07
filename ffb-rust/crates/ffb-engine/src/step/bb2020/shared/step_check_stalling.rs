use ffb_model::model::game::Game;
use ffb_model::report::mixed::report_staller_detected::ReportStallerDetected;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2020.shared.StepCheckStalling.
///
/// Checks if the ball-carrier is stalling (BB2020).
///
/// Java init param: IGNORE_ACTED_FLAG (default: true).
/// Java start(): if performCheck() → findStallingPlayer() → if found, add staller.
/// Then NEXT_STEP.
///
/// Full stalling check requires prayer state and pathfinding infrastructure.
/// `ignore_acted_flag` is kept as a field for future use; the Java init param
/// IGNORE_ACTED_FLAG is handled via set_parameter.
///
/// Note: `StepParameter::IgnoreActedFlag` is handled via `set_parameter`.
pub struct StepCheckStalling {
    /// Java: fIgnoreActedFlag (default true in Java init)
    pub ignore_acted_flag: bool,
}

impl StepCheckStalling {
    pub fn new() -> Self {
        Self {
            ignore_acted_flag: true,
        }
    }

    /// Java: getResult().addReport(new ReportStallerDetected(stallingPlayer.getId()))
    /// Called when a stalling player is found. Wired here for structural completeness;
    /// the caller (start) currently skips detection because pathfinding is not yet ported.
    pub fn report_staller_detected(game: &mut Game, player_id: Option<String>) {
        game.report_list.add(ReportStallerDetected::new(player_id));
    }
}

impl Default for StepCheckStalling {
    fn default() -> Self { Self::new() }
}

impl Step for StepCheckStalling {
    fn id(&self) -> StepId { StepId::CheckStalling }

    fn start(&mut self, _game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // no-op: stalling detection requires pathfinding — headless conservatively reports no stall.
        // Java: if (performCheck()) { stallingPlayer = findStallingPlayer(); if (stallingPlayer != null) {
        //           addReport(new ReportStallerDetected(stallingPlayer.getId()));
        //           prayerState.addStaller(stallingPlayer); } }
        StepOutcome::next()
    }

    fn handle_command(&mut self, _action: &Action, _game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        StepOutcome::next()
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::IgnoreActedFlag(v) => { self.ignore_acted_flag = *v; true }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::StepAction;
    use ffb_model::enums::Rules;
    use ffb_model::report::report_id::ReportId;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2020)
    }

    #[test]
    fn new_has_ignore_acted_flag_true() {
        let step = StepCheckStalling::new();
        assert!(step.ignore_acted_flag);
    }

    #[test]
    fn start_returns_next_step() {
        let mut game = make_game();
        let mut step = StepCheckStalling::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn handle_command_returns_next_step() {
        let mut game = make_game();
        let mut step = StepCheckStalling::new();
        let out = step.handle_command(&Action::Acknowledge, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn set_parameter_ignore_acted_flag_accepted() {
        let mut step = StepCheckStalling::new();
        assert!(step.ignore_acted_flag); // default = true
        assert!(step.set_parameter(&StepParameter::IgnoreActedFlag(false)));
        assert!(!step.ignore_acted_flag);
        assert!(step.set_parameter(&StepParameter::IgnoreActedFlag(true)));
        assert!(step.ignore_acted_flag);
    }

    #[test]
    fn set_parameter_unknown_returns_false() {
        let mut step = StepCheckStalling::new();
        assert!(!step.set_parameter(&StepParameter::EndTurn(true)));
    }

    #[test]
    fn report_staller_detected_with_player_id_emits_report() {
        let mut game = make_game();
        StepCheckStalling::report_staller_detected(&mut game, Some("home_01".into()));
        assert!(game.report_list.has_report(ReportId::STALLER_DETECTED));
    }

    #[test]
    fn report_staller_detected_without_player_id_emits_report() {
        let mut game = make_game();
        StepCheckStalling::report_staller_detected(&mut game, None);
        assert!(game.report_list.has_report(ReportId::STALLER_DETECTED));
    }
}
