/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandRemoveSketches`.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandRemoveSketches {
    /// Java: `ids`
    pub ids: Vec<String>,
}

impl ClientCommandRemoveSketches {
    pub fn new() -> Self { Self::default() }

    pub fn with_ids(ids: Vec<String>) -> Self {
        Self { ids }
    }

    pub fn get_ids(&self) -> &[String] { &self.ids }

    pub fn add_id(&mut self, id: impl Into<String>) {
        self.ids.push(id.into());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_stored() {
        let cmd = ClientCommandRemoveSketches::with_ids(vec!["id1".to_string(), "id2".to_string()]);
        assert_eq!(cmd.get_ids().len(), 2);
        assert_eq!(cmd.get_ids()[0], "id1");
    }

    #[test]
    fn default_is_empty() {
        let cmd = ClientCommandRemoveSketches::new();
        assert!(cmd.ids.is_empty());
    }

    #[test]
    fn add_id_increments_len() {
        let mut cmd = ClientCommandRemoveSketches::new();
        cmd.add_id("sk-1");
        assert_eq!(cmd.get_ids().len(), 1);
        cmd.add_id("sk-2");
        assert_eq!(cmd.get_ids().len(), 2);
    }

    #[test]
    fn debug_format_nonempty() {
        let cmd = ClientCommandRemoveSketches::default();
        assert!(!format!("{cmd:?}").is_empty());
    }

    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandRemoveSketches::default().clone();
    }
}
