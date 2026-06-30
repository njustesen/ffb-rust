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
