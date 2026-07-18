use ffb_model::model::Player;
use crate::modifiers::armor_modifier::ArmorModifier;
use crate::modifiers::armor_modifier_context::ArmorModifierContext;

/// 1:1 translation of com.fumbbl.ffb.modifiers.VariableArmourModifier.
/// getModifier uses attacker.getSkillIntValue(registeredTo) — currently stubbed to 0.
pub struct VariableArmourModifier {
    pub name: String,
    pub foul_assist_modifier: bool,
    pub registered_to: Option<String>,
    modifier_fn: Option<Box<dyn Fn(Option<&Player>, &Player) -> i32 + Send + Sync>>,
    applies_to_context: Option<Box<dyn Fn(&ArmorModifierContext<'_>) -> bool + Send + Sync>>,
}

impl VariableArmourModifier {
    pub fn new(name: impl Into<String>, foul_assist_modifier: bool) -> Self {
        Self { name: name.into(), foul_assist_modifier, registered_to: None, modifier_fn: None, applies_to_context: None }
    }

    pub fn with_modifier_fn(mut self, f: impl Fn(Option<&Player>, &Player) -> i32 + Send + Sync + 'static) -> Self {
        self.modifier_fn = Some(Box::new(f));
        self
    }

    pub fn with_predicate(mut self, f: impl Fn(&ArmorModifierContext<'_>) -> bool + Send + Sync + 'static) -> Self {
        self.applies_to_context = Some(Box::new(f));
        self
    }
}

impl ArmorModifier for VariableArmourModifier {
    fn get_modifier(&self, attacker: Option<&Player>, defender: &Player) -> i32 {
        if let Some(f) = &self.modifier_fn {
            return f(attacker, defender);
        }
        // Java default: attacker.getSkillIntValue(registeredTo)
        attacker.map(|a| a.get_skill_int_value(self.registered_to.as_deref().unwrap_or(""))).unwrap_or(0)
    }
    fn get_name(&self) -> &str { &self.name }
    fn is_foul_assist_modifier(&self) -> bool { self.foul_assist_modifier }
    fn applies_to_context(&self, context: &ArmorModifierContext<'_>) -> bool {
        self.applies_to_context.as_ref().map(|f| f(context)).unwrap_or(true)
    }
    fn registered_to(&self) -> Option<&str> { self.registered_to.as_deref() }
    fn set_registered_to(&mut self, skill_id: Option<String>) { self.registered_to = skill_id; }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_stores_name_and_foul_flag() {
        let m = VariableArmourModifier::new("Mighty Blow", false);
        assert_eq!(m.get_name(), "Mighty Blow");
        assert!(!m.is_foul_assist_modifier());
    }

    #[test]
    fn registered_to_defaults_none() {
        let m = VariableArmourModifier::new("x", false);
        assert!(m.registered_to().is_none());
    }

    #[test]
    fn foul_assist_flag_propagates() {
        let m_foul = VariableArmourModifier::new("Foul Assist", true);
        assert!(m_foul.is_foul_assist_modifier());
        let m_no_foul = VariableArmourModifier::new("x", false);
        assert!(!m_no_foul.is_foul_assist_modifier());
    }

    #[test]
    fn set_registered_to_stores_value() {
        let mut m = VariableArmourModifier::new("x", false);
        m.set_registered_to(Some("DODGE".to_string()));
        assert_eq!(m.registered_to(), Some("DODGE"));
    }

    #[test]
    fn set_registered_to_then_clear_to_none() {
        let mut m = VariableArmourModifier::new("x", false);
        m.set_registered_to(Some("DODGE".to_string()));
        m.set_registered_to(None);
        assert!(m.registered_to().is_none());
    }

    #[test]
    fn with_modifier_fn_overrides_default_lookup() {
        use ffb_model::enums::{PlayerType, PlayerGender};
        let m = VariableArmourModifier::new("test", false).with_modifier_fn(|_a, _d| 2);
        let p = Player {
            id: "p".into(), name: "p".into(), nr: 1, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        };
        assert_eq!(m.get_modifier(Some(&p), &p), 2);
    }

    #[test]
    fn with_predicate_overrides_default_true() {
        use ffb_model::enums::Rules;
        use ffb_model::model::{Game, Team};
        let m = VariableArmourModifier::new("test", false).with_predicate(|_ctx| false);
        let team = Team {
            id: "t".into(), name: "T".into(), race: "human".into(),
            roster_id: "human".into(), coach: "Coach".into(),
            rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0,
            cheerleaders: 0, assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0,
            team_value: 0, treasury: 0, special_rules: vec![], players: vec![],
            vampire_lord: false,
            necromancer: false,
        };
        let game = Game::new(team.clone(), team, Rules::Bb2025);
        let defender = Player::default();
        let ctx = ArmorModifierContext::new(&game, None, &defender, false, false);
        assert!(!m.applies_to_context(&ctx));
    }
}
