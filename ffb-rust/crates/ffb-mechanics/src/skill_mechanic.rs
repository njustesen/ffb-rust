use ffb_model::enums::{PlayerState, TurnMode};
use ffb_model::types::FieldCoordinate;
use ffb_model::model::{FieldModel, Game, Player};
use crate::mechanic::{Mechanic, MechanicType};

/// 1:1 translation of com.fumbbl.ffb.mechanics.SkillMechanic.
pub trait SkillMechanic: Mechanic {
    fn get_type(&self) -> MechanicType { MechanicType::SKILL }

    fn eligible_for_pro(&self, game: &Game, player: &Player, original_bombardier: Option<&str>) -> bool;
    fn is_valid_assist(&self, using_multi_block: bool, field_model: &FieldModel, player: &Player) -> bool;
    fn is_valid_pushback_square(&self, field_model: &FieldModel, coordinate: FieldCoordinate) -> bool;
    fn can_prevent_strip_ball(&self, player_state: PlayerState) -> bool;
    fn allows_cancelling_guard(&self, turn_mode: TurnMode) -> bool;
    fn calculate_player_level(&self, game: &Game, player: &Player) -> String;
    fn can_always_assist_foul(&self, game: &Game, assistant: &Player) -> bool;
    fn animosity_exists(&self, thrower: &Player, catcher: &Player) -> bool;
}
