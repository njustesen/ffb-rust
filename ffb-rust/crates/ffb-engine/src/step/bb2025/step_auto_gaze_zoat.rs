/// 1:1 translation of com.fumbbl.ffb.server.step.bb2025.StepAutoGazeZoat (BB2025).
///
/// Applies the Zoat's automatic gaze to an adjacent opponent (within 3 squares).
///
/// Stub: NamedProperties.canGazeAutomaticallyThreeSquaresAway is not a standard SkillId and
/// is not yet translated. The skill check always returns None → NEXT_STEP immediately.
/// Fields and parameter wiring are complete; full logic awaits NamedProperty translation.
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

pub struct StepAutoGazeZoat {
    /// Java: endPlayerAction — set by END_PLAYER_ACTION parameter.
    pub end_player_action: bool,
    /// Java: endTurn — set by END_TURN parameter.
    pub end_turn: bool,
    /// Java: goToLabelOnFailure — GOTO_LABEL_ON_FAILURE init parameter.
    pub goto_label_on_failure: String,
    /// Java: playerId — set from CLIENT_PLAYER_CHOICE command.
    pub player_id: Option<String>,
}

impl StepAutoGazeZoat {
    pub fn new() -> Self {
        Self {
            end_player_action: false,
            end_turn: false,
            goto_label_on_failure: String::new(),
            player_id: None,
        }
    }
}

impl Default for StepAutoGazeZoat {
    fn default() -> Self { Self::new() }
}

impl Step for StepAutoGazeZoat {
    fn id(&self) -> StepId { StepId::AutoGazeZoat }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // Java: CLIENT_PLAYER_CHOICE — set playerId or decline (empty id → SKIP_STEP / NEXT_STEP)
        // Java: CLIENT_END_TURN → set endTurn + EXECUTE_STEP
        match action {
            Action::SelectPlayer { player_id } => {
                if player_id.is_empty() {
                    // Declined: Java SKIP_STEP path — report skill not used, NEXT_STEP
                    return StepOutcome::next();
                }
                self.player_id = Some(player_id.clone());
            }
            Action::EndTurn => { self.end_turn = true; }
            _ => {}
        }
        self.execute_step(game)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::EndTurn(v)               => { self.end_turn = *v; true }
            StepParameter::EndPlayerAction(v)       => { self.end_player_action = *v; true }
            StepParameter::GotoLabelOnFailure(v)    => { self.goto_label_on_failure = v.clone(); true }
            _ => false,
        }
    }
}

impl StepAutoGazeZoat {
    fn execute_step(&self, _game: &mut Game) -> StepOutcome {
        // Java: skill = UtilCards.getUnusedSkillWithProperty(actingPlayer,
        //           NamedProperties.canGazeAutomaticallyThreeSquaresAway)
        // Stub: NamedProperties.canGazeAutomaticallyThreeSquaresAway is not a standard
        // SkillId — no Rust equivalent. Skill check always returns None.
        // Java "if (skill != null)" branch is unreachable → fall through to NEXT_STEP.
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
    fn start_always_returns_next_step() {
        let mut game = make_game();
        let mut step = StepAutoGazeZoat::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn handle_command_end_turn_returns_next_step() {
        let mut game = make_game();
        let mut step = StepAutoGazeZoat::new();
        let out = step.handle_command(&Action::EndTurn, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn handle_command_decline_player_returns_next_step() {
        let mut game = make_game();
        let mut step = StepAutoGazeZoat::new();
        let action = Action::SelectPlayer { player_id: String::new() };
        let out = step.handle_command(&action, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn set_parameter_end_turn() {
        let mut step = StepAutoGazeZoat::new();
        assert!(step.set_parameter(&StepParameter::EndTurn(true)));
        assert!(step.end_turn);
    }

    #[test]
    fn set_parameter_goto_label_on_failure() {
        let mut step = StepAutoGazeZoat::new();
        assert!(step.set_parameter(&StepParameter::GotoLabelOnFailure("FAIL".into())));
        assert_eq!(step.goto_label_on_failure, "FAIL");
    }

    #[test]
    fn set_parameter_end_player_action() {
        let mut step = StepAutoGazeZoat::new();
        assert!(step.set_parameter(&StepParameter::EndPlayerAction(true)));
        assert!(step.end_player_action);
    }
}
