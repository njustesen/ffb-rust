/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandSketchSetColor`.
/// Java field is named `rbg` (sic — typo for rgb) but we use `rgb` in Rust.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandSketchSetColor {
    /// Java: `sketchIds`
    pub sketch_ids: Vec<String>,
    /// Java: `rbg` (sic — typo for rgb in Java source)
    pub rgb: i32,
}

impl ClientCommandSketchSetColor {
    pub fn new() -> Self { Self::default() }

    pub fn with_color(sketch_ids: Vec<String>, rgb: i32) -> Self {
        Self { sketch_ids, rgb }
    }

    pub fn get_sketch_ids(&self) -> &[String] { &self.sketch_ids }
    pub fn get_rgb(&self) -> i32 { self.rgb }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields_stored() {
        let cmd = ClientCommandSketchSetColor::with_color(
            vec!["s1".to_string(), "s2".to_string()],
            0xFF0000,
        );
        assert_eq!(cmd.get_sketch_ids().len(), 2);
        assert_eq!(cmd.get_rgb(), 0xFF0000);
    }

    #[test]
    fn default_is_empty() {
        let cmd = ClientCommandSketchSetColor::new();
        assert!(cmd.sketch_ids.is_empty());
        assert_eq!(cmd.rgb, 0);
    }
}
