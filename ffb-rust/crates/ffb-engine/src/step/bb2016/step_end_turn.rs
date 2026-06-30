/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2016.StepEndTurn` (BB2016).
///
/// Ends the current player turn. Handles special-case turn modes (Blitz, KickoffReturn,
/// PassBlock, IllegalSubstitution, Swarming → publish EndTurn and return immediately),
/// then the full pipeline for normal turn-end (touchdowns, secret weapons, knockouts,
/// half/game-end, argue-the-call, bribes).
///
/// BB2016 differs from BB2020 in that there is no `Swarming`, `StarOfTheShow`, or
/// `useStarOfTheShow` concept; and `fWithinSecretWeaponHandling` is the gate for
/// secret-weapon dialogs (identical to BB2020 in that regard).
///
/// TODOs (most of this step requires untranslated utilities):
///  - checkTouchdown / UtilServerSteps not translated.
///  - markPlayedAndSecretWeapons not translated.
///  - KickoffSequence / EndGameSequence push not translated.
///  - checkEndOfHalf not translated.
///  - argueTheCall dialogs and bribes dialogs not translated.
///  - UtilServerCards.deactivateCard() not translated.
///  - recoverKnockout / heatExhaust not translated.
///  - UtilServerTimer not translated.
///  - FUMBBL request update not translated.
///
/// Mirrors Java `com.fumbbl.ffb.server.step.bb2016.StepEndTurn`.
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use ffb_model::enums::TurnMode;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

pub struct StepEndTurn {
    /// Java: fTouchdown
    pub touchdown: Option<bool>,
    /// Java: fBribesChoiceHome
    pub bribes_choice_home: Option<bool>,
    /// Java: fBribesChoiceAway
    pub bribes_choice_away: Option<bool>,
    /// Java: fArgueTheCallChoiceHome
    pub argue_the_call_choice_home: Option<bool>,
    /// Java: fArgueTheCallChoiceAway
    pub argue_the_call_choice_away: Option<bool>,
    /// Java: fNextSequencePushed
    pub next_sequence_pushed: bool,
    /// Java: fRemoveUsedSecretWeapons
    pub remove_used_secret_weapons: bool,
    /// Java: fNewHalf
    pub new_half: bool,
    /// Java: fEndGame
    pub end_game: bool,
    /// Java: fWithinSecretWeaponHandling
    pub within_secret_weapon_handling: bool,
    /// Java: turnNr
    pub turn_nr: i32,
    /// Java: half
    pub half: i32,
    /// Java: touchdown player resolved (downstream consumption)
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
            next_sequence_pushed: false,
            remove_used_secret_weapons: false,
            new_half: false,
            end_game: false,
            within_secret_weapon_handling: false,
            turn_nr: 0,
            half: 0,
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
                // Java: CLIENT_ARGUE_THE_CALL → argueTheCall(team, playerIds); fWithinSecretWeaponHandling = true
                self.within_secret_weapon_handling = true;
            }
            Action::UseBribe { use_bribe: _ } => {
                // Java: CLIENT_USE_INDUCEMENT (AVOID_BAN) → useSecretWeaponBribes; fWithinSecretWeaponHandling = true
                self.within_secret_weapon_handling = true;
            }
            _ => {}
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::TouchdownPlayerId(v) => { self.touchdown_player_id = v.clone(); true }
            StepParameter::EndGame(v) => { self.end_game = *v; true }
            StepParameter::NewHalf(v) => { self.new_half = *v; true }
            _ => false,
        }
    }
}

impl StepEndTurn {
    fn execute_step(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
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

            // TODO(end_turn_bb2016): checkTouchdown not translated.
            // TODO(end_turn_bb2016): markPlayedAndSecretWeapons not translated.
            // TODO(end_turn_bb2016): KickoffSequence / EndGameSequence push not translated.
            // TODO(end_turn_bb2016): checkEndOfHalf not translated.
            // TODO(end_turn_bb2016): recoverKnockout / heatExhaust not translated.
            // TODO(end_turn_bb2016): deactivateCards not translated.
        }

        // TODO(end_turn_bb2016): argueTheCall dialogs not translated.
        // TODO(end_turn_bb2016): bribes dialogs not translated.
        // TODO(end_turn_bb2016): argue_the_call_choice / bribes_choice negotiation not translated.

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
    use ffb_model::util::rng::GameRng;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2016)
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
    fn within_secret_weapon_handling_skips_early_return() {
        let mut game = make_game();
        // Even with Blitz mode, if within_secret_weapon_handling is true, skip the early return
        game.turn_mode = TurnMode::Blitz;
        let mut step = StepEndTurn::new();
        step.within_secret_weapon_handling = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        // Should NOT publish EndTurn since within_secret_weapon_handling gate is active
        assert!(!out.published.iter().any(|p| matches!(p, StepParameter::EndTurn(true))));
    }
}
