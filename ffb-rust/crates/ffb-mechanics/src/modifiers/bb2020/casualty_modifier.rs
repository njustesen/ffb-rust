pub struct CasualtyModifier {
    pub name: String,
    pub modifier: i32,
    applies_to_context: Option<Box<dyn Fn(&ffb_model::model::Player) -> bool + Send + Sync>>,
}

impl CasualtyModifier {
    pub fn new(name: impl Into<String>, modifier: i32) -> Self {
        Self { name: name.into(), modifier, applies_to_context: None }
    }

    pub fn with_predicate(mut self, f: impl Fn(&ffb_model::model::Player) -> bool + Send + Sync + 'static) -> Self {
        self.applies_to_context = Some(Box::new(f));
        self
    }

    pub fn get_modifier(&self) -> i32 { self.modifier }
    pub fn get_name(&self) -> &str { &self.name }
    pub fn applies_to_context(&self, player: &ffb_model::model::Player) -> bool {
        self.applies_to_context.as_ref().map(|f| f(player)).unwrap_or(true)
    }
    pub fn report_string(&self) -> String { format!("{} {}", self.modifier, self.name) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_modifier_returns_value() {
        let m = CasualtyModifier::new("Test", 2);
        assert_eq!(m.get_modifier(), 2);
    }

    #[test]
    fn applies_to_context_true_when_no_predicate() {
        use ffb_model::enums::{PlayerType, PlayerGender};
        let m = CasualtyModifier::new("Test", 1);
        let p = ffb_model::model::Player {
            id: "p".into(), name: "p".into(), nr: 1, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
};
        assert!(m.applies_to_context(&p));
    }

    #[test]
    fn report_string_includes_name_and_modifier() {
        let m = CasualtyModifier::new("Mighty Blow", 1);
        let s = m.report_string();
        assert!(s.contains("Mighty Blow"));
        assert!(s.contains('1'));
    }
}
