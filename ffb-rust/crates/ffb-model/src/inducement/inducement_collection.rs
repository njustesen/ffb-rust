use crate::inducement::usage::Usage;
use crate::model::game_options::GameOptions;
use crate::model::team::Team;
use crate::option::game_option_id::GameOptionId as GameOptionEnum;

/// The Java `InducementType` subclasses' overrides (anonymous classes in the collections):
/// `useReducedCostId(Team)` and `availability(Team, GameOptions)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InducementTypeRule {
    /// No override.
    #[default]
    Plain,
    /// `useReducedCostId`: the team has the special rule Bribery and Corruption (bribes, biasedRef).
    ReducedForBriberyAndCorruption,
    /// `useReducedCostId`: the roster has the keyword Master Chef (halflingMasterChef).
    ReducedForMasterChefKeyword,
    /// `availability`: capped by `INDUCEMENT_PART_TIME_COACH_TOTAL_MAX - team.assistantCoaches`.
    PartTimeCoachTotal,
    /// `availability`: capped by `INDUCEMENT_TEMP_CHEERLEADER_TOTAL_MAX - team.cheerleaders`.
    TempCheerleaderTotal,
    /// `availability`: 0 unless the team has this special rule (display name).
    RequiresSpecialRule(&'static str),
    /// `availability`: 0 unless the roster has an apothecary (wanderingApothecaries).
    RequiresApothecary,
    /// `availability`: the WIZARD_AVAILABLE flag while the max option is unchanged (wizard).
    Wizard,
}

/// Describes a single inducement type — Java `InducementType`: its name, usages, the game options
/// that hold its cost and maximum (and their reduced variants), and its per-type overrides.
#[derive(Debug, Clone)]
pub struct InducementTypeEntry {
    pub id: String,
    pub usages: Vec<Usage>,
    /// Java `maxId`.
    pub max_id: Option<&'static str>,
    /// Java `costId`.
    pub cost_id: Option<&'static str>,
    /// Java `reducedCostId`.
    pub reduced_cost_id: Option<&'static str>,
    /// Java `reducedMaxId`.
    pub reduced_max_id: Option<&'static str>,
    pub rule: InducementTypeRule,
}

impl InducementTypeEntry {
    pub fn new(id: impl Into<String>, usages: Vec<Usage>) -> Self {
        Self {
            id: id.into(), usages, max_id: None, cost_id: None, reduced_cost_id: None, reduced_max_id: None,
            rule: InducementTypeRule::Plain,
        }
    }

    /// The four option ids, as the Java constructor chain fills them: the short constructors pass
    /// `reducedCostId = costId` and `reducedMaxId = maxId`.
    pub fn with_ids(mut self, max_id: Option<&'static str>, cost_id: Option<&'static str>,
                    reduced_cost_id: Option<&'static str>, reduced_max_id: Option<&'static str>) -> Self {
        self.max_id = max_id;
        self.cost_id = cost_id;
        self.reduced_cost_id = reduced_cost_id;
        self.reduced_max_id = reduced_max_id;
        self
    }

    pub fn with_rule(mut self, rule: InducementTypeRule) -> Self {
        self.rule = rule;
        self
    }

    /// Java `useReducedCostId(Team)` (default false; overridden per type).
    fn use_reduced_cost_id(&self, team: &Team) -> bool {
        match self.rule {
            InducementTypeRule::ReducedForBriberyAndCorruption => {
                let name = crate::model::special_rule::SpecialRule::BRIBERY_AND_CORRUPTION.get_rule_name();
                team.special_rules.iter().any(|r| r == name)
            }
            InducementTypeRule::ReducedForMasterChefKeyword => team.roster.as_ref()
                .map_or(false, |r| r.keywords.iter().any(|k| k == "Master Chef")),
            _ => false,
        }
    }

    /// Java `getActualCostId(Team)`.
    pub fn actual_cost_id(&self, team: &Team) -> Option<&'static str> {
        if self.use_reduced_cost_id(team) { self.reduced_cost_id } else { self.cost_id }
    }

    /// Java `getActualMaxId(Team)` (`useReducedMaxId` = `useReducedCostId`).
    pub fn actual_max_id(&self, team: &Team) -> Option<&'static str> {
        if self.use_reduced_cost_id(team) { self.reduced_max_id } else { self.max_id }
    }

    /// Java `InducementType.availability(Team, GameOptions)` with the collections' overrides.
    pub fn availability(&self, team: &Team, options: &GameOptions) -> i32 {
        // Java base: `getOptionWithDefault(getActualMaxId(team))`, an int option's value, else 0.
        let int_option = |id: Option<&'static str>| -> i32 {
            id.and_then(GameOptionEnum::for_name)
                .map(|o| options.get_option_with_default(o).get_value_as_string().parse().unwrap_or(0))
                .unwrap_or(0)
        };
        let base = || int_option(self.actual_max_id(team));
        match self.rule {
            InducementTypeRule::PartTimeCoachTotal | InducementTypeRule::TempCheerleaderTotal => {
                let mut availability = base();
                if availability > 0 {
                    let (total_id, current) = if self.rule == InducementTypeRule::PartTimeCoachTotal {
                        (crate::option::game_option_id::INDUCEMENT_PART_TIME_COACH_TOTAL_MAX, team.assistant_coaches)
                    } else {
                        (crate::option::game_option_id::INDUCEMENT_TEMP_CHEERLEADER_TOTAL_MAX, team.cheerleaders)
                    };
                    let max = int_option(Some(total_id)) - current;
                    availability = max.max(0).min(availability);
                }
                availability
            }
            InducementTypeRule::RequiresSpecialRule(rule) => {
                if !team.special_rules.iter().any(|r| r == rule) { 0 } else { base() }
            }
            InducementTypeRule::RequiresApothecary => {
                if !team.roster.as_ref().map_or(false, |r| r.apothecary) { 0 } else { base() }
            }
            InducementTypeRule::Wizard => {
                let changed = self.actual_max_id(team).and_then(GameOptionEnum::for_name)
                    .map_or(false, |o| options.get_option_with_default(o).is_changed());
                if !changed {
                    let enabled = GameOptionEnum::for_name(crate::option::game_option_id::WIZARD_AVAILABLE)
                        .map(|o| options.get_option_with_default(o).get_value_as_string())
                        .map_or(false, |v| v == "true");
                    if enabled { 1 } else { 0 }
                } else {
                    base()
                }
            }
            _ => base(),
        }
    }
}

/// 1:1 translation of `com.fumbbl.ffb.inducement.InducementCollection` (abstract base).
///
/// Holds the common set of inducement types shared by all editions.
#[derive(Debug, Default)]
pub struct InducementCollection {
    types: Vec<InducementTypeEntry>,
}

impl InducementCollection {
    pub fn new() -> Self {
        use crate::option::game_option_id::*;
        let types = vec![
            InducementTypeEntry::new("extraTeamTraining", vec![Usage::REROLL]).with_ids(
                Some(INDUCEMENT_EXTRA_TRAINING_MAX), Some(INDUCEMENT_EXTRA_TRAINING_COST),
                Some(INDUCEMENT_EXTRA_TRAINING_COST), Some(INDUCEMENT_EXTRA_TRAINING_MAX)),
            InducementTypeEntry::new("wanderingApothecaries", vec![Usage::APOTHECARY]).with_ids(
                Some(INDUCEMENT_APOS_MAX), Some(INDUCEMENT_APOS_COST),
                Some(INDUCEMENT_APOS_COST), Some(INDUCEMENT_APOS_MAX))
                .with_rule(InducementTypeRule::RequiresApothecary),
            InducementTypeEntry::new("starPlayers", vec![Usage::STAR]).with_ids(
                Some(INDUCEMENT_STARS_MAX), None, None, Some(INDUCEMENT_STARS_MAX)),
            InducementTypeEntry::new("mercenaries", vec![Usage::LONER]),
            InducementTypeEntry::new("wizard", vec![Usage::SPELL]).with_ids(
                Some(INDUCEMENT_WIZARDS_MAX), Some(INDUCEMENT_WIZARDS_COST), None, Some(INDUCEMENT_WIZARDS_MAX))
                .with_rule(InducementTypeRule::Wizard),
        ];
        Self { types }
    }

    /// Java: `getTypes()` — all types (base + sub-types).
    pub fn get_types(&self) -> &[InducementTypeEntry] {
        &self.types
    }

    /// Java: `getKey()` — returns class simple name.
    pub fn get_key(&self) -> &str {
        "InducementCollection"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_collection_has_five_types() {
        let c = InducementCollection::new();
        assert_eq!(c.get_types().len(), 5);
    }

    #[test]
    fn has_extra_team_training() {
        let c = InducementCollection::new();
        assert!(c.get_types().iter().any(|t| t.id == "extraTeamTraining"));
    }

}
