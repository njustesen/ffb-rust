use crate::enums::TeamStatus;

/// 1:1 translation of com.fumbbl.ffb.factory.TeamStatusFactory.
pub struct TeamStatusFactory;

impl Default for TeamStatusFactory {
    fn default() -> Self { TeamStatusFactory }
}

impl TeamStatusFactory {
    pub fn for_name(&self, name: &str) -> Option<TeamStatus> {
        TeamStatus::from_name(name)
    }

    pub fn initialize(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn for_name_returns_known_status() {
        assert_eq!(TeamStatusFactory::default().for_name("Active"), Some(TeamStatus::Active));
        assert_eq!(TeamStatusFactory::default().for_name("New"), Some(TeamStatus::New));
    }

    #[test]
    fn for_name_unknown_returns_none() {
        assert_eq!(TeamStatusFactory::default().for_name("invalid"), None);
    }

    #[test]
    fn initialize_does_not_panic() {
        let mut f = TeamStatusFactory::default();
        f.initialize();
    }

    #[test]
    fn for_name_empty_string_returns_none() {
        assert_eq!(TeamStatusFactory::default().for_name(""), None);
    }
}
