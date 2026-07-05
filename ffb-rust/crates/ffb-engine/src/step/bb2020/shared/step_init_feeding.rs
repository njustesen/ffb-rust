use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

/// Initialises a Nurgle Rotspawn feeding turn (BB2020).
/// Needs `GotoLabelOnEnd`. Handles forced vs. chosen feeding.
/// headless: feeding target selection not yet ported.
/// Mirrors Java `com.fumbbl.ffb.server.step.bb2020.shared.StepInitFeeding`.
pub struct StepInitFeeding {
    pub goto_label_on_end: String,
    pub feed_on_player_choice: Option<bool>,
    pub feeding_allowed: Option<bool>,
    pub end_player_action: bool,
    pub end_turn: bool,
}

impl StepInitFeeding {
    pub fn new(goto_label_on_end: String) -> Self {
        Self {
            goto_label_on_end,
            feed_on_player_choice: None,
            feeding_allowed: None,
            end_player_action: false,
            end_turn: false,
        }
    }
}

impl Step for StepInitFeeding {
    fn id(&self) -> StepId { StepId::InitFeeding }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnEnd(v) => { self.goto_label_on_end = v.clone(); true }
            StepParameter::FeedOnPlayerChoice(v) => { self.feed_on_player_choice = Some(*v); true }
            StepParameter::FeedingAllowed(v) => { self.feeding_allowed = Some(*v); true }
            StepParameter::EndPlayerAction(v) => { self.end_player_action = *v; true }
            StepParameter::EndTurn(v) => { self.end_turn = *v; true }
            _ => false,
        }
    }
}

impl StepInitFeeding {
    fn execute_step(&self, _game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // headless: set up feeding target selection sequence — infra not ported
        StepOutcome::cont()
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
        Game::new(home, away, Rules::Bb2020)
    }

    #[test]
    fn id_is_init_feeding() {
        assert_eq!(StepInitFeeding::new("x".into()).id(), StepId::InitFeeding);
    }

    #[test]
    fn start_returns_cont() {
        let mut game = make_game();
        let mut step = StepInitFeeding::new("end".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::Continue);
    }

    #[test]
    fn set_parameter_feeding_allowed_accepted() {
        let mut step = StepInitFeeding::new("end".into());
        assert!(step.set_parameter(&StepParameter::FeedingAllowed(true)));
        assert_eq!(step.feeding_allowed, Some(true));
    }

    #[test]
    fn set_parameter_end_turn_accepted() {
        let mut step = StepInitFeeding::new("end".into());
        assert!(step.set_parameter(&StepParameter::EndTurn(true)));
        assert!(step.end_turn);
    }

    #[test]
    fn set_parameter_feed_on_player_choice_accepted() {
        let mut step = StepInitFeeding::new("end".into());
        assert!(step.set_parameter(&StepParameter::FeedOnPlayerChoice(true)));
        assert_eq!(step.feed_on_player_choice, Some(true));
    }
}
