use crate::enums::ReRollSource;
use crate::model::re_roll_sources::ReRollSources;

/// 1:1 translation of com.fumbbl.ffb.factory.ReRollSourceFactory.
pub struct ReRollSourceFactory {
    re_roll_sources: ReRollSources,
}

impl Default for ReRollSourceFactory {
    fn default() -> Self {
        ReRollSourceFactory { re_roll_sources: ReRollSources::new() }
    }
}

impl ReRollSourceFactory {
    pub fn for_name(&self, name: &str) -> Option<ReRollSource> {
        self.re_roll_sources.for_name(name).cloned()
    }

    pub fn initialize(&mut self) {}

    /// Java `ReRollSourceFactory.forName(name)` (`values().get(name.toLowerCase())`) on one shared
    /// registry -- for the dialog JSON decoders (BACKLOG §O).
    pub fn for_name_shared(name: &str) -> Option<&'static ReRollSource> {
        static REGISTRY: std::sync::OnceLock<ReRollSources> = std::sync::OnceLock::new();
        REGISTRY.get_or_init(ReRollSources::new).for_name(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn for_name_returns_known_source() {
        let f = ReRollSourceFactory::default();
        assert!(f.for_name("Pro").is_some());
    }

    #[test]
    fn for_name_unknown_returns_none() {
        let f = ReRollSourceFactory::default();
        assert!(f.for_name("NOT_VALID").is_none());
    }

    #[test]
    fn for_name_a_second_known_variant() {
        // "Team ReRoll" is stored lowercased; lookup is case-insensitive
        let f = ReRollSourceFactory::default();
        assert!(f.for_name("Team ReRoll").is_some());
    }

    #[test]
    fn for_name_empty_string_returns_none() {
        let f = ReRollSourceFactory::default();
        assert!(f.for_name("").is_none());
    }
}
