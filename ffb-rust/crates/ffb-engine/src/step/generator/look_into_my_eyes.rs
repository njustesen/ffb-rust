/// Root-level abstract base for the LookIntoMyEyes step sequence generator.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.LookIntoMyEyes`.

#[derive(Debug, Clone, Default)]
pub struct LookIntoMyEyesParams {
    pub push_select: bool,
    pub goto_on_end: Option<String>,
}

pub struct LookIntoMyEyes;

impl LookIntoMyEyes {
    pub fn new() -> Self { Self }
}

impl Default for LookIntoMyEyes {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn look_into_my_eyes_params_default_no_push_select() {
        let p = LookIntoMyEyesParams::default();
        assert!(!p.push_select);
    }

    #[test]
    fn look_into_my_eyes_params_default_no_goto() {
        let p = LookIntoMyEyesParams::default();
        assert!(p.goto_on_end.is_none());
    }

    #[test]
    fn look_into_my_eyes_struct_is_default() {
        let _ = LookIntoMyEyes::default();
    }
}
