/// 1:1 translation of ClientCommandTeamSetupLoad (Java field: fSetupName).
#[derive(Debug, Clone, Default)]
pub struct ClientCommandTeamSetupLoad {
    pub setup_name: Option<String>,
}

impl ClientCommandTeamSetupLoad {
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
        let cmd = ClientCommandTeamSetupLoad::new();
        assert!(cmd.get_setup_name().is_none());
    }

    #[test]
    fn with_setup_name_stores_value() {
        let cmd = ClientCommandTeamSetupLoad::with_setup_name("default-setup");
        assert_eq!(cmd.get_setup_name(), Some("default-setup"));
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ClientCommandTeamSetupLoad::default()).is_empty());
    }

}
