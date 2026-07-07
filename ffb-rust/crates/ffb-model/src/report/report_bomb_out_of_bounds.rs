use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportBombOutOfBounds.java`.
/// No fields — the report carries only its identity.
#[derive(Debug, Clone, Default)]
pub struct ReportBombOutOfBounds;

impl ReportBombOutOfBounds {
    pub fn new() -> Self { Self }
}

impl IReport for ReportBombOutOfBounds {
    fn get_id(&self) -> ReportId { ReportId::BOMB_OUT_OF_BOUNDS }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_id() {
        assert_eq!(ReportBombOutOfBounds::new().get_id(), ReportId::BOMB_OUT_OF_BOUNDS);
    }

    #[test]
    fn get_name() {
        assert_eq!(ReportBombOutOfBounds::new().get_name(), "bombOutOfBounds");
    }

    #[test]
    fn default_works() {
        let r = ReportBombOutOfBounds::default();
        assert_eq!(r.get_id(), ReportId::BOMB_OUT_OF_BOUNDS);
    }

    #[test]
    fn new_and_default_same_name() {
        assert_eq!(ReportBombOutOfBounds::new().get_name(), ReportBombOutOfBounds::default().get_name());
    }

    #[test]
    fn clone_preserves_id() {
        let r = ReportBombOutOfBounds::new();
        let c = r.clone();
        assert_eq!(c.get_id(), ReportId::BOMB_OUT_OF_BOUNDS);
    }
}
