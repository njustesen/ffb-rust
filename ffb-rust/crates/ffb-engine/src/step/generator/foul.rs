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
