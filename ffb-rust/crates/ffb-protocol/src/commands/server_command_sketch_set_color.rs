/// 1:1 translation of `com.fumbbl.ffb.net.commands.ServerCommandSketchSetColor`.
/// Sets the color of one or more sketches identified by ID.
#[derive(Debug, Clone, Default)]
pub struct ServerCommandSketchSetColor {
    /// Java: `coach` — coach who owns the sketches.
    pub coach: String,
    /// Java: `sketchIds` — IDs of sketches to recolor.
    pub sketch_ids: Vec<String>,
    /// Java: `rbg` — packed RGB color value (note: Java field is named `rbg`).
    pub rbg: i32,
}

impl ServerCommandSketchSetColor {
    pub fn new(coach: impl Into<String>, sketch_ids: Vec<String>, rbg: i32) -> Self {
        Self { coach: coach.into(), sketch_ids, rbg }
    }
    pub fn get_coach(&self) -> &str { &self.coach }
    pub fn get_sketch_ids(&self) -> &[String] { &self.sketch_ids }
    pub fn get_rbg(&self) -> i32 { self.rbg }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields_stored() {
        let ids = vec!["s1".to_string(), "s2".to_string()];
        let cmd = ServerCommandSketchSetColor::new("Bob", ids.clone(), 0xFF0000);
        assert_eq!(cmd.get_coach(), "Bob");
        assert_eq!(cmd.get_sketch_ids(), ids.as_slice());
        assert_eq!(cmd.get_rbg(), 0xFF0000);
    }

    #[test]
    fn default_empty() {
        let cmd = ServerCommandSketchSetColor::default();
        assert!(cmd.coach.is_empty());
        assert!(cmd.sketch_ids.is_empty());
        assert_eq!(cmd.rbg, 0);
    }
}
