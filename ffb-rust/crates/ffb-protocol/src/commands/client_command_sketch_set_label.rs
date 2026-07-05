/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandSketchSetLabel`.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandSketchSetLabel {
    /// Java: `sketchIds`
    pub sketch_ids: Vec<String>,
    /// Java: `label`
    pub label: Option<String>,
}

impl ClientCommandSketchSetLabel {
    pub fn new() -> Self { Self::default() }

    pub fn with_label(sketch_ids: Vec<String>, label: impl Into<String>) -> Self {
        Self { sketch_ids, label: Some(label.into()) }
    }

    pub fn get_sketch_ids(&self) -> &[String] { &self.sketch_ids }
    pub fn get_label(&self) -> Option<&str> { self.label.as_deref() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields_stored() {
        let cmd = ClientCommandSketchSetLabel::with_label(
            vec!["s1".to_string()],
            "attack",
        );
        assert_eq!(cmd.get_sketch_ids().len(), 1);
        assert_eq!(cmd.get_label(), Some("attack"));
    }

    #[test]
    fn default_is_empty() {
        let cmd = ClientCommandSketchSetLabel::new();
        assert!(cmd.sketch_ids.is_empty());
        assert!(cmd.label.is_none());
    }
}
