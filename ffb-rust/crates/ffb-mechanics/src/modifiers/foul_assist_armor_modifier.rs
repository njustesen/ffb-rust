use ffb_model::model::Player;
use crate::modifiers::armor_modifier::ArmorModifier;
use crate::modifiers::armor_modifier_context::ArmorModifierContext;

/// 1:1 translation of com.fumbbl.ffb.factory.FoulAssistArmorModifier.
/// Extends StaticArmourModifier; appliesToContext checks isFoul && foulAssists == modifier.
pub struct FoulAssistArmorModifier {
    name: String,
    modifier: i32,
    foul_assist_modifier: bool,
}

impl FoulAssistArmorModifier {
    pub fn new(name: impl Into<String>, modifier: i32, foul_assist_modifier: bool) -> Self {
        Self { name: name.into(), modifier, foul_assist_modifier }
    }
}

impl ArmorModifier for FoulAssistArmorModifier {
    fn get_modifier(&self, _attacker: Option<&Player>, _defender: &Player) -> i32 {
        self.modifier
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn is_foul_assist_modifier(&self) -> bool {
        self.foul_assist_modifier
    }

    fn applies_to_context(&self, context: &ArmorModifierContext<'_>) -> bool {
        context.is_foul() && context.get_foul_assists() == self.modifier
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::model::{Game, Player, team::Team};
    use ffb_model::enums::{PlayerType, PlayerGender, Rules};

    fn dummy_player(id: &str) -> Player {
        Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        }
    }

    fn make_team(id: &str, players: Vec<Player>) -> Team {
        Team {
            id: id.into(), name: id.into(), race: "human".into(), roster_id: "human".into(),
            coach: "c".into(), rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0,
            assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: vec![], players,
            vampire_lord: false,
            necromancer: false,
        }
    }

    fn make_game() -> Game {
        let home = make_team("home", vec![dummy_player("a"), dummy_player("d")]);
        let away = make_team("away", vec![]);
        Game::new(home, away, Rules::Bb2025)
    }

    #[test]
    fn applies_when_foul_and_matching_assists() {
        let m = FoulAssistArmorModifier::new("2 Offensive Assists", 2, true);
        let game = make_game();
        let attacker = dummy_player("a");
        let defender = dummy_player("d");
        let ctx = ArmorModifierContext::new_with_foul_assists(&game, Some(&attacker), &defender, false, true, 2);
        assert!(m.applies_to_context(&ctx));
    }

    #[test]
    fn does_not_apply_when_foul_but_wrong_assists() {
        let m = FoulAssistArmorModifier::new("2 Offensive Assists", 2, true);
        let game = make_game();
        let attacker = dummy_player("a");
        let defender = dummy_player("d");
        let ctx = ArmorModifierContext::new_with_foul_assists(&game, Some(&attacker), &defender, false, true, 3);
        assert!(!m.applies_to_context(&ctx));
    }

    #[test]
    fn does_not_apply_when_not_foul() {
        let m = FoulAssistArmorModifier::new("2 Offensive Assists", 2, true);
        let game = make_game();
        let attacker = dummy_player("a");
        let defender = dummy_player("d");
        let ctx = ArmorModifierContext::new_with_foul_assists(&game, Some(&attacker), &defender, false, false, 2);
        assert!(!m.applies_to_context(&ctx));
    }
}
