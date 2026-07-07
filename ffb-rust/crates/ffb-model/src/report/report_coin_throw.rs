use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportCoinThrow.java`.
#[derive(Debug, Clone)]
pub struct ReportCoinThrow {
    pub coin_throw_heads: bool,
    pub coach: String,
    pub coin_choice_heads: bool,
}

impl ReportCoinThrow {
    pub fn new(coin_throw_heads: bool, coach: String, coin_choice_heads: bool) -> Self {
        Self { coin_throw_heads, coach, coin_choice_heads }
    }

    pub fn is_coin_throw_heads(&self) -> bool { self.coin_throw_heads }
    pub fn get_coach(&self) -> &str { &self.coach }
    pub fn is_coin_choice_heads(&self) -> bool { self.coin_choice_heads }
}

impl IReport for ReportCoinThrow {
    fn get_id(&self) -> ReportId { ReportId::COIN_THROW }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportCoinThrow {
        ReportCoinThrow::new(true, "CoachA".into(), false)
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::COIN_THROW);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "coinThrow");
    }

    #[test]
    fn get_coach() {
        assert_eq!(make().get_coach(), "CoachA");
    }

    #[test]
    fn coin_throw_heads() {
        assert!(make().is_coin_throw_heads());
    }

    #[test]
    fn coin_choice_tails() {
        assert!(!make().is_coin_choice_heads());
    }
}
