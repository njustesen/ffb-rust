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
