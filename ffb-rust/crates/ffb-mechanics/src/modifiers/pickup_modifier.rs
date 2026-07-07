use crate::modifiers::modifier_type::ModifierType;
use crate::modifiers::pickup_context::PickupContext;

/// 1:1 translation of com.fumbbl.ffb.modifiers.PickupModifier.
pub struct PickupModifier {
    pub name: String,
    pub reporting_string: String,
    pub modifier: i32,
    pub modifier_type: ModifierType,
    applies_to_context: Option<Box<dyn Fn(&PickupContext<'_>) -> bool + Send + Sync>>,
}

impl PickupModifier {
    pub fn new(name: impl Into<String>, modifier: i32, modifier_type: ModifierType) -> Self {
        let n = name.into();
        Self {
            reporting_string: n.clone(),
            name: n,
            modifier,
            modifier_type,
            applies_to_context: None,
        }
    }

    pub fn new_full(
        name: impl Into<String>,
        reporting_string: impl Into<String>,
        modifier: i32,
        modifier_type: ModifierType,
    ) -> Self {
        Self {
            name: name.into(),
            reporting_string: reporting_string.into(),
            modifier,
            modifier_type,
            applies_to_context: None,
        }
    }

    pub fn with_predicate(
        mut self,
        f: impl Fn(&PickupContext<'_>) -> bool + Send + Sync + 'static,
    ) -> Self {
        self.applies_to_context = Some(Box::new(f));
        self
    }

    pub fn get_modifier(&self) -> i32 { self.modifier }
    pub fn get_type(&self) -> ModifierType { self.modifier_type }
    pub fn get_name(&self) -> &str { &self.name }
    pub fn get_report_string(&self) -> &str { &self.reporting_string }

    pub fn is_modifier_included(&self) -> bool {
        self.modifier_type == ModifierType::TACKLEZONE
    }

    pub fn applies_to_context(&self, context: &PickupContext<'_>) -> bool {
        self.applies_to_context.as_ref().map(|f| f(context)).unwrap_or(true)
    }
}

impl Default for PickupModifier {
    fn default() -> Self {
        PickupModifier::new("", 0, ModifierType::REGULAR)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_stores_fields() {
        let m = PickupModifier::new("1 Tacklezone", 1, ModifierType::TACKLEZONE);
        assert_eq!(m.get_name(), "1 Tacklezone");
        assert_eq!(m.get_modifier(), 1);
        assert_eq!(m.get_type(), ModifierType::TACKLEZONE);
    }

    #[test]
    fn tacklezone_is_modifier_included() {
        assert!(PickupModifier::new("tz", 1, ModifierType::TACKLEZONE).is_modifier_included());
    }

    #[test]
    fn regular_is_not_modifier_included() {
        assert!(!PickupModifier::new("x", 0, ModifierType::REGULAR).is_modifier_included());
    }
    #[test]
    fn applies_to_context_returns_true_without_predicate() {
        use ffb_model::enums::Rules;
        use ffb_model::model::{Game, Team, Player};
        let m = PickupModifier::new("x", 1, ModifierType::REGULAR);
        let game = Game::new(
            Team { id: "h".into(), name: "H".into(), race: "x".into(), roster_id: "x".into(), coach: "c".into(),
                rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0, prayers_to_nuffle: 0,
                bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0, assistant_coaches: 0,
                fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0,
                special_rules: vec![], players: vec![], vampire_lord: false, necromancer: false },
            Team { id: "a".into(), name: "A".into(), race: "x".into(), roster_id: "x".into(), coach: "c".into(),
                rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0, prayers_to_nuffle: 0,
                bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0, assistant_coaches: 0,
                fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0,
                special_rules: vec![], players: vec![], vampire_lord: false, necromancer: false },
            Rules::Bb2025,
        );
        let player = Player::default();
        let ctx = crate::modifiers::pickup_context::PickupContext::new(&game, &player);
        assert!(m.applies_to_context(&ctx));
    }
}
