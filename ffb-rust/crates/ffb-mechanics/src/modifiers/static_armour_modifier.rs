use ffb_model::model::Player;
use crate::modifiers::armor_modifier::ArmorModifier;
use crate::modifiers::armor_modifier_context::ArmorModifierContext;

/// 1:1 translation of com.fumbbl.ffb.modifiers.StaticArmourModifier.
pub struct StaticArmourModifier {
    pub name: String,
    pub modifier: i32,
    pub foul_assist_modifier: bool,
    pub chainsaw: bool,
    pub registered_to: Option<String>,
    applies_to_context: Option<Box<dyn Fn(&ArmorModifierContext<'_>) -> bool + Send + Sync>>,
}

impl StaticArmourModifier {
    pub fn new(name: impl Into<String>, modifier: i32, foul_assist_modifier: bool) -> Self {
        Self { name: name.into(), modifier, foul_assist_modifier, chainsaw: false, registered_to: None, applies_to_context: None }
    }

    pub fn new_with_chainsaw(name: impl Into<String>, modifier: i32, foul_assist_modifier: bool, chainsaw: bool) -> Self {
        Self { name: name.into(), modifier, foul_assist_modifier, chainsaw, registered_to: None, applies_to_context: None }
    }

    pub fn with_predicate(mut self, f: impl Fn(&ArmorModifierContext<'_>) -> bool + Send + Sync + 'static) -> Self {
        self.applies_to_context = Some(Box::new(f));
        self
    }

    pub fn is_chainsaw(&self) -> bool { self.chainsaw }
}

impl ArmorModifier for StaticArmourModifier {
    fn get_modifier(&self, _attacker: Option<&Player>, _defender: &Player) -> i32 { self.modifier }
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
    fn new_stores_name_modifier_and_foul_flag() {
        let m = StaticArmourModifier::new("Mighty Blow", 1, false);
        assert_eq!(m.get_name(), "Mighty Blow");
        let p = dummy_player();
        assert_eq!(m.get_modifier(None, &p), 1);
        assert!(!m.is_foul_assist_modifier());
    }

    #[test]
    fn chainsaw_flag_defaults_false() {
        assert!(!StaticArmourModifier::new("x", 0, false).is_chainsaw());
    }

    #[test]
    fn chainsaw_flag_can_be_set() {
        let m = StaticArmourModifier::new_with_chainsaw("Chainsaw +3", 3, false, true);
        assert!(m.is_chainsaw());
    }
}
