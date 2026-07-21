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
    fn serde_round_trip_empty() {
        let p = BlockRollProperties::new();
        let s = serde_json::to_string(&p).unwrap();
        let back: BlockRollProperties = serde_json::from_str(&s).unwrap();
        assert!(back.re_roll_sources.is_empty());
    }

}
