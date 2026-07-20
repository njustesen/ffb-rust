/// Shared helpers for the pre-game inducement-buying steps
/// (`bb2016::start::StepBuyInducements`, `bb2020::start::StepBuyCardsAndInducements`,
/// `bb2025::start::StepBuyInducements`).
///
/// The Java monolith resolves the buyable catalog via `InducementTypeFactory`, which is not
/// ported (no client-side factory registry in this engine). Instead each edition's catalog is
/// data-driven from `data/inducements/bb20xx_inducements.json` (loaded as `InducementsJson` /
/// `InducementJson` — see `ffb_model::data::loader`). These helpers apply the same filtering
/// and purchase-application rules across all three editions.
///
/// Roster-derived entries (`availability: "roster_star"` / `"roster_staff"`) are out of scope
/// for this pass — star players / infamous staff purchasing needs roster data plumbed through
/// beyond the catalog and is left for a follow-up phase.
use ffb_model::data::roster_json::InducementJson;
use ffb_model::inducement::inducement::Inducement;
use ffb_model::inducement::usage::Usage;
use ffb_model::model::inducement_set::InducementSet;
use ffb_model::model::team::Team;
use ffb_mechanics::inducement::special_rule_matches;
use crate::action::InducementPurchase;

/// Whether `def` is purchasable by `team` right now: it must have at least one purchasable
/// slot (`max_count > 0`), and if it has an `availability` gate, the team must satisfy it.
/// `roster_star` / `roster_staff` gated entries are never purchasable here (see module docs).
pub fn is_available_for_team(def: &InducementJson, team: &Team) -> bool {
    if def.max_count <= 0 {
        return false;
    }
    if def.availability.is_empty() {
        return true;
    }
    match def.availability.strip_prefix("special_rule:") {
        Some(key) => special_rule_matches(&team.special_rules, key),
        None => false, // roster_star / roster_staff — out of scope for now
    }
}

/// The full `(id, cost)` catalog available to `team` right now — i.e. what
/// `AgentPrompt::BuyInducements`/`BuyPrayersAndInducements` should list as `available`.
pub fn available_list(catalog: &[InducementJson], team: &Team) -> Vec<(String, i32)> {
    catalog.iter()
        .filter(|def| is_available_for_team(def, team))
        .map(|def| (def.id.clone(), def.cost))
        .collect()
}

/// Applies a single inducement id + quantity to `team`/`inducement_set`, clamped to
/// `def.max_count` and to what fits in `remaining_budget`. Deducts the spent gold from
/// `team.treasury` (floored at 0) and returns the amount actually spent.
///
/// Ids with a dedicated `Team` scalar field (bribes, halflingMasterChef,
/// bloodweiserBabes/bloodweiserKegs, riotousRookies, cheerleaders, assistantCoaches) increment
/// that field directly; everything else (wizard, cards, weatherMage, biasedReferee,
/// mortuaryAssistant/igor/plagueDoctor, partTimeCoach, tempCheerleader, bugmansXXXXXX,
/// prayers, throwARock, briberyAndCorruption, teamMascot) is recorded in the team's
/// `InducementSet`, keyed by catalog id, with `usages` parsed from the catalog's `usage` field.
fn existing_qty(def: &InducementJson, team: &Team, inducement_set: &InducementSet) -> i32 {
    match def.id.as_str() {
        "bribes" => team.bribes,
        "halflingMasterChef" => team.master_chefs,
        "bloodweiserBabes" | "bloodweiserKegs" => team.bloodweiser_kegs,
        "riotousRookies" => team.riotous_rookies,
        "cheerleaders" => team.cheerleaders,
        "assistantCoaches" => team.assistant_coaches,
        _ => inducement_set.get(&def.id).map_or(0, |i| i.get_value()),
    }
}

fn apply_one(def: &InducementJson, requested: i32, remaining_budget: i32, team: &mut Team, inducement_set: &mut InducementSet) -> (i32, u32) {
    let existing = existing_qty(def, team, inducement_set);
    let room = (def.max_count - existing).max(0);
    let mut qty = requested.min(room);
    if def.cost > 0 {
        qty = qty.min(remaining_budget / def.cost);
    }
    if qty <= 0 {
        return (0, 0);
    }
    let cost = def.cost * qty;
    team.treasury = (team.treasury - cost).max(0);
    match def.id.as_str() {
        "bribes" => team.bribes += qty,
        "halflingMasterChef" => team.master_chefs += qty,
        "bloodweiserBabes" | "bloodweiserKegs" => team.bloodweiser_kegs += qty,
        "riotousRookies" => team.riotous_rookies += qty,
        "cheerleaders" => team.cheerleaders += qty,
        "assistantCoaches" => team.assistant_coaches += qty,
        _ => {
            let usages = Usage::parse_list(&def.usage);
            inducement_set.add_inducement(Inducement::new(def.id.clone(), existing + qty, usages));
        }
    }
    (cost, qty as u32)
}

/// Applies a full purchase list (as received via `Action::BuyInducements`) against `team` and
/// its `inducement_set`, in order, each purchase's affordability checked against the remaining
/// budget after prior purchases in the same list. Purchases for ids not in the buyable catalog
/// (unknown id, or gated out for this team) are skipped. Returns the total gold spent plus the
/// `(inducement_id, quantity)` actually applied for each purchase that resulted in a nonzero
/// quantity — callers use this to emit `GameEvent::BuyInducement` for coverage tracking.
pub fn apply_purchases(
    catalog: &[InducementJson],
    team: &mut Team,
    inducement_set: &mut InducementSet,
    purchases: &[InducementPurchase],
    budget: i32,
) -> (i32, Vec<(String, u32)>) {
    let mut remaining = budget;
    let mut spent = 0;
    let mut applied = Vec::new();
    for purchase in purchases {
        let def = match catalog.iter().find(|d| d.id == purchase.id) {
            Some(d) if is_available_for_team(d, team) => d,
            _ => continue,
        };
        let (cost, qty) = apply_one(def, purchase.count as i32, remaining, team, inducement_set);
        remaining -= cost;
        spent += cost;
        if qty > 0 { applied.push((def.id.clone(), qty)); }
    }
    (spent, applied)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::data::loader::BB2016_INDUCEMENTS;
    use ffb_model::data::loader::BB2020_INDUCEMENTS;

    fn make_team() -> Team {
        Team {
            id: "home".into(), name: "Home".into(), race: "human".into(),
            roster_id: "human".into(), coach: "c".into(),
            rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0,
            cheerleaders: 0, assistant_coaches: 0, fan_factor: 0,
            dedicated_fans: 0, team_value: 0, treasury: 1_000_000,
            special_rules: vec![], players: vec![],
            vampire_lord: false, necromancer: false,
        }
    }

    #[test]
    fn available_list_excludes_gated_special_rule_by_default() {
        let team = make_team();
        let list = available_list(&BB2016_INDUCEMENTS.inducements, &team);
        assert!(!list.iter().any(|(id, _)| id == "igor"), "igor requires Sylvanian Spotlight");
    }

    #[test]
    fn available_list_includes_special_rule_gated_item_when_team_has_rule() {
        let mut team = make_team();
        team.special_rules.push("Sylvanian Spotlight".into());
        let list = available_list(&BB2016_INDUCEMENTS.inducements, &team);
        assert!(list.iter().any(|(id, _)| id == "igor"));
    }

    #[test]
    fn available_list_excludes_roster_star() {
        let team = make_team();
        let list = available_list(&BB2016_INDUCEMENTS.inducements, &team);
        assert!(!list.iter().any(|(id, _)| id == "starPlayer"));
    }

    #[test]
    fn available_list_excludes_zero_max_count_entries() {
        let mut team = make_team();
        team.special_rules.push("Bribery and Corruption".into());
        let list = available_list(&BB2020_INDUCEMENTS.inducements, &team);
        // briberyAndCorruption has max_count 0 — auto-granted elsewhere, never purchasable here.
        assert!(!list.iter().any(|(id, _)| id == "briberyAndCorruption"));
    }

    #[test]
    fn apply_purchases_increments_team_field_and_deducts_treasury() {
        let mut team = make_team();
        let mut set = InducementSet::new();
        let purchases = vec![InducementPurchase { id: "bribes".into(), count: 2 }];
        let (spent, _applied) = apply_purchases(&BB2016_INDUCEMENTS.inducements, &mut team, &mut set, &purchases, 1_000_000);
        assert_eq!(spent, 200_000);
        assert_eq!(team.bribes, 2);
        assert_eq!(team.treasury, 800_000);
    }

    #[test]
    fn apply_purchases_clamps_to_max_count() {
        let mut team = make_team();
        let mut set = InducementSet::new();
        // bribes max_count is 3.
        let purchases = vec![InducementPurchase { id: "bribes".into(), count: 10 }];
        let (spent, _applied) = apply_purchases(&BB2016_INDUCEMENTS.inducements, &mut team, &mut set, &purchases, 10_000_000);
        assert_eq!(team.bribes, 3);
        assert_eq!(spent, 300_000);
    }

    #[test]
    fn apply_purchases_clamps_to_budget() {
        let mut team = make_team();
        let mut set = InducementSet::new();
        // Only enough gold for 1 bribe (100,000).
        let purchases = vec![InducementPurchase { id: "bribes".into(), count: 3 }];
        let (spent, _applied) = apply_purchases(&BB2016_INDUCEMENTS.inducements, &mut team, &mut set, &purchases, 150_000);
        assert_eq!(team.bribes, 1);
        assert_eq!(spent, 100_000);
    }

    #[test]
    fn apply_purchases_unknown_id_is_skipped() {
        let mut team = make_team();
        let mut set = InducementSet::new();
        let purchases = vec![InducementPurchase { id: "nonexistent".into(), count: 1 }];
        let (spent, _applied) = apply_purchases(&BB2016_INDUCEMENTS.inducements, &mut team, &mut set, &purchases, 1_000_000);
        assert_eq!(spent, 0);
    }

    #[test]
    fn apply_purchases_gated_item_is_skipped_without_special_rule() {
        let mut team = make_team();
        let mut set = InducementSet::new();
        let purchases = vec![InducementPurchase { id: "igor".into(), count: 1 }];
        let (spent, _applied) = apply_purchases(&BB2016_INDUCEMENTS.inducements, &mut team, &mut set, &purchases, 1_000_000);
        assert_eq!(spent, 0);
        assert!(set.get("igor").is_none());
    }

    #[test]
    fn apply_purchases_routes_wizard_into_inducement_set() {
        let mut team = make_team();
        let mut set = InducementSet::new();
        let purchases = vec![InducementPurchase { id: "wizard".into(), count: 1 }];
        let (spent, _applied) = apply_purchases(&BB2016_INDUCEMENTS.inducements, &mut team, &mut set, &purchases, 1_000_000);
        assert_eq!(spent, 150_000);
        assert_eq!(set.get("wizard").map(|i| i.get_value()), Some(1));
        assert!(set.get("wizard").unwrap().has_usage(Usage::SPELL));
    }

    #[test]
    fn apply_purchases_accumulates_existing_inducement_set_value() {
        let mut team = make_team();
        let mut set = InducementSet::new();
        set.add_inducement(Inducement::new("wizard", 1, vec![Usage::SPELL]));
        let purchases = vec![InducementPurchase { id: "wizard".into(), count: 1 }];
        apply_purchases(&BB2016_INDUCEMENTS.inducements, &mut team, &mut set, &purchases, 1_000_000);
        assert_eq!(set.get("wizard").map(|i| i.get_value()), Some(1), "wizard max_count is 1, second purchase clamped to 0");
    }

    #[test]
    fn apply_purchases_spreads_budget_across_multiple_purchases() {
        let mut team = make_team();
        let mut set = InducementSet::new();
        let purchases = vec![
            InducementPurchase { id: "bribes".into(), count: 1 }, // 100,000
            InducementPurchase { id: "halflingMasterChef".into(), count: 1 }, // 300,000
        ];
        let (spent, _applied) = apply_purchases(&BB2016_INDUCEMENTS.inducements, &mut team, &mut set, &purchases, 350_000);
        // bribes fits (100,000 spent, 250,000 left); master chef (300,000) doesn't fit remaining 250,000.
        assert_eq!(spent, 100_000);
        assert_eq!(team.bribes, 1);
        assert_eq!(team.master_chefs, 0);
    }
}
