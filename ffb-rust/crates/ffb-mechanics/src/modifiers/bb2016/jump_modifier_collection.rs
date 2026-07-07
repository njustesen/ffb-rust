use crate::modifiers::jump_modifier::JumpModifier;
use crate::modifiers::jump_context::JumpContext;
use crate::modifiers::jump_modifier_collection::JumpModifierCollection as BaseJumpModifierCollection;

pub struct JumpModifierCollection {
    inner: BaseJumpModifierCollection,
}

impl JumpModifierCollection {
    pub fn new() -> Self { Self { inner: BaseJumpModifierCollection::new() } }
    pub fn get_modifiers(&self) -> &[JumpModifier] { self.inner.get_modifiers() }
    pub fn find_applicable<'a>(&'a self, ctx: &JumpContext<'_>) -> Vec<&'a JumpModifier> { self.inner.find_applicable(ctx) }
}

impl Default for JumpModifierCollection {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modifiers::jump_modifier::JumpModifier;
    use crate::modifiers::modifier_type::ModifierType;

    #[test]
    fn has_zero_modifiers() {
        // bb2016 adds no extra jump modifiers on top of empty base
        assert_eq!(JumpModifierCollection::new().get_modifiers().len(), 0);
    }

    #[test]
    fn default_creates_empty_collection() {
        let col = JumpModifierCollection::default();
        assert_eq!(col.get_modifiers().len(), 0);
    }

    #[test]
    fn find_applicable_on_empty_returns_empty() {
        use crate::modifiers::jump_context::JumpContext;
        use ffb_model::enums::Rules;
        use ffb_model::model::{Game, Team};
        use ffb_model::types::FieldCoordinate;
        use ffb_model::enums::{PlayerType, PlayerGender};
        use ffb_model::model::Player;
        let home = Team { id: "h".into(), name: "h".into(), race: "h".into(), roster_id: "h".into(), coach: "c".into(),
            rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0, prayers_to_nuffle: 0,
            bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0, assistant_coaches: 0,
            fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: vec![], players: vec![], vampire_lord: false, necromancer: false };
        let away = home.clone();
        let game = Game::new(home, away, Rules::Bb2016);
        let player = Player { id: "p".into(), name: "p".into(), nr: 1, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(), niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None, is_big_guy: false, ..Default::default() };
        let ctx = JumpContext::new(&game, &player, FieldCoordinate::new(5, 5), FieldCoordinate::new(6, 6));
        let col = JumpModifierCollection::new();
        assert_eq!(col.find_applicable(&ctx).len(), 0);
    }

    #[test]
    fn get_modifiers_returns_slice() {
        let col = JumpModifierCollection::new();
        let mods = col.get_modifiers();
        assert!(mods.is_empty());
    }
}
