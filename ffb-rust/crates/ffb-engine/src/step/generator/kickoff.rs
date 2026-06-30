/// Root-level abstract base for the Kickoff step sequence generator.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.Kickoff`.

#[derive(Debug, Clone, Default)]
pub struct KickoffParams {
    pub with_coin_choice: bool,
}

pub struct Kickoff;

impl Kickoff {
    pub fn new() -> Self { Self }
}

impl Default for Kickoff {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kickoff_params_default_no_coin_choice() {
        let p = KickoffParams::default();
        assert!(!p.with_coin_choice);
    }

    #[test]
    fn kickoff_params_can_set_coin_choice() {
        let p = KickoffParams { with_coin_choice: true };
        assert!(p.with_coin_choice);
    }

    #[test]
    fn kickoff_struct_is_default() {
        let _ = Kickoff::default();
    }
}
