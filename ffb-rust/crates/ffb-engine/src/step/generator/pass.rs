/// Root-level abstract base for the Pass step sequence generator.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.Pass`.
use ffb_model::types::FieldCoordinate;

#[derive(Debug, Clone, Default)]
pub struct PassParams {
    pub target_coordinate: Option<FieldCoordinate>,
}

pub struct Pass;

impl Pass {
    pub fn new() -> Self { Self }
}

impl Default for Pass {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pass_params_default_no_target() {
        let p = PassParams::default();
        assert!(p.target_coordinate.is_none());
    }

    #[test]
    fn pass_params_can_set_target() {
        let coord = FieldCoordinate::new(5, 7);
        let p = PassParams { target_coordinate: Some(coord) };
        assert!(p.target_coordinate.is_some());
    }

    #[test]
    fn pass_struct_is_default() {
        let _ = Pass::default();
    }

    #[test]
    fn params_with_fields_set() {
        let coord = FieldCoordinate::new(3, 5);
        let p = PassParams { target_coordinate: Some(coord) };
        assert!(p.target_coordinate.is_some());
    }

    #[test]
    fn params_clone() {
        let coord = FieldCoordinate::new(2, 4);
        let p = PassParams { target_coordinate: Some(coord) };
        let q = p.clone();
        assert!(q.target_coordinate.is_some());
    }
}
