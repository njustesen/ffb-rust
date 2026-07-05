use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};
use crate::inducement::inducement::Inducement;
use crate::inducement::usage::Usage;

/// 1:1 translation of `com.fumbbl.ffb.model.InducementSet`.
/// Tracks all inducements, cards, and prayers available to one team for a game.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InducementSet {
    /// Java: `fInducements` — keyed by InducementType name.
    inducements: HashMap<String, Inducement>,
    /// Java: `fCardsAvailable` — card names drawn but not yet played.
    cards_available: HashSet<String>,
    /// Java: `fCardsActive` — card names currently in play.
    cards_active: HashSet<String>,
    /// Java: `fCardsDeactivated` — card names that have been used.
    cards_deactivated: HashSet<String>,
    /// Java: `prayers` — prayer names available this drive.
    prayers: HashSet<String>,
    /// Java: `fStarPlayerPositionIds` — FUMBBL communication only, no change tracking.
    star_player_position_ids: HashSet<String>,
}

impl InducementSet {
    pub fn new() -> Self {
        Self::default()
    }

    // ---- Inducements -------------------------------------------------------

    /// Java: `InducementSet.get(InducementType)` — returns a clone of the entry.
    pub fn get(&self, type_id: &str) -> Option<Inducement> {
        self.inducements.get(type_id).cloned()
    }

    /// Java: `InducementSet.getInducementMapping()` — returns a copy of the map.
    pub fn get_inducement_mapping(&self) -> HashMap<String, Inducement> {
        self.inducements.clone()
    }

    /// Java: `InducementSet.getInducementTypes()` — type names.
    pub fn get_inducement_types(&self) -> Vec<&str> {
        self.inducements.keys().map(|s| s.as_str()).collect()
    }

    /// Java: `InducementSet.getInducements()` — cloned slice.
    pub fn get_inducements(&self) -> Vec<Inducement> {
        self.inducements.values().cloned().collect()
    }

    /// Java: `InducementSet.addInducement(Inducement)` — null-safe, keyed by type name.
    pub fn add_inducement(&mut self, inducement: Inducement) {
        self.inducements.insert(inducement.type_id.clone(), inducement);
    }

    /// Java: `InducementSet.removeInducement(Inducement)` — null-safe.
    pub fn remove_inducement(&mut self, type_id: &str) {
        self.inducements.remove(type_id);
    }

    /// Java: `InducementSet.hasUsesLeft(InducementType)`.
    pub fn has_uses_left(&self, type_id: &str) -> bool {
        self.get(type_id).map_or(false, |i| i.get_uses_left() > 0)
    }

    /// Java: `UtilServerInducementUse.useInducement(type, 1, set)` — consume one charge.
    /// Returns true and increments `uses` if charges remain; returns false otherwise.
    pub fn use_one_of(&mut self, type_id: &str) -> bool {
        if let Some(ind) = self.inducements.get_mut(type_id) {
            if ind.get_uses_left() > 0 {
                ind.set_uses(ind.get_uses() + 1);
                return true;
            }
        }
        false
    }

    /// Consume one charge of an inducement matching `usage`. Returns the type_id that was used.
    pub fn use_one_for_usage(&mut self, usage: Usage) -> Option<String> {
        let type_id = self.for_usage(usage).map(|s| s.to_string())?;
        if self.use_one_of(&type_id) { Some(type_id) } else { None }
    }

    /// Java: `InducementSet.value(Usage)` — value of the first inducement with that usage.
    pub fn value(&self, usage: Usage) -> i32 {
        self.inducements.values()
            .find(|i| i.has_usage(usage))
            .map_or(0, |i| i.get_value())
    }

    /// Java: `InducementSet.forUsage(Usage)` — type_id of the first matching inducement.
    pub fn for_usage(&self, usage: Usage) -> Option<&str> {
        self.inducements.values()
            .find(|i| i.has_usage(usage))
            .map(|i| i.type_id.as_str())
    }

    // ---- Prayers -----------------------------------------------------------

    /// Java: `InducementSet.addPrayer(Prayer)`.
    pub fn add_prayer(&mut self, prayer_name: impl Into<String>) {
        self.prayers.insert(prayer_name.into());
    }

    /// Java: `InducementSet.removePrayer(Prayer)`.
    pub fn remove_prayer(&mut self, prayer_name: &str) {
        self.prayers.remove(prayer_name);
    }

    /// Java: `InducementSet.getPrayers()`.
    pub fn get_prayers(&self) -> Vec<&str> {
        self.prayers.iter().map(|s| s.as_str()).collect()
    }

    pub fn has_prayer(&self, prayer_name: &str) -> bool {
        self.prayers.contains(prayer_name)
    }

    // ---- Cards -------------------------------------------------------------

    /// Java: `InducementSet.addAvailableCard(Card)`.
    pub fn add_available_card(&mut self, card_name: impl Into<String>) {
        self.cards_available.insert(card_name.into());
    }

    /// Java: `InducementSet.removeAvailableCard(Card)`.
    pub fn remove_available_card(&mut self, card_name: &str) {
        self.cards_available.remove(card_name);
    }

    /// Java: `InducementSet.getAvailableCards()`.
    pub fn get_available_cards(&self) -> Vec<&str> {
        self.cards_available.iter().map(|s| s.as_str()).collect()
    }

    /// Java: `InducementSet.isAvailable(Card)`.
    pub fn is_available(&self, card_name: &str) -> bool {
        self.cards_available.contains(card_name)
    }

    /// Java: `InducementSet.activateCard(Card)` — moves from available → active.
    pub fn activate_card(&mut self, card_name: &str) {
        if self.cards_available.remove(card_name) {
            self.cards_active.insert(card_name.to_string());
        }
    }

    /// Java: `InducementSet.deactivateCard(Card)` — moves from active → deactivated.
    pub fn deactivate_card(&mut self, card_name: &str) {
        if self.cards_active.remove(card_name) {
            self.cards_deactivated.insert(card_name.to_string());
        }
    }

    /// Java: `InducementSet.getActiveCards()`.
    pub fn get_active_cards(&self) -> Vec<&str> {
        self.cards_active.iter().map(|s| s.as_str()).collect()
    }

    /// Java: `InducementSet.getDeactivatedCards()`.
    pub fn get_deactivated_cards(&self) -> Vec<&str> {
        self.cards_deactivated.iter().map(|s| s.as_str()).collect()
    }

    /// Java: `InducementSet.isDeactivated(Card)`.
    pub fn is_deactivated(&self, card_name: &str) -> bool {
        self.cards_deactivated.contains(card_name)
    }

    /// Java: `InducementSet.isActive(Card)`.
    pub fn is_active(&self, card_name: &str) -> bool {
        self.cards_active.contains(card_name)
    }

    /// Java: `InducementSet.getAllCards()` — available + active + deactivated.
    pub fn get_all_cards(&self) -> Vec<&str> {
        let mut all: Vec<&str> = vec![];
        all.extend(self.get_available_cards());
        all.extend(self.get_active_cards());
        all.extend(self.get_deactivated_cards());
        all
    }

    // ---- Star players ------------------------------------------------------

    /// Java: `InducementSet.addStarPlayerPositionId(String)`.
    pub fn add_star_player_position_id(&mut self, position_id: impl Into<String>) {
        self.star_player_position_ids.insert(position_id.into());
    }

    /// Java: `InducementSet.getStarPlayerPositionIds()`.
    pub fn get_star_player_position_ids(&self) -> Vec<&str> {
        self.star_player_position_ids.iter().map(|s| s.as_str()).collect()
    }

    // ---- Bulk operations ---------------------------------------------------

    /// Java: `InducementSet.add(InducementSet)` — merges another set into this one.
    pub fn add(&mut self, other: &InducementSet) {
        for inducement in other.get_inducements() {
            let mut init = Inducement::new(
                inducement.type_id.clone(),
                inducement.value,
                inducement.usages.clone(),
            );
            init.set_uses(inducement.uses);
            self.add_inducement(init);
        }
        for card in other.get_available_cards() {
            self.add_available_card(card);
            if other.is_active(card) || other.is_deactivated(card) {
                self.activate_card(card);
                if other.is_deactivated(card) {
                    self.deactivate_card(card);
                }
            }
        }
        for prayer in other.get_prayers() {
            self.add_prayer(prayer);
        }
    }

    /// Java: `InducementSet.clear()`.
    pub fn clear(&mut self) {
        self.inducements.clear();
        self.cards_available.clear();
        self.cards_active.clear();
        self.cards_deactivated.clear();
        self.prayers.clear();
    }

    /// Java: `InducementSet.totalInducements()` — sum of values excluding certain usages, plus cards.
    pub fn total_inducements(&self) -> i32 {
        let excluded: &[Usage] = Usage::exclude_from_count();
        let inducement_total: i32 = self.inducements.values()
            .filter(|i| !i.usages.iter().all(|u| excluded.contains(u)))
            .map(|i| i.get_value())
            .sum();
        inducement_total + self.get_all_cards().len() as i32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_bribe() -> Inducement {
        Inducement::new("BRIBE", 2, vec![Usage::AVOID_BAN])
    }

    fn make_cheerleader() -> Inducement {
        Inducement::new("EXTRA_CHEERLEADER", 1, vec![Usage::ADD_CHEERLEADER])
    }

    #[test]
    fn add_and_get_inducement() {
        let mut s = InducementSet::new();
        s.add_inducement(make_bribe());
        assert!(s.get("BRIBE").is_some());
        assert_eq!(s.get("BRIBE").unwrap().get_value(), 2);
    }

    #[test]
    fn has_uses_left_false_when_absent() {
        let s = InducementSet::new();
        assert!(!s.has_uses_left("BRIBE"));
    }

    #[test]
    fn has_uses_left_true_when_present() {
        let mut s = InducementSet::new();
        s.add_inducement(make_bribe());
        assert!(s.has_uses_left("BRIBE"));
    }

    #[test]
    fn has_uses_left_false_when_exhausted() {
        let mut s = InducementSet::new();
        let mut b = make_bribe();
        b.set_uses(2);
        s.add_inducement(b);
        assert!(!s.has_uses_left("BRIBE"));
    }

    #[test]
    fn value_returns_zero_when_absent() {
        let s = InducementSet::new();
        assert_eq!(s.value(Usage::ADD_CHEERLEADER), 0);
    }

    #[test]
    fn value_returns_count_for_usage() {
        let mut s = InducementSet::new();
        s.add_inducement(make_cheerleader());
        assert_eq!(s.value(Usage::ADD_CHEERLEADER), 1);
    }

    #[test]
    fn for_usage_returns_type_id() {
        let mut s = InducementSet::new();
        s.add_inducement(make_bribe());
        assert_eq!(s.for_usage(Usage::AVOID_BAN), Some("BRIBE"));
    }

    #[test]
    fn remove_inducement() {
        let mut s = InducementSet::new();
        s.add_inducement(make_bribe());
        s.remove_inducement("BRIBE");
        assert!(s.get("BRIBE").is_none());
    }

    #[test]
    fn card_lifecycle() {
        let mut s = InducementSet::new();
        s.add_available_card("Chop Block");
        assert!(s.is_available("Chop Block"));
        s.activate_card("Chop Block");
        assert!(!s.is_available("Chop Block"));
        assert!(s.is_active("Chop Block"));
        s.deactivate_card("Chop Block");
        assert!(!s.is_active("Chop Block"));
        assert!(s.is_deactivated("Chop Block"));
    }

    #[test]
    fn get_all_cards_includes_all_states() {
        let mut s = InducementSet::new();
        s.add_available_card("A");
        s.add_available_card("B");
        s.activate_card("A");
        let all = s.get_all_cards();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn prayer_add_remove() {
        let mut s = InducementSet::new();
        s.add_prayer("Nuffle's Blessing");
        assert!(s.has_prayer("Nuffle's Blessing"));
        s.remove_prayer("Nuffle's Blessing");
        assert!(!s.has_prayer("Nuffle's Blessing"));
    }

    #[test]
    fn clear_empties_all() {
        let mut s = InducementSet::new();
        s.add_inducement(make_bribe());
        s.add_available_card("Chop Block");
        s.add_prayer("Nuffle's Blessing");
        s.clear();
        assert!(s.get("BRIBE").is_none());
        assert!(s.get_all_cards().is_empty());
        assert!(s.get_prayers().is_empty());
    }

    #[test]
    fn total_inducements_excludes_reroll_argue() {
        let mut s = InducementSet::new();
        s.add_inducement(Inducement::new("REROLL_ARGUE", 2, vec![Usage::REROLL_ARGUE]));
        s.add_inducement(make_bribe());
        // REROLL_ARGUE is in exclude_from_count, so only bribe counts
        assert_eq!(s.total_inducements(), 2); // bribe value=2
    }

    #[test]
    fn add_merges_inducements() {
        let mut a = InducementSet::new();
        a.add_inducement(make_bribe());
        let mut b = InducementSet::new();
        b.add_inducement(make_cheerleader());
        a.add(&b);
        assert!(a.get("BRIBE").is_some());
        assert!(a.get("EXTRA_CHEERLEADER").is_some());
    }
}
