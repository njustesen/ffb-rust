/// 1:1 translation of `com.fumbbl.ffb.server.step.mixed.end.StepDedicatedFans`.
///
/// Calculates dedicated-fan modifiers at the end of a game based on result and concession.
/// Java: uses `score + penaltyScore` for tie-breaking; `penaltyScore` is now in `TeamResult`.
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};

/// Java: `StepDedicatedFans` (mixed/end, BB2020 + BB2025).
pub struct StepDedicatedFans;

impl StepDedicatedFans {
    pub fn new() -> Self { Self }

    /// Java: `modifier(roll, dedicatedFans, winning, conceded)`
    fn modifier(roll: i32, dedicated_fans: i32, winning: bool, conceded: bool) -> i32 {
        if conceded {
            -i32::max(i32::min(roll, dedicated_fans - 1), 0)
        } else if winning {
            if roll >= dedicated_fans { 1 } else { 0 }
        } else {
            if roll >= dedicated_fans { 0 } else { -1 }
        }
    }

    fn execute_step(&self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let home_conceded = game.game_result.home.conceded && !game.conceded_legally;
        let away_conceded = game.game_result.away.conceded && !game.conceded_legally;

        let home_die: u32 = if home_conceded { 3 } else { 6 };
        let away_die: u32 = if away_conceded { 3 } else { 6 };

        // Determine conceded team and winning team
        let home_id = game.team_home.id.clone();
        let away_id = game.team_away.id.clone();

        let conceded_id: Option<String>;
        // winning_home: None = draw, Some(true) = home wins, Some(false) = away wins
        let winning_home: Option<bool>;

        if home_conceded {
            conceded_id = Some(home_id.clone());
            winning_home = Some(false); // away wins
        } else if away_conceded {
            conceded_id = Some(away_id.clone());
            winning_home = Some(true); // home wins
        } else {
            conceded_id = None;
            // Java: score + penaltyScore
            let home_total = game.game_result.home.score + game.game_result.home.penalty_score;
            let away_total = game.game_result.away.score + game.game_result.away.penalty_score;
            if home_total > away_total {
                winning_home = Some(true);
            } else if away_total > home_total {
                winning_home = Some(false);
            } else {
                winning_home = None; // draw
            }
        }

        if let Some(home_wins) = winning_home {
            let roll_home = rng.die(home_die);
            let roll_away = rng.die(away_die);

            let modifier_home = Self::modifier(
                roll_home,
                game.team_home.dedicated_fans,
                home_wins,
                conceded_id.as_deref() == Some(&home_id),
            );
            let modifier_away = Self::modifier(
                roll_away,
                game.team_away.dedicated_fans,
                !home_wins,
                conceded_id.as_deref() == Some(&away_id),
            );

            game.game_result.home.dedicated_fans_modifier = modifier_home;
            game.game_result.away.dedicated_fans_modifier = modifier_away;
        }
        // If draw (winningTeam == null): Java adds empty ReportDedicatedFans, no modifiers set

        StepOutcome::next()
    }
}

impl Default for StepDedicatedFans {
    fn default() -> Self { Self::new() }
}

impl Step for StepDedicatedFans {
    fn id(&self) -> StepId { StepId::DedicatedFans }

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
    use crate::step::framework::test_team;
    use ffb_model::enums::Rules;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    #[test]
    fn id_is_dedicated_fans() {
        assert_eq!(StepDedicatedFans::new().id(), StepId::DedicatedFans);
    }

    #[test]
    fn draw_sets_no_modifier() {
        let mut step = StepDedicatedFans::new();
        let mut game = make_game();
        // Both teams 0-0, no concession → draw
        let mut rng = GameRng::new(0);
        step.start(&mut game, &mut rng);
        assert_eq!(game.game_result.home.dedicated_fans_modifier, 0);
        assert_eq!(game.game_result.away.dedicated_fans_modifier, 0);
    }

    #[test]
    fn winning_team_gets_positive_or_zero_modifier() {
        let mut step = StepDedicatedFans::new();
        let mut game = make_game();
        game.game_result.home.score = 2;
        game.game_result.away.score = 0;
        game.team_home.dedicated_fans = 3;
        game.team_away.dedicated_fans = 3;
        let mut rng = GameRng::new(42);
        step.start(&mut game, &mut rng);
        assert!(game.game_result.home.dedicated_fans_modifier >= 0);
    }

    #[test]
    fn conceding_team_gets_negative_modifier() {
        let mut step = StepDedicatedFans::new();
        let mut game = make_game();
        game.game_result.home.conceded = true;
        game.team_home.dedicated_fans = 3;
        game.team_away.dedicated_fans = 3;
        let mut rng = GameRng::new(0);
        step.start(&mut game, &mut rng);
        assert!(game.game_result.home.dedicated_fans_modifier <= 0);
    }

    #[test]
    fn modifier_concede_caps_at_dedicated_fans_minus_one() {
        // modifier(5, 3, _, conceded=true) → -min(min(5,2),0) → -2
        let m = StepDedicatedFans::modifier(5, 3, false, true);
        assert_eq!(m, -2);
    }

    #[test]
    fn modifier_winning_roll_at_or_above_gives_plus_one() {
        let m = StepDedicatedFans::modifier(4, 4, true, false);
        assert_eq!(m, 1);
        let m2 = StepDedicatedFans::modifier(3, 4, true, false);
        assert_eq!(m2, 0);
    }
}
