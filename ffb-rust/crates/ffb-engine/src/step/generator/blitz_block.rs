/// Root-level abstract base for the BlitzBlock step sequence generator.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.BlitzBlock`.

#[derive(Debug, Clone, Default)]
pub struct BlitzBlockParams {
    pub block_defender_id: Option<String>,
    pub multi_block_defender_id: Option<String>,
    pub using_stab: bool,
    pub using_chainsaw: bool,
    pub using_vomit: bool,
    pub frenzy_block: bool,
    pub ask_for_block_kind: bool,
    pub publish_defender: bool,
    pub using_breathe_fire: bool,
    pub using_chomp: bool,
}

pub struct BlitzBlock;

impl BlitzBlock {
    pub fn new() -> Self { Self }
}

impl Default for BlitzBlock {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blitz_block_params_default_no_defender() {
        let p = BlitzBlockParams::default();
        assert!(p.block_defender_id.is_none());
        assert!(p.multi_block_defender_id.is_none());
    }

    #[test]
    fn blitz_block_params_default_all_bools_false() {
        let p = BlitzBlockParams::default();
        assert!(!p.using_stab);
        assert!(!p.using_chainsaw);
        assert!(!p.using_vomit);
        assert!(!p.frenzy_block);
        assert!(!p.ask_for_block_kind);
        assert!(!p.publish_defender);
        assert!(!p.using_breathe_fire);
        assert!(!p.using_chomp);
    }

    #[test]
    fn blitz_block_struct_is_default() {
        let _ = BlitzBlock::default();
    }
}
