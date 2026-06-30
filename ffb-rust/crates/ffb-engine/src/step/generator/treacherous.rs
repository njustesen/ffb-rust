/// Root-level abstract base for the Treacherous step sequence generator.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.Treacherous`.

#[derive(Debug, Clone, Default)]
pub struct TreacherousParams {
    pub failure_label: Option<String>,
}

pub struct Treacherous;

impl Treacherous {
    pub fn new() -> Self { Self }
}

impl Default for Treacherous {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn treacherous_params_default_no_label() {
        let p = TreacherousParams::default();
        assert!(p.failure_label.is_none());
    }

    #[test]
    fn treacherous_params_can_set_label() {
        let p = TreacherousParams { failure_label: Some("end".to_string()) };
        assert_eq!(p.failure_label.as_deref(), Some("end"));
    }

    #[test]
    fn treacherous_struct_is_default() {
        let _ = Treacherous::default();
    }
}
