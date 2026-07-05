use serde::{Deserialize, Serialize};
use crate::enums::ReRollSource;

/// 1:1 translation of com.fumbbl.ffb.model.BlockRollProperties.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BlockRollProperties {
    pub re_roll_sources: Vec<ReRollSource>,
}

impl BlockRollProperties {
    pub fn new() -> Self { BlockRollProperties::default() }

    pub fn add_re_roll_source(&mut self, source: ReRollSource) {
        self.re_roll_sources.push(source);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn new_starts_empty() {
        assert!(BlockRollProperties::new().re_roll_sources.is_empty());
    }
    #[test]
    fn add_re_roll_source_accumulates() {
        let mut p = BlockRollProperties::new();
        let source = ReRollSource::new("teamReRoll");
        p.add_re_roll_source(source);
        assert_eq!(p.re_roll_sources.len(), 1);
    }
}
