use crate::model::SkillUse;

/// 1:1 translation of com.fumbbl.ffb.factory.SkillUseFactory.
pub struct SkillUseFactory;

impl Default for SkillUseFactory {
    fn default() -> Self { SkillUseFactory }
}

impl SkillUseFactory {
    pub fn for_name(&self, name: &str) -> Option<SkillUse> {
        SkillUse::for_name(name)
    }

    pub fn initialize(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn for_name_returns_known_skill_use() {
        let f = SkillUseFactory::default();
        assert_eq!(f.for_name("wouldNotHelp"), Some(SkillUse::WOULD_NOT_HELP));
    }

    #[test]
    fn for_name_unknown_returns_none() {
        assert_eq!(SkillUseFactory::default().for_name("invalid"), None);
    }

    #[test]
    fn for_name_stop_opponent_returns_some() {
        let f = SkillUseFactory::default();
        assert!(f.for_name("stopOpponent").is_some());
    }

    #[test]
    fn initialize_does_not_panic() {
        let mut f = SkillUseFactory::default();
        f.initialize();
    }
}
