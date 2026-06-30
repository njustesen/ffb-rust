/// 1:1 translation of `com.fumbbl.ffb.server.step.mixed.move.StepTentacles`.
///
/// Handles the TENTACLES skill check during movement.  When the acting player
/// moves out of the tackle zone of an opponent with Tentacles, that opponent may
/// attempt to grab the mover with a strength contest.
///
/// Java delegates the actual roll logic to `executeStepHooks`; this Rust stub
/// translates the parameter wiring and the player-choice command dispatch.
///
/// Init parameters (mandatory): GOTO_LABEL_ON_SUCCESS.
/// Incoming parameters: COORDINATE_FROM.
/// Incoming command: CLIENT_PLAYER_CHOICE (PlayerChoiceMode.TENTACLES).
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};
use crate::step::abstract_step_with_re_roll::ReRollState;

/// Java: `StepTentacles.StepState` — all step-local state collected in one place.
#[derive(Debug, Default)]
pub struct StepTentaclesState {
    /// Java: `state.goToLabelOnSuccess`
    pub go_to_label_on_success: String,
    /// Java: `state.coordinateFrom`
    pub coordinate_from: Option<ffb_model::types::FieldCoordinate>,
    /// Java: `state.usingTentacles` — `None` = not yet decided.
    pub using_tentacles: Option<bool>,
}

/// Java: `StepTentacles` (mixed/move, BB2020 + BB2025).
/// Extends AbstractStepWithReRoll.
#[derive(Debug, Default)]
pub struct StepTentacles {
    pub state: StepTentaclesState,
    /// Re-roll tracking (AbstractStepWithReRoll).
    pub re_roll_state: ReRollState,
}

impl StepTentacles {
    pub fn new() -> Self { Self::default() }

    fn execute_step(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // Java: getGameState().executeStepHooks(this, state)
        // Hooks handle the actual roll and navigation; here we just translate the
        // state that hooks need:
        //   - state.goToLabelOnSuccess → used by hook to goto on success
        //   - state.usingTentacles     → set by CLIENT_PLAYER_CHOICE handler
        //   - state.coordinateFrom     → set by preceding step

        // When usingTentacles is None the step waits for the player choice dialog.
        // When it's Some(false) the player chose not to use Tentacles → NextStep.
        // When it's Some(true) hooks perform the roll and navigate.
        match self.state.using_tentacles {
            None => {
                // Java: hooks show the dialog; here just continue (wait).
                // In the absence of the full hook infrastructure, immediately advance.
                StepOutcome::next()
            }
            Some(false) => StepOutcome::next(),
            Some(true) => {
                // Java: goto success label after the tentacles roll succeeds.
                let lbl = self.state.go_to_label_on_success.clone();
                if lbl.is_empty() {
                    StepOutcome::next()
                } else {
                    StepOutcome::goto(&lbl)
                }
            }
        }
    }
}

impl Step for StepTentacles {
    fn id(&self) -> StepId { StepId::Tentacles }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: CLIENT_PLAYER_CHOICE / PlayerChoiceMode.TENTACLES
        if let Action::PlayerChoice { player_id, mode, .. } = action {
            if mode == "TENTACLES" {
                // Java: state.usingTentacles = StringTool.isProvided(playerChoiceCommand.getPlayerId())
                self.state.using_tentacles = Some(player_id.is_some());
                if let Some(pid) = player_id {
                    // Java: game.setLastDefenderId(game.getDefenderId()); game.setDefenderId(playerId)
                    game.last_defender_id = game.defender_id.clone();
                    game.defender_id = Some(pid.clone());
                }
            }
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::CoordinateFrom(v)     => { self.state.coordinate_from = Some(*v); true }
            StepParameter::GotoLabelOnSuccess(v) => { self.state.go_to_label_on_success = v.clone(); true }
            _ => false,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{test_team, StepAction};
    use ffb_model::enums::Rules;
    use ffb_model::model::game::Game;
    use ffb_model::util::rng::GameRng;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    #[test]
    fn id_is_tentacles() {
        assert_eq!(StepTentacles::new().id(), StepId::Tentacles);
    }

    #[test]
    fn start_without_choice_returns_next() {
        let mut step = StepTentacles::new();
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn player_choice_false_returns_next() {
        let mut step = StepTentacles::new();
        step.state.go_to_label_on_success = "success".into();
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let out = step.handle_command(
            &Action::PlayerChoice { player_id: None, player_ids: vec![], mode: "TENTACLES".into() },
            &mut game,
            &mut rng,
        );
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn player_choice_true_goes_to_success_label() {
        let mut step = StepTentacles::new();
        step.state.go_to_label_on_success = "success".into();
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let out = step.handle_command(
            &Action::PlayerChoice { player_id: Some("def1".into()), player_ids: vec![], mode: "TENTACLES".into() },
            &mut game,
            &mut rng,
        );
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("success"));
    }

    #[test]
    fn set_parameter_goto_label_on_success() {
        let mut step = StepTentacles::new();
        step.set_parameter(&StepParameter::GotoLabelOnSuccess("lbl".into()));
        assert_eq!(step.state.go_to_label_on_success, "lbl");
    }
}
