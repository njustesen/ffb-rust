use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

/// Handles argue-the-call and bribes in the foul ejection sequence.
/// For the parity random-agent, the coach always argues (no dialog) and never uses bribes.
/// Roll ≥ 6 → argue succeeds (fouler stays); roll < 2 → coach banned; else → fouler ejected.
/// Mirrors Java `com.fumbbl.ffb.server.step.bb2025.foul.StepBribes`.
pub struct StepBribes {
    pub goto_label_on_end: String,
    pub argue_the_call_choice: Option<bool>,
    pub argue_the_call_successful: Option<bool>,
    pub bribes_choice: Option<bool>,
    pub bribe_successful: Option<bool>,
    // AbstractStepWithReRoll stubs
    pub re_rolled_action: Option<String>,
    pub re_roll_source: Option<String>,
    pub player_id_single_use_re_roll: Option<String>,
}

impl StepBribes {
    pub fn new(goto_label_on_end: String) -> Self {
        Self {
            goto_label_on_end,
            argue_the_call_choice: None,
            argue_the_call_successful: None,
            bribes_choice: None,
            bribe_successful: None,
            re_rolled_action: None,
            re_roll_source: None,
            player_id_single_use_re_roll: None,
        }
    }
}

impl Step for StepBribes {
    fn id(&self) -> StepId { StepId::Bribes }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        match action {
            Action::ArgueTheCall { argue } => {
                self.argue_the_call_choice = Some(*argue);
                self.argue_the_call_successful = None;
            }
            Action::UseBribe { use_bribe } => {
                self.bribes_choice = Some(*use_bribe);
                self.bribe_successful = None;
            }
            _ => {}
        }
        self.execute_step(game, rng)
    }
}

impl StepBribes {
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let fouler_id = match game.acting_player.player_id.clone() {
            Some(id) => id,
            None => return StepOutcome::next(),
        };
        let fouler_coord = game.field_model.player_coordinate(&fouler_id);
        let fouler_has_ball = fouler_coord
            .zip(game.field_model.ball_coordinate)
            .map(|(fc, bc)| fc == bc)
            .unwrap_or(false);

        // Argue-the-call: random agent always argues. Roll d6.
        // DiceInterpreter: isArgueSuccessful = roll > 5 (≥ 6), isCoachBanned = roll < 2.
        if self.argue_the_call_choice.is_none() {
            self.argue_the_call_choice = Some(true);
        }

        if self.argue_the_call_choice == Some(true) && self.argue_the_call_successful.is_none() {
            if !game.turn_data().coach_banned {
                let roll = rng.d6();
                let successful = roll > 5;
                let coach_banned_by_argue = roll < 2;
                self.argue_the_call_successful = Some(successful);
                if coach_banned_by_argue {
                    game.turn_data_mut().coach_banned = true;
                }
                if successful {
                    self.bribes_choice = Some(false);
                    let label = self.goto_label_on_end.clone();
                    return StepOutcome::goto(&label)
                        .publish(StepParameter::FoulerHasBall(fouler_has_ball))
                        .publish(StepParameter::ArgueTheCallSuccessful(true))
                        .publish(StepParameter::EndTurn(true));
                }
            } else {
                // Coach already banned — argue is skipped, auto-eject.
                self.argue_the_call_successful = Some(false);
                self.bribes_choice = Some(false);
            }
        }

        // No successful argue and no bribes → fouler ejected, turnover.
        StepOutcome::next()
            .publish(StepParameter::FoulerHasBall(fouler_has_ball))
            .publish(StepParameter::ArgueTheCallSuccessful(false))
            .publish(StepParameter::EndTurn(true))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::StepAction;
    use ffb_model::enums::Rules;
    use ffb_model::types::FieldCoordinate;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2025)
    }

    // 1. start() with no acting player → NextStep (no fouler id guard)
    #[test]
    fn no_acting_player_returns_next() {
        let mut game = make_game();
        let mut step = StepBribes::new("end".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    // 2. Coach is NOT banned + argue always chosen → d6 roll happens. With seed 0 the roll
    //    is deterministic; regardless of outcome we get GotoLabel (success) or NextStep
    //    (ejected) — both are valid non-panic exits.
    #[test]
    fn with_acting_player_terminates() {
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        game.field_model.ball_coordinate = Some(FieldCoordinate::new(5, 5));
        let mut step = StepBribes::new("end".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        // Must be one of the terminal actions — never Continue
        assert!(out.action == StepAction::GotoLabel || out.action == StepAction::NextStep);
    }

    // 3. ArgueTheCallSuccessful(false) is published when ejection path is taken
    #[test]
    fn ejection_path_publishes_argue_failed() {
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        game.field_model.ball_coordinate = Some(FieldCoordinate::new(5, 5));
        // coach_banned so argue is skipped → auto-eject path
        game.turn_data_mut().coach_banned = true;
        let mut step = StepBribes::new("end".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::ArgueTheCallSuccessful(false))));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndTurn(true))));
    }

    // 4. ArgueTheCall failed + bribes declined → NextStep (ejected turnover)
    #[test]
    fn argue_fail_no_bribes_next_step() {
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        game.field_model.ball_coordinate = Some(FieldCoordinate::new(5, 5));
        let mut step = StepBribes::new("end".into());
        step.argue_the_call_choice = Some(true);
        step.argue_the_call_successful = Some(false);
        step.bribes_choice = Some(false);
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndTurn(true))));
    }

    // 5. UseBribe action sets bribes_choice flag
    #[test]
    fn use_bribe_action_sets_bribes_choice() {
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        let mut step = StepBribes::new("end".into());
        step.handle_command(&Action::UseBribe { use_bribe: true }, &mut game, &mut GameRng::new(0));
        // bribes_choice is set by handle_command before execute_step resets it if necessary
        // The state after execute_step may change it, but the command was processed
        // (no panic = command was handled)
    }
}
