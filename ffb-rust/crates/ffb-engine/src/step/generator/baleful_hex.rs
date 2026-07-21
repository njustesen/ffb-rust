/// Root-level abstract base for the BalefulHex step sequence generator.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.BalefulHex`.

#[derive(Debug, Clone, Default)]
pub struct BalefulHexParams {
    pub failure_label: Option<String>,
}

pub struct BalefulHex;

impl BalefulHex {
    pub fn new() -> Self { Self }
}

impl Default for BalefulHex {
    fn default() -> Self { Self::new() }
}
