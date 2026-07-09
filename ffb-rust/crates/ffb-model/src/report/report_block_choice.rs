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

impl ReportBlockChoice {
    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "nrOfDice": self.nr_of_dice,
            "blockRoll": self.block_roll,
            "diceIndex": self.dice_index,
            "blockResult": self.block_result,
            "defenderId": self.defender_id,
            "suppressExtraEffectHandling": self.suppress_extra_effect_handling,
            "showNameInReport": self.show_name_in_report,
            "blockRollId": self.block_roll_id,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            nr_of_dice: json["nrOfDice"].as_i64().unwrap_or(0) as i32,
            block_roll: json["blockRoll"].as_array()
                .map(|a| a.iter().map(|v| v.as_i64().unwrap_or(0) as i32).collect())
                .unwrap_or_default(),
            dice_index: json["diceIndex"].as_i64().unwrap_or(0) as i32,
            block_roll_id: json["blockRollId"].as_i64().unwrap_or(0) as i32,
            block_result: json["blockResult"].as_str().unwrap_or("").to_string(),
            defender_id: json["defenderId"].as_str().unwrap_or("").to_string(),
            suppress_extra_effect_handling: json["suppressExtraEffectHandling"].as_bool().unwrap_or(false),
            show_name_in_report: json["showNameInReport"].as_bool().unwrap_or(false),
        }
    }
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

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportBlockChoice::from_json(&json);
        assert_eq!(restored.nr_of_dice, original.nr_of_dice);
        assert_eq!(restored.block_roll, original.block_roll);
        assert_eq!(restored.dice_index, original.dice_index);
        assert_eq!(restored.block_roll_id, original.block_roll_id);
        assert_eq!(restored.block_result, original.block_result);
        assert_eq!(restored.defender_id, original.defender_id);
        assert_eq!(restored.suppress_extra_effect_handling, original.suppress_extra_effect_handling);
        assert_eq!(restored.show_name_in_report, original.show_name_in_report);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("blockChoice"));
    }
}
