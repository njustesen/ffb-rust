/// 1:1 translation of `com.fumbbl.ffb.server.step.game.start.UtilInducementSequence`.
///
/// Utility class with a single static method that calculates how much inducement
/// gold a team has available, taking into account TV differences and petty cash
/// already used.
use ffb_model::model::game::Game;

/// Java: `UtilInducementSequence` (static utility class — no instance state).
pub struct UtilInducementSequence;

impl UtilInducementSequence {
    /// Java: `calculateInducementGold(Game, boolean)`.
    ///
    /// Returns the amount of inducement gold available to the home team (`home == true`)
    /// or the away team (`home == false`).
    ///
    /// Logic (1:1 Java):
    /// 1. Start with each team's `petty_cash_transferred` from their `TeamResult`.
    /// 2. If away TV > home TV and the difference exceeds the home team's petty cash,
    ///    set home petty cash to that difference.
    /// 3. Symmetrically for the away team.
    /// 4. Subtract `petty_cash_used` from each (floor at 0).
    /// 5. Return the relevant team's value.
    pub fn calculate_inducement_gold(game: Option<&Game>, home: bool) -> i32 {
        let game = match game {
            Some(g) => g,
            None => return 0,
        };
        let result = &game.game_result;
        let mut inducement_gold_home = result.home.petty_cash_transferred;
        let mut inducement_gold_away = result.away.petty_cash_transferred;
        let home_tv = result.home.team_value;
        let away_tv = result.away.team_value;
        if (away_tv > home_tv) && ((away_tv - home_tv) > inducement_gold_home) {
            inducement_gold_home = away_tv - home_tv;
        }
        if (home_tv > away_tv) && ((home_tv - away_tv) > inducement_gold_away) {
            inducement_gold_away = home_tv - away_tv;
        }
        inducement_gold_home = (inducement_gold_home - result.home.petty_cash_used).max(0);
        inducement_gold_away = (inducement_gold_away - result.away.petty_cash_used).max(0);
        if home { inducement_gold_home } else { inducement_gold_away }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use ffb_model::model::team::Team;
    use ffb_model::model::game::Game;

    fn make_team(id: &str) -> Team {
        Team {
            id: id.into(), name: id.into(), race: "human".into(),
            roster_id: "human".into(), coach: "c".into(),
            rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0,
            cheerleaders: 0, assistant_coaches: 0, fan_factor: 0,
            dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: vec![], players: vec![],
        }
    }

    fn make_game() -> Game {
        Game::new(make_team("home"), make_team("away"), Rules::Bb2025)
    }

    #[test]
    fn none_game_returns_zero() {
        assert_eq!(UtilInducementSequence::calculate_inducement_gold(None, true), 0);
        assert_eq!(UtilInducementSequence::calculate_inducement_gold(None, false), 0);
    }

    #[test]
    fn equal_tv_no_petty_cash_returns_zero() {
        let game = make_game();
        // Both teams have TV=0, no petty cash — both get 0.
        assert_eq!(UtilInducementSequence::calculate_inducement_gold(Some(&game), true), 0);
        assert_eq!(UtilInducementSequence::calculate_inducement_gold(Some(&game), false), 0);
    }

    #[test]
    fn tv_difference_fills_underdog_gold() {
        let mut game = make_game();
        // Away team has 100k more TV → home should get 100k inducement gold.
        game.game_result.home.team_value = 1_000_000;
        game.game_result.away.team_value = 1_100_000;
        assert_eq!(UtilInducementSequence::calculate_inducement_gold(Some(&game), true), 100_000);
        assert_eq!(UtilInducementSequence::calculate_inducement_gold(Some(&game), false), 0);
    }

    #[test]
    fn petty_cash_transferred_used_as_baseline() {
        let mut game = make_game();
        // Both teams equal TV but home transferred 50k petty cash manually.
        game.game_result.home.team_value = 1_000_000;
        game.game_result.away.team_value = 1_000_000;
        game.game_result.home.petty_cash_transferred = 50_000;
        assert_eq!(UtilInducementSequence::calculate_inducement_gold(Some(&game), true), 50_000);
        assert_eq!(UtilInducementSequence::calculate_inducement_gold(Some(&game), false), 0);
    }

    #[test]
    fn petty_cash_used_is_subtracted() {
        let mut game = make_game();
        // Home has 100k TV advantage (away is underdog), away also used 30k already.
        game.game_result.home.team_value = 1_100_000;
        game.game_result.away.team_value = 1_000_000;
        game.game_result.away.petty_cash_used = 30_000;
        // Away's gold: max(0, 100_000 − 30_000) = 70_000
        assert_eq!(UtilInducementSequence::calculate_inducement_gold(Some(&game), false), 70_000);
    }

    #[test]
    fn petty_cash_used_floors_at_zero() {
        let mut game = make_game();
        // Away transferred 10k but used 50k → should floor at 0.
        game.game_result.home.team_value = 1_000_000;
        game.game_result.away.team_value = 1_000_000;
        game.game_result.away.petty_cash_transferred = 10_000;
        game.game_result.away.petty_cash_used = 50_000;
        assert_eq!(UtilInducementSequence::calculate_inducement_gold(Some(&game), false), 0);
    }

    #[test]
    fn tv_diff_uses_larger_of_tv_diff_or_petty_cash() {
        let mut game = make_game();
        // Away TV 200k above home, but home already has 50k petty_cash_transferred.
        // TV diff (200k) > existing petty_cash (50k) → use TV diff.
        game.game_result.home.team_value = 1_000_000;
        game.game_result.away.team_value = 1_200_000;
        game.game_result.home.petty_cash_transferred = 50_000;
        assert_eq!(UtilInducementSequence::calculate_inducement_gold(Some(&game), true), 200_000);
    }
}
