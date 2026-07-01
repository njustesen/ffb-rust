use ffb_model::types::FieldCoordinate;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2020.move.StepShadowing.
///
/// BB2020 logic is identical to BB2025.
///
/// Handles the Shadowing skill: an opposing player with Shadowing may move to follow
/// a dodging/jumping player.
///
/// Delegates to `executeStepHooks(this, state)` in Java, meaning all logic lives in
/// hook implementations (not in the step itself).
///
/// DEFERRED(executeStepHooks): all shadowing logic lives in hook implementations attached to the
/// game state (eligibility check, shadower movement, RE_ROLL_USED publishing). The hook dispatch
/// system is not yet ported — stub always emits NEXT_STEP.
pub struct StepShadowing {
    /// Java: state.coordinateFrom
    pub coordinate_from: Option<FieldCoordinate>,
    /// Java: state.defenderPosition
    pub defender_position: Option<FieldCoordinate>,
    /// Java: state.usingDivingTackle
    pub using_diving_tackle: bool,
    /// Java: state.usingShadowing (Boolean tristate)
    pub using_shadowing: Option<bool>,
    /// Java: state.shadowerWasPreviousDefender
    pub shadower_was_previous_defender: bool,
    // AbstractStepWithReRoll stubs
    pub re_rolled_action: Option<String>,
    pub re_roll_source: Option<String>,
}

impl StepShadowing {
    pub fn new() -> Self {
        Self {
            coordinate_from: None,
            defender_position: None,
            using_diving_tackle: false,
            using_shadowing: None,
            shadower_was_previous_defender: false,
            re_rolled_action: None,
            re_roll_source: None,
        }
    }
}

impl Default for StepShadowing {
    fn default() -> Self { Self::new() }
}

impl Step for StepShadowing {
    fn id(&self) -> StepId { StepId::Shadowing }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: case CLIENT_PLAYER_CHOICE:
        //   if PlayerChoiceMode.SHADOWING == playerChoiceMode:
        //     state.usingShadowing = StringTool.isProvided(playerId)
        //     if defenderId provided && defenderId == playerId → shadowerWasPreviousDefender = true
        //     else → game.setDefenderId(playerId)
        if let Action::PlayerChoice { player_id, mode, .. } = action {
            if mode == "SHADOWING" {
                self.using_shadowing = Some(player_id.is_some());
                if let Some(ref pid) = player_id {
                    let is_previous_defender = game.defender_id.as_deref()
                        .map(|did| did == pid.as_str())
                        .unwrap_or(false);
                    if is_previous_defender {
                        self.shadower_was_previous_defender = true;
                    } else {
                        game.defender_id = Some(pid.clone());
                    }
                }
            }
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::CoordinateFrom(v) => { self.coordinate_from = Some(*v); true }
            StepParameter::DefenderPosition(v) => { self.defender_position = Some(*v); true }
            StepParameter::UsingDivingTackle(v) => { self.using_diving_tackle = *v; true }
            // Java: JUMPED → state.usingShadowing = false
            StepParameter::Jumped(_) => { self.using_shadowing = Some(false); true }
            StepParameter::UsingShadowing(v) => { self.using_shadowing = *v; true }
            StepParameter::ShadowerWasPreviousDefender(v) => {
                self.shadower_was_previous_defender = *v; true
            }
            _ => false,
        }
    }
}

impl StepShadowing {
    fn execute_step(&self, _game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // Java: getGameState().executeStepHooks(this, state)
        // All shadowing logic is in hook implementations attached to the game state.
        // DEFERRED(executeStepHooks): hook dispatch system not yet ported.
        StepOutcome::next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::{StepAction, StepParameter};
    use ffb_model::enums::Rules;
    use ffb_model::util::rng::GameRng;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2020)
    }

    #[test]
    fn start_returns_next_step() {
        let mut game = make_game();
        let mut step = StepShadowing::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn jumped_parameter_sets_using_shadowing_false() {
        let mut step = StepShadowing::new();
        step.using_shadowing = Some(true);
        assert!(step.set_parameter(&StepParameter::Jumped(true)));
        assert_eq!(step.using_shadowing, Some(false));
    }

    #[test]
    fn coordinate_from_parameter_accepted() {
        let mut step = StepShadowing::new();
        let coord = FieldCoordinate::new(3, 4);
        assert!(step.set_parameter(&StepParameter::CoordinateFrom(coord)));
        assert_eq!(step.coordinate_from, Some(coord));
    }

    #[test]
    fn using_diving_tackle_parameter_accepted() {
        let mut step = StepShadowing::new();
        assert!(step.set_parameter(&StepParameter::UsingDivingTackle(true)));
        assert!(step.using_diving_tackle);
    }

    #[test]
    fn unrecognised_parameter_returns_false() {
        let mut step = StepShadowing::new();
        assert!(!step.set_parameter(&StepParameter::EndTurn(true)));
    }
}
