/// Root-level abstract base for the RadingParty step sequence generator.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.RadingParty`.

#[derive(Debug, Clone, Default)]
pub struct RadingPartyParams {
    pub failure_label: Option<String>,
    pub success_label: Option<String>,
}

pub struct RadingParty;

impl RadingParty {
    pub fn new() -> Self { Self }
}

impl Default for RadingParty {
    fn default() -> Self { Self::new() }
}
