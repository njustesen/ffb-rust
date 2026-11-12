//! Java `minimumInducementCost(Team)`, a private method that is identical in
//! `bb2020.start.StepBuyCardsAndInducements` and `bb2025.start.StepBuyInducements`:
//!
//! ```java
//! return Stream.concat(Stream.concat(
//!         roster STAR positions' cost (> 0),
//!         factory.allTypes() with getActualCostId(team) != null && name != "card"
//!             → UtilGameOption.getIntOption(game, type.getActualCostId(team))),
//!         roster MERCENARY positions' cost + INDUCEMENT_MERCENARIES_EXTRA_COST)
//!     .min().orElse(Integer.MAX_VALUE);
//! ```
//!
//! The buy step shows a team its dialog only when this minimum is affordable. Both editions'
//! `InducementCollection`s contain `briberyAndCorruption` priced by `INDUCEMENT_DUMMY` (and
//! bb2025 also `throwARock`), whose factory default is 0, so the minimum is 0 and every team that
//! reaches its phase is shown a dialog. That is why Java reports a purchase for BOTH teams in
//! every unequal-TV parity game, the overdog with 0 gold included (BACKLOG §I.9).
//!
//! Stars and mercenaries are not modelled here: a star counts only at a cost above 0 and a
//! mercenary costs at least INDUCEMENT_MERCENARIES_EXTRA_COST on top of its position, so neither can
//! go below the 0 that `INDUCEMENT_DUMMY` already contributes. The halfling Master Chef reduced
//! price (a roster keyword, not carried on the Rust `Team`) cannot go below it either.
use ffb_model::enums::Rules;
use ffb_model::model::game::Game;
use ffb_model::model::special_rule::SpecialRule;
use ffb_model::model::team::Team;
use ffb_model::option::game_option_id::*;
use ffb_model::option::util_game_option::get_int_option;

/// `getActualCostId(team)` of every inducement type in the edition's collection (the shared base
/// `InducementCollection` plus the edition subclass), skipping types with a null cost id and the
/// bb2020 `card` type exactly as the Java filter does.
fn actual_cost_ids(rules: Rules, team: &Team) -> Vec<&'static str> {
    let bribery = team.special_rules.iter()
        .any(|r| r == SpecialRule::BRIBERY_AND_CORRUPTION.get_rule_name());
    // Base collection: extraTeamTraining, wanderingApothecaries, wizard (stars/mercs: null cost).
    let mut ids = vec![INDUCEMENT_EXTRA_TRAINING_COST, INDUCEMENT_APOS_COST, INDUCEMENT_WIZARDS_COST];
    let bribes = if bribery { INDUCEMENT_BRIBES_REDUCED_COST } else { INDUCEMENT_BRIBES_COST };
    let biased_ref = if bribery { INDUCEMENT_BIASED_REF_REDUCED_COST } else { INDUCEMENT_BIASED_REF_COST };
    match rules {
        Rules::Bb2020 => ids.extend([
            INDUCEMENT_KEGS_COST, bribes, INDUCEMENT_PRAYERS_COST, INDUCEMENT_DUMMY,
            INDUCEMENT_CHEFS_COST, INDUCEMENT_MORTUARY_ASSISTANTS_COST, INDUCEMENT_PLAGUE_DOCTORS_COST,
            INDUCEMENT_RIOTOUS_ROOKIES_COST, INDUCEMENT_TEMP_CHEERLEADER_COST,
            INDUCEMENT_PART_TIME_COACH_COST, biased_ref, INDUCEMENT_WEATHER_MAGE_COST,
        ]),
        _ => ids.extend([
            INDUCEMENT_MASCOT_COST, INDUCEMENT_PRAYERS_COST, INDUCEMENT_PART_TIME_COACH_COST,
            INDUCEMENT_TEMP_CHEERLEADER_COST, INDUCEMENT_WEATHER_MAGE_COST, INDUCEMENT_KEGS_COST,
            bribes, INDUCEMENT_MORTUARY_ASSISTANTS_COST, INDUCEMENT_DUMMY, INDUCEMENT_PLAGUE_DOCTORS_COST,
            INDUCEMENT_RIOTOUS_ROOKIES_COST, INDUCEMENT_DUMMY, INDUCEMENT_CHEFS_COST, biased_ref,
        ]),
    }
    ids
}

/// Java `minimumInducementCost(Team)` (see module docs).
pub fn minimum_inducement_cost(game: &Game, team: &Team) -> i32 {
    actual_cost_ids(game.rules, team).into_iter()
        .map(|id| get_int_option(game, id))
        .min()
        .unwrap_or(i32::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;

    #[test]
    fn minimum_is_the_zero_cost_dummy_in_bb2020_and_bb2025() {
        for rules in [Rules::Bb2020, Rules::Bb2025] {
            let game = Game::new(test_team("home", 0), test_team("away", 0), rules);
            assert_eq!(minimum_inducement_cost(&game, &game.team_home), 0, "{rules:?}");
        }
    }

    #[test]
    fn an_explicit_dummy_price_raises_the_minimum_to_the_cheapest_real_inducement() {
        let mut game = Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025);
        game.options.set(INDUCEMENT_DUMMY, "1000000");
        // Cheapest bb2025 factory price: Part-time Assistant Coach / Temp Cheerleader at 20k.
        assert_eq!(minimum_inducement_cost(&game, &game.team_home), 20_000);
    }
}
