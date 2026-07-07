use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportBiasedRef.java`.
#[derive(Debug, Clone)]
pub struct ReportBiasedRef {
    pub foul_spotted: bool,
    pub roll: i32,
}

impl ReportBiasedRef {
    pub fn new(foul_spotted: bool, roll: i32) -> Self {
        Self { foul_spotted, roll }
    }

    pub fn is_foul_spotted(&self) -> bool { self.foul_spotted }
    pub fn get_roll(&self) -> i32 { self.roll }
}

impl IReport for ReportBiasedRef {
    fn get_id(&self) -> ReportId { ReportId::BIASED_REF }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportBiasedRef {
        ReportBiasedRef::new(true, 3)
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::BIASED_REF); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "biasedRef"); }

    #[test]
    fn get_roll() { assert_eq!(make().get_roll(), 3); }

    #[test]
    fn is_foul_spotted() { assert!(make().is_foul_spotted()); }

    #[test]
    fn not_foul_spotted() {
        let r = ReportBiasedRef::new(false, 1);
        assert!(!r.is_foul_spotted());
        assert_eq!(r.get_roll(), 1);
    }
}
