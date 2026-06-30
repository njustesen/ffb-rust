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
