/// Root-level abstract base for the EndTurn step sequence generator.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.EndTurn`.

#[derive(Debug, Clone, Default)]
pub struct EndTurnParams {
    pub check_forgo: bool,
}

pub struct EndTurn;

impl EndTurn {
    pub fn new() -> Self { Self }
}

impl Default for EndTurn {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn end_turn_params_default_no_forgo() {
        let p = EndTurnParams::default();
        assert!(!p.check_forgo);
    }

    #[test]
    fn end_turn_params_can_set_forgo() {
        let p = EndTurnParams { check_forgo: true };
        assert!(p.check_forgo);
    }

    #[test]
    fn end_turn_struct_is_default() {
        let _ = EndTurn::default();
    }

    #[test]
    fn params_with_fields_set() {
        let p = EndTurnParams { check_forgo: true };
        assert!(p.check_forgo);
    }

    #[test]
    fn params_clone() {
        let p = EndTurnParams { check_forgo: true };
        let q = p.clone();
        assert!(q.check_forgo);
    }
}
