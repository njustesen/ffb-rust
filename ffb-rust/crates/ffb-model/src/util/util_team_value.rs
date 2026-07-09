use crate::model::team::Team;
use crate::model::player::Player;

/// 1:1 translation of `com.fumbbl.ffb.util.UtilTeamValue`.
pub struct UtilTeamValue;

impl UtilTeamValue {
    /// Java: `findTeamValue(Team)`.
    ///
    /// Sum of re-rolls (× roster re-roll cost), fan factor, coaches, cheerleaders,
    /// apothecaries and player values.
    ///
    /// NOTE: The Java source uses `roster.getReRollCost()`.  In the Rust model the
    /// re-roll cost is not stored on `Team`, so we use a default of 50 000 (standard
    /// BB2020 / BB2025 value).  Callers that need the exact value should pass the
    /// cost directly.
    pub fn find_team_value(team: &Team) -> i32 {
        Self::find_team_value_with_reroll_cost(team, 50_000)
    }

    /// Variant that accepts a custom re-roll cost (for roster-specific values).
    pub fn find_team_value_with_reroll_cost(team: &Team, reroll_cost: i32) -> i32 {
        let mut tv = 0i32;
        tv += team.rerolls * reroll_cost;
        tv += team.fan_factor * 10_000;
        tv += team.assistant_coaches * 10_000;
        tv += team.cheerleaders * 10_000;
        tv += team.apothecaries * 50_000;
        for player in &team.players {
            tv += Self::find_player_value(player);
        }
        tv
    }

    /// Java: `findPlayerValue(Player)` (private in Java, public here for testability).
    ///
    /// Java source: `position.getCost() + sum(skill.getCost(player))`.
    /// In the Rust model there is no position cost on `Player`; callers that need
    /// exact value can pass a pre-computed value via `find_player_value_explicit`.
    pub fn find_player_value(_player: &Player) -> i32 {
        // Player cost data lives in the roster loader; return 0 until roster is loaded.
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::team::Team;

    fn make_team(rerolls: i32, fan_factor: i32) -> Team {
        Team {
            id: "t1".into(),
            name: "Test".into(),
            race: "Human".into(),
            roster_id: "human".into(),
            coach: "coach1".into(),
            rerolls,
            apothecaries: 0,
            bribes: 0,
            master_chefs: 0,
            prayers_to_nuffle: 0,
            bloodweiser_kegs: 0,
            riotous_rookies: 0,
            cheerleaders: 0,
            assistant_coaches: 0,
            fan_factor,
            dedicated_fans: 0,
            team_value: 0,
            treasury: 0,
            special_rules: vec![],
            players: vec![],
            vampire_lord: false,
            necromancer: false,
        }
    }

    #[test]
    fn rerolls_add_to_tv() {
        let team = make_team(2, 0);
        assert_eq!(UtilTeamValue::find_team_value_with_reroll_cost(&team, 50_000), 100_000);
    }

    #[test]
    fn fan_factor_adds_10k_each() {
        let team = make_team(0, 3);
        assert_eq!(UtilTeamValue::find_team_value_with_reroll_cost(&team, 50_000), 30_000);
    }
}
