/// 1:1 translation of `com.fumbbl.ffb.net.commands.ServerCommandSketchSetLabel`.
/// Sets the text label of one or more sketches identified by ID.
#[derive(Debug, Clone, Default)]
pub struct ServerCommandSketchSetLabel {
    /// Java: `coach` — coach who owns the sketches.
    pub coach: String,
    /// Java: `sketchIds` — IDs of sketches to relabel.
    pub sketch_ids: Vec<String>,
    /// Java: `label` — the new label text.
    pub label: String,
}

impl ServerCommandSketchSetLabel {
    pub fn new(
        coach: impl Into<String>,
        sketch_ids: Vec<String>,
        label: impl Into<String>,
    ) -> Self {
        Self { coach: coach.into(), sketch_ids, label: label.into() }
    }
    pub fn get_coach(&self) -> &str { &self.coach }
    pub fn get_sketch_ids(&self) -> &[String] { &self.sketch_ids }
    pub fn get_label(&self) -> &str { &self.label }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields_stored() {
        let ids = vec!["s1".to_string()];
        let cmd = ServerCommandSketchSetLabel::new("Carol", ids.clone(), "Arrow");
        assert_eq!(cmd.get_coach(), "Carol");
        assert_eq!(cmd.get_sketch_ids(), ids.as_slice());
        assert_eq!(cmd.get_label(), "Arrow");
    }

    #[test]
    fn default_empty() {
        let cmd = ServerCommandSketchSetLabel::default();
        assert!(cmd.coach.is_empty());
        assert!(cmd.label.is_empty());
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ServerCommandSketchSetLabel::default()).is_empty());
    }


    #[test]
    fn clone_does_not_panic() {
        let _ = ServerCommandSketchSetLabel::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ServerCommandSketchSetLabel::default());
        assert!(s.contains("ServerCommandSketchSetLabel"));
    }
}
