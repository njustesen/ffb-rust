use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportPuntDistance.java`.
#[derive(Debug, Clone)]
pub struct ReportPuntDistance {
    pub roll: i32,
    pub out_of_bounds: bool,
}

impl ReportPuntDistance {
    pub fn new(roll: i32, out_of_bounds: bool) -> Self {
        Self { roll, out_of_bounds }
    }

    pub fn get_roll(&self) -> i32 { self.roll }
    pub fn is_out_of_bounds(&self) -> bool { self.out_of_bounds }
}

impl IReport for ReportPuntDistance {
    fn get_id(&self) -> ReportId { ReportId::PUNT_DISTANCE_ROLL }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_id() {
        assert_eq!(ReportPuntDistance::new(4, false).get_id(), ReportId::PUNT_DISTANCE_ROLL);
    }

    #[test]
    fn get_name() {
        assert_eq!(ReportPuntDistance::new(4, false).get_name(), "puntDistanceRoll");
    }

    #[test]
    fn fields() {
        let r = ReportPuntDistance::new(4, true);
        assert_eq!(r.get_roll(), 4);
        assert!(r.is_out_of_bounds());
    }
}
