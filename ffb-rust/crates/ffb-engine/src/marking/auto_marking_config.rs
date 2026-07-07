/// 1:1 translation of com.fumbbl.ffb.server.marking.AutoMarkingConfig.
use ffb_model::marking::sort_mode::SortMode;
use ffb_model::model::skill_def::SkillId;
use crate::marking::auto_marking_record::{AutoMarkingRecord, Builder};

pub struct AutoMarkingConfig {
    pub separator: String,
    pub markings: Vec<AutoMarkingRecord>,
    /// Java: transient field, not serialized. Set when loading config from the site.
    pub sort_mode: SortMode,
}

impl AutoMarkingConfig {
    pub fn new() -> Self {
        Self {
            separator: String::new(),
            markings: Vec::new(),
            sort_mode: SortMode::Default,
        }
    }

    pub fn get_markings(&self) -> &[AutoMarkingRecord] { &self.markings }
    pub fn get_separator(&self) -> &str { &self.separator }
    pub fn set_separator(&mut self, separator: impl Into<String>) { self.separator = separator.into(); }
    pub fn get_sort_mode(&self) -> SortMode { self.sort_mode }
    pub fn set_sort_mode(&mut self, sort_mode: SortMode) { self.sort_mode = sort_mode; }

    /// Java: AutoMarkingConfig.defaults(SkillFactory skillFactory).
    /// Returns the default set of auto-marking records (gained skills only, one letter each).
    pub fn defaults() -> Vec<AutoMarkingRecord> {
        [
            (SkillId::Block,          "B"),
            (SkillId::Tackle,         "T"),
            (SkillId::Dodge,          "D"),
            (SkillId::MightyBlow,     "M"),
            (SkillId::SneakyGit,      "Sg"),
            (SkillId::Claw,           "C"),
            (SkillId::DivingTackle,   "Dt"),
            (SkillId::DirtyPlayer,    "Dp"),
            (SkillId::SideStep,       "S"),
            (SkillId::Guard,          "G"),
            (SkillId::Wrestle,        "W"),
        ]
        .iter()
        .map(|(skill_id, marking)| {
            Builder::new()
                .with_skill(*skill_id)
                .with_marking(*marking)
                .with_gained_only(true)
                .build()
        })
        .collect()
    }
}

impl Default for AutoMarkingConfig {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_has_empty_markings() {
        let c = AutoMarkingConfig::new();
        assert!(c.get_markings().is_empty());
        assert!(c.get_separator().is_empty());
        assert_eq!(c.get_sort_mode(), SortMode::Default);
    }

    #[test]
    fn set_separator() {
        let mut c = AutoMarkingConfig::new();
        c.set_separator("/");
        assert_eq!(c.get_separator(), "/");
    }

    #[test]
    fn set_sort_mode() {
        let mut c = AutoMarkingConfig::new();
        c.set_sort_mode(SortMode::None);
        assert_eq!(c.get_sort_mode(), SortMode::None);
    }

    #[test]
    fn defaults_has_11_records() {
        let defaults = AutoMarkingConfig::defaults();
        assert_eq!(defaults.len(), 11);
    }

    #[test]
    fn defaults_block_marking_is_b() {
        let defaults = AutoMarkingConfig::defaults();
        let block_record = defaults.iter()
            .find(|r| r.skills().contains(&ffb_model::model::skill_def::SkillId::Block));
        assert!(block_record.is_some());
        assert_eq!(block_record.unwrap().marking(), "B");
    }

    #[test]
    fn defaults_all_gained_only() {
        let defaults = AutoMarkingConfig::defaults();
        assert!(defaults.iter().all(|r| r.is_gained_only()));
    }

    #[test]
    fn add_marking_record() {
        let mut c = AutoMarkingConfig::new();
        c.markings.push(Builder::new()
            .with_skill(ffb_model::model::skill_def::SkillId::Block)
            .with_marking("B")
            .build());
        assert_eq!(c.get_markings().len(), 1);
    }

    #[test]
    fn default_same_as_new() {
        let c = AutoMarkingConfig::default();
        assert!(c.get_markings().is_empty());
    }

    #[test]
    fn defaults_tackle_marking_is_t() {
        let defaults = AutoMarkingConfig::defaults();
        let tackle = defaults.iter()
            .find(|r| r.skills().contains(&ffb_model::model::skill_def::SkillId::Tackle));
        assert!(tackle.is_some());
        assert_eq!(tackle.unwrap().marking(), "T");
    }
}
