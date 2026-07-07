/// 1:1 translation of com.fumbbl.ffb.net.commands.ClientCommandUseApothecaries.
///
/// Java: `fInjuryDescriptions` is a List<InjuryDescription>. InjuryDescription is a complex
/// object — DEFERRED: stored as raw JSON strings until InjuryDescription is fully translated.

#[derive(Debug, Clone, Default)]
pub struct ClientCommandUseApothecaries {
    /// Java: `fInjuryDescriptions` — DEFERRED: InjuryDescription is complex; stored as JSON strings.
    pub injury_description_json: Vec<String>,
}

impl ClientCommandUseApothecaries {
    pub fn new() -> Self {
        Self::default()
    }

    /// Java: `getInjuryDescriptions()` — DEFERRED: returns raw JSON strings for each InjuryDescription.
    pub fn get_injury_description_json(&self) -> &[String] {
        &self.injury_description_json
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_has_empty_descriptions() {
        let cmd = ClientCommandUseApothecaries::new();
        assert!(cmd.get_injury_description_json().is_empty());
    }

    #[test]
    fn stores_injury_descriptions() {
        let cmd = ClientCommandUseApothecaries {
            injury_description_json: vec![
                r#"{"playerId":"p1","apothecaryUsed":true}"#.to_string(),
            ],
        };
        assert_eq!(cmd.get_injury_description_json().len(), 1);
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ClientCommandUseApothecaries::default()).is_empty());
    }

}
