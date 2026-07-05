/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandPositionSelection`.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandPositionSelection {
    /// Java: `position` (String[])
    pub position: Vec<String>,
    /// Java: `teamId`
    pub team_id: Option<String>,
}

impl ClientCommandPositionSelection {
    pub fn new() -> Self { Self::default() }

    pub fn with_team(team_id: impl Into<String>, position: Vec<String>) -> Self {
        Self { position, team_id: Some(team_id.into()) }
    }

    pub fn get_position(&self) -> &[String] { &self.position }
    pub fn get_team_id(&self) -> Option<&str> { self.team_id.as_deref() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields_stored() {
        let cmd = ClientCommandPositionSelection::with_team(
            "team1",
            vec!["Lineman".to_string(), "Blitzer".to_string()],
        );
        assert_eq!(cmd.get_team_id(), Some("team1"));
        assert_eq!(cmd.get_position().len(), 2);
    }

    #[test]
    fn default_is_empty() {
        let cmd = ClientCommandPositionSelection::new();
        assert!(cmd.team_id.is_none());
        assert!(cmd.position.is_empty());
    }
}
