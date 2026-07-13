use ffb_model::enums::{PlayerState, PlayerType, TurnMode};
use ffb_model::types::FieldCoordinate;
use ffb_model::model::{FieldModel, Game, Player};
use ffb_model::model::property::named_properties::NamedProperties;
use crate::mechanic::{Mechanic, MechanicType};
use crate::skill_mechanic::SkillMechanic as SkillMechanicTrait;

/// Java `bb2020/Animosity.Evaluator.values(Skill, Player)` — unlike bb2025's evaluator, bb2020
/// does NOT normalize through `Keyword.forName` (`split()` returns raw values), since bb2020
/// configures Animosity against raw roster position ids (e.g. "underworld.skaven.thrower") or
/// bare race names (e.g. "goblin"), matched directly against `catcher.getPositionId()`/`getRace()`.
fn animosity_values(thrower: &Player, animosity: ffb_model::model::skill_def::SkillId) -> std::collections::HashSet<String> {
    let mut values = thrower.temporary_skill_values(animosity);
    values.insert(thrower.skill_value_excluding_temporary_ones(animosity).unwrap_or_else(|| "all".to_string()));
    values.into_iter().flat_map(|v| v.split(';').map(str::to_string).collect::<Vec<_>>()).collect()
}

/// 1:1 translation of com.fumbbl.ffb.mechanics.bb2020.SkillMechanic.
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
        "Rookie".to_string()
    }

    fn can_always_assist_foul(&self, game: &Game, assistant: &Player) -> bool {
        game.options.is_enabled("sneakyGitAsFoulGuard")
            && assistant.has_skill_property(NamedProperties::CAN_ALWAYS_ASSIST_FOULS)
    }

    fn animosity_exists(&self, thrower: &Player, catcher: &Player) -> bool {
        // Java: `thrower.getTeam().getId().equals(catcher.getTeam().getId())` is not checked
        // here — see the identical note in bb2025's `animosity_exists`.
        let Some(animosity) = thrower.skill_id_with_property(NamedProperties::HAS_TO_ROLL_TO_PASS_BALL_ON) else {
            return false;
        };
        if catcher.player_type == PlayerType::Mercenary || catcher.player_type == PlayerType::Star {
            return false;
        }
        let pattern: std::collections::HashSet<String> = [
            Some("all".to_string()),
            Some(catcher.position_id.to_lowercase()),
            catcher.race.as_ref().map(|r| r.to_lowercase()),
        ]
        .into_iter()
        .flatten()
        .collect();
        animosity_values(thrower, animosity)
            .iter()
            .map(|v| v.to_lowercase())
            .any(|v| pattern.contains(&v))
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
        // PlayerState::Standing has tacklezones
        assert!(SkillMechanic.can_prevent_strip_ball(PlayerState(PS_STANDING)));
    }

    #[test]
    fn can_prevent_strip_ball_false_when_prone() {
        // PlayerState::Prone has no tacklezones
        assert!(!SkillMechanic.can_prevent_strip_ball(PlayerState(PS_PRONE)));
    }

    #[test]
    fn animosity_exists_false_without_skill() {
        use ffb_model::model::Player;
        let p1 = Player::default();
        let p2 = Player::default();
        assert!(!SkillMechanic.animosity_exists(&p1, &p2));
    }

    fn animosity_player(value: Option<&str>, catcher_position_id: &str, catcher_race: Option<&str>) -> (Player, Player) {
        use ffb_model::model::Player;
        use ffb_model::model::skill_def::SkillWithValue;
        let mut thrower = Player::default();
        thrower.starting_skills.push(match value {
            Some(v) => SkillWithValue::with_value(ffb_model::model::SkillId::Animosity, v),
            None => SkillWithValue::new(ffb_model::model::SkillId::Animosity),
        });
        let mut catcher = Player::default();
        catcher.position_id = catcher_position_id.to_string();
        catcher.race = catcher_race.map(String::from);
        (thrower, catcher)
    }

    #[test]
    fn animosity_exists_true_when_configured_all() {
        let (thrower, catcher) = animosity_player(Some("all"), "underworld.skaven.lineman", Some("underworld.lrb6"));
        assert!(SkillMechanic.animosity_exists(&thrower, &catcher));
    }

    #[test]
    fn animosity_exists_true_when_position_id_matches() {
        let (thrower, catcher) = animosity_player(
            Some("underworld.skaven.thrower;underworld.troll.warpstone"),
            "underworld.skaven.thrower",
            Some("underworld.lrb6"),
        );
        assert!(SkillMechanic.animosity_exists(&thrower, &catcher));
    }

    #[test]
    fn animosity_exists_true_when_race_matches() {
        let (thrower, catcher) = animosity_player(Some("Skaven"), "underworld.skaven.lineman", Some("Skaven"));
        assert!(SkillMechanic.animosity_exists(&thrower, &catcher));
    }

    #[test]
    fn animosity_exists_false_when_neither_matches() {
        let (thrower, catcher) = animosity_player(Some("goblin"), "underworld.skaven.lineman", Some("Skaven"));
        assert!(!SkillMechanic.animosity_exists(&thrower, &catcher));
    }

    #[test]
    fn animosity_exists_false_for_mercenary_catcher() {
        let (thrower, mut catcher) = animosity_player(Some("all"), "underworld.skaven.lineman", Some("Skaven"));
        catcher.player_type = PlayerType::Mercenary;
        assert!(!SkillMechanic.animosity_exists(&thrower, &catcher));
    }

    #[test]
    fn animosity_exists_false_for_star_catcher() {
        let (thrower, mut catcher) = animosity_player(Some("all"), "underworld.skaven.lineman", Some("Skaven"));
        catcher.player_type = PlayerType::Star;
        assert!(!SkillMechanic.animosity_exists(&thrower, &catcher));
    }
}
