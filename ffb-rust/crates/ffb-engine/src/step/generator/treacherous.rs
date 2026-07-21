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
