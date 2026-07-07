use ffb_model::model::Player;
use crate::modifiers::player_stat_key::PlayerStatKey;
use crate::modifiers::stat_based_roll_modifier::StatBasedRollModifier;

/// 1:1 translation of com.fumbbl.ffb.modifiers.StatBasedRollModifierFactory.
pub struct StatBasedRollModifierFactory {
    pub name: String,
    pub stat_key: PlayerStatKey,
}

impl StatBasedRollModifierFactory {
    pub fn new(name: impl Into<String>, stat_key: PlayerStatKey) -> Self {
        Self { name: name.into(), stat_key }
    }

    pub fn create(&self, player: &Player) -> StatBasedRollModifier {
        let value = match self.stat_key {
            PlayerStatKey::AG => player.agility_with_modifiers(),
            PlayerStatKey::AV => player.armour_with_modifiers(),
            PlayerStatKey::MA => player.movement_with_modifiers(),
            PlayerStatKey::PA => player.passing_with_modifiers(),
            PlayerStatKey::ST => player.strength_with_modifiers(),
        };
        StatBasedRollModifier::new(&self.name, value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{PlayerType, PlayerGender};

    fn player_with_stats(agility: i32, strength: i32, armour: i32) -> Player {
        Player {
            id: "p".into(), name: "p".into(), nr: 1, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength, agility, passing: 4, armour,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        }
    }

    #[test]
    fn create_with_ag_reads_agility() {
        let factory = StatBasedRollModifierFactory::new("Agility", PlayerStatKey::AG);
        let p = player_with_stats(3, 4, 8);
        let m = factory.create(&p);
        assert_eq!(m.get_modifier(), 3);
        assert_eq!(m.get_report_string(), "Agility");
    }

    #[test]
    fn create_with_st_reads_strength() {
        let factory = StatBasedRollModifierFactory::new("Strength", PlayerStatKey::ST);
        let p = player_with_stats(3, 5, 8);
        let m = factory.create(&p);
        assert_eq!(m.get_modifier(), 5);
    }

    #[test]
    fn create_with_av_reads_armour() {
        let factory = StatBasedRollModifierFactory::new("Armour", PlayerStatKey::AV);
        let p = player_with_stats(3, 4, 9);
        let m = factory.create(&p);
        assert_eq!(m.get_modifier(), 9);
    }
}
