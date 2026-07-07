/// 1:1 translation of `com.fumbbl.ffb.net.commands.ServerCommandTeamSetupList`.
/// Sends the list of saved team setup names to the client.
#[derive(Debug, Clone, Default)]
pub struct ServerCommandTeamSetupList {
    /// Java: `fSetupNames` — ordered list of saved setup names.
    pub setup_names: Vec<String>,
}

impl ServerCommandTeamSetupList {
    pub fn new(setup_names: Vec<String>) -> Self { Self { setup_names } }
    pub fn get_setup_names(&self) -> &[String] { &self.setup_names }
    pub fn add_setup_name(&mut self, name: impl Into<String>) { self.setup_names.push(name.into()); }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields_stored() {
        let cmd = ServerCommandTeamSetupList::new(vec!["Wide".to_string(), "Cage".to_string()]);
        assert_eq!(cmd.get_setup_names(), &["Wide", "Cage"]);
    }

    #[test]
    fn add_name() {
        let mut cmd = ServerCommandTeamSetupList::default();
        cmd.add_setup_name("Press");
        assert_eq!(cmd.setup_names.len(), 1);
        assert_eq!(cmd.setup_names[0], "Press");
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ServerCommandTeamSetupList::default()).is_empty());
    }


    #[test]
    fn clone_does_not_panic() {
        let _ = ServerCommandTeamSetupList::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ServerCommandTeamSetupList::default());
        assert!(s.contains("ServerCommandTeamSetupList"));
    }
}
