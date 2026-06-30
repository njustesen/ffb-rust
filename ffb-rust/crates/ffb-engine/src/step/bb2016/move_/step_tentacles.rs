use ffb_model::types::FieldCoordinate;
use ffb_model::model::game::Game;
#[allow(unused_imports)]
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2016.move.StepTentacles.
///
/// BB2016 StepTentacles holds a `StepState { goToLabelOnSuccess, coordinateFrom, usingTentacles }`
/// and delegates all logic to `executeStepHooks(this, state)`.
///
/// The step resolves the TENTACLES skill: if an adjacent player with Tentacles is present
/// when the acting player tries to leave the tackle zone, a contested strength roll is made.
/// If the Tentacles player wins, the acting player's move is cancelled (GoTo success label = stuck).
/// If the acting player wins, movement continues normally (NEXT_STEP).
///
/// Init params: GOTO_LABEL_ON_SUCCESS (mandatory), COORDINATE_FROM (mandatory).
///
/// Client command handled:
/// - CLIENT_PLAYER_CHOICE (TENTACLES mode): sets usingTentacles + tentacles player ID.
///
/// Since the hook infrastructure is not yet ported, this is implemented as a
/// minimal stub that:
/// - Handles CLIENT_PLAYER_CHOICE to set usingTentacles
/// - Defers the roll to TODO
/// - Returns NEXT_STEP (no tentacles block)
///
/// TODO(hooks): executeStepHooks infrastructure not yet ported.
/// TODO(tentaclesRoll): contested strength roll (actingPlayer.str vs tentaclesPlayer.str) not yet ported.
/// TODO(successGoto): GOTO_LABEL_ON_SUCCESS when Tentacles wins not yet ported.
/// TODO(modifier): TentaclesModifierFactory not yet ported.
pub struct StepTentacles {
    /// Java: StepState.goToLabelOnSuccess
    pub goto_label_on_success: String,
    /// Java: StepState.coordinateFrom
    pub coordinate_from: Option<FieldCoordinate>,
    /// Java: StepState.usingTentacles (the player being grabbed)
    pub using_tentacles: bool,
    /// Java: tentacles player ID (set via CLIENT_PLAYER_CHOICE)
    pub tentacles_player_id: Option<String>,
}

impl StepTentacles {
    pub fn new(goto_label_on_success: String) -> Self {
        Self {
            goto_label_on_success,
            coordinate_from: None,
            using_tentacles: false,
            tentacles_player_id: None,
        }
    }
}

impl Default for StepTentacles {
    fn default() -> Self { Self::new(String::new()) }
}

impl Step for StepTentacles {
    fn id(&self) -> StepId { StepId::Tentacles }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        match action {
            // Java: CLIENT_PLAYER_CHOICE (TENTACLES mode) → usingTentacles = true, tentaclesPlayerId = id
            // In Rust, SelectPlayer is the closest equivalent to CLIENT_PLAYER_CHOICE.
            // TODO(playerChoice): Java CLIENT_PLAYER_CHOICE has a "mode" field not in Rust Action::SelectPlayer
            Action::SelectPlayer { player_id } => {
                self.tentacles_player_id = Some(player_id.clone());
                self.using_tentacles = true;
            }
            _ => {}
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnSuccess(v) => { self.goto_label_on_success = v.clone(); true }
            StepParameter::CoordinateFrom(v) => { self.coordinate_from = Some(*v); true }
            _ => false,
        }
    }
}

impl StepTentacles {
    fn execute_step(&mut self, _game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // TODO(hooks): executeStepHooks not yet ported
        // TODO(tentaclesRoll): if usingTentacles → roll contested strength
        //   - Tentacles wins → GOTO_LABEL(fGotoLabelOnSuccess) = move cancelled
        //   - Acting player wins → NEXT_STEP = move continues
        // For now: always NEXT_STEP (no tentacles effect)
        StepOutcome::next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::{StepAction, StepParameter};
    use ffb_model::enums::Rules;
    use ffb_model::types::FieldCoordinate;
    use ffb_model::util::rng::GameRng;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2016)
    }

    #[test]
    fn start_returns_next_step_stub() {
        let mut game = make_game();
        let mut step = StepTentacles::new("success".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn select_player_sets_using_tentacles() {
        let mut game = make_game();
        let mut step = StepTentacles::new("success".into());
        let action = Action::SelectPlayer { player_id: "tentpid".into() };
        step.handle_command(&action, &mut game, &mut GameRng::new(0));
        assert!(step.using_tentacles);
        assert_eq!(step.tentacles_player_id.as_deref(), Some("tentpid"));
    }

    #[test]
    fn unknown_command_does_not_set_tentacles() {
        let mut game = make_game();
        let mut step = StepTentacles::new("success".into());
        step.using_tentacles = false;
        let action = Action::EndTurn;
        step.handle_command(&action, &mut game, &mut GameRng::new(0));
        assert!(!step.using_tentacles);
    }

    #[test]
    fn set_parameter_goto_label_on_success_accepted() {
        let mut step = StepTentacles::new("old".into());
        assert!(step.set_parameter(&StepParameter::GotoLabelOnSuccess("new".into())));
        assert_eq!(step.goto_label_on_success, "new");
    }

    #[test]
    fn set_parameter_coordinate_from_accepted() {
        let mut step = StepTentacles::new("success".into());
        let coord = FieldCoordinate::new(5, 5);
        assert!(step.set_parameter(&StepParameter::CoordinateFrom(coord)));
        assert_eq!(step.coordinate_from, Some(coord));
    }

    #[test]
    fn unrecognised_parameter_returns_false() {
        let mut step = StepTentacles::new("success".into());
        assert!(!step.set_parameter(&StepParameter::EndTurn(true)));
    }

    #[test]
    fn other_command_still_returns_next_step() {
        let mut game = make_game();
        let mut step = StepTentacles::new("success".into());
        let out = step.handle_command(&Action::EndTurn, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn acknowledge_command_does_not_set_tentacles() {
        let mut game = make_game();
        let mut step = StepTentacles::new("success".into());
        let action = Action::Acknowledge;
        step.handle_command(&action, &mut game, &mut GameRng::new(0));
        assert!(!step.using_tentacles);
        assert!(step.tentacles_player_id.is_none());
    }
}
