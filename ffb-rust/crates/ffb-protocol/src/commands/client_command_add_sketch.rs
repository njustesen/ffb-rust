/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandAddSketch`.
/// Java holds a `Sketch` which is a client-side rendering class (path data, color, label).
/// Full Sketch serialization is deferred; only the sketch ID is carried here for now.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandAddSketch {
    /// Identifies the sketch being added. Full Sketch serialization is DEFERRED.
    pub sketch_id: Option<String>,
}

impl ClientCommandAddSketch {
    pub fn new() -> Self { Self::default() }

    pub fn with_sketch_id(sketch_id: impl Into<String>) -> Self {
        Self { sketch_id: Some(sketch_id.into()) }
    }

    pub fn get_sketch_id(&self) -> Option<&str> { self.sketch_id.as_deref() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sketch_id_stored() {
        let cmd = ClientCommandAddSketch::with_sketch_id("sk-123");
        assert_eq!(cmd.get_sketch_id(), Some("sk-123"));
    }

    #[test]
    fn default_is_none() {
        let cmd = ClientCommandAddSketch::new();
        assert!(cmd.sketch_id.is_none());
    }

    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ClientCommandAddSketch::new()).is_empty());
    }


    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandAddSketch::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ClientCommandAddSketch::default());
        assert!(s.contains("ClientCommandAddSketch"));
    }
}
