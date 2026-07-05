use ffb_model::types::FieldCoordinate;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandSketchAddCoordinate`.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandSketchAddCoordinate {
    /// Java: `sketchId`
    pub sketch_id: Option<String>,
    /// Java: `coordinate`
    pub coordinate: Option<FieldCoordinate>,
}

impl ClientCommandSketchAddCoordinate {
    pub fn new() -> Self { Self::default() }

    pub fn with_sketch(sketch_id: impl Into<String>, coordinate: FieldCoordinate) -> Self {
        Self { sketch_id: Some(sketch_id.into()), coordinate: Some(coordinate) }
    }

    pub fn get_sketch_id(&self) -> Option<&str> { self.sketch_id.as_deref() }
    pub fn get_coordinate(&self) -> Option<FieldCoordinate> { self.coordinate }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields_stored() {
        let coord = FieldCoordinate::new(2, 9);
        let cmd = ClientCommandSketchAddCoordinate::with_sketch("sketch1", coord);
        assert_eq!(cmd.get_sketch_id(), Some("sketch1"));
        assert_eq!(cmd.get_coordinate(), Some(coord));
    }

    #[test]
    fn default_is_empty() {
        let cmd = ClientCommandSketchAddCoordinate::new();
        assert!(cmd.sketch_id.is_none());
        assert!(cmd.coordinate.is_none());
    }
}
