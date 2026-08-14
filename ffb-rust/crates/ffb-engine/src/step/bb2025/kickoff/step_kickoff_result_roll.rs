use ffb_model::enums::KickoffResult;
use ffb_model::events::GameEvent;
use ffb_model::model::game::Game;
use ffb_model::option::game_option_id;
use ffb_model::report::report_kickoff_result::ReportKickoffResult;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

/// Rolls 2d6 and maps the result to the BB2025 kickoff event table; publishes
/// `KickoffResult`.
///
/// BB2025 kickoff table (2d6 total):
///  2  → Get the Ref
///  3  → Time-out
///  4  → Solid Defence
///  5  → High Kick
///  6  → Cheering Fans
///  7  → Brilliant Coaching
///  8  → Weather Change
///  9  → Quick Snap
///  10 → Charge
///  11 → Dodgy Snack
///  12 → Pitch Invasion
///
/// Overtime options (GameOptionId::OVERTIME_KICK_OFF_RESULTS) implemented for all
/// non-dialog paths. client-only: blitzOrSolidDefence dialog path (Blitz/SolidDefence result) — dialog is client-side.
///
/// Mirrors Java `com.fumbbl.ffb.server.step.bb2025.kickoff.StepKickoffResultRoll`.
pub struct StepKickoffResultRoll {
    /// Java: fKickoffResult — None means "not yet rolled".
    pub kickoff_result: Option<KickoffResult>,
    /// Java: rollKickoff (int[]) — individual dice for ReportKickoffResult.
    kickoff_roll: Vec<i32>,
}

impl StepKickoffResultRoll {
    pub fn new() -> Self {
        Self { kickoff_result: None, kickoff_roll: Vec::new() }
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
        // Java CLIENT_KICK_OFF_RESULT_CHOICE: set fKickoffResult from the command.
        // client-only: Action::KickoffResultChoice arrives from dialog; headless never receives this
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, _param: &StepParameter) -> bool { false }
}

impl StepKickoffResultRoll {
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        if self.kickoff_result.is_none() {
            let overtime_option = game.options.get(game_option_id::OVERTIME_KICK_OFF_RESULTS).unwrap_or("all");
            if game.half < 3 || overtime_option == "all" {
                // Java: rollKickoff = getDiceRoller().rollKickoff() → two d6 values
                let d1 = rng.d6();
                let d2 = rng.d6();
                self.kickoff_roll = vec![d1, d2];
                self.kickoff_result = Some(kickoff_result_for_roll_in(game.rules, d1 + d2));
            } else if overtime_option == "randomBlitzOrSolidDefence" {
                let valid_rolls: [[i32; 2]; 6] = [[1, 3], [2, 2], [3, 1], [6, 4], [5, 5], [4, 6]];
                let index = (rng.d6() - 1) as usize;
                let pair = valid_rolls[index.min(5)];
                self.kickoff_roll = vec![pair[0], pair[1]];
                self.kickoff_result = Some(kickoff_result_for_roll_in(game.rules, pair[0] + pair[1]));
            } else if overtime_option == "blitz" {
                // Java: com.fumbbl.ffb.kickoff.bb2025.KickoffResult.CHARGE — BB2025 renamed the
                // "Blitz" kickoff result to "Charge"; the OVERTIME_KICK_OFF_BLITZ option name is
                // legacy but still maps to the BB2025 CHARGE result, NOT the (non-BB2025) Blitz variant.
                self.kickoff_result = Some(KickoffResult::Charge);
            } else if overtime_option == "solidDefence" {
                self.kickoff_result = Some(KickoffResult::SolidDefence);
            } else {
                // client-only: DialogKickOffResultChoice for blitzOrSolidDefence — headless auto-rolls
                let d1 = rng.d6();
                let d2 = rng.d6();
                self.kickoff_roll = vec![d1, d2];
                self.kickoff_result = Some(kickoff_result_for_roll_in(game.rules, d1 + d2));
            }
        }

        let result = self.kickoff_result.unwrap();
        // Java: getResult().addReport(new ReportKickoffResult(fKickoffResult, rollKickoff))
        game.report_list.add(ReportKickoffResult::new(result, self.kickoff_roll.clone()));
        StepOutcome::next()
            .with_event(GameEvent::KickoffResultEvent { result })
            .publish(StepParameter::KickoffResult(result))
    }
}

/// Kickoff event table mapping (2d6 → KickoffResult), per edition.
///
/// Java has one `KickoffResultMapping` per `@RulesCollection`; BB2020's differs from BB2025's on
/// exactly two rolls, everything else is identical:
///
/// | roll | `kickoff/bb2020/KickoffResultMapping` | `kickoff/bb2025/KickoffResultMapping` |
/// |------|---------------------------------------|---------------------------------------|
/// | 10   | `BLITZ`                               | `CHARGE`                              |
/// | 11   | `OFFICIOUS_REF`                       | `DODGY_SNACK`                         |
///
/// Both replacements happen to draw the same dice SHAPE as the BB2025 event they displaced
/// (`StepBlitzTurn` opens with a d3 just like Charge; Officious Ref's d6/d6/d11/d6 matches Dodgy
/// Snack), so using the BB2025 table for a BB2020 game kept the shared dice stream aligned and
/// diverged only the board — which is why it survived so long undetected.
fn kickoff_result_for_roll_in(rules: ffb_model::enums::Rules, roll: i32) -> KickoffResult {
    if rules == ffb_model::enums::Rules::Bb2020 {
        match roll {
            10 => return KickoffResult::Blitz,
            11 => return KickoffResult::OficiousRef,
            _ => {}
        }
    }
    match roll {
        2  => KickoffResult::GetTheRef,
        3  => KickoffResult::TimeOut,
        4  => KickoffResult::SolidDefence,
        5  => KickoffResult::HighKick,
        6  => KickoffResult::CheeringFans,
        7  => KickoffResult::BrilliantCoaching,
        8  => KickoffResult::WeatherChange,
        9  => KickoffResult::QuickSnap,
        10 => KickoffResult::Charge,
        11 => KickoffResult::DodgySnack,
        12 => KickoffResult::PitchInvasion,
        // Out-of-range (should never happen with 2d6):
        _ => KickoffResult::BrilliantCoaching,
    }
}

/// The BB2025 table — the historical entry point, kept for the existing tests.
#[cfg(test)]
fn kickoff_result_for_roll(roll: i32) -> KickoffResult {
    kickoff_result_for_roll_in(ffb_model::enums::Rules::Bb2025, roll)
}

#[cfg(test)]
mod tests {
    /// Java has one `KickoffResultMapping` per edition. BB2020's rolls 10/11 are BLITZ and
    /// OFFICIOUS_REF where BB2025's are CHARGE and DODGY_SNACK; every other roll is shared.
    /// Using the BB2025 table for a BB2020 game applied the wrong kickoff event outright, and
    /// because each replacement draws the same dice SHAPE as the event it displaced (StepBlitzTurn
    /// opens with a d3 like Charge; Officious Ref's d6/d6/d11/d6 matches Dodgy Snack) the dice
    /// stream stayed aligned and only the board diverged (bb2020 human seed 23 i=138).
    #[test]
    fn bb2020_and_bb2025_kickoff_tables_differ_only_on_rolls_10_and_11() {
        use ffb_model::enums::Rules;
        for roll in 2..=12 {
            let bb2020 = kickoff_result_for_roll_in(Rules::Bb2020, roll);
            let bb2025 = kickoff_result_for_roll_in(Rules::Bb2025, roll);
            match roll {
                10 => {
                    assert_eq!(bb2020, KickoffResult::Blitz);
                    assert_eq!(bb2025, KickoffResult::Charge);
                }
                11 => {
                    assert_eq!(bb2020, KickoffResult::OficiousRef);
                    assert_eq!(bb2025, KickoffResult::DodgySnack);
                }
                _ => assert_eq!(bb2020, bb2025, "roll {roll} must be identical across editions"),
            }
        }
        // BB2016 shares the BB2025 arm of this helper (it has its own step file).
        assert_eq!(kickoff_result_for_roll_in(Rules::Bb2016, 10), KickoffResult::Charge);
    }

    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::{StepAction, StepParameter};
    use ffb_model::enums::Rules;
    use ffb_model::util::rng::GameRng;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2025)
    }

    #[test]
    fn start_returns_next_step() {
        let mut game = make_game();
        let mut step = StepKickoffResultRoll::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn start_publishes_kickoff_result_parameter() {
        let mut game = make_game();
        let mut step = StepKickoffResultRoll::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::KickoffResult(_))));
    }

    #[test]
    fn kickoff_result_stored_after_roll() {
        let mut game = make_game();
        let mut step = StepKickoffResultRoll::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert!(step.kickoff_result.is_some());
    }

    #[test]
    fn pre_set_result_reused_without_re_roll() {
        let mut game = make_game();
        let mut step = StepKickoffResultRoll::new();
        step.kickoff_result = Some(KickoffResult::HighKick);
        let out = step.start(&mut game, &mut GameRng::new(0));
        // Should reuse the pre-set result, not overwrite it.
        assert_eq!(step.kickoff_result, Some(KickoffResult::HighKick));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::KickoffResult(KickoffResult::HighKick))));
    }

    /// Java: OVERTIME_KICK_OFF_BLITZ maps to `com.fumbbl.ffb.kickoff.bb2025.KickoffResult.CHARGE`,
    /// not a "Blitz" result (BB2025 renamed the "Blitz" kickoff result to "Charge").
    #[test]
    fn overtime_blitz_option_maps_to_charge_result() {
        use ffb_model::option::game_option_id;
        let mut game = make_game();
        game.half = 3; // half >= 3 so the "all" branch is skipped
        game.options.set(game_option_id::OVERTIME_KICK_OFF_RESULTS, "blitz");
        let mut step = StepKickoffResultRoll::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(step.kickoff_result, Some(KickoffResult::Charge));
    }

    #[test]
    fn kickoff_table_all_rolls() {
        let cases = [
            (2,  KickoffResult::GetTheRef),
            (3,  KickoffResult::TimeOut),
            (4,  KickoffResult::SolidDefence),
            (5,  KickoffResult::HighKick),
            (6,  KickoffResult::CheeringFans),
            (7,  KickoffResult::BrilliantCoaching),
            (8,  KickoffResult::WeatherChange),
            (9,  KickoffResult::QuickSnap),
            (10, KickoffResult::Charge),
            (11, KickoffResult::DodgySnack),
            (12, KickoffResult::PitchInvasion),
        ];
        for (roll, expected) in cases {
            assert_eq!(kickoff_result_for_roll(roll), expected, "roll={roll}");
        }
    }

    /// ReportKickoffResult is added to report_list after the roll.
    #[test]
    fn report_kickoff_result_added_to_report_list() {
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game();
        let mut step = StepKickoffResultRoll::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert!(
            game.report_list.has_report(ReportId::KICKOFF_RESULT),
            "expected KICKOFF_RESULT in report_list"
        );
    }

    /// The report contains the same kickoff_result that was stored in the step.
    #[test]
    fn report_kickoff_result_matches_rolled_result() {
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game();
        let mut step = StepKickoffResultRoll::new();
        step.start(&mut game, &mut GameRng::new(42));
        assert!(step.kickoff_result.is_some());
        assert!(game.report_list.has_report(ReportId::KICKOFF_RESULT));
        // kickoff_roll should contain exactly 2 dice values in [1,6].
        assert_eq!(step.kickoff_roll.len(), 2);
        for &die in &step.kickoff_roll {
            assert!((1..=6).contains(&die), "die value {die} out of range");
        }
    }
}
