/// 1:1 translation of com.fumbbl.ffb.server.step.action.move.StepDivingTackle (COMMON rules)
/// and its BB2025 hook com.fumbbl.ffb.server.skillbehaviour.bb2025.DivingTackleBehaviour.
///
/// Handles the Diving Tackle skill prompt during a dodge.
/// Needs GOTO_LABEL_ON_SUCCESS init parameter.
/// Expects COORDINATE_FROM, COORDINATE_TO, DODGE_ROLL parameters.
///
/// Stub: find_eligible_diving_tacklers always returns empty → USING_DIVING_TACKLE(false)
/// published and NEXT_STEP returned. Full implementation requires DodgeModifierFactory,
/// UtilPlayer.findEligibleDivingTacklers, and interactive coach prompts.
use ffb_model::types::FieldCoordinate;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

pub struct StepDivingTackle {
    /// Java: state.goToLabelOnSuccess — GOTO_LABEL_ON_SUCCESS init parameter.
    pub goto_label_on_success: String,
    /// Java: state.coordinateFrom — set by COORDINATE_FROM parameter.
    pub coordinate_from: Option<FieldCoordinate>,
    /// Java: state.coordinateTo — set by COORDINATE_TO parameter.
    pub coordinate_to: Option<FieldCoordinate>,
    /// Java: state.dodgeRoll — set by DODGE_ROLL parameter.
    pub dodge_roll: i32,
    /// Java: state.usingDivingTackle — None = waiting for coach decision,
    /// Some(false) = not using, Some(true) = using.
    pub using_diving_tackle: Option<bool>,
    /// Java: state.usingBreakTackle
    pub using_break_tackle: bool,
    /// Java: state.usingModifyingSkill
    pub using_modifying_skill: Option<bool>,
}

impl StepDivingTackle {
    pub fn new() -> Self {
        Self {
            goto_label_on_success: String::new(),
            coordinate_from: None,
            coordinate_to: None,
            dodge_roll: 0,
            using_diving_tackle: None,
            using_break_tackle: false,
            using_modifying_skill: None,
        }
    }
}

impl Default for StepDivingTackle {
    fn default() -> Self { Self::new() }
}

impl Step for StepDivingTackle {
    fn id(&self) -> StepId { StepId::DivingTackle }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // Java: handleCommand receives CLIENT_PLAYER_CHOICE(DIVING_TACKLE) → usingDivingTackle.
        // Stub: dialog/player-choice not yet implemented; execute step directly.
        self.execute_step(game)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnSuccess(v) => { self.goto_label_on_success = v.clone(); true }
            StepParameter::CoordinateFrom(v) => { self.coordinate_from = Some(*v); true }
            StepParameter::CoordinateTo(v) => { self.coordinate_to = Some(*v); true }
            StepParameter::DodgeRoll(v) => { self.dodge_roll = *v; true }
            StepParameter::UsingBreakTackle(v) => { self.using_break_tackle = *v; true }
            StepParameter::UsingModifyingSkill(v) => { self.using_modifying_skill = Some(*v); true }
            StepParameter::UsingDivingTackle(v) => { self.using_diving_tackle = Some(*v); true }
            _ => false,
        }
    }
}

impl StepDivingTackle {
    /// Java: DivingTackleBehaviour(BB2025).handleExecuteStepHook
    fn execute_step(&mut self, _game: &mut Game) -> StepOutcome {
        // Java: if (usingDivingTackle == null) { check eligible tacklers + maybe prompt }
        if self.using_diving_tackle.is_none() {
            // Stub: UtilPlayer.findEligibleDivingTacklers → always empty.
            // With no eligible tacklers, skip straight to usingDivingTackle = false.
            self.using_diving_tackle = Some(false);
        }

        // Java: if (usingDivingTackle != null) { publish; if true → GOTO_LABEL else → NEXT_STEP }
        let using = self.using_diving_tackle.unwrap_or(false);
        let outcome = StepOutcome::next()
            .publish(StepParameter::UsingDivingTackle(using));

        if using {
            // Java: GOTO_LABEL(goToLabelOnSuccess)
            StepOutcome::goto(&self.goto_label_on_success)
                .publish(StepParameter::UsingDivingTackle(true))
        } else {
            // Java: NEXT_STEP
            outcome
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::StepAction;
    use ffb_model::model::game::Game;
    use ffb_model::enums::Rules;
    use crate::step::framework::test_team;
    use ffb_model::util::rng::GameRng;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    #[test]
    fn no_eligible_tacklers_returns_next_step() {
        let mut step = StepDivingTackle::new();
        step.goto_label_on_success = "DT".into();
        let outcome = step.start(&mut make_game(), &mut GameRng::new(0));
        assert_eq!(outcome.action, StepAction::NextStep);
    }

    #[test]
    fn using_diving_tackle_false_published_by_default() {
        let mut step = StepDivingTackle::new();
        let outcome = step.start(&mut make_game(), &mut GameRng::new(0));
        assert!(outcome.published.iter().any(|p| matches!(p, StepParameter::UsingDivingTackle(false))));
    }

    #[test]
    fn set_using_diving_tackle_true_goes_to_label() {
        let mut step = StepDivingTackle::new();
        step.goto_label_on_success = "DT_LABEL".into();
        step.using_diving_tackle = Some(true);
        let outcome = step.start(&mut make_game(), &mut GameRng::new(0));
        assert_eq!(outcome.action, StepAction::GotoLabel);
        assert_eq!(outcome.goto_label.as_deref(), Some("DT_LABEL"));
    }

    #[test]
    fn parameters_stored_correctly() {
        let mut step = StepDivingTackle::new();
        let coord = FieldCoordinate::new(3, 5);
        assert!(step.set_parameter(&StepParameter::GotoLabelOnSuccess("S".into())));
        assert!(step.set_parameter(&StepParameter::CoordinateFrom(coord)));
        assert!(step.set_parameter(&StepParameter::CoordinateTo(coord)));
        assert!(step.set_parameter(&StepParameter::DodgeRoll(3)));
        assert!(step.set_parameter(&StepParameter::UsingBreakTackle(true)));
        assert!(step.set_parameter(&StepParameter::UsingModifyingSkill(false)));
        assert!(step.set_parameter(&StepParameter::UsingDivingTackle(true)));
        assert_eq!(step.goto_label_on_success, "S");
        assert_eq!(step.dodge_roll, 3);
        assert!(step.using_break_tackle);
        assert_eq!(step.using_diving_tackle, Some(true));
    }
}
