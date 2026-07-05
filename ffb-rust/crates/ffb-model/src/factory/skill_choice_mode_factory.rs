use crate::model::SkillChoiceMode;

/// 1:1 translation of com.fumbbl.ffb.factory.SkillChoiceModeFactory (if exists).
pub struct SkillChoiceModeFactory;

impl Default for SkillChoiceModeFactory {
    fn default() -> Self { SkillChoiceModeFactory }
}

impl SkillChoiceModeFactory {
    pub fn for_name(&self, name: &str) -> Option<SkillChoiceMode> {
        SkillChoiceMode::for_name(name)
    }

    pub fn initialize(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn for_name_returns_known_mode() {
        let f = SkillChoiceModeFactory::default();
        assert_eq!(f.for_name("intensiveTraining"), Some(SkillChoiceMode::INTENSIVE_TRAINING));
    }

    #[test]
    fn for_name_unknown_returns_none() {
        assert_eq!(SkillChoiceModeFactory::default().for_name("invalid"), None);
    }
}
