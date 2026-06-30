/// Root-level abstract base for the Foul step sequence generator.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.Foul`.

#[derive(Debug, Clone, Default)]
pub struct FoulParams {
    pub fouled_defender_id: Option<String>,
    pub using_chainsaw: bool,
}

pub struct Foul;

impl Foul {
    pub fn new() -> Self { Self }
}

impl Default for Foul {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn foul_params_default_no_defender() {
        let p = FoulParams::default();
        assert!(p.fouled_defender_id.is_none());
    }

    #[test]
    fn foul_params_default_not_using_chainsaw() {
        let p = FoulParams::default();
        assert!(!p.using_chainsaw);
    }

    #[test]
    fn foul_struct_is_default() {
        let _ = Foul::default();
    }
}
