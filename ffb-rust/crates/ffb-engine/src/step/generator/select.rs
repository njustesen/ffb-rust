/// Root-level abstract base for the Select step sequence generator.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.Select`.
use ffb_model::model::block_target::BlockTarget;

#[derive(Debug, Clone, Default)]
pub struct SelectParams {
    /// BlockTarget list.
    pub block_targets: Vec<BlockTarget>,
    pub update_persistence: bool,
}

pub struct Select;

impl Select {
    pub fn new() -> Self { Self }
}

impl Default for Select {
    fn default() -> Self { Self::new() }
}
