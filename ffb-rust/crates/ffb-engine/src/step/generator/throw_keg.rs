/// Root-level abstract base for the ThrowKeg step sequence generator.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.ThrowKeg`.

#[derive(Debug, Clone, Default)]
pub struct ThrowKegParams {
    pub player_id: Option<String>,
}

pub struct ThrowKeg;

impl ThrowKeg {
    pub fn new() -> Self { Self }
}

impl Default for ThrowKeg {
    fn default() -> Self { Self::new() }
}
