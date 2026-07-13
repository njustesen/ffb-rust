use ffb_model::enums::{Keyword, PlayerState, TurnMode};
use ffb_model::types::FieldCoordinate;
use ffb_model::model::{FieldModel, Game, Player};
use ffb_model::model::property::named_properties::NamedProperties;
use crate::mechanic::{Mechanic, MechanicType};
use crate::skill_mechanic::SkillMechanic as SkillMechanicTrait;

/// Java `Animosity.Evaluator.values(Skill, Player)` (bb2025 variant) — temp values plus the
/// non-temporary configured value (or "all" if unconfigured), each part normalized through
/// `Keyword.forName(value).getName()`.
fn animosity_values(thrower: &Player, animosity: ffb_model::model::skill_def::SkillId) -> std::collections::HashSet<String> {
    let mut values = thrower.temporary_skill_values(animosity);
    values.insert(thrower.skill_value_excluding_temporary_ones(animosity).unwrap_or_else(|| "all".to_string()));
    values
        .into_iter()
        .flat_map(|v| v.split(';').map(|s| Keyword::for_name(s).get_name().to_string()).collect::<Vec<_>>())
        .collect()
}

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
        // Java: `thrower.getTeam().getId().equals(catcher.getTeam().getId())` is not checked
        // here — thrower/catcher are always resolved from the same team by the caller (a
        // pass/hand-off target is always a teammate), so the trait signature (no `Game`) is
        // left unchanged rather than threading team lookup through every mechanic call site.
        let Some(animosity) = thrower.skill_id_with_property(NamedProperties::HAS_TO_ROLL_TO_PASS_BALL_ON) else {
            return false;
        };
        let pattern: std::collections::HashSet<String> = std::iter::once("all".to_string())
            .chain(catcher.keywords.iter().map(|k| Keyword::for_name(k).get_name().to_lowercase()))
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
        assert!(SkillMechanic.can_prevent_strip_ball(PlayerState(PS_STANDING)));
    }

    #[test]
    fn can_prevent_strip_ball_false_when_prone() {
        assert!(!SkillMechanic.can_prevent_strip_ball(PlayerState(PS_PRONE)));
    }

    #[test]
    fn animosity_exists_false_without_skill() {
        use ffb_model::model::Player;
        let p1 = Player::default();
        let p2 = Player::default();
        assert!(!SkillMechanic.animosity_exists(&p1, &p2));
    }

    fn animosity_player(value: Option<&str>, catcher_keywords: Vec<&str>) -> (Player, Player) {
        use ffb_model::model::Player;
        use ffb_model::model::skill_def::SkillWithValue;
        let mut thrower = Player::default();
        thrower.starting_skills.push(match value {
            Some(v) => SkillWithValue::with_value(ffb_model::model::SkillId::Animosity, v),
            None => SkillWithValue::new(ffb_model::model::SkillId::Animosity),
        });
        let mut catcher = Player::default();
        catcher.keywords = catcher_keywords.into_iter().map(String::from).collect();
        (thrower, catcher)
    }

    #[test]
    fn animosity_exists_true_when_configured_all() {
        let (thrower, catcher) = animosity_player(Some("all"), vec!["Goblin"]);
        assert!(SkillMechanic.animosity_exists(&thrower, &catcher));
    }

    #[test]
    fn animosity_exists_true_when_unconfigured_defaults_to_all() {
        let (thrower, catcher) = animosity_player(None, vec!["Goblin"]);
        assert!(SkillMechanic.animosity_exists(&thrower, &catcher));
    }

    #[test]
    fn animosity_exists_true_when_catcher_keyword_matches_configured_value() {
        let (thrower, catcher) = animosity_player(Some("Goblin"), vec!["Lineman", "Goblin"]);
        assert!(SkillMechanic.animosity_exists(&thrower, &catcher));
    }

    #[test]
    fn animosity_exists_false_when_catcher_keyword_does_not_match() {
        let (thrower, catcher) = animosity_player(Some("Goblin"), vec!["Troll"]);
        assert!(!SkillMechanic.animosity_exists(&thrower, &catcher));
    }

    #[test]
    fn animosity_exists_temporary_value_overrides_configured_value() {
        use ffb_model::model::skill_def::SkillWithValue;
        let (mut thrower, catcher) = animosity_player(Some("Goblin"), vec!["Troll"]);
        thrower.temporary_skills.push(SkillWithValue::with_value(ffb_model::model::SkillId::Animosity, "Troll"));
        assert!(SkillMechanic.animosity_exists(&thrower, &catcher));
    }
}
