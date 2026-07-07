use ffb_model::enums::{ApothecaryType, PlayerState};
use ffb_model::model::{Game, Player};
use crate::mechanic::{Mechanic, MechanicType};
use crate::apothecary_mechanic::ApothecaryMechanic as ApothecaryMechanicTrait;

/// 1:1 translation of com.fumbbl.ffb.mechanics.bb2016.ApothecaryMechanic.
pub struct ApothecaryMechanic;

impl ApothecaryMechanic {
    pub fn new() -> Self { Self }
}

impl Default for ApothecaryMechanic {
    fn default() -> Self { Self::new() }
}

impl Mechanic for ApothecaryMechanic {
    fn get_type(&self) -> MechanicType { MechanicType::APOTHECARY }
}

impl ApothecaryMechanicTrait for ApothecaryMechanic {
    fn apothecary_types(&self, _game: &Game, _defender: &Player, _player_state: PlayerState) -> Vec<ApothecaryType> {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{PS_SERIOUS_INJURY, PlayerType, PlayerGender, Rules};
    use ffb_model::model::{Game, Team};
    use crate::apothecary_mechanic::ApothecaryMechanic as ApoTrait;

    fn bare_team(id: &str) -> Team {
        Team {
            id: id.into(), name: id.into(), race: "human".into(), roster_id: "human".into(), coach: "c".into(),
            rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0, prayers_to_nuffle: 0,
            bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0, assistant_coaches: 0,
            fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: vec![], players: vec![],
            vampire_lord: false,
            necromancer: false,
        }
    }

    fn bare_player(id: &str) -> Player {
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

    #[test]
    fn apothecary_types_always_empty_in_bb2016() {
        let game = Game::new(bare_team("home"), bare_team("away"), Rules::Bb2016);
        let p = bare_player("p1");
        let result = ApothecaryMechanic.apothecary_types(&game, &p, PlayerState(PS_SERIOUS_INJURY));
        assert!(result.is_empty());
    }

    #[test]
    fn mechanic_type_is_apothecary() {
        assert_eq!(Mechanic::get_type(&ApothecaryMechanic), MechanicType::APOTHECARY);
    }

    #[test]
    fn apothecary_types_empty_regardless_of_player_state() {
        use ffb_model::enums::PS_KNOCKED_OUT;
        let game = Game::new(bare_team("home"), bare_team("away"), Rules::Bb2016);
        let p = bare_player("p1");
        assert!(ApothecaryMechanic.apothecary_types(&game, &p, PlayerState(PS_KNOCKED_OUT)).is_empty());
    }

    #[test]
    fn default_creates_instance() {
        let m = ApothecaryMechanic::default();
        assert_eq!(Mechanic::get_type(&m), MechanicType::APOTHECARY);
    }

    #[test]
    fn apothecary_types_empty_for_star_player_too() {
        use ffb_model::enums::PlayerType;
        let game = Game::new(bare_team("home"), bare_team("away"), Rules::Bb2016);
        let mut p = bare_player("p1");
        p.player_type = PlayerType::Star;
        // BB2016 always returns empty regardless of player type
        assert!(ApothecaryMechanic.apothecary_types(&game, &p, PlayerState(PS_SERIOUS_INJURY)).is_empty());
    }

    #[test]
    fn get_name_returns_apothecary_string() {
        use crate::mechanic::Mechanic;
        assert_eq!(Mechanic::get_name(&ApothecaryMechanic), "APOTHECARY");
    }
}
