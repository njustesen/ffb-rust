use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportBlockChoice.java`.
/// `BlockResult` is stored as its name string (not yet a full Rust type).
#[derive(Debug, Clone)]
pub struct ReportBlockChoice {
    pub nr_of_dice: i32,
    pub block_roll: Vec<i32>,
    pub dice_index: i32,
    pub block_roll_id: i32,
    /// `BlockResult` name (replaces the Java `BlockResult` object).
    pub block_result: String,
    pub defender_id: String,
    /// Renamed from `suppressExtraEffectHandling` (negated boolean kept for backwards compat in Java).
    pub suppress_extra_effect_handling: bool,
    pub show_name_in_report: bool,
}

impl ReportBlockChoice {
    pub fn new(
        nr_of_dice: i32,
        block_roll: Vec<i32>,
        dice_index: i32,
        block_result: String,
        defender_id: String,
        suppress_extra_effect_handling: bool,
        show_name_in_report: bool,
        block_roll_id: i32,
    ) -> Self {
        Self {
            nr_of_dice,
            block_roll,
            dice_index,
            block_roll_id,
            block_result,
            defender_id,
            suppress_extra_effect_handling,
            show_name_in_report,
        }
    }

    pub fn get_nr_of_dice(&self) -> i32 { self.nr_of_dice }
    pub fn get_block_roll(&self) -> &[i32] { &self.block_roll }
    pub fn get_dice_index(&self) -> i32 { self.dice_index }
    pub fn get_block_result(&self) -> &str { &self.block_result }
    pub fn get_defender_id(&self) -> &str { &self.defender_id }
    pub fn is_suppress_extra_effect_handling(&self) -> bool { self.suppress_extra_effect_handling }
    pub fn is_show_name_in_report(&self) -> bool { self.show_name_in_report }
    pub fn get_block_roll_id(&self) -> i32 { self.block_roll_id }
}

impl IReport for ReportBlockChoice {
    fn get_id(&self) -> ReportId { ReportId::BLOCK_CHOICE }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportBlockChoice {
        ReportBlockChoice::new(
            2,
            vec![3, 5],
            1,
            "PUSHED".into(),
            "def1".into(),
            false,
            true,
            42,
        )
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::BLOCK_CHOICE);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "blockChoice");
    }

    #[test]
    fn get_nr_of_dice() {
        assert_eq!(make().get_nr_of_dice(), 2);
    }

    #[test]
    fn block_roll_and_result() {
        let r = make();
        assert_eq!(r.get_block_roll(), &[3, 5]);
        assert_eq!(r.get_block_result(), "PUSHED");
    }

    #[test]
    fn defender_and_flags() {
        let r = make();
        assert_eq!(r.get_defender_id(), "def1");
        assert!(!r.is_suppress_extra_effect_handling());
        assert!(r.is_show_name_in_report());
        assert_eq!(r.get_block_roll_id(), 42);
    }
}
