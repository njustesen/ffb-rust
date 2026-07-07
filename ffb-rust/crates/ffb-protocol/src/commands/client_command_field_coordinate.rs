/// 1:1 translation of ClientCommandFieldCoordinate (Java field: fieldCoordinate).
use ffb_model::types::FieldCoordinate;

#[derive(Debug, Clone, Default)]
pub struct ClientCommandFieldCoordinate {
    pub field_coordinate: Option<FieldCoordinate>,
}

impl ClientCommandFieldCoordinate {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_coordinate(c: FieldCoordinate) -> Self {
        Self { field_coordinate: Some(c) }
    }

    pub fn get_field_coordinate(&self) -> Option<FieldCoordinate> {
        self.field_coordinate
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::types::FieldCoordinate;

    #[test]
    fn default_has_no_coordinate() {
        let cmd = ClientCommandFieldCoordinate::new();
        assert!(cmd.get_field_coordinate().is_none());
    }

    #[test]
    fn with_coordinate_stores_value() {
        let coord = FieldCoordinate::new(3, 5);
        let cmd = ClientCommandFieldCoordinate::with_coordinate(coord);
        assert_eq!(cmd.get_field_coordinate(), Some(FieldCoordinate::new(3, 5)));
    }

    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ClientCommandFieldCoordinate::new()).is_empty());
    }


    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandFieldCoordinate::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ClientCommandFieldCoordinate::default());
        assert!(s.contains("ClientCommandFieldCoordinate"));
    }
}
