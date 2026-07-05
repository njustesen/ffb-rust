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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn for_name_returns_known_distance() {
        let f = PassingDistanceFactory::default();
        assert!(f.for_name("Quick Pass").is_some());
    }

    #[test]
    fn for_name_unknown_returns_none() {
        assert_eq!(PassingDistanceFactory::default().for_name("invalid"), None);
    }
}
