/// 1:1 translation of com.fumbbl.ffb.server.util.UtilServerInjury.
///
/// Static utility methods for injury handling: evaluating injury contexts, regeneration,
/// injury side-effects (raise dead, pump-up), and dropping players.
///
/// Stub — full method bodies are in `crate::step::util_server_injury` (step-layer utility module).
/// This outer module exists to satisfy import paths; all real logic lives in the step module.
pub struct UtilServerInjury;

impl UtilServerInjury {
    pub fn new() -> Self { Self }
}

impl Default for UtilServerInjury {
    fn default() -> Self { Self::new() }
}
