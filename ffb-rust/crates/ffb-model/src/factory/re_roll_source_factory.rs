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
}
