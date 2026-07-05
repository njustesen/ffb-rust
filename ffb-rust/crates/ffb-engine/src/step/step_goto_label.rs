use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

/// 1:1 translation of com.fumbbl.ffb.server.step.StepGotoLabel.
///
/// Jumps to a given label. Init params: GOTO_LABEL (mandatory), ALTERNATE_GOTO_LABEL (optional).
/// Accepts USE_ALTERNATE_LABEL step param to switch to the alternate label.
pub struct StepGotoLabel {
    /// Java: fGotoLabel — mandatory init param.
    goto_label: String,
    /// Java: alternateLabel — optional init param.
    alternate_label: Option<String>,
    /// Java: useAlternateLabel — set via USE_ALTERNATE_LABEL step param.
    use_alternate_label: bool,
}

impl StepGotoLabel {
    pub fn new() -> Self {
        Self {
            goto_label: String::new(),
            alternate_label: None,
            use_alternate_label: false,
        }
    }
}

impl Default for StepGotoLabel {
    fn default() -> Self { Self::new() }
}

impl Step for StepGotoLabel {
    fn id(&self) -> StepId { StepId::GotoLabel }

    fn start(&mut self, _game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        let label = if self.use_alternate_label {
            self.alternate_label.as_deref().unwrap_or(&self.goto_label)
        } else {
            &self.goto_label
        };
        StepOutcome::goto(label)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.start(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabel(v)          => { self.goto_label = v.clone(); true }
            StepParameter::AlternateGotoLabel(v) => { self.alternate_label = Some(v.clone()); true }
            StepParameter::UseAlternateLabel(v)  => { self.use_alternate_label = *v; true }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::enums::Rules;
    use ffb_model::util::rng::GameRng;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2016)
    }

    #[test]
    fn id_is_goto_label() {
        assert_eq!(StepGotoLabel::new().id(), StepId::GotoLabel);
    }

    #[test]
    fn goto_label_param_accepted() {
        let mut step = StepGotoLabel::new();
        assert!(step.set_parameter(&StepParameter::GotoLabel("end".into())));
        assert_eq!(step.goto_label, "end");
    }

    #[test]
    fn alternate_goto_label_param_accepted() {
        let mut step = StepGotoLabel::new();
        assert!(step.set_parameter(&StepParameter::AlternateGotoLabel("alt".into())));
        assert_eq!(step.alternate_label.as_deref(), Some("alt"));
    }

    #[test]
    fn use_alternate_label_param_accepted() {
        let mut step = StepGotoLabel::new();
        assert!(step.set_parameter(&StepParameter::UseAlternateLabel(true)));
        assert!(step.use_alternate_label);
    }

    #[test]
    fn start_goes_to_goto_label() {
        let mut step = StepGotoLabel::new();
        step.goto_label = "end".into();
        let out = step.start(&mut make_game(), &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("end"));
    }

    #[test]
    fn start_uses_alternate_when_flag_set() {
        let mut step = StepGotoLabel::new();
        step.goto_label = "end".into();
        step.alternate_label = Some("alt".into());
        step.use_alternate_label = true;
        let out = step.start(&mut make_game(), &mut GameRng::new(0));
        assert_eq!(out.goto_label.as_deref(), Some("alt"));
    }

    #[test]
    fn start_uses_primary_when_alternate_flag_false() {
        let mut step = StepGotoLabel::new();
        step.goto_label = "end".into();
        step.alternate_label = Some("alt".into());
        step.use_alternate_label = false;
        let out = step.start(&mut make_game(), &mut GameRng::new(0));
        assert_eq!(out.goto_label.as_deref(), Some("end"));
    }

    #[test]
    fn unrecognised_parameter_returns_false() {
        let mut step = StepGotoLabel::new();
        assert!(!step.set_parameter(&StepParameter::EndTurn(true)));
    }
}
