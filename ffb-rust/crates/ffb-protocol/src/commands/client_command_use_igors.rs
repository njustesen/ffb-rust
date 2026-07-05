/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandUseIgors`.
/// Sent when Necromantic team uses Igors for injury recovery.
/// Note: InjuryDescription serialised as raw JSON strings; full type not yet ported.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandUseIgors {
    /// Java: `injuryDescriptions` — simplified to JSON strings pending InjuryDescription port.
    pub injury_description_json: Vec<String>,
}

impl ClientCommandUseIgors {
    pub fn new() -> Self { Self::default() }
    pub fn get_injury_descriptions(&self) -> &[String] { &self.injury_description_json }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default_empty() {
        assert!(ClientCommandUseIgors::new().injury_description_json.is_empty());
    }
}
