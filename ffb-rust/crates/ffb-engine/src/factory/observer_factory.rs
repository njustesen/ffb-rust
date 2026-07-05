/// Translation of com.fumbbl.ffb.server.factory.ObserverFactory.
///
/// Java: Set<ConditionalModelChangeObserver> populated by Scanner reflection.
/// Rust: explicit registration via initialize(). Exposes getObservers() for iteration.
use ffb_model::enums::Rules;
use crate::model::change::chomp_removal_observer::ChompRemovalObserver;
use crate::model::change::conditional_model_change_observer::ConditionalModelChangeObserver;

pub struct ObserverFactory {
    /// Java: Set<ConditionalModelChangeObserver> observers
    observers: Vec<Box<dyn ConditionalModelChangeObserver>>,
}

impl ObserverFactory {
    pub fn new() -> Self { Self { observers: Vec::new() } }

    /// Java: initialize(Game game) — Scanner populates observers by @RulesCollection annotation.
    pub fn initialize(&mut self, rules: Rules) {
        self.observers.clear();
        if rules == Rules::Bb2025 {
            self.observers.push(Box::new(ChompRemovalObserver::new()));
        }
    }

    /// Java: getObservers() — the set of all registered observers.
    pub fn get_observers(&self) -> &[Box<dyn ConditionalModelChangeObserver>] {
        &self.observers
    }

    /// Java: forName(String pName) — always returns null in Java (no name lookup).
    pub fn for_name(&self, name: &str) -> Option<&dyn ConditionalModelChangeObserver> {
        self.observers.iter().find(|o| o.get_name() == name).map(|o| o.as_ref())
    }

    pub fn is_empty(&self) -> bool { self.observers.is_empty() }
}

impl Default for ObserverFactory {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;

    #[test]
    fn new_factory_is_empty() {
        assert!(ObserverFactory::new().is_empty());
    }

    #[test]
    fn initialize_bb2025_registers_chomp_removal_observer() {
        let mut f = ObserverFactory::new();
        f.initialize(Rules::Bb2025);
        assert!(!f.is_empty());
        assert!(f.for_name("ChompRemovalObserver").is_some());
    }

    #[test]
    fn initialize_bb2016_is_empty() {
        let mut f = ObserverFactory::new();
        f.initialize(Rules::Bb2016);
        assert!(f.is_empty());
    }

    #[test]
    fn initialize_bb2020_is_empty() {
        let mut f = ObserverFactory::new();
        f.initialize(Rules::Bb2020);
        assert!(f.is_empty());
    }

    #[test]
    fn get_observers_returns_empty_slice_before_init() {
        assert!(ObserverFactory::new().get_observers().is_empty());
    }

    #[test]
    fn for_name_returns_none_for_unknown() {
        let mut f = ObserverFactory::new();
        f.initialize(Rules::Bb2025);
        assert!(f.for_name("NonExistentObserver").is_none());
    }
}
