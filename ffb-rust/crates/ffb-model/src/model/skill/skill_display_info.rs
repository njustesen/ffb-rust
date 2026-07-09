use serde::{Deserialize, Serialize};

/// 1:1 translation of com.fumbbl.ffb.model.skill.SkillDisplayInfo.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SkillDisplayInfo {
    pub name: String,
    pub short_name: String,
    pub description: String,
}

impl SkillDisplayInfo {
    pub fn new(name: String, short_name: String, description: String) -> Self {
        Self { name, short_name, description }
    }
    pub fn get_name(&self) -> &str { &self.name }
    pub fn get_short_name(&self) -> &str { &self.short_name }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_empty() {
        assert!(SkillDisplayInfo::default().name.is_empty());
    }

    #[test]
    fn new_sets_fields() {
        let info = SkillDisplayInfo::new("Block".to_string(), "Blk".to_string(), "desc".to_string());
        assert_eq!(info.get_name(), "Block");
        assert_eq!(info.get_short_name(), "Blk");
    }
}
