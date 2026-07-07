/// 1:1 translation of `com.fumbbl.ffb.net.commands.ServerCommandRemoveSketches`.
/// Instructs clients to remove specific sketches by ID.
#[derive(Debug, Clone, Default)]
pub struct ServerCommandRemoveSketches {
    /// Java: `coach` — the coach whose sketches are being removed.
    pub coach: String,
    /// Java: `ids` — sketch IDs to remove.
    pub ids: Vec<String>,
}

impl ServerCommandRemoveSketches {
    pub fn new(coach: impl Into<String>, ids: Vec<String>) -> Self {
        Self { coach: coach.into(), ids }
    }
    pub fn get_coach(&self) -> &str { &self.coach }
    pub fn get_ids(&self) -> &[String] { &self.ids }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields_stored() {
        let cmd = ServerCommandRemoveSketches::new("Bob", vec!["id1".into()]);
        assert_eq!(cmd.get_coach(), "Bob");
        assert_eq!(cmd.get_ids(), &["id1"]);
    }

    #[test]
    fn default_empty() {
        let cmd = ServerCommandRemoveSketches::default();
        assert!(cmd.ids.is_empty());
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ServerCommandRemoveSketches::default()).is_empty());
    }


    #[test]
    fn clone_does_not_panic() {
        let _ = ServerCommandRemoveSketches::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ServerCommandRemoveSketches::default());
        assert!(s.contains("ServerCommandRemoveSketches"));
    }
}
