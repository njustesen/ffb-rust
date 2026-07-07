use ffb_model::enums::{ApothecaryType, PlayerState, PlayerType};
use ffb_model::model::{Game, Player};
use crate::mechanic::{Mechanic, MechanicType};
use crate::apothecary_mechanic::ApothecaryMechanic as ApothecaryMechanicTrait;

/// 1:1 translation of com.fumbbl.ffb.mechanics.bb2020.ApothecaryMechanic.
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
    fn apothecary_types(&self, game: &Game, defender: &Player, player_state: PlayerState) -> Vec<ApothecaryType> {
        let mut valid_types = Vec::new();
        if defender.is_zapped() || defender.player_type == PlayerType::Star {
            return valid_types;
        }
        let turn_data = if game.team_home.player(&defender.id).is_some() {
            &game.turn_data_home
        } else {
            &game.turn_data_away
        };
        let team_has_wandering_apo = turn_data.wandering_apothecaries > 0;
        let team_can_use_plague_doctor = turn_data.plague_doctors > 0 && player_state.is_ko();
        if defender.player_type == PlayerType::Mercenary {
            if team_has_wandering_apo {
                valid_types.push(ApothecaryType::Wandering);
            }
        } else if defender.is_journeyman() {
            if team_has_wandering_apo {
                valid_types.push(ApothecaryType::Wandering);
            }
            if team_can_use_plague_doctor {
                valid_types.push(ApothecaryType::Plague);
            }
        } else {
            if turn_data.apothecaries > turn_data.wandering_apothecaries {
                valid_types.push(ApothecaryType::Team);
            } else if team_has_wandering_apo {
                valid_types.push(ApothecaryType::Wandering);
            }
            if team_can_use_plague_doctor {
                valid_types.push(ApothecaryType::Plague);
            }
        }
        valid_types
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{PS_SERIOUS_INJURY, PS_KNOCKED_OUT, PlayerGender, Rules};
    use ffb_model::model::Team;
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

    fn player(id: &str, player_type: PlayerType) -> Player {
        Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "pos".into(),
            player_type, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        }
    }

    #[test]
    fn star_player_returns_empty() {
        let game = Game::new(bare_team("home"), bare_team("away"), Rules::Bb2020);
        let p = player("p1", PlayerType::Star);
        assert!(ApothecaryMechanic.apothecary_types(&game, &p, PlayerState(PS_SERIOUS_INJURY)).is_empty());
    }

    #[test]
    fn regular_player_with_team_apo_returns_team_type() {
        let mut home = bare_team("home");
        let p = player("p1", PlayerType::Regular);
        home.players.push(p.clone());
        let mut game = Game::new(home, bare_team("away"), Rules::Bb2020);
        game.turn_data_home.apothecaries = 1;
        let result = ApothecaryMechanic.apothecary_types(&game, &p, PlayerState(PS_SERIOUS_INJURY));
        assert_eq!(result, vec![ApothecaryType::Team]);
    }

    #[test]
    fn regular_player_ko_with_plague_doctor_gets_plague_type() {
        let mut home = bare_team("home");
        let p = player("p1", PlayerType::Regular);
        home.players.push(p.clone());
        let mut game = Game::new(home, bare_team("away"), Rules::Bb2020);
        game.turn_data_home.plague_doctors = 1;
        let result = ApothecaryMechanic.apothecary_types(&game, &p, PlayerState(PS_KNOCKED_OUT));
        assert!(result.contains(&ApothecaryType::Plague));
    }

    #[test]
    fn no_apo_returns_empty() {
        let mut home = bare_team("home");
        let p = player("p1", PlayerType::Regular);
        home.players.push(p.clone());
        let game = Game::new(home, bare_team("away"), Rules::Bb2020);
        assert!(ApothecaryMechanic.apothecary_types(&game, &p, PlayerState(PS_SERIOUS_INJURY)).is_empty());
    }

    #[test]
    fn mercenary_player_with_wandering_apo_returns_wandering() {
        let mut home = bare_team("home");
        let p = player("p1", PlayerType::Mercenary);
        home.players.push(p.clone());
        let mut game = Game::new(home, bare_team("away"), Rules::Bb2020);
        game.turn_data_home.wandering_apothecaries = 1;
        let result = ApothecaryMechanic.apothecary_types(&game, &p, PlayerState(PS_SERIOUS_INJURY));
        assert!(result.contains(&ApothecaryType::Wandering));
    }

    #[test]
    fn plague_doctor_not_applied_to_serious_injury_for_regular() {
        // Plague doctor only activates on KO (is_ko check)
        let mut home = bare_team("home");
        let p = player("p1", PlayerType::Regular);
        home.players.push(p.clone());
        let mut game = Game::new(home, bare_team("away"), Rules::Bb2020);
        game.turn_data_home.plague_doctors = 1;
        let result = ApothecaryMechanic.apothecary_types(&game, &p, PlayerState(PS_SERIOUS_INJURY));
        assert!(!result.contains(&ApothecaryType::Plague));
    }
}
