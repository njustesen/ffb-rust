/// 1:1 translation of com.fumbbl.ffb.server.util.UtilServerInjury.
///
/// Static utility methods for injury handling: evaluating injury contexts, regeneration,
/// injury side-effects (raise dead, pump-up), and dropping players.
///
/// DEFERRED: all methods require GameState and IStep which are not yet fully ported.
/// Method signatures are declared so callers can compile; bodies are no-ops or DEFERRED.
pub struct UtilServerInjury;

impl UtilServerInjury {
    pub fn new() -> Self { Self }
}

impl Default for UtilServerInjury {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn struct_can_be_created() {
        let _ = UtilServerInjury::new();
    }

    #[test]
    fn default_creates_instance() {
        let _ = UtilServerInjury::default();
    }
}
