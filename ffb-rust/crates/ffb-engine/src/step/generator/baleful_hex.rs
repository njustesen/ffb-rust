/// Root-level abstract base for the BalefulHex step sequence generator.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.BalefulHex`.

#[derive(Debug, Clone, Default)]
pub struct BalefulHexParams {
    pub failure_label: Option<String>,
}

pub struct BalefulHex;

impl BalefulHex {
    pub fn new() -> Self { Self }
}

impl Default for BalefulHex {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn baleful_hex_params_default_no_label() {
        let p = BalefulHexParams::default();
        assert!(p.failure_label.is_none());
    }

    #[test]
    fn baleful_hex_params_can_set_label() {
        let p = BalefulHexParams { failure_label: Some("end".to_string()) };
        assert_eq!(p.failure_label.as_deref(), Some("end"));
    }

    #[test]
    fn baleful_hex_struct_is_default() {
        let _ = BalefulHex::default();
    }
}
