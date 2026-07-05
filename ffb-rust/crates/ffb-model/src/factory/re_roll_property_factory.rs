use crate::enums::ReRollProperty;

/// 1:1 translation of com.fumbbl.ffb.factory.ReRollPropertyFactory.
pub struct ReRollPropertyFactory;

impl Default for ReRollPropertyFactory {
    fn default() -> Self { ReRollPropertyFactory }
}

impl ReRollPropertyFactory {
    pub fn for_name(&self, name: &str) -> Option<ReRollProperty> {
        ReRollProperty::from_name(name)
    }

    pub fn initialize(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn for_name_returns_known_property() {
        assert_eq!(ReRollPropertyFactory::default().for_name("TRR"), Some(ReRollProperty::Trr));
        assert_eq!(ReRollPropertyFactory::default().for_name("PRO"), Some(ReRollProperty::Pro));
    }

    #[test]
    fn for_name_unknown_returns_none() {
        assert_eq!(ReRollPropertyFactory::default().for_name("invalid"), None);
    }
}
