pub mod engine;
pub mod agent;
pub mod action;
pub mod legal_actions;

/// Parity debug tracing, enabled by setting the FFB_TRACE env var.
/// Used to gate stderr diagnostics (dodge rolls, negatrait rolls, agent picks)
/// that align with the Java ParityRunner's -Dffb.diceTrace output.
pub fn parity_trace_enabled() -> bool {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var_os("FFB_TRACE").is_some())
}
