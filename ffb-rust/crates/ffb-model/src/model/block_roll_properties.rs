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
