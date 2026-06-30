use crate::enums::PassingDistance;

/// 1:1 translation of com.fumbbl.ffb.factory.PassingDistanceFactory.
pub struct PassingDistanceFactory;

impl Default for PassingDistanceFactory {
    fn default() -> Self { Self }
}

impl PassingDistanceFactory {
    pub fn for_name(&self, name: &str) -> Option<PassingDistance> {
        PassingDistance::from_name(name)
    }

    pub fn initialize(&mut self) {}
}
