use crate::modifiers::jump_context::JumpContext;
use crate::modifiers::modifier_type::ModifierType;

/// 1:1 translation of com.fumbbl.ffb.modifiers.JumpModifier.
pub struct JumpModifier {
    pub name: String,
    pub reporting_string: String,
    pub modifier: i32,
    pub multiplier: i32,
    pub modifier_type: ModifierType,
    applies_to_context: Option<Box<dyn Fn(&JumpContext<'_>) -> bool + Send + Sync>>,
}

impl JumpModifier {
    pub fn new(name: impl Into<String>, modifier: i32, modifier_type: ModifierType) -> Self {
        let n = name.into();
        Self {
            reporting_string: n.clone(),
            name: n,
            modifier,
            multiplier: 1,
            modifier_type,
            applies_to_context: None,
        }
    }

    /// Java: JumpModifier(String name, String reportingString, int modifier, int multiplier, ModifierType type)
    pub fn new_full(
        name: impl Into<String>,
        reporting_string: impl Into<String>,
        modifier: i32,
        multiplier: i32,
        modifier_type: ModifierType,
    ) -> Self {
        Self {
            name: name.into(),
            reporting_string: reporting_string.into(),
            modifier,
            multiplier,
            modifier_type,
            applies_to_context: None,
        }
    }

    pub fn with_predicate(
        mut self,
        f: impl Fn(&JumpContext<'_>) -> bool + Send + Sync + 'static,
    ) -> Self {
        self.applies_to_context = Some(Box::new(f));
        self
    }

    pub fn get_modifier(&self) -> i32 { self.modifier }
    pub fn get_multiplier(&self) -> i32 { self.multiplier }
    pub fn get_type(&self) -> ModifierType { self.modifier_type }
    pub fn get_name(&self) -> &str { &self.name }
    pub fn get_report_string(&self) -> &str { &self.reporting_string }

    pub fn is_modifier_included(&self) -> bool {
        self.modifier_type == ModifierType::TACKLEZONE
            || self.modifier_type == ModifierType::PREHENSILE_TAIL
    }

    pub fn applies_to_context(&self, context: &JumpContext<'_>) -> bool {
        self.applies_to_context.as_ref().map(|f| f(context)).unwrap_or(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_stores_fields() {
        let m = JumpModifier::new("1 Tacklezone", 1, ModifierType::TACKLEZONE);
        assert_eq!(m.get_name(), "1 Tacklezone");
        assert_eq!(m.get_modifier(), 1);
        assert_eq!(m.get_type(), ModifierType::TACKLEZONE);
        assert_eq!(m.get_multiplier(), 1);
    }

    #[test]
    fn tacklezone_is_modifier_included() {
        assert!(JumpModifier::new("tz", 1, ModifierType::TACKLEZONE).is_modifier_included());
    }

    #[test]
    fn regular_is_not_modifier_included() {
        assert!(!JumpModifier::new("x", 0, ModifierType::REGULAR).is_modifier_included());
    }

    #[test]
    fn new_full_sets_multiplier_and_reporting_string() {
        let m = JumpModifier::new_full("Jump", "jump report", 1, 2, ModifierType::TACKLEZONE);
        assert_eq!(m.get_name(), "Jump");
        assert_eq!(m.get_report_string(), "jump report");
        assert_eq!(m.get_multiplier(), 2);
        assert_eq!(m.get_modifier(), 1);
    }

    #[test]
    fn applies_to_context_without_predicate_returns_true() {
        use ffb_model::types::FieldCoordinate;
        let game = {
            use ffb_model::enums::Rules;
            ffb_model::model::Game::new(
                ffb_model::model::Team {
                    id: "home".into(), name: "H".into(), race: "human".into(),
                    roster_id: "human".into(), coach: "c".into(),
                    rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
                    prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0,
                    cheerleaders: 0, assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0,
                    team_value: 0, treasury: 0, special_rules: vec![], players: vec![],
                    vampire_lord: false, necromancer: false,
                },
                ffb_model::model::Team {
                    id: "away".into(), name: "A".into(), race: "human".into(),
                    roster_id: "human".into(), coach: "c".into(),
                    rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
                    prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0,
                    cheerleaders: 0, assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0,
                    team_value: 0, treasury: 0, special_rules: vec![], players: vec![],
                    vampire_lord: false, necromancer: false,
                },
                Rules::Bb2025,
            )
        };
        let player = ffb_model::model::Player::default();
        let m = JumpModifier::new("x", 0, ModifierType::REGULAR);
        let ctx = crate::modifiers::jump_context::JumpContext::new(
            &game, &player,
            FieldCoordinate { x: 0, y: 0 },
            FieldCoordinate { x: 1, y: 1 },
        );
        assert!(m.applies_to_context(&ctx));
    }
}
