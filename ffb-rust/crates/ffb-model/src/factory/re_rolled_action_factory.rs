use crate::model::re_rolled_action::ReRolledAction;
use crate::model::re_rolled_actions::ReRolledActions;

/// 1:1 translation of com.fumbbl.ffb.factory.ReRolledActionFactory.
pub struct ReRolledActionFactory {
    re_rolled_actions: ReRolledActions,
}

impl Default for ReRolledActionFactory {
    fn default() -> Self {
        ReRolledActionFactory { re_rolled_actions: ReRolledActions::new() }
    }
}

impl ReRolledActionFactory {
    pub fn for_name(&self, name: &str) -> Option<ReRolledAction> {
        self.re_rolled_actions.for_name(name).cloned()
    }

    pub fn initialize(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn for_name_returns_known_action() {
        let f = ReRolledActionFactory::default();
        assert!(f.for_name("Dodge").is_some());
        assert_eq!(f.for_name("dodge").unwrap().get_name(), "Dodge");
    }

    #[test]
    fn for_name_unknown_returns_none() {
        let f = ReRolledActionFactory::default();
        assert!(f.for_name("NOT_AN_ACTION").is_none());
    }

    #[test]
    fn initialize_does_not_panic() {
        let mut f = ReRolledActionFactory::default();
        f.initialize();
    }

    #[test]
    fn for_name_a_second_known_variant() {
        let f = ReRolledActionFactory::default();
        assert!(f.for_name("Go For It").is_some());
    }

    #[test]
    fn for_name_empty_string_returns_none() {
        let f = ReRolledActionFactory::default();
        assert!(f.for_name("").is_none());
    }
}
