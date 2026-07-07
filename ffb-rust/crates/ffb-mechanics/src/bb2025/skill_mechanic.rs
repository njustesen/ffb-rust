use ffb_model::enums::{PlayerState, TurnMode};
use ffb_model::types::FieldCoordinate;
use ffb_model::model::{FieldModel, Game, Player};
use ffb_model::model::property::named_properties::NamedProperties;
use crate::mechanic::{Mechanic, MechanicType};
use crate::skill_mechanic::SkillMechanic as SkillMechanicTrait;

/// 1:1 translation of com.fumbbl.ffb.mechanics.bb2025.SkillMechanic.
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

fn modes_allowing_pro(turn_mode: TurnMode) -> bool {
    matches!(turn_mode, TurnMode::Regular | TurnMode::Blitz | TurnMode::BombHome | TurnMode::BombAway)
}

impl SkillMechanicTrait for SkillMechanic {
    fn eligible_for_pro(&self, game: &Game, player: &Player, original_bombardier: Option<&str>) -> bool {
        let player_state = game.field_model.player_state(&player.id).unwrap_or_default();
        let acting = &game.acting_player;
        (!acting.standing_up || acting.has_acted)
            && !player_state.is_prone_or_stunned()
            && !player_state.is_stunned()
            && acting.player_id.as_deref() == Some(&player.id)
            && modes_allowing_pro(game.turn_mode)
            && (!game.turn_mode.is_bomb_turn() || original_bombardier == Some(&player.id))
    }

    fn is_valid_assist(&self, using_multi_block: bool, field_model: &FieldModel, player: &Player) -> bool {
        !(using_multi_block && field_model.is_multi_block_target(&player.id))
    }

    fn is_valid_pushback_square(&self, field_model: &FieldModel, coordinate: FieldCoordinate) -> bool {
        !field_model.was_multi_block_target_square(coordinate)
    }

    fn can_prevent_strip_ball(&self, player_state: PlayerState) -> bool {
        player_state.has_tacklezones()
    }

    fn allows_cancelling_guard(&self, turn_mode: TurnMode) -> bool {
        TurnMode::Blitz != turn_mode
    }

    fn calculate_player_level(&self, _game: &Game, _player: &Player) -> String {
        // TODO: player.skill_infos() — counts PLAYER-category non-stat-decrease skills gained
        // This requires SkillDisplayInfo translation. Return stub.
        "Rookie".to_string()
    }

    fn can_always_assist_foul(&self, _game: &Game, assistant: &Player) -> bool {
        assistant.has_skill_property(NamedProperties::CAN_ALWAYS_ASSIST_FOULS)
    }

    fn animosity_exists(&self, thrower: &Player, catcher: &Player) -> bool {
        // TODO: requires getSkillWithProperty(NamedProperties.hasToRollToPassBallOn) and AnimosityValueEvaluator
        let _ = (thrower, catcher);
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{PlayerState, PS_STANDING, PS_PRONE, TurnMode};
    use crate::skill_mechanic::SkillMechanic as SkillTrait;

    #[test]
    fn allows_cancelling_guard_false_on_blitz() {
        assert!(!SkillMechanic.allows_cancelling_guard(TurnMode::Blitz));
    }

    #[test]
    fn allows_cancelling_guard_true_on_regular() {
        assert!(SkillMechanic.allows_cancelling_guard(TurnMode::Regular));
    }

    #[test]
    fn can_prevent_strip_ball_true_when_standing() {
        assert!(SkillMechanic.can_prevent_strip_ball(PlayerState(PS_STANDING)));
    }

    #[test]
    fn can_prevent_strip_ball_false_when_prone() {
        assert!(!SkillMechanic.can_prevent_strip_ball(PlayerState(PS_PRONE)));
    }

    #[test]
    fn animosity_exists_returns_false_stub() {
        use ffb_model::model::Player;
        let p1 = Player::default();
        let p2 = Player::default();
        assert!(!SkillMechanic.animosity_exists(&p1, &p2));
    }
}
