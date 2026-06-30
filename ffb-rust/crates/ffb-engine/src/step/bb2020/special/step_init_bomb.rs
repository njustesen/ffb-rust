use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

/// Initialises the bomb throw sequence (BB2020).
/// Mirrors Java `com.fumbbl.ffb.server.step.bb2020.special.StepInitBomb`.
pub struct StepInitBomb {
    goto_label_on_end: Option<String>,
    catcher_id: Option<String>,
    pass_fumble: bool,
    bomb_out_of_bounds: bool,
    dont_drop_fumble: bool,
}

impl StepInitBomb {
    pub fn new() -> Self {
        Self {
            goto_label_on_end: None,
            catcher_id: None,
            pass_fumble: false,
            bomb_out_of_bounds: false,
            dont_drop_fumble: false,
        }
    }

    fn execute_step(&mut self) -> StepOutcome {
        // Java: if fPassFumble → fCatcherId = null
        if self.pass_fumble {
            self.catcher_id = None;
        }
        // Java: if fBombOutOfBounds → fCatcherId = null
        if self.bomb_out_of_bounds {
            self.catcher_id = None;
        }

        // Java leaveStep: publish CatcherId always
        if self.catcher_id.is_some() {
            if let Some(ref label) = self.goto_label_on_end.clone() {
                // Java: leaveStep(fGotoLabelOnEnd) → GOTO_LABEL
                return StepOutcome::goto(label)
                    .publish(StepParameter::CatcherId(self.catcher_id.clone()));
            }
        }

        // Java: leaveStep(null) → NEXT_STEP (no catcher or no goto label)
        StepOutcome::next()
            .publish(StepParameter::CatcherId(self.catcher_id.clone()))
    }
}

impl Default for StepInitBomb {
    fn default() -> Self { Self::new() }
}

impl Step for StepInitBomb {
    fn id(&self) -> StepId { StepId::InitBomb }

    fn start(&mut self, _game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step()
    }

    fn handle_command(&mut self, _action: &Action, _game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        StepOutcome::next()
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::BombOutOfBounds(v) => {
                self.bomb_out_of_bounds = *v;
                true
            }
            StepParameter::CatcherId(v) => {
                self.catcher_id = v.clone();
                true
            }
            StepParameter::PassFumble(v) => {
                self.pass_fumble = *v;
                true
            }
            StepParameter::GotoLabelOnEnd(v) => {
                self.goto_label_on_end = Some(v.clone());
                true
            }
            StepParameter::DontDropFumble(v) => {
                self.dont_drop_fumble = *v;
                true
            }
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
    use ffb_model::model::game::Game;
    use ffb_model::util::rng::GameRng;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2020)
    }

    #[test]
    fn id_is_init_bomb() {
        let step = StepInitBomb::new();
        assert_eq!(step.id(), StepId::InitBomb);
    }

    #[test]
    fn no_catcher_returns_next_step() {
        let mut step = StepInitBomb::new();
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let outcome = step.start(&mut game, &mut rng);
        assert_eq!(outcome.action, StepAction::NextStep);
        // CatcherId(None) should be published
        assert!(outcome.published.iter().any(|p| matches!(p, StepParameter::CatcherId(None))));
    }

    #[test]
    fn pass_fumble_clears_catcher() {
        let mut step = StepInitBomb::new();
        // Set a catcher first
        assert!(step.set_parameter(&StepParameter::CatcherId(Some("player-1".to_string()))));
        // Set pass fumble
        assert!(step.set_parameter(&StepParameter::PassFumble(true)));

        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let outcome = step.start(&mut game, &mut rng);

        // Should return NextStep and publish CatcherId(None)
        assert_eq!(outcome.action, StepAction::NextStep);
        assert!(outcome.published.iter().any(|p| matches!(p, StepParameter::CatcherId(None))));
    }

    #[test]
    fn bomb_out_of_bounds_clears_catcher() {
        let mut step = StepInitBomb::new();
        assert!(step.set_parameter(&StepParameter::CatcherId(Some("player-2".to_string()))));
        assert!(step.set_parameter(&StepParameter::BombOutOfBounds(true)));

        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let outcome = step.start(&mut game, &mut rng);

        assert_eq!(outcome.action, StepAction::NextStep);
        assert!(outcome.published.iter().any(|p| matches!(p, StepParameter::CatcherId(None))));
    }

    #[test]
    fn catcher_with_goto_label_returns_goto() {
        let mut step = StepInitBomb::new();
        assert!(step.set_parameter(&StepParameter::CatcherId(Some("player-3".to_string()))));
        assert!(step.set_parameter(&StepParameter::GotoLabelOnEnd("label_catch".to_string())));

        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let outcome = step.start(&mut game, &mut rng);

        assert_eq!(outcome.action, StepAction::GotoLabel);
        assert_eq!(outcome.goto_label.as_deref(), Some("label_catch"));
        assert!(outcome.published.iter().any(|p| matches!(p, StepParameter::CatcherId(Some(_)))));
    }

    #[test]
    fn set_bomb_out_of_bounds_accepted() {
        let mut step = StepInitBomb::new();
        assert!(step.set_parameter(&StepParameter::BombOutOfBounds(true)));
    }

    #[test]
    fn set_parameter_unknown_returns_false() {
        let mut step = StepInitBomb::new();
        assert!(!step.set_parameter(&StepParameter::EndTurn(true)));
    }
}
