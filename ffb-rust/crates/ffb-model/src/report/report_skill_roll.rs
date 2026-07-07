/// 1:1 translation of `ReportSkillRoll.java` — abstract base for skill-roll reports.
/// Rust does not have inheritance, so concrete reports embed or copy these fields.
/// This struct can be used as a field in concrete report structs.
#[derive(Debug, Clone, Default)]
pub struct ReportSkillRoll {
    pub player_id: Option<String>,
    pub successful: bool,
    pub roll: i32,
    pub minimum_roll: i32,
    pub re_rolled: bool,
    /// Modifier names (sorted) — replaces `List<RollModifier<?>>`.
    pub roll_modifier_names: Vec<String>,
}

impl ReportSkillRoll {
    pub fn new(
        player_id: Option<String>,
        successful: bool,
        roll: i32,
        minimum_roll: i32,
        re_rolled: bool,
        roll_modifiers: Vec<String>,
    ) -> Self {
        let mut roll_modifier_names = roll_modifiers;
        roll_modifier_names.sort();
        Self { player_id, successful, roll, minimum_roll, re_rolled, roll_modifier_names }
    }

    pub fn get_player_id(&self) -> Option<&str> {
        self.player_id.as_deref()
    }

    pub fn is_successful(&self) -> bool {
        self.successful
    }

    pub fn get_roll(&self) -> i32 {
        self.roll
    }

    pub fn get_minimum_roll(&self) -> i32 {
        self.minimum_roll
    }

    pub fn is_re_rolled(&self) -> bool {
        self.re_rolled
    }

    pub fn get_roll_modifiers(&self) -> &[String] {
        &self.roll_modifier_names
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modifiers_sorted() {
        let r = ReportSkillRoll::new(None, true, 4, 2, false, vec!["Zon".into(), "Alpha".into()]);
        assert_eq!(r.roll_modifier_names, vec!["Alpha", "Zon"]);
    }

    #[test]
    fn is_re_rolled() {
        let r = ReportSkillRoll::new(Some("p1".into()), false, 1, 3, true, vec![]);
        assert!(r.is_re_rolled());
        assert!(!r.is_successful());
    }

    #[test]
    fn default_has_no_player_id() {
        let r = ReportSkillRoll::default();
        assert!(r.get_player_id().is_none());
        assert!(!r.is_successful());
        assert_eq!(r.get_roll(), 0);
        assert_eq!(r.get_minimum_roll(), 0);
    }

    #[test]
    fn get_player_id_and_roll_accessors() {
        let r = ReportSkillRoll::new(Some("p2".into()), true, 5, 3, false, vec![]);
        assert_eq!(r.get_player_id(), Some("p2"));
        assert_eq!(r.get_roll(), 5);
        assert_eq!(r.get_minimum_roll(), 3);
    }

    #[test]
    fn modifiers_empty_when_none_provided() {
        let r = ReportSkillRoll::new(None, true, 4, 2, false, vec![]);
        assert!(r.get_roll_modifiers().is_empty());
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ReportSkillRoll::default()).is_empty());
    }

}
