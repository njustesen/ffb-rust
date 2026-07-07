/// 1:1 translation of `com.fumbbl.ffb.server.step.mixed.end.StepPenaltyShootout`.
///
/// Step in the end-game sequence to handle the penalty shootout.
/// If the game is past half 2 and scores are tied, runs a best-of-5 shootout
/// until one team wins (each round: rollPenaltyShootout for each team; winner
/// of the round gets +1 penalty score; highest after 5 rounds wins).
///
/// Java: `SHOOTOUT_LIMIT = 5`. `rollPenaltyShootout()` = `rollDice(6)`.
/// Java serialisation fields: `homeConfirmed`, `awayConfirmed` — not needed in
/// headless Rust (no client dialogs), kept as struct fields for completeness.
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use ffb_model::report::mixed::report_penalty_shootout::ReportPenaltyShootout;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};

/// Java: `StepPenaltyShootout` (mixed/end, BB2020 + BB2025).
pub struct StepPenaltyShootout {
    /// Java: homeConfirmed
    pub home_confirmed: bool,
    /// Java: awayConfirmed
    pub away_confirmed: bool,
}

impl StepPenaltyShootout {
    pub fn new() -> Self {
        Self { home_confirmed: false, away_confirmed: false }
    }

    /// Java: `toOrdinal(int number)`
    fn to_ordinal(n: i32) -> String {
        match n {
            1 => "1st".into(),
            2 => "2nd".into(),
            3 => "3rd".into(),
            _ => format!("{}th", n),
        }
    }

    fn execute_step(&self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let home_score = game.game_result.home.score;
        let away_score = game.game_result.away.score;

        if game.half > 2 && home_score == away_score {
            const SHOOTOUT_LIMIT: i32 = 5;
            let mut penalty_score_home = 0i32;
            let mut penalty_score_away = 0i32;

            while penalty_score_home + penalty_score_away < SHOOTOUT_LIMIT {
                // Java: int currentPenalty = penaltyScoreAway + penaltyScoreHome + 1 (1-indexed round)
                let current_penalty = penalty_score_home + penalty_score_away + 1;

                // Java: DiceRoller.rollPenaltyShootout() = rollDice(6) = D6
                let roll_home = rng.d6();
                let roll_away = rng.d6();

                let home_team_won_penalty: Option<bool>;
                if roll_away > roll_home {
                    home_team_won_penalty = Some(false);
                    penalty_score_away += 1;
                } else if roll_away < roll_home {
                    home_team_won_penalty = Some(true);
                    penalty_score_home += 1;
                } else {
                    home_team_won_penalty = None; // tied roll → nobody scores this round
                }

                // Java: if (penaltyScoreHome + penaltyScoreAway == SHOOTOUT_LIMIT) { set penalty scores + teamId }
                let winning_team = if penalty_score_home + penalty_score_away == SHOOTOUT_LIMIT {
                    game.game_result.home.penalty_score = penalty_score_home;
                    game.game_result.away.penalty_score = penalty_score_away;
                    if penalty_score_home > penalty_score_away {
                        Some(game.team_home.id.clone())
                    } else {
                        Some(game.team_away.id.clone())
                    }
                } else {
                    None
                };

                let round_str = Self::to_ordinal(current_penalty);

                // Java: getResult().addReport(new ReportPenaltyShootout(rollHome, penaltyScoreHome, rollAway, penaltyScoreAway, homeTeamWonPenalty, round, teamId))
                game.report_list.add(ReportPenaltyShootout::new(
                    roll_home,
                    penalty_score_home,
                    roll_away,
                    penalty_score_away,
                    home_team_won_penalty,
                    Some(round_str),
                    winning_team,
                ));
            }

            // Java: after the loop sets CONTINUE (dialog); we skip the dialog
            // and fall through to NEXT_STEP since Rust has no client confirmation.
        }

        StepOutcome::next()
    }
}

impl Default for StepPenaltyShootout {
    fn default() -> Self { Self::new() }
}

impl Step for StepPenaltyShootout {
    fn id(&self) -> StepId { StepId::PenaltyShootout }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    /// Java: CLIENT_CONFIRM sets homeConfirmed/awayConfirmed.
    /// When both confirmed → NEXT_STEP. In headless Rust we skip this and just proceed.
    fn handle_command(&mut self, _action: &Action, _game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        StepOutcome::next()
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
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    #[test]
    fn penalty_shootout_reports_added() {
        let mut step = StepPenaltyShootout::new();
        let mut game = make_game();
        game.half = 3;
        game.game_result.home.score = 1;
        game.game_result.away.score = 1;
        step.start(&mut game, &mut GameRng::new(42));
        assert!(
            game.report_list.has_report(ReportId::PENALTY_SHOOTOUT),
            "should add at least one ReportPenaltyShootout during shootout"
        );
    }

    #[test]
    fn no_penalty_shootout_reports_when_no_overtime() {
        let mut step = StepPenaltyShootout::new();
        let mut game = make_game();
        game.half = 2; // not past half 2
        game.game_result.home.score = 1;
        game.game_result.away.score = 1;
        step.start(&mut game, &mut GameRng::new(0));
        assert!(
            !game.report_list.has_report(ReportId::PENALTY_SHOOTOUT),
            "should not add ReportPenaltyShootout when not in overtime"
        );
    }

    #[test]
    fn id_is_penalty_shootout() {
        assert_eq!(StepPenaltyShootout::new().id(), StepId::PenaltyShootout);
    }

    #[test]
    fn no_overtime_no_change() {
        let mut step = StepPenaltyShootout::new();
        let mut game = make_game();
        game.half = 2;
        game.game_result.home.score = 1;
        game.game_result.away.score = 1;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(matches!(out.action, StepAction::NextStep));
        assert_eq!(game.game_result.home.penalty_score, 0);
        assert_eq!(game.game_result.away.penalty_score, 0);
    }

    #[test]
    fn no_tie_no_shootout() {
        let mut step = StepPenaltyShootout::new();
        let mut game = make_game();
        game.half = 3;
        game.game_result.home.score = 2;
        game.game_result.away.score = 1;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(matches!(out.action, StepAction::NextStep));
        assert_eq!(game.game_result.home.penalty_score, 0);
        assert_eq!(game.game_result.away.penalty_score, 0);
    }

    #[test]
    fn overtime_tie_resolves_shootout_total_5() {
        let mut step = StepPenaltyShootout::new();
        let mut game = make_game();
        game.half = 3;
        game.game_result.home.score = 1;
        game.game_result.away.score = 1;
        let out = step.start(&mut game, &mut GameRng::new(42));
        assert!(matches!(out.action, StepAction::NextStep));
        let total = game.game_result.home.penalty_score + game.game_result.away.penalty_score;
        assert_eq!(total, 5);
    }

    #[test]
    fn one_team_wins_penalty_shootout() {
        let mut step = StepPenaltyShootout::new();
        let mut game = make_game();
        game.half = 3;
        game.game_result.home.score = 0;
        game.game_result.away.score = 0;
        step.start(&mut game, &mut GameRng::new(0));
        let total = game.game_result.home.penalty_score + game.game_result.away.penalty_score;
        assert_eq!(total, 5);
        assert_ne!(game.game_result.home.penalty_score, game.game_result.away.penalty_score);
    }

    #[test]
    fn default_constructs() {
        let s = StepPenaltyShootout::default();
        assert!(!s.home_confirmed);
        assert!(!s.away_confirmed);
    }

    #[test]
    fn set_parameter_always_false() {
        let mut step = StepPenaltyShootout::new();
        assert!(!step.set_parameter(&StepParameter::EndPlayerAction(true)));
    }
}
