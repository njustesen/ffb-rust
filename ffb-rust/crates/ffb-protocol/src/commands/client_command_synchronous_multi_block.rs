use ffb_model::model::BlockTarget;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandSynchronousMultiBlock`.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandSynchronousMultiBlock {
    /// Java: `selectedBlockTargets`
    pub selected_block_targets: Vec<BlockTarget>,
}

impl ClientCommandSynchronousMultiBlock {
    pub fn new() -> Self { Self::default() }

    pub fn with_targets(selected_block_targets: Vec<BlockTarget>) -> Self {
        Self { selected_block_targets }
    }

    pub fn get_selected_block_targets(&self) -> &[BlockTarget] { &self.selected_block_targets }

    pub fn add_target(&mut self, target: BlockTarget) {
        self.selected_block_targets.push(target);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::model::BlockKind;

    #[test]
    fn targets_stored() {
        let t = BlockTarget::new("p1", BlockKind::BLOCK, None);
        let cmd = ClientCommandSynchronousMultiBlock::with_targets(vec![t]);
        assert_eq!(cmd.get_selected_block_targets().len(), 1);
    }

    #[test]
    fn default_is_empty() {
        let cmd = ClientCommandSynchronousMultiBlock::new();
        assert!(cmd.selected_block_targets.is_empty());
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ClientCommandSynchronousMultiBlock::default()).is_empty());
    }


    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandSynchronousMultiBlock::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ClientCommandSynchronousMultiBlock::default());
        assert!(s.contains("ClientCommandSynchronousMultiBlock"));
    }
}
