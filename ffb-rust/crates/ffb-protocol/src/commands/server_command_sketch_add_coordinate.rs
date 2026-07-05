use ffb_model::types::FieldCoordinate;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ServerCommandSketchAddCoordinate`.
/// Adds a point to a sketch path on the field.
#[derive(Debug, Clone)]
pub struct ServerCommandSketchAddCoordinate {
    /// Java: `coach` — coach who owns the sketch.
    pub coach: String,
    /// Java: `sketchId` — the sketch being extended.
    pub sketch_id: String,
    /// Java: `coordinate` — the new field coordinate to add.
    pub coordinate: FieldCoordinate,
}

impl ServerCommandSketchAddCoordinate {
    pub fn new(
        coach: impl Into<String>,
        sketch_id: impl Into<String>,
        coordinate: FieldCoordinate,
    ) -> Self {
        Self { coach: coach.into(), sketch_id: sketch_id.into(), coordinate }
    }
    pub fn get_coach(&self) -> &str { &self.coach }
    pub fn get_sketch_id(&self) -> &str { &self.sketch_id }
    pub fn get_coordinate(&self) -> FieldCoordinate { self.coordinate }
}

impl Default for ServerCommandSketchAddCoordinate {
    fn default() -> Self {
        Self {
            coach: String::new(),
            sketch_id: String::new(),
            coordinate: FieldCoordinate::new(0, 0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields_stored() {
        let cmd = ServerCommandSketchAddCoordinate::new("Alice", "sk1", FieldCoordinate::new(5, 3));
        assert_eq!(cmd.get_coach(), "Alice");
        assert_eq!(cmd.get_sketch_id(), "sk1");
        assert_eq!(cmd.get_coordinate(), FieldCoordinate::new(5, 3));
    }
}
