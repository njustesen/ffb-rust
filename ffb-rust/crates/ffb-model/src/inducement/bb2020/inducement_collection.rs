use crate::inducement::inducement_collection::{InducementCollection as BaseCollection, InducementTypeEntry};
use crate::inducement::usage::Usage;

/// 1:1 translation of `com.fumbbl.ffb.inducement.bb2020.InducementCollection`.
///
/// BB2020-specific inducement types (extends base collection).
pub struct InducementCollection {
    base: BaseCollection,
    sub_types: Vec<InducementTypeEntry>,
}

impl Default for InducementCollection {
    fn default() -> Self {
        Self::new()
    }
}

impl InducementCollection {
    pub fn new() -> Self {
        use crate::inducement::inducement_collection::InducementTypeRule as Rule;
        use crate::model::special_rule::SpecialRule;
        use crate::option::game_option_id::*;
        // `with_ids(max, cost, reducedCost, reducedMax)` as each Java constructor call fills them
        // (bb2020's discounted types use the 10-argument constructor: reducedMax = max).
        let sub_types = vec![
            InducementTypeEntry::new("bloodweiserBabes", vec![Usage::KNOCKOUT_RECOVERY])
                .with_ids(Some(INDUCEMENT_KEGS_MAX), Some(INDUCEMENT_KEGS_COST),
                    Some(INDUCEMENT_KEGS_COST), Some(INDUCEMENT_KEGS_MAX)),
            InducementTypeEntry::new("card", vec![]),
            InducementTypeEntry::new("bribes", vec![Usage::AVOID_BAN])
                .with_ids(Some(INDUCEMENT_BRIBES_MAX), Some(INDUCEMENT_BRIBES_COST),
                    Some(INDUCEMENT_BRIBES_REDUCED_COST), Some(INDUCEMENT_BRIBES_MAX))
                .with_rule(Rule::ReducedForBriberyAndCorruption),
            InducementTypeEntry::new("prayers", vec![Usage::GAME_MODIFICATION])
                .with_ids(Some(INDUCEMENT_PRAYERS_MAX), Some(INDUCEMENT_PRAYERS_COST),
                    Some(INDUCEMENT_PRAYERS_COST), Some(INDUCEMENT_PRAYERS_MAX)),
            InducementTypeEntry::new("briberyAndCorruption", vec![Usage::REROLL_ARGUE])
                .with_ids(Some(INDUCEMENT_DUMMY), Some(INDUCEMENT_DUMMY), Some(INDUCEMENT_DUMMY), Some(INDUCEMENT_DUMMY)),
            InducementTypeEntry::new("halflingMasterChef", vec![Usage::STEAL_REROLL])
                .with_ids(Some(INDUCEMENT_CHEFS_MAX), Some(INDUCEMENT_CHEFS_COST),
                    Some(INDUCEMENT_CHEFS_REDUCED_COST), Some(INDUCEMENT_CHEFS_MAX))
                .with_rule(Rule::ReducedForMasterChefKeyword),
            InducementTypeEntry::new("mortuaryAssistant", vec![Usage::REGENERATION])
                .with_ids(Some(INDUCEMENT_MORTUARY_ASSISTANTS_MAX), Some(INDUCEMENT_MORTUARY_ASSISTANTS_COST),
                    Some(INDUCEMENT_MORTUARY_ASSISTANTS_COST), Some(INDUCEMENT_MORTUARY_ASSISTANTS_MAX))
                .with_rule(Rule::RequiresSpecialRule(SpecialRule::SYLVANIAN_SPOTLIGHT.get_rule_name())),
            InducementTypeEntry::new("plagueDoctor", vec![Usage::REGENERATION, Usage::APOTHECARY_JOURNEYMEN])
                .with_ids(Some(INDUCEMENT_PLAGUE_DOCTORS_MAX), Some(INDUCEMENT_PLAGUE_DOCTORS_COST),
                    Some(INDUCEMENT_PLAGUE_DOCTORS_COST), Some(INDUCEMENT_PLAGUE_DOCTORS_MAX))
                .with_rule(Rule::RequiresSpecialRule(SpecialRule::FAVOURED_OF_NURGLE.get_rule_name())),
            InducementTypeEntry::new("riotousRookies", vec![Usage::ADD_LINEMEN])
                .with_ids(Some(INDUCEMENT_RIOTOUS_ROOKIES_MAX), Some(INDUCEMENT_RIOTOUS_ROOKIES_COST),
                    Some(INDUCEMENT_RIOTOUS_ROOKIES_COST), Some(INDUCEMENT_RIOTOUS_ROOKIES_MAX))
                .with_rule(Rule::RequiresSpecialRule(SpecialRule::LOW_COST_LINEMEN.get_rule_name())),
            InducementTypeEntry::new("tempCheerleader", vec![Usage::ADD_CHEERLEADER])
                .with_ids(Some(INDUCEMENT_TEMP_CHEERLEADER_MAX), Some(INDUCEMENT_TEMP_CHEERLEADER_COST),
                    Some(INDUCEMENT_TEMP_CHEERLEADER_COST), Some(INDUCEMENT_TEMP_CHEERLEADER_MAX))
                .with_rule(Rule::TempCheerleaderTotal),
            InducementTypeEntry::new("partTimeCoach", vec![Usage::ADD_COACH])
                .with_ids(Some(INDUCEMENT_PART_TIME_COACH_MAX), Some(INDUCEMENT_PART_TIME_COACH_COST),
                    Some(INDUCEMENT_PART_TIME_COACH_COST), Some(INDUCEMENT_PART_TIME_COACH_MAX))
                .with_rule(Rule::PartTimeCoachTotal),
            InducementTypeEntry::new("biasedRef", vec![Usage::ADD_TO_ARGUE_ROLL, Usage::SPOT_FOUL])
                .with_ids(Some(INDUCEMENT_BIASED_REF_MAX), Some(INDUCEMENT_BIASED_REF_COST),
                    Some(INDUCEMENT_BIASED_REF_REDUCED_COST), Some(INDUCEMENT_BIASED_REF_MAX))
                .with_rule(Rule::ReducedForBriberyAndCorruption),
            InducementTypeEntry::new("weatherMage", vec![Usage::CHANGE_WEATHER])
                .with_ids(Some(INDUCEMENT_WEATHER_MAGE_MAX), Some(INDUCEMENT_WEATHER_MAGE_COST),
                    None, Some(INDUCEMENT_WEATHER_MAGE_MAX)),
            InducementTypeEntry::new("infamousStaff", vec![Usage::STAFF])
                .with_ids(Some(INDUCEMENT_STAFF_MAX), None, None, Some(INDUCEMENT_STAFF_MAX)),
            InducementTypeEntry::new("bugmansXXXXXX", vec![Usage::REROLL_ONES_ON_KOS])
                .with_ids(Some(INDUCEMENT_STAFF_MAX), None, None, Some(INDUCEMENT_STAFF_MAX)),
        ];
        Self { base: BaseCollection::new(), sub_types }
    }

    pub fn get_all_types(&self) -> Vec<&InducementTypeEntry> {
        self.base.get_types().iter().chain(self.sub_types.iter()).collect()
    }

    pub fn get_key(&self) -> &str {
        "InducementCollection"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bb2020_collection_has_prayers() {
        let c = InducementCollection::new();
        assert!(c.get_all_types().iter().any(|t| t.id == "prayers"));
    }

    #[test]
    fn has_biased_ref() {
        let c = InducementCollection::new();
        assert!(c.get_all_types().iter().any(|t| t.id == "biasedRef"));
    }

    #[test]
    fn total_types_count_is_at_least_15() {
        let c = InducementCollection::new();
        assert!(c.get_all_types().len() >= 15);
    }
}
