/// 1:1 translation of com.fumbbl.ffb.server.step.bb2025.StepThenIStartedBlastin (BB2025).
///
/// Resolves the "Then I Started Blastin'!" ability: throw a keg at a target, causing injury.
///
/// Commands: CLIENT_TARGET_SELECTED (target selection), CLIENT_END_TURN.
///
/// Stub: NamedProperties.canBlastRemotePlayer not translated → skill check always fails.
/// InjuryTypeThenIStartedBlastin not translated → NEXT_STEP immediately.
use ffb_model::model::game::Game;
use ffb_model::report::mixed::report_then_i_started_blastin::ReportThenIStartedBlastin;
use ffb_model::report::report_id::ReportId;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

pub struct StepThenIStartedBlastin {
    /// Java: gotoLabelOnEnd — GOTO_LABEL_ON_END init parameter.
    pub goto_label_on_end: String,
    /// Java: roll — the skill die result.
    pub roll: i32,
}

impl StepThenIStartedBlastin {
    pub fn new() -> Self {
        Self {
            goto_label_on_end: String::new(),
            roll: 0,
        }
    }
}

impl Default for StepThenIStartedBlastin {
    fn default() -> Self { Self::new() }
}

impl Step for StepThenIStartedBlastin {
    fn id(&self) -> StepId { StepId::ThenIStartedBlastin }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // Stub: NamedProperties.canBlastRemotePlayer not translated → NEXT_STEP
        // Java: when skill is present, rolls and emits ReportThenIStartedBlastin(actorId, defenderId, roll, success, roll==1)
        let acting_id = game.acting_player.player_id.clone();
        let defender_id = game.defender_id.clone();
        game.report_list.add(ReportThenIStartedBlastin::new(
            acting_id,
            defender_id,
            self.roll,
            false,
            self.roll == 1,
        ));
        StepOutcome::next()
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        match action {
            Action::SelectPlayer { player_id } => {
                // Java: if target is not on playing team (opponent) → addReport(ReportThenIStartedBlastin(... 0, true, false))
                let is_opponent = game.inactive_team().player(player_id).is_some();
                if is_opponent {
                    let acting_id = game.acting_player.player_id.clone();
                    game.report_list.add(ReportThenIStartedBlastin::new(
                        acting_id,
                        Some(player_id.clone()),
                        0,
                        true,
                        false,
                    ));
                }
            }
            Action::EndTurn => {
                // Java: restoreTurnModes + publish END_PLAYER_ACTION + NEXT_STEP
            }
            _ => {}
        }
        StepOutcome::next()
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnEnd(v) => { self.goto_label_on_end = v.clone(); true }
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

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2025)
    }

    #[test]
    fn start_returns_next_step() {
        let mut game = make_game();
        let mut step = StepThenIStartedBlastin::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn handle_end_turn_returns_next_step() {
        let mut game = make_game();
        let mut step = StepThenIStartedBlastin::new();
        let out = step.handle_command(&Action::EndTurn, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn set_parameter_goto_label_on_end() {
        let mut step = StepThenIStartedBlastin::new();
        assert!(step.set_parameter(&StepParameter::GotoLabelOnEnd("END".into())));
        assert_eq!(step.goto_label_on_end, "END");
    }

    #[test]
    fn start_adds_then_i_started_blastin_report() {
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game();
        let mut step = StepThenIStartedBlastin::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::THEN_I_STARTED_BLASTIN));
    }

    #[test]
    fn select_opponent_adds_then_i_started_blastin_report() {
        use ffb_model::model::player::Player;
        use ffb_model::report::report_id::ReportId;
        let home = test_team("home", 0);
        let mut away = test_team("away", 0);
        away.players.push(Player { id: "away_p1".into(), name: "A".into(), ..Default::default() });
        let mut game = Game::new(home, away, Rules::Bb2025);
        game.home_playing = true;
        let mut step = StepThenIStartedBlastin::new();
        let action = Action::SelectPlayer { player_id: "away_p1".into() };
        step.handle_command(&action, &mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::THEN_I_STARTED_BLASTIN));
    }
}
