/// Root-level abstract base for the Select step sequence generator.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.Select`.
// TODO: replace String placeholder with ffb_model::model::BlockTarget once available

#[derive(Debug, Clone, Default)]
pub struct SelectParams {
    /// BlockTarget list — using String as placeholder until BlockTarget is re-exported
    /// from ffb_model. TODO: use proper BlockTarget type.
    pub block_target_ids: Vec<String>,
    pub update_persistence: bool,
}

pub struct Select;

impl Select {
    pub fn new() -> Self { Self }
}

impl Default for Select {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn select_params_default_empty_targets() {
        let p = SelectParams::default();
        assert!(p.block_target_ids.is_empty());
    }

    #[test]
    fn select_params_default_no_update_persistence() {
        let p = SelectParams::default();
        assert!(!p.update_persistence);
    }

    #[test]
    fn select_struct_is_default() {
        let _ = Select::default();
    }
}
