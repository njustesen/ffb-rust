use ffb_model::enums::ReRollSource;
use ffb_model::model::game::Game;
use ffb_model::report::bb2025::report_getting_even_roll::ReportGettingEvenRoll;
use ffb_model::report::report_id::ReportId;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::util_server_re_roll::{ask_for_reroll_if_available, use_reroll};

/// Java: ReRolledActions.GETTING_EVEN.getName() == "Getting Even"
const RE_ROLLED_ACTION_GETTING_EVEN: &str = "Getting Even";

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2025.shared.StepGettingEven.
/// Opposing team may use their apothecary after a casualty (Getting Even apothecary rule).
/// client-only: opposing apo dialog (DialogUseApothecaryParameter for opposing team) — headless auto-skips.
pub struct StepGettingEven {
    /// Java: playerId
    pub player_id: Option<String>,
    /// Java: keyword (Keyword enum) — stored as name until Keyword enum is ported
    pub keyword_name: Option<String>,
    // AbstractStepWithReRoll stubs
    pub re_rolled_action: Option<String>,
    pub re_roll_source: Option<String>,
}

impl StepGettingEven {
    pub fn new() -> Self {
        Self { player_id: None, keyword_name: None, re_rolled_action: None, re_roll_source: None }
    }
}

impl Default for StepGettingEven {
    fn default() -> Self { Self::new() }
}

impl Step for StepGettingEven {
    fn id(&self) -> StepId { StepId::GettingEven }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: AbstractStepWithReRoll.handleCommand consumes CLIENT_USE_RE_ROLL;
        // a "no" answer clears the re-roll source so doRoll becomes false.
        if let Action::UseReRoll { use_reroll: false } = action {
            self.re_roll_source = None;
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::PlayerId(v) => { self.player_id = Some(v.clone()); true }
            _ => false,
        }
    }
}

/// Java: MINIMUM_ROLL = 4
const MINIMUM_ROLL: i32 = 4;

impl StepGettingEven {
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let player_id = self.player_id.clone();

        // Java: boolean rerolled = ReRolledActions.GETTING_EVEN == getReRolledAction();
        let rerolled = self.re_rolled_action.as_deref() == Some(RE_ROLLED_ACTION_GETTING_EVEN);
        let mut do_roll = true;

        if rerolled {
            // Java: if (getReRollSource() == null || !UtilServerReRoll.useReRoll(this, getReRollSource(), player)) doRoll = false;
            let can_use = match (&self.re_roll_source, &player_id) {
                (Some(source_name), Some(pid)) => {
                    let source = ReRollSource::new(source_name.as_str());
                    use_reroll(game, &source, pid)
                }
                _ => false,
            };
            if !can_use {
                do_roll = false;
            }
        }

        if do_roll {
            // Java: roll = DiceRoller.rollSkill(); success = roll >= MINIMUM_ROLL
            let roll = rng.d6();
            let successful = roll >= MINIMUM_ROLL;
            let keyword = self.keyword_name.clone().unwrap_or_default();
            game.report_list.add(ReportGettingEvenRoll::new(
                player_id.clone(), successful, roll, MINIMUM_ROLL, rerolled, keyword,
            ));

            if successful {
                // Java: game.addHatred(player, keyword);
                // client/model-only: FieldModel.addHatred only notifies observers (client sync)
                // and updates PlayerResult stats — no gameplay mechanic reads it, so there is
                // no engine-side effect to replicate here.
            } else if self.re_rolled_action.is_none() {
                // Java: if (getReRolledAction() == null && UtilServerReRoll.askForReRollIfAvailable(...)) { CONTINUE; return; }
                if let Some(prompt) = ask_for_reroll_if_available(game, RE_ROLLED_ACTION_GETTING_EVEN, MINIMUM_ROLL, false) {
                    self.re_rolled_action = Some(RE_ROLLED_ACTION_GETTING_EVEN.into());
                    self.re_roll_source = Some("TRR".into());
                    return StepOutcome::cont().with_prompt(prompt);
                }
            }
        }

        // client-only: offer opposing apothecary dialog — client-side
        StepOutcome::next()
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
        Game::new(home, away, Rules::Bb2025)
    }

    #[test]
    fn start_returns_next() {
        let mut game = make_game();
        let mut step = StepGettingEven::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn set_parameter_player_id_accepted() {
        let mut step = StepGettingEven::new();
        assert!(step.set_parameter(&StepParameter::PlayerId("p1".into())));
        assert_eq!(step.player_id.as_deref(), Some("p1"));
    }

    #[test]
    fn handle_command_returns_next() {
        let mut game = make_game();
        let mut step = StepGettingEven::new();
        let out = step.handle_command(&Action::EndTurn, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn start_adds_getting_even_roll_report() {
        let mut game = make_game();
        let mut step = StepGettingEven::new();
        step.player_id = Some("p1".into());
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::GETTING_EVEN_ROLL));
    }

    #[test]
    fn keyword_is_included_in_report() {
        let mut game = make_game();
        let mut step = StepGettingEven::new();
        step.player_id = Some("p1".into());
        step.keyword_name = Some("Agility".into());
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::GETTING_EVEN_ROLL));
    }

    fn seed_for_d6(target: i32) -> u64 {
        for s in 0u64..10_000 {
            if GameRng::new(s).d6() == target { return s; }
        }
        panic!("no seed for d6={}", target);
    }

    /// Java: on a failed roll (roll < MINIMUM_ROLL), StepGettingEven.executeStep offers a
    /// re-roll via UtilServerReRoll.askForReRollIfAvailable and sets nextAction=CONTINUE
    /// when one is available, instead of unconditionally proceeding to NEXT_STEP.
    #[test]
    fn failed_roll_with_trr_available_offers_reroll_instead_of_next_step() {
        let seed = seed_for_d6(1); // roll=1 < MINIMUM_ROLL(4) -> failure
        let mut game = make_game();
        game.home_playing = true;
        game.turn_data_home.rerolls = 1;
        let mut step = StepGettingEven::new();
        step.player_id = Some("p1".into());
        let out = step.start(&mut game, &mut GameRng::new(seed));
        assert_eq!(out.action, StepAction::Continue);
        assert!(out.prompt.is_some());
        // TRR not yet consumed while waiting for the player's answer.
        assert_eq!(game.turn_data_home.rerolls, 1);
    }

    /// Declining the offered re-roll (Action::UseReRoll { use_reroll: false }) must clear the
    /// re-roll source so the step falls through to NEXT_STEP without rolling again.
    #[test]
    fn declining_reroll_after_offer_finishes_step() {
        let seed = seed_for_d6(1);
        let mut game = make_game();
        game.home_playing = true;
        game.turn_data_home.rerolls = 1;
        let mut step = StepGettingEven::new();
        step.player_id = Some("p1".into());
        let out = step.start(&mut game, &mut GameRng::new(seed));
        assert_eq!(out.action, StepAction::Continue);

        let out2 = step.handle_command(
            &Action::UseReRoll { use_reroll: false },
            &mut game,
            &mut GameRng::new(seed),
        );
        assert_eq!(out2.action, StepAction::NextStep);
        // TRR must remain unspent since the coach declined.
        assert_eq!(game.turn_data_home.rerolls, 1);
    }

    /// Accepting the re-roll consumes the TRR and re-rolls, ultimately finishing the step.
    #[test]
    fn accepting_reroll_consumes_trr_and_completes() {
        let seed = seed_for_d6(1);
        let mut game = make_game();
        game.home_playing = true;
        game.turn_data_home.rerolls = 1;
        let mut step = StepGettingEven::new();
        step.player_id = Some("p1".into());
        let out = step.start(&mut game, &mut GameRng::new(seed));
        assert_eq!(out.action, StepAction::Continue);

        let out2 = step.handle_command(
            &Action::UseReRoll { use_reroll: true },
            &mut game,
            &mut GameRng::new(seed),
        );
        assert_eq!(out2.action, StepAction::NextStep);
        assert_eq!(game.turn_data_home.rerolls, 0);
        assert!(game.turn_data_home.reroll_used);
    }

    /// Without an available TRR, a failed roll must fall straight through to NEXT_STEP
    /// (matching Java when askForReRollIfAvailable returns false).
    #[test]
    fn failed_roll_without_trr_returns_next_step() {
        let seed = seed_for_d6(1);
        let mut game = make_game();
        let mut step = StepGettingEven::new();
        step.player_id = Some("p1".into());
        let out = step.start(&mut game, &mut GameRng::new(seed));
        assert_eq!(out.action, StepAction::NextStep);
    }
}
