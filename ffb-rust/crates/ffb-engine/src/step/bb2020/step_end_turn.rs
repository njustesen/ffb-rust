/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2020.StepEndTurn` (BB2020).
///
/// Ends the current player turn. Handles special-case turn modes (Blitz, KickoffReturn,
/// PassBlock, IllegalSubstitution, Swarming → publish EndTurn and return immediately),
/// then the full pipeline for normal turn-end (touchdowns, stallers, secret weapons,
/// knockouts, half/game-end, prayers, argue-the-call, bribes).
///
/// TODOs (most of this step requires untranslated utilities):
///  - checkTouchdown / UtilServerSteps not translated.
///  - handleStallers / markPlayedAndSecretWeapons not translated.
///  - KickoffSequence / EndGameSequence push not translated.
///  - checkEndOfHalf not translated.
///  - argueTheCall dialogs and bribes dialogs not translated.
///  - getFaintingCount / deactivateCardsAndPrayers not translated.
///  - PrayerHandlerFactory not translated.
///  - UtilServerCards.deactivateCard() not translated.
///  - StarOfTheShow / canGrantReRollAfterTouchdown detection not translated.
///
/// Mirrors Java `com.fumbbl.ffb.server.step.bb2020.StepEndTurn`.
use std::collections::HashSet;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use ffb_model::enums::TurnMode;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

pub struct StepEndTurn {
    pub touchdown: Option<bool>,
    pub bribes_choice_home: Option<bool>,
    pub bribes_choice_away: Option<bool>,
    pub argue_the_call_choice_home: Option<bool>,
    pub argue_the_call_choice_away: Option<bool>,
    pub use_star_of_the_show: Option<bool>,
    pub next_sequence_pushed: bool,
    pub remove_used_secret_weapons: bool,
    pub new_half: bool,
    pub end_game: bool,
    pub within_secret_weapon_handling: bool,
    pub turn_nr: i32,
    pub half: i32,
    pub player_ids_natural_ones: Vec<String>,
    pub player_ids_failed_bribes: HashSet<String>,
    pub player_ids_argued: HashSet<String>,
    pub touchdown_player_id: Option<String>,
}

impl StepEndTurn {
    pub fn new() -> Self {
        Self {
            touchdown: None,
            bribes_choice_home: None,
            bribes_choice_away: None,
            argue_the_call_choice_home: None,
            argue_the_call_choice_away: None,
            use_star_of_the_show: None,
            next_sequence_pushed: false,
            remove_used_secret_weapons: false,
            new_half: false,
            end_game: false,
            within_secret_weapon_handling: false,
            turn_nr: 0,
            half: 0,
            player_ids_natural_ones: Vec::new(),
            player_ids_failed_bribes: HashSet::new(),
            player_ids_argued: HashSet::new(),
            touchdown_player_id: None,
        }
    }
}

impl Default for StepEndTurn {
    fn default() -> Self { Self::new() }
}

impl Step for StepEndTurn {
    fn id(&self) -> StepId { StepId::EndTurn }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        match action {
            Action::ArgueTheCall { argue: _ } => {
                self.within_secret_weapon_handling = true;
            }
            Action::UseBribe { use_bribe: _ } => {
                self.within_secret_weapon_handling = true;
            }
            Action::UseSkill { skill_id: _, use_skill } => {
                // TODO(end_turn): canGrantReRollAfterTouchdown detection not translated.
                // For now, treat any UseSkill as StarOfTheShow toggle.
                self.use_star_of_the_show = Some(*use_skill);
            }
            Action::UseReRoll { use_reroll: _ } => {
                // Re-roll for argue-the-call: just fall through to execute_step
            }
            _ => {}
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::TouchdownPlayerId(v) => {
                self.touchdown_player_id = v.clone();
                true
            }
            StepParameter::EndGame(v) => { self.end_game = *v; true }
            StepParameter::NewHalf(v) => { self.new_half = *v; true }
            _ => false,
        }
    }
}

impl StepEndTurn {
    fn execute_step(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // TODO(end_turn): game.field_model.clear_multi_block_targets() not translated.
        // TODO(end_turn): pass_state.reset() not translated.

        // Java: if (turnNr == 0) { turnNr = game.getTurnData().getTurnNr(); half = game.getHalf(); }
        if self.turn_nr == 0 {
            self.turn_nr = game.turn_data().turn_nr;
            self.half = game.half;
        }

        // Java: if not within secret weapon handling AND turn mode is special mode:
        //       publish EndTurn=true and return NextStep.
        if !self.within_secret_weapon_handling {
            let mode = game.turn_mode;
            if matches!(
                mode,
                TurnMode::Blitz
                | TurnMode::KickoffReturn
                | TurnMode::PassBlock
                | TurnMode::IllegalSubstitution
                | TurnMode::Swarming
            ) {
                return StepOutcome::next().publish(StepParameter::EndTurn(true));
            }

            // TODO(end_turn): checkTouchdown not translated.
            // TODO(end_turn): handleStallers not translated.
            // TODO(end_turn): markPlayedAndSecretWeapons not translated.
            // TODO(end_turn): kickoff / endGame sequence push not translated.
            // TODO(end_turn): checkEndOfHalf not translated.
        }

        // TODO(end_turn): argueTheCall dialogs not translated.
        // TODO(end_turn): bribes dialogs not translated.
        // TODO(end_turn): argue_the_call_choice / bribes_choice negotiation not translated.
        // TODO(end_turn): getFaintingCount / deactivateCardsAndPrayers not translated.

        // Simplified: advance and publish params.
        StepOutcome::next()
            .publish(StepParameter::TouchdownPlayerId(self.touchdown_player_id.clone()))
            .publish(StepParameter::EndGame(self.end_game))
            .publish(StepParameter::NewHalf(self.new_half))
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
    fn start_returns_next_step() {
        let mut game = make_game();
        let mut step = StepEndTurn::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn blitz_mode_publishes_end_turn_and_returns_next() {
        let mut game = make_game();
        game.turn_mode = TurnMode::Blitz;
        let mut step = StepEndTurn::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndTurn(true))));
    }

    #[test]
    fn kickoff_return_mode_publishes_end_turn() {
        let mut game = make_game();
        game.turn_mode = TurnMode::KickoffReturn;
        let mut step = StepEndTurn::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndTurn(true))));
    }

    #[test]
    fn pass_block_mode_publishes_end_turn() {
        let mut game = make_game();
        game.turn_mode = TurnMode::PassBlock;
        let mut step = StepEndTurn::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndTurn(true))));
    }

    #[test]
    fn swarming_mode_publishes_end_turn() {
        let mut game = make_game();
        game.turn_mode = TurnMode::Swarming;
        let mut step = StepEndTurn::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndTurn(true))));
    }

    #[test]
    fn illegal_substitution_mode_publishes_end_turn() {
        let mut game = make_game();
        game.turn_mode = TurnMode::IllegalSubstitution;
        let mut step = StepEndTurn::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndTurn(true))));
    }

    #[test]
    fn regular_mode_publishes_touchdown_player_id() {
        let mut game = make_game();
        game.turn_mode = TurnMode::Regular;
        let mut step = StepEndTurn::new();
        step.touchdown_player_id = Some("scorer".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::TouchdownPlayerId(Some(id)) if id == "scorer")));
    }

    #[test]
    fn regular_mode_publishes_end_game() {
        let mut game = make_game();
        game.turn_mode = TurnMode::Regular;
        let mut step = StepEndTurn::new();
        step.end_game = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndGame(true))));
    }

    #[test]
    fn regular_mode_publishes_new_half() {
        let mut game = make_game();
        game.turn_mode = TurnMode::Regular;
        let mut step = StepEndTurn::new();
        step.new_half = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::NewHalf(true))));
    }

    #[test]
    fn set_parameter_touchdown_player_id_accepted() {
        let mut step = StepEndTurn::new();
        assert!(step.set_parameter(&StepParameter::TouchdownPlayerId(Some("p1".into()))));
        assert_eq!(step.touchdown_player_id, Some("p1".into()));
    }

    #[test]
    fn set_parameter_end_game_accepted() {
        let mut step = StepEndTurn::new();
        assert!(step.set_parameter(&StepParameter::EndGame(true)));
        assert!(step.end_game);
    }

    #[test]
    fn set_parameter_new_half_accepted() {
        let mut step = StepEndTurn::new();
        assert!(step.set_parameter(&StepParameter::NewHalf(true)));
        assert!(step.new_half);
    }

    #[test]
    fn turn_nr_initialized_on_first_run() {
        let mut game = make_game();
        game.turn_data_home.turn_nr = 3;
        game.turn_mode = TurnMode::Regular;
        let mut step = StepEndTurn::new();
        assert_eq!(step.turn_nr, 0);
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(step.turn_nr, 3);
    }

    #[test]
    fn within_secret_weapon_handling_set_by_argue_the_call() {
        let mut game = make_game();
        let mut step = StepEndTurn::new();
        step.handle_command(&Action::ArgueTheCall { argue: true }, &mut game, &mut GameRng::new(0));
        assert!(step.within_secret_weapon_handling);
    }

    #[test]
    fn within_secret_weapon_handling_set_by_use_bribe() {
        let mut game = make_game();
        let mut step = StepEndTurn::new();
        step.handle_command(&Action::UseBribe { use_bribe: false }, &mut game, &mut GameRng::new(0));
        assert!(step.within_secret_weapon_handling);
    }
}
