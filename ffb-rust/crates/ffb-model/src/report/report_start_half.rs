use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportStartHalf.java`.
#[derive(Debug, Clone)]
pub struct ReportStartHalf {
    pub half: i32,
}

impl ReportStartHalf {
    pub fn new(half: i32) -> Self {
        Self { half }
    }

    pub fn get_half(&self) -> i32 { self.half }
}

impl IReport for ReportStartHalf {
    fn get_id(&self) -> ReportId { ReportId::START_HALF }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportStartHalf {
        ReportStartHalf::new(1)
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::START_HALF);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "startHalf");
    }

    #[test]
    fn get_half() {
        assert_eq!(make().get_half(), 1);
        assert_eq!(ReportStartHalf::new(2).get_half(), 2);
    }
}
