use ffb_model::model::Player;
use crate::modifiers::injury_modifier::InjuryModifier;
use crate::modifiers::injury_modifier_context::InjuryModifierContext;

/// 1:1 translation of com.fumbbl.ffb.modifiers.StaticInjuryModifier.
pub struct StaticInjuryModifier {
    pub name: String,
    pub modifier: i32,
    pub niggling_injury_modifier: bool,
    pub registered_to: Option<String>,
    applies_to_context: Option<Box<dyn Fn(&InjuryModifierContext<'_>) -> bool + Send + Sync>>,
}

impl StaticInjuryModifier {
    pub fn new(name: impl Into<String>, modifier: i32, niggling_injury_modifier: bool) -> Self {
        Self { name: name.into(), modifier, niggling_injury_modifier, registered_to: None, applies_to_context: None }
    }

    pub fn with_predicate(mut self, f: impl Fn(&InjuryModifierContext<'_>) -> bool + Send + Sync + 'static) -> Self {
        self.applies_to_context = Some(Box::new(f));
        self
    }
}

impl InjuryModifier for StaticInjuryModifier {
    fn get_modifier(&self, _attacker: Option<&Player>, _defender: &Player) -> i32 { self.modifier }
    fn get_name(&self) -> &str { &self.name }
    fn is_niggling_injury_modifier(&self) -> bool { self.niggling_injury_modifier }
    fn applies_to_context(&self, context: &InjuryModifierContext<'_>) -> bool {
        self.applies_to_context.as_ref().map(|f| f(context)).unwrap_or(true)
    }
    fn registered_to(&self) -> Option<&str> { self.registered_to.as_deref() }
    fn set_registered_to(&mut self, skill_id: Option<String>) { self.registered_to = skill_id; }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_player() -> Player {
        use ffb_model::enums::{PlayerType, PlayerGender};
        Player {
            id: "p".into(), name: "p".into(), nr: 1, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            ..Default::default()
        }
    }

    #[test]
    fn new_stores_fields() {
        let m = StaticInjuryModifier::new("Stunty", 1, false);
        let p = dummy_player();
        assert_eq!(m.get_name(), "Stunty");
        assert_eq!(m.get_modifier(None, &p), 1);
        assert!(!m.is_niggling_injury_modifier());
    }

    #[test]
    fn niggling_flag_propagates() {
        let m = StaticInjuryModifier::new("Niggling", 0, true);
        assert!(m.is_niggling_injury_modifier());
    }

    #[test]
    fn registered_to_defaults_none() {
        let m = StaticInjuryModifier::new("x", 0, false);
        assert!(m.registered_to().is_none());
    }
}
