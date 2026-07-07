/// 1:1 translation of ClientCommandTeamSetupDelete (Java field: fSetupName).
#[derive(Debug, Clone, Default)]
pub struct ClientCommandTeamSetupDelete {
    pub setup_name: Option<String>,
}

impl ClientCommandTeamSetupDelete {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_setup_name(name: impl Into<String>) -> Self {
        Self { setup_name: Some(name.into()) }
    }

    pub fn get_setup_name(&self) -> Option<&str> {
        self.setup_name.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_has_no_setup_name() {
        let cmd = ClientCommandTeamSetupDelete::new();
        assert!(cmd.get_setup_name().is_none());
    }

    #[test]
    fn with_setup_name_stores_value() {
        let cmd = ClientCommandTeamSetupDelete::with_setup_name("my-setup");
        assert_eq!(cmd.get_setup_name(), Some("my-setup"));
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ClientCommandTeamSetupDelete::default()).is_empty());
    }


    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandTeamSetupDelete::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ClientCommandTeamSetupDelete::default());
        assert!(s.contains("ClientCommandTeamSetupDelete"));
    }
}
