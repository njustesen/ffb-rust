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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn select_params_default_empty_targets() {
        let p = SelectParams::default();
        assert!(p.block_targets.is_empty());
    }

    #[test]
    fn select_params_default_no_update_persistence() {
        let p = SelectParams::default();
        assert!(!p.update_persistence);
    }

    #[test]
    fn select_struct_is_default() {
        let _ = Select::default();
    }

    #[test]
    fn select_params_accepts_block_target() {
        let bt = BlockTarget::default();
        let p = SelectParams { block_targets: vec![bt], ..Default::default() };
        assert_eq!(p.block_targets.len(), 1);
    }
}
