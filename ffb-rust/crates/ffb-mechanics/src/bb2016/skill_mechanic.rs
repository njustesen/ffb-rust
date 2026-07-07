use ffb_model::enums::{PlayerState, TurnMode};
use ffb_model::types::FieldCoordinate;
use ffb_model::model::{FieldModel, Game, Player};
use ffb_model::model::property::named_properties::NamedProperties;
use crate::mechanic::{Mechanic, MechanicType};
use crate::skill_mechanic::SkillMechanic as SkillMechanicTrait;

/// 1:1 translation of com.fumbbl.ffb.mechanics.bb2016.SkillMechanic.
pub struct SkillMechanic;

impl SkillMechanic {
    pub fn new() -> Self { Self }
}

impl Default for SkillMechanic {
    fn default() -> Self { Self::new() }
}

impl Mechanic for SkillMechanic {
    fn get_type(&self) -> MechanicType { MechanicType::SKILL }
}

impl SkillMechanicTrait for SkillMechanic {
    fn eligible_for_pro(&self, _game: &Game, _player: &Player, _original_bombardier: Option<&str>) -> bool {
        true
    }

    fn is_valid_assist(&self, _using_multi_block: bool, _field_model: &FieldModel, _player: &Player) -> bool {
        true
    }

    fn is_valid_pushback_square(&self, _field_model: &FieldModel, _coordinate: FieldCoordinate) -> bool {
        true
    }

    fn can_prevent_strip_ball(&self, _player_state: PlayerState) -> bool {
        true
    }

    fn allows_cancelling_guard(&self, _turn_mode: TurnMode) -> bool {
        false
    }

    fn calculate_player_level(&self, _game: &Game, player: &Player) -> String {
        let old_spps = player.career_spps;
        if old_spps > 175 {
            "Legend".to_string()
        } else if old_spps > 75 {
            "Super Star".to_string()
        } else if old_spps > 50 {
            "Star".to_string()
        } else if old_spps > 30 {
            "Emerging".to_string()
        } else if old_spps > 15 {
            "Veteran".to_string()
        } else if old_spps > 5 {
            "Experienced".to_string()
        } else {
            "Rookie".to_string()
        }
    }

    fn can_always_assist_foul(&self, game: &Game, assistant: &Player) -> bool {
        game.options.is_enabled("sneakyGitAsFoulGuard")
            && assistant.has_skill_property(NamedProperties::CAN_ALWAYS_ASSIST_FOULS)
    }

    fn animosity_exists(&self, _thrower: &Player, _catcher: &Player) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{PlayerState, PS_STANDING, PS_PRONE, PlayerType, PlayerGender, TurnMode};
    use crate::skill_mechanic::SkillMechanic as SkillTrait;

    fn bare_player(id: &str, career_spps: i32) -> Player {
        Player {
            id: id.into(), name: id.into(), nr: 1,
            position_id: "pos".into(),
            player_type: PlayerType::Regular,
            gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![],
            current_spps: career_spps, career_spps,
            race: None,
            is_big_guy: false,
            ..Default::default()
        }
    }

    #[test]
    fn allows_cancelling_guard_is_always_false() {
        assert!(!SkillMechanic.allows_cancelling_guard(TurnMode::Regular));
        assert!(!SkillMechanic.allows_cancelling_guard(TurnMode::Blitz));
    }

    #[test]
    fn can_prevent_strip_ball_is_always_true() {
        assert!(SkillMechanic.can_prevent_strip_ball(PlayerState(PS_STANDING)));
        assert!(SkillMechanic.can_prevent_strip_ball(PlayerState(PS_PRONE)));
    }

    #[test]
    fn animosity_exists_always_false() {
        let p1 = bare_player("p1", 0);
        let p2 = bare_player("p2", 0);
        assert!(!SkillMechanic.animosity_exists(&p1, &p2));
    }

    fn bare_team(id: &str) -> ffb_model::model::Team {
        ffb_model::model::Team {
            id: id.into(), name: id.into(), race: "human".into(), roster_id: "human".into(), coach: "c".into(),
            rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0, prayers_to_nuffle: 0,
            bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0, assistant_coaches: 0,
            fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: vec![], players: vec![],
            vampire_lord: false, necromancer: false,
        }
    }

    fn bare_game() -> Game {
        Game::new(bare_team("home"), bare_team("away"), ffb_model::enums::Rules::Bb2016)
    }

    #[test]
    fn calculate_player_level_rookie() {
        let p = bare_player("p1", 0);
        assert_eq!(SkillMechanic.calculate_player_level(&bare_game(), &p), "Rookie");
    }

    #[test]
    fn calculate_player_level_legend() {
        let p = bare_player("legend", 176);
        assert_eq!(SkillMechanic.calculate_player_level(&bare_game(), &p), "Legend");
    }

    #[test]
    fn calculate_player_level_boundaries() {
        // Thresholds: >5=Experienced, >15=Veteran, >30=Emerging, >50=Star, >75=SuperStar, >175=Legend
        let g = bare_game();
        assert_eq!(SkillMechanic.calculate_player_level(&g, &bare_player("x", 6)), "Experienced");
        assert_eq!(SkillMechanic.calculate_player_level(&g, &bare_player("x", 16)), "Veteran");
        assert_eq!(SkillMechanic.calculate_player_level(&g, &bare_player("x", 31)), "Emerging");
        assert_eq!(SkillMechanic.calculate_player_level(&g, &bare_player("x", 51)), "Star");
        assert_eq!(SkillMechanic.calculate_player_level(&g, &bare_player("x", 76)), "Super Star");
    }

}
