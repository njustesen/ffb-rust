/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2016.end.StepWinnings`.
///
/// Step in end game sequence to roll winnings (BB2016).
/// - Rolls D6 for both teams; adds fame and +1 if tied/winning.
/// - Admin mode: if the winning team's roll is under 3, the whole roll (both teams) is redone
///   automatically (Java: `StepWinnings.executeStep()` admin-mode reroll block).
/// - Winner may re-roll via dialog (client-only, headless skips).
/// - If team conceded illegally: transfer their winnings to the opposing team.
/// - Emits ReportWinningsRoll twice (initial + concede adjustment).
///
/// client-only: DialogWinningsReRollParameter — winner re-roll choice requires dialog; headless skips.
/// client-only: WINNINGS re-roll (AbstractStepWithReRoll) — coach triggers re-roll via dialog; headless skips.
/// WinningsRoll GameEvent wired.
use ffb_model::events::GameEvent;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use ffb_model::report::bb2016::report_winnings_roll::ReportWinningsRoll;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};

/// Java: `StepWinnings` (bb2016/end).
pub struct StepWinnings;

impl StepWinnings {
    pub fn new() -> Self { Self }

    /// Java: `rollWinnings()` — rolls D6 for both teams and applies base winnings.
    fn roll_winnings(game: &mut Game, rng: &mut GameRng) -> (i32, i32) {
        let score_diff_home = game.game_result.home.score - game.game_result.away.score;
        // Roll home
        let roll_home = rng.d6();
        let winnings_home = roll_home + game.game_result.home.fame + if score_diff_home >= 0 { 1 } else { 0 };
        game.game_result.home.winnings = winnings_home * 10_000;
        // Roll away
        let roll_away = rng.d6();
        let winnings_away = roll_away + game.game_result.away.fame + if score_diff_home <= 0 { 1 } else { 0 };
        game.game_result.away.winnings = winnings_away * 10_000;
        (roll_home, roll_away)
    }

    /// Java: `concedeWinnings()` — transfer winnings on illegal concede.
    fn concede_winnings(game: &mut Game) {
        if game.game_result.home.conceded && !game.conceded_legally {
            game.game_result.away.winnings += game.game_result.home.winnings;
            game.game_result.home.winnings = 0;
        }
        if game.game_result.away.conceded && !game.conceded_legally {
            game.game_result.home.winnings += game.game_result.away.winnings;
            game.game_result.away.winnings = 0;
        }
    }

    /// Java: `if (game.isAdminMode()) { if (((scoreDiffHome > 0) && (rollHome < 3))
    ///           || ((scoreDiffHome < 0) && (rollAway < 3))) { reportWinnings = rollWinnings(); } }`
    /// True when the winning team rolled less than 3 in admin mode, triggering an automatic re-roll.
    fn admin_mode_should_reroll(admin_mode: bool, score_diff_home: i32, roll_home: i32, roll_away: i32) -> bool {
        admin_mode
            && ((score_diff_home > 0 && roll_home < 3) || (score_diff_home < 0 && roll_away < 3))
    }

    fn execute_step(game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let (mut roll_home, mut roll_away) = Self::roll_winnings(game, rng);
        // Java: if (game.isAdminMode()) { roll winnings, reroll on a 1 or 2 --> ... <-- }
        let score_diff_home = game.game_result.home.score - game.game_result.away.score;
        if Self::admin_mode_should_reroll(game.admin_mode, score_diff_home, roll_home, roll_away) {
            let (rh2, ra2) = Self::roll_winnings(game, rng);
            roll_home = rh2;
            roll_away = ra2;
        }
        // Capture winnings after roll, before concession transfer
        let home_id = game.team_home.id.clone();
        let away_id = game.team_away.id.clone();
        let winnings_home_initial = game.game_result.home.winnings;
        let winnings_away_initial = game.game_result.away.winnings;
        let fame_home = game.game_result.home.fame;
        let fame_away = game.game_result.away.fame;
        // Java: getResult().addReport(reportWinnings) — initial roll report
        game.report_list.add(ReportWinningsRoll::new(roll_home, winnings_home_initial, roll_away, winnings_away_initial));
        // client-only: DialogWinningsReRollParameter shown to winning coach — headless skips re-roll
        Self::concede_winnings(game);
        // Java: getResult().addReport(concedeWinnings()) — concede transfer report (if any)
        let concede_home = game.game_result.home.winnings;
        let concede_away = game.game_result.away.winnings;
        if concede_home != winnings_home_initial || concede_away != winnings_away_initial {
            game.report_list.add(ReportWinningsRoll::new(0, concede_home, 0, concede_away));
        }
        StepOutcome::next()
            .with_event(GameEvent::WinningsRoll {
                team_id: home_id.clone(),
                base: fame_home,
                roll: roll_home,
                total: winnings_home_initial,
            })
            .with_event(GameEvent::WinningsRoll {
                team_id: away_id.clone(),
                base: fame_away,
                roll: roll_away,
                total: winnings_away_initial,
            })
    }
}

impl Default for StepWinnings {
    fn default() -> Self { Self::new() }
}

impl Step for StepWinnings {
    fn id(&self) -> StepId { StepId::Winnings }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        Self::execute_step(game, rng)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        Self::execute_step(game, rng)
    }

    fn set_parameter(&mut self, _param: &StepParameter) -> bool { false }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::enums::Rules;

    fn make_game(home_score: i32, away_score: i32) -> Game {
        let mut game = Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2016);
        game.game_result.home.score = home_score;
        game.game_result.away.score = away_score;
        game
    }

    #[test]
    fn id_is_winnings() {
        assert_eq!(StepWinnings::new().id(), StepId::Winnings);
    }

    #[test]
    fn start_returns_next_step() {
        let mut game = make_game(0, 0);
        let out = StepWinnings::new().start(&mut game, &mut GameRng::new(0));
        assert!(matches!(out.action, StepAction::NextStep));
    }

    #[test]
    fn winnings_set_after_roll() {
        let mut game = make_game(0, 0);
        StepWinnings::new().start(&mut game, &mut GameRng::new(0));
        assert!(game.game_result.home.winnings > 0);
        assert!(game.game_result.away.winnings > 0);
    }

    #[test]
    fn home_illegal_concede_transfers_winnings() {
        let mut game = make_game(3, 1);
        game.game_result.home.conceded = true;
        game.conceded_legally = false;
        StepWinnings::new().start(&mut game, &mut GameRng::new(42));
        assert_eq!(game.game_result.home.winnings, 0);
        assert!(game.game_result.away.winnings > 0);
    }

    #[test]
    fn away_illegal_concede_transfers_winnings() {
        let mut game = make_game(1, 3);
        game.game_result.away.conceded = true;
        game.conceded_legally = false;
        StepWinnings::new().start(&mut game, &mut GameRng::new(42));
        assert_eq!(game.game_result.away.winnings, 0);
        assert!(game.game_result.home.winnings > 0);
    }

    #[test]
    fn winner_gets_extra_gold() {
        // home wins: score_diff_home > 0 → home_winnings += 1 extra
        let mut game = make_game(3, 0);
        game.game_result.home.fame = 0;
        game.game_result.away.fame = 0;
        StepWinnings::new().start(&mut game, &mut GameRng::new(1));
        // Minimum roll on D6 = 1; home gets 1+1 = 20000 minimum.
        assert!(game.game_result.home.winnings >= 20_000);
    }

    #[test]
    fn start_adds_winnings_roll_report() {
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game(0, 0);
        StepWinnings::new().start(&mut game, &mut GameRng::new(0));
        assert!(
            game.report_list.has_report(ReportId::WINNINGS_ROLL),
            "StepWinnings should add ReportWinningsRoll"
        );
    }

    #[test]
    fn admin_mode_should_reroll_matches_java_condition() {
        // Java: if (((scoreDiffHome > 0) && (rollHome < 3)) || ((scoreDiffHome < 0) && (rollAway < 3)))
        // Before the fix, StepWinnings never consulted `game.admin_mode` at all, so this
        // condition (and the corresponding re-roll) was never wired up.
        assert!(StepWinnings::admin_mode_should_reroll(true, 5, 1, 6)); // home winning, rolled low
        assert!(StepWinnings::admin_mode_should_reroll(true, -5, 6, 2)); // away winning, rolled low
        assert!(!StepWinnings::admin_mode_should_reroll(false, 5, 1, 6)); // not admin mode
        assert!(!StepWinnings::admin_mode_should_reroll(true, 5, 3, 6)); // home rolled 3+, no reroll
        assert!(!StepWinnings::admin_mode_should_reroll(true, 0, 1, 1)); // tie: no "winning" team
    }

    #[test]
    fn admin_mode_low_roll_gets_rerolled() {
        // Find a seed whose first D6 roll for the winning (home) team is < 3, then verify that
        // with admin_mode=true the final winnings reflect the *third* d6 call (i.e. a full
        // second `roll_winnings()` pass, consuming 2 extra RNG rolls), not the first.
        let mut seed_and_first_roll = None;
        for seed in 0u64..200 {
            let mut probe_rng = GameRng::new(seed);
            let first_roll = probe_rng.d6();
            if first_roll < 3 {
                seed_and_first_roll = Some((seed, first_roll));
                break;
            }
        }
        let (seed, first_roll) = seed_and_first_roll.expect("expected a seed with first roll < 3 within 200 tries");

        // Predict the rerolled home value: roll_winnings() consumes d6() for home then away;
        // a second full call consumes 2 more — the 3rd overall d6() call is the rerolled home roll.
        let mut predict_rng = GameRng::new(seed);
        let _first_home = predict_rng.d6();
        let _first_away = predict_rng.d6();
        let rerolled_home = predict_rng.d6();

        let mut game_admin = make_game(3, 0); // home winning
        game_admin.game_result.home.fame = 0;
        game_admin.game_result.away.fame = 0;
        game_admin.admin_mode = true;
        StepWinnings::new().start(&mut game_admin, &mut GameRng::new(seed));

        // fame=0, home winning (score_diff_home>0) → +1 bonus.
        assert_eq!(
            game_admin.game_result.home.winnings,
            (rerolled_home + 1) * 10_000,
            "admin mode should use the re-rolled home value ({}), not the original low roll ({})",
            rerolled_home, first_roll
        );
    }

    #[test]
    fn illegal_concede_adds_second_winnings_roll_report() {
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game(3, 1);
        game.game_result.home.conceded = true;
        game.conceded_legally = false;
        StepWinnings::new().start(&mut game, &mut GameRng::new(42));
        // Should have at least 2 WINNINGS_ROLL reports (initial + concede transfer)
        let count = game.report_list.get_reports().iter()
            .filter(|r| r.get_id() == ReportId::WINNINGS_ROLL)
            .count();
        assert!(count >= 2, "illegal concede should add a second ReportWinningsRoll, got {}", count);
    }
}
