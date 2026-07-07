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

    #[test]
    fn getter_returns_slice() {
        let mut cmd = ClientCommandUseIgors::new();
        cmd.injury_description_json.push("{}".into());
        assert_eq!(cmd.get_injury_descriptions().len(), 1);
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ClientCommandUseIgors::default()).is_empty());
    }


    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandUseIgors::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ClientCommandUseIgors::default());
        assert!(s.contains("ClientCommandUseIgors"));
    }
}
