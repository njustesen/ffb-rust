/// Root-level abstract base for the CatchOfTheDay step sequence generator.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.CatchOfTheDay`.

#[derive(Debug, Clone, Default)]
pub struct CatchOfTheDayParams {
    pub failure_label: Option<String>,
}

pub struct CatchOfTheDay;

impl CatchOfTheDay {
    pub fn new() -> Self { Self }
}

impl Default for CatchOfTheDay {
    fn default() -> Self { Self::new() }
}
