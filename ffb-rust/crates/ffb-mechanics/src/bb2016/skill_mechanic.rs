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
