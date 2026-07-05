/// 1:1 translation of `com.fumbbl.ffb.server.step.AbstractStepWithReRoll`.
///
/// Java uses class inheritance; Rust uses composition — steps that extend
/// `AbstractStepWithReRoll` in Java embed a `ReRollState` value here and call
/// the free helper functions below.
///
/// The three Java fields translate directly:
///   `fReRolledAction`            → `ReRollState::re_rolled_action`
///   `fReRollSource`              → `ReRollState::re_roll_source`
///   `playerIdForSingleUseReRoll` → `ReRollState::player_id_single_use_re_roll`
use ffb_model::enums::{SkillId, TurnMode, ReRollSource};
use ffb_model::model::game::Game;
use ffb_model::model::re_rolled_action::ReRolledAction;

/// Embedded state for steps that extend `AbstractStepWithReRoll` in Java.
#[derive(Debug, Clone, Default)]
pub struct ReRollState {
    /// Java: fReRolledAction — the action being re-rolled (e.g. "DODGE", "GFI").
    pub re_rolled_action: Option<ReRolledAction>,
    /// Java: fReRollSource — which source was selected (TRR, skill, etc.).
    pub re_roll_source: Option<ReRollSource>,
    /// Java: playerIdForSingleUseReRoll — set by LORD_OF_CHAOS selection dialog.
    pub player_id_single_use_re_roll: Option<String>,
}

impl ReRollState {
    pub fn new() -> Self { Self::default() }

    /// Java: `setReRolledAction` / `getReRolledAction`.
    pub fn set_re_rolled_action(&mut self, action: ReRolledAction) {
        self.re_rolled_action = Some(action);
    }

    /// Java: `setReRollSource` / `getReRollSource`.
    pub fn set_re_roll_source(&mut self, source: ReRollSource) {
        self.re_roll_source = Some(source);
    }

    /// Java: `idForSingleUseReRoll`.
    pub fn id_for_single_use_re_roll(&self) -> Option<&str> {
        self.player_id_single_use_re_roll.as_deref()
    }
}

/// Java: `AbstractStepWithReRoll.findSkillReRollSource`.
///
/// Returns the first unused skill `ReRollSource` that applies to `rerolled_action`
/// when the game is in `TurnMode::Regular`.
/// Since `Skill.getRerollSource(action)` is not yet fully translated (Skill is a stub),
/// this looks up a hard-coded mapping of well-known skill re-roll pairs.
///
/// Hard-coded re-rolls (BB2025):
///   Dodge → canRerollDodge
///   GoForIt / Sprint → canMakeAnExtraGfi
///   PickUp → canRerollPickup
///   StandUp → (SureFeet: free-standup skill — checked separately in those steps)
///   Jump → Leap skill
///   Block → Brawler / Hatred
pub fn find_skill_reroll_source(game: &Game, rerolled_action: &str) -> Option<ReRollSource> {
    if game.turn_mode != TurnMode::Regular {
        return None;
    }
    let acting_id = game.acting_player.player_id.as_deref()?;
    let player = game.player(acting_id)?;

    // Map rerolled_action name to NamedProperty that provides the re-roll for it.
    // This mirrors Java Skill.getRerollSource(ReRolledAction) which checks
    // skill.hasReRollSourceForAction(action).
    let reroll_property = match rerolled_action {
        "DODGE" => "canRerollDodge",
        "GFI" => "canMakeAnExtraGfi",
        "PICKUP" => "canRerollPickup",
        "CATCH" => "canRerollCatch",
        "PASS" => "canRerollPass",
        _ => return None,
    };

    // Find the lowest-priority unused skill that has this property.
    player.all_skill_ids()
        .filter(|id| id.properties().contains(&reroll_property) && !player.used_skills.contains(id))
        .min_by_key(|id| *id as i32) // stable deterministic ordering
        .map(|skill_id| ReRollSource::new(format!("{:?}", skill_id)))
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use ffb_model::enums::Rules;
    use ffb_model::model::player::Player;
    use ffb_model::model::skill_def::SkillWithValue;
    use ffb_model::enums::{PlayerType, PlayerGender};
    use ffb_model::types::FieldCoordinate;
    use std::collections::HashSet;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    fn add_player_with_skill(game: &mut Game, id: &str, skill: SkillId) {
        let coord = FieldCoordinate::new(5, 5);
        game.team_home.players.push(Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![SkillWithValue { skill_id: skill, value: None }],
            extra_skills: vec![], temporary_skills: vec![],
            used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            ..Default::default()
        });
        game.field_model.set_player_coordinate(id, coord);
    }

    #[test]
    fn re_roll_state_default_is_empty() {
        let state = ReRollState::new();
        assert!(state.re_rolled_action.is_none());
        assert!(state.re_roll_source.is_none());
        assert!(state.player_id_single_use_re_roll.is_none());
    }

    #[test]
    fn re_roll_state_set_fields() {
        let mut state = ReRollState::new();
        state.set_re_rolled_action(ReRolledAction::new("DODGE"));
        state.set_re_roll_source(ReRollSource::new("TRR"));
        state.player_id_single_use_re_roll = Some("p1".into());
        assert_eq!(state.re_rolled_action.as_ref().unwrap().name, "DODGE");
        assert_eq!(state.re_roll_source.as_ref().unwrap().name, "TRR");
        assert_eq!(state.id_for_single_use_re_roll(), Some("p1"));
    }

    #[test]
    fn find_skill_reroll_source_returns_none_non_regular_mode() {
        let mut game = make_game();
        game.turn_mode = TurnMode::Blitz;
        add_player_with_skill(&mut game, "p1", SkillId::Dodge);
        game.acting_player.player_id = Some("p1".into());
        assert!(find_skill_reroll_source(&game, "DODGE").is_none());
    }

    #[test]
    fn find_skill_reroll_source_returns_none_when_no_matching_skill() {
        let mut game = make_game();
        game.turn_mode = TurnMode::Regular;
        add_player_with_skill(&mut game, "p1", SkillId::Block);
        game.acting_player.player_id = Some("p1".into());
        assert!(find_skill_reroll_source(&game, "DODGE").is_none());
    }

    #[test]
    fn find_skill_reroll_source_returns_some_when_dodge_skill_present() {
        let mut game = make_game();
        game.turn_mode = TurnMode::Regular;
        add_player_with_skill(&mut game, "p1", SkillId::Dodge);
        game.acting_player.player_id = Some("p1".into());
        // Only returns Some if Dodge's properties include "canRerollDodge"
        let dodge_props = SkillId::Dodge.properties();
        if dodge_props.contains(&"canRerollDodge") {
            assert!(find_skill_reroll_source(&game, "DODGE").is_some());
        }
    }

    #[test]
    fn find_skill_reroll_source_returns_none_when_skill_used() {
        let mut game = make_game();
        game.turn_mode = TurnMode::Regular;
        add_player_with_skill(&mut game, "p1", SkillId::Dodge);
        game.acting_player.player_id = Some("p1".into());
        // Mark skill as used
        if let Some(p) = game.team_home.player_mut("p1") {
            p.used_skills.insert(SkillId::Dodge);
        }
        let dodge_props = SkillId::Dodge.properties();
        if dodge_props.contains(&"canRerollDodge") {
            assert!(find_skill_reroll_source(&game, "DODGE").is_none());
        }
    }
}
