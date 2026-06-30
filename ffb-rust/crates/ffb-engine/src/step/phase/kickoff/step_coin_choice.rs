/// 1:1 translation of com.fumbbl.ffb.server.step.phase.kickoff.StepCoinChoice.
///
/// Waits for CLIENT_COIN_CHOICE (Action::CoinChoice), then flips the coin.
/// Publishes StepParameter::ChoosingTeamId for all subsequent steps on the stack.
use ffb_model::model::game::Game;
use ffb_model::prompts::AgentPrompt;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepId, StepOutcome, StepParameter};

pub struct StepCoinChoice {
    /// Java: fCoinChoiceHeads — home coach's guess; None until CLIENT_COIN_CHOICE received.
    coin_choice_heads: Option<bool>,
}

impl StepCoinChoice {
    pub fn new() -> Self {
        Self { coin_choice_heads: None }
    }
}

impl Default for StepCoinChoice {
    fn default() -> Self { Self::new() }
}

impl Step for StepCoinChoice {
    fn id(&self) -> StepId { StepId::CoinChoice }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        if let Action::CoinChoice { heads } = action {
            self.coin_choice_heads = Some(*heads);
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, _param: &StepParameter) -> bool { false }
}

impl StepCoinChoice {
    fn execute_step(&self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        if self.coin_choice_heads.is_none() {
            // Java: UtilServerDialog.showDialog(getGameState(), new DialogCoinChoiceParameter(), false)
            return StepOutcome::cont().with_prompt(AgentPrompt::CoinChoice { is_home: game.home_playing });
        }
        let coin_choice_heads = self.coin_choice_heads.unwrap();
        // Java: getGameState().getDiceRoller().throwCoin() → rollDice(2) == 1 → rng.bool()
        let coin_throw_heads = rng.bool();
        // Java: the coach who is "home playing" makes the call
        // home wins the toss iff their guess matched the coin (when home is playing),
        // or their guess did NOT match the coin (when away is playing).
        let choosing_team_id = if (game.home_playing && (coin_throw_heads != coin_choice_heads))
            || (!game.home_playing && (coin_throw_heads == coin_choice_heads))
        {
            game.team_away.id.clone()
        } else {
            game.team_home.id.clone()
        };
        StepOutcome::next().publish(StepParameter::ChoosingTeamId(Some(choosing_team_id)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::enums::Rules;
    use ffb_model::util::rng::GameRng;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    #[test]
    fn start_without_choice_returns_cont() {
        let mut game = make_game();
        let mut step = StepCoinChoice::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::Continue);
    }

    #[test]
    fn coin_choice_command_triggers_execute() {
        let mut game = make_game();
        let mut step = StepCoinChoice::new();
        let out = step.handle_command(&Action::CoinChoice { heads: true }, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn publishes_choosing_team_id() {
        let mut game = make_game();
        let mut step = StepCoinChoice::new();
        let out = step.handle_command(&Action::CoinChoice { heads: true }, &mut game, &mut GameRng::new(0));
        let has_param = out.published.iter().any(|p| matches!(p, StepParameter::ChoosingTeamId(_)));
        assert!(has_param, "should publish ChoosingTeamId");
    }

    #[test]
    fn set_parameter_returns_false() {
        let mut step = StepCoinChoice::new();
        assert!(!step.set_parameter(&StepParameter::EndTurn(false)));
    }

    #[test]
    fn non_coin_choice_action_still_returns_cont() {
        let mut game = make_game();
        let mut step = StepCoinChoice::new();
        let out = step.handle_command(&Action::Acknowledge, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::Continue);
    }
}
