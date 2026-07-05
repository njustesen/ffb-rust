use crate::model::ConcedeGameStatus;

/// 1:1 translation of com.fumbbl.ffb.factory.ConcedeGameStatusFactory.
pub struct ConcedeGameStatusFactory;

impl Default for ConcedeGameStatusFactory {
    fn default() -> Self { ConcedeGameStatusFactory }
}

impl ConcedeGameStatusFactory {
    pub fn for_name(&self, name: &str) -> Option<ConcedeGameStatus> {
        ConcedeGameStatus::from_name(name)
    }

    pub fn initialize(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn for_name_returns_known_status() {
        assert!(ConcedeGameStatusFactory::default().for_name("requested").is_some());
        assert!(ConcedeGameStatusFactory::default().for_name("confirmed").is_some());
    }

    #[test]
    fn for_name_unknown_returns_none() {
        assert_eq!(ConcedeGameStatusFactory::default().for_name("invalid"), None);
    }
}
