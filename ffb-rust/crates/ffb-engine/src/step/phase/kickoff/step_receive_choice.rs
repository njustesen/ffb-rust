/// 1:1 translation of com.fumbbl.ffb.server.step.phase.kickoff.StepReceiveChoice.
///
/// Expects StepParameter::ChoosingTeamId from StepCoinChoice.
/// Waits for CLIENT_RECEIVE_CHOICE (Action::ReceiveChoice), then sets game.home_playing,
/// game.home_first_offense and game.setup_offense.
use ffb_model::model::game::Game;
use ffb_model::prompts::AgentPrompt;
use ffb_model::report::report_id::ReportId;
use ffb_model::report::report_receive_choice::ReportReceiveChoice;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepId, StepOutcome, StepParameter};

pub struct StepReceiveChoice {
    /// Java: fChoosingTeamId — set by StepParameter::ChoosingTeamId.
    choosing_team_id: Option<String>,
    /// Java: fReceiveChoice — home coach's choice; None until CLIENT_RECEIVE_CHOICE received.
    receive_choice: Option<bool>,
}

impl StepReceiveChoice {
    pub fn new() -> Self {
        Self { choosing_team_id: None, receive_choice: None }
    }
}

impl Default for StepReceiveChoice {
    fn default() -> Self { Self::new() }
}

impl Step for StepReceiveChoice {
    fn id(&self) -> StepId { StepId::ReceiveChoice }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        if let StepParameter::ChoosingTeamId(id) = param {
            self.choosing_team_id = id.clone();  // id is Option<String>
            return true;
        }
        false
    }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        if let Action::ReceiveChoice { receive } = action {
            self.receive_choice = Some(*receive);
        }
        self.execute_step(game)
    }
}

impl StepReceiveChoice {
    fn execute_step(&self, game: &mut Game) -> StepOutcome {
        if self.receive_choice.is_none() {
            // Java: UtilServerDialog.showDialog(getGameState(), new DialogReceiveChoiceParameter(choosingTeamId), false)
            let team_id = self.choosing_team_id.clone().unwrap_or_default();
            return StepOutcome::cont().with_prompt(AgentPrompt::ReceiveChoice { team_id });
        }
        let receive_choice = self.receive_choice.unwrap();
        // Java: if (game.getTeamHome().getId().equals(fChoosingTeamId)) { game.setHomePlaying(!fReceiveChoice); }
        //       else { game.setHomePlaying(fReceiveChoice); }
        if self.choosing_team_id.as_deref() == Some(game.team_home.id.as_str()) {
            game.home_playing = !receive_choice;
            // Java: getResult().addReport(new ReportReceiveChoice(game.getTeamHome().getId(), fReceiveChoice))
            game.report_list.add(ReportReceiveChoice::new(game.team_home.id.clone(), receive_choice));
        } else {
            game.home_playing = receive_choice;
            // Java: getResult().addReport(new ReportReceiveChoice(game.getTeamAway().getId(), fReceiveChoice))
            game.report_list.add(ReportReceiveChoice::new(game.team_away.id.clone(), receive_choice));
        }
        game.home_first_offense = !game.home_playing;
        game.setup_offense = false;
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
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    #[test]
    fn start_without_choice_returns_cont() {
        let mut game = make_game();
        let mut step = StepReceiveChoice::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::Continue);
    }

    #[test]
    fn receive_choice_command_triggers_execute() {
        let mut game = make_game();
        let mut step = StepReceiveChoice::new();
        step.set_parameter(&StepParameter::ChoosingTeamId(Some("home".to_string())));
        let out = step.handle_command(&Action::ReceiveChoice { receive: true }, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn home_choosing_receive_sets_home_playing_false() {
        let mut game = make_game();
        let mut step = StepReceiveChoice::new();
        step.set_parameter(&StepParameter::ChoosingTeamId(Some("home".to_string())));
        step.handle_command(&Action::ReceiveChoice { receive: true }, &mut game, &mut GameRng::new(0));
        // home chose receive → home NOT playing (i.e. home_playing = !true = false, home receives)
        assert!(!game.home_playing);
        assert!(game.home_first_offense);
        assert!(!game.setup_offense);
    }

    #[test]
    fn away_choosing_receive_sets_home_playing_true() {
        let mut game = make_game();
        let mut step = StepReceiveChoice::new();
        step.set_parameter(&StepParameter::ChoosingTeamId(Some("away".to_string())));
        step.handle_command(&Action::ReceiveChoice { receive: true }, &mut game, &mut GameRng::new(0));
        // away chose receive → home_playing = receive_choice = true (home kicks)
        assert!(game.home_playing);
        assert!(!game.home_first_offense);
        assert!(!game.setup_offense);
    }

    #[test]
    fn set_parameter_accepts_choosing_team_id() {
        let mut step = StepReceiveChoice::new();
        assert!(step.set_parameter(&StepParameter::ChoosingTeamId(Some("home".to_string()))));
        assert!(!step.set_parameter(&StepParameter::EndTurn(false)));
    }

    #[test]
    fn home_choosing_adds_receive_choice_report() {
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game();
        let mut step = StepReceiveChoice::new();
        step.set_parameter(&StepParameter::ChoosingTeamId(Some("home".to_string())));
        step.handle_command(&Action::ReceiveChoice { receive: true }, &mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::RECEIVE_CHOICE));
    }

    #[test]
    fn away_choosing_adds_receive_choice_report() {
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game();
        let mut step = StepReceiveChoice::new();
        step.set_parameter(&StepParameter::ChoosingTeamId(Some("away".to_string())));
        step.handle_command(&Action::ReceiveChoice { receive: false }, &mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::RECEIVE_CHOICE));
    }
}
