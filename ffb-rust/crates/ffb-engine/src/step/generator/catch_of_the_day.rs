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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catch_of_the_day_params_default_no_label() {
        let p = CatchOfTheDayParams::default();
        assert!(p.failure_label.is_none());
    }

    #[test]
    fn catch_of_the_day_params_can_set_label() {
        let p = CatchOfTheDayParams { failure_label: Some("end".to_string()) };
        assert_eq!(p.failure_label.as_deref(), Some("end"));
    }

    #[test]
    fn catch_of_the_day_struct_is_default() {
        let _ = CatchOfTheDay::default();
    }
}
