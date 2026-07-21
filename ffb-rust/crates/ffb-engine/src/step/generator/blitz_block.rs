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
