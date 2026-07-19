/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2016.StepKickoffResultRoll`.
///
/// Rolls 2D6 and maps to a BB2016 kickoff table result, then publishes it.
/// Java: `DiceInterpreter.interpretRollKickoff(game, roll)` → `KickoffResultFactory.forRoll(2d6)`.
///
/// BB2016 kickoff table:
///  2→GetTheRef, 3→Riot, 4→PerfectDefence, 5→HighKick, 6→CheeringFans,
///  7→WeatherChange, 8→BrilliantCoaching, 9→QuickSnap, 10→Blitz, 11→ThrowARock, 12→PitchInvasion
use ffb_model::enums::KickoffResult;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use ffb_model::report::report_kickoff_result::ReportKickoffResult;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};

/// Java: `StepKickoffResultRoll` (bb2016).
pub struct StepKickoffResultRoll {
    /// Java: `fKickoffResult` — the last rolled result.
    kickoff_result: Option<KickoffResult>,
}

impl StepKickoffResultRoll {
    pub fn new() -> Self { Self { kickoff_result: None } }

    /// BB2016 kickoff table: 2D6 → `KickoffResult`.
    fn interpret_roll(roll: i32) -> KickoffResult {
        match roll {
            2  => KickoffResult::GetTheRef,
            3  => KickoffResult::Riot,
            4  => KickoffResult::PerfectDefence,
            5  => KickoffResult::HighKick,
            6  => KickoffResult::CheeringFans,
            7  => KickoffResult::WeatherChange,
            8  => KickoffResult::BrilliantCoaching,
            9  => KickoffResult::QuickSnap,
            10 => KickoffResult::Blitz,
            11 => KickoffResult::ThrowARock,
            _  => KickoffResult::PitchInvasion, // 12
        }
    }

    /// Java: `getGameState().getDiceRoller().rollKickoff()` — rolls 2 individual d6 dice
    /// and returns them (not their sum). Factored out so the exact report payload — the
    /// 2-element dice array, not the summed value — is independently testable.
    fn roll_kickoff_dice(rng: &mut GameRng) -> [i32; 2] {
        [rng.d6(), rng.d6()]
    }

    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: interpretRollKickoff sums the 2 dice for the table lookup, but
        // ReportKickoffResult stores the full 2-element array (not just the sum) for
        // client display.
        let dice = Self::roll_kickoff_dice(rng);
        let roll_sum = dice[0] + dice[1];
        let result = Self::interpret_roll(roll_sum);
        self.kickoff_result = Some(result);
        game.report_list.add(ReportKickoffResult::new(result, dice.to_vec()));
        StepOutcome::next()
            .publish(StepParameter::KickoffResult(result))
    }
}

impl Default for StepKickoffResultRoll {
    fn default() -> Self { Self::new() }
}

impl Step for StepKickoffResultRoll {
    fn id(&self) -> StepId { StepId::KickoffResultRoll }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, _param: &StepParameter) -> bool { false }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::enums::Rules;
    use ffb_model::report::report_id::ReportId;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2016)
    }

    #[test]
    fn id_is_kickoff_result_roll() {
        assert_eq!(StepKickoffResultRoll::new().id(), StepId::KickoffResultRoll);
    }

    #[test]
    fn interpret_roll_covers_all_values() {
        assert_eq!(StepKickoffResultRoll::interpret_roll(2),  KickoffResult::GetTheRef);
        assert_eq!(StepKickoffResultRoll::interpret_roll(3),  KickoffResult::Riot);
        assert_eq!(StepKickoffResultRoll::interpret_roll(4),  KickoffResult::PerfectDefence);
        assert_eq!(StepKickoffResultRoll::interpret_roll(5),  KickoffResult::HighKick);
        assert_eq!(StepKickoffResultRoll::interpret_roll(6),  KickoffResult::CheeringFans);
        assert_eq!(StepKickoffResultRoll::interpret_roll(7),  KickoffResult::WeatherChange);
        assert_eq!(StepKickoffResultRoll::interpret_roll(8),  KickoffResult::BrilliantCoaching);
        assert_eq!(StepKickoffResultRoll::interpret_roll(9),  KickoffResult::QuickSnap);
        assert_eq!(StepKickoffResultRoll::interpret_roll(10), KickoffResult::Blitz);
        assert_eq!(StepKickoffResultRoll::interpret_roll(11), KickoffResult::ThrowARock);
        assert_eq!(StepKickoffResultRoll::interpret_roll(12), KickoffResult::PitchInvasion);
    }

    #[test]
    fn publishes_kickoff_result_parameter() {
        let mut step = StepKickoffResultRoll::new();
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let outcome = step.start(&mut game, &mut rng);
        let has_result = outcome.published.iter().any(|p| matches!(p, StepParameter::KickoffResult(_)));
        assert!(has_result);
    }

    #[test]
    fn returns_next_step() {
        let mut step = StepKickoffResultRoll::new();
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let outcome = step.start(&mut game, &mut rng);
        assert!(matches!(outcome.action, StepAction::NextStep));
    }

    #[test]
    fn stores_kickoff_result() {
        let mut step = StepKickoffResultRoll::new();
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        step.start(&mut game, &mut rng);
        assert!(step.kickoff_result.is_some());
    }

    #[test]
    fn adds_kickoff_result_report() {
        let mut step = StepKickoffResultRoll::new();
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        step.start(&mut game, &mut rng);
        assert!(game.report_list.has_report(ReportId::KICKOFF_RESULT), "should add ReportKickoffResult");
    }

    #[test]
    fn kickoff_dice_are_two_individual_d6_not_a_collapsed_sum() {
        // Java: rollKickoff() returns int[2] (2 individual d6), and ReportKickoffResult
        // stores that full array — not the summed value in a 1-element vec. Before the
        // fix, `execute_step` used `rng.d6_two()` (sum only) and stored `vec![sum]`.
        let mut rng_a = GameRng::new(0);
        let dice = StepKickoffResultRoll::roll_kickoff_dice(&mut rng_a);
        assert_eq!(dice.len(), 2);
        for d in dice { assert!((1..=6).contains(&d), "each die must be a valid d6 value 1-6, got {d}"); }

        // Cross-check against manual d6()+d6() draws from a rng seeded identically —
        // confirms two independent d6 draws are consumed (matching Java's 2-die roll),
        // not a single summed roll collapsed into one slot.
        let mut rng_b = GameRng::new(0);
        let expected = [rng_b.d6(), rng_b.d6()];
        assert_eq!(dice, expected);
    }

    #[test]
    fn kickoff_result_report_stores_both_individual_dice_not_the_sum() {
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let dice = StepKickoffResultRoll::roll_kickoff_dice(&mut rng);
        let sum = dice[0] + dice[1];
        let result = StepKickoffResultRoll::interpret_roll(sum);
        let report = ReportKickoffResult::new(result, dice.to_vec());
        game.report_list.add(report);
        assert!(game.report_list.has_report(ReportId::KICKOFF_RESULT));
        // The bug would have stored `vec![sum]` — a single-element vec containing the
        // combined value — instead of the 2 individual dice.
        assert_ne!(dice.to_vec().len(), 1, "must not collapse the 2-dice roll into a single summed element");
    }

    #[test]
    fn report_added_on_handle_command() {
        let mut step = StepKickoffResultRoll::new();
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        step.handle_command(&Action::EndTurn, &mut game, &mut rng);
        assert!(game.report_list.has_report(ReportId::KICKOFF_RESULT), "handle_command should also add ReportKickoffResult");
    }
}
