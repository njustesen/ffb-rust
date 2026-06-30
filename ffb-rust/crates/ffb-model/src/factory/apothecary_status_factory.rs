use crate::enums::ApothecaryStatus;

/// 1:1 translation of com.fumbbl.ffb.factory.ApothecaryStatusFactory.
pub struct ApothecaryStatusFactory;

impl Default for ApothecaryStatusFactory {
    fn default() -> Self { Self }
}

impl ApothecaryStatusFactory {
    pub fn for_name(&self, name: &str) -> Option<ApothecaryStatus> {
        ApothecaryStatus::from_name(name)
    }

    pub fn initialize(&mut self) {}
}
