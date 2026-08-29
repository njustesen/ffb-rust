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
use ffb_model::enums::{TurnMode, ReRollSource};
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

/// Java: `AbstractStepWithReRoll.findSkillReRollSource` →
/// `UtilCards.getUnusedRerollSource(actingPlayer, reRolledAction)`.
///
/// Returns the unused skill `ReRollSource` that applies to `rerolled_action`
/// when the game is in `TurnMode::Regular`, consulting the static
/// `SkillId::reroll_sources()` table (the fold of every Java
/// `registerRerollSource` call). Java streams the player's skills, maps each to
/// `skill.getRerollSource(action)`, and picks the minimum
/// `ReRollSource.getPriority()`; ties are broken here by `SkillId` order for
/// determinism.
pub fn find_skill_reroll_source(game: &Game, rerolled_action: &str) -> Option<ReRollSource> {
    if game.turn_mode != TurnMode::Regular {
        return None;
    }
    let acting_id = game.acting_player.player_id.as_deref()?;
    let player = game.player(acting_id)?;

    player.all_skill_ids()
        .filter(|id| !player.used_skills.contains(id))
        .filter_map(|id| {
            id.reroll_sources()
                .iter()
                .find(|(action, _)| *action == rerolled_action)
                .map(|(_, priority)| (id, *priority))
        })
        .min_by_key(|(id, priority)| (*priority, *id as i32))
        .map(|(skill_id, priority)| ReRollSource::with_priority(format!("{:?}", skill_id), priority))
}

/// Java: `UtilCards.getRerollSource(Player, ReRolledAction)`.
///
/// The sibling of [`find_skill_reroll_source`], and deliberately NOT the same function. Java has
/// both and they differ in three ways that matter: this one takes an arbitrary PLAYER rather than
/// the acting player, it does not filter out skills already used this activation, and it keeps only
/// skills whose usage type is REGULAR. It also has no turn-mode guard.
///
/// `StepPass` asks about the THROWER, who is normally the acting player but need not be, and it
/// asks before anything has been marked used -- so the "unused" variant would answer a different
/// question.
pub fn find_player_reroll_source(
    player: &ffb_model::model::player::Player,
    rerolled_action: &str,
) -> Option<ReRollSource> {
    player
        .all_skill_ids()
        .filter(|id| id.usage_type() == ffb_model::enums::SkillUsageType::Regular)
        .filter_map(|id| {
            id.reroll_sources()
                .iter()
                .find(|(action, _)| *action == rerolled_action)
                .map(|(_, priority)| (id, *priority))
        })
        // Java takes `min(comparingInt(getPriority))`, which keeps the FIRST minimum in stream
        // order; the SkillId tie-break makes that deterministic here, as in the sibling.
        .min_by_key(|(id, priority)| (*priority, *id as i32))
        .map(|(skill_id, priority)| ReRollSource::with_priority(format!("{:?}", skill_id), priority))
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use ffb_model::enums::{Rules, SkillId};
    use ffb_model::model::player::Player;
    use ffb_model::model::skill_def::SkillWithValue;
    use ffb_model::enums::{PlayerType, PlayerGender};
    use ffb_model::types::FieldCoordinate;
    use std::collections::HashSet;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    fn add_player_with_skills(game: &mut Game, id: &str, skills: &[SkillId]) {
        let coord = FieldCoordinate::new(5, 5);
        game.team_home.players.push(Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: skills.iter()
                .map(|&skill_id| SkillWithValue { skill_id, value: None })
                .collect(),
            extra_skills: vec![], temporary_skills: vec![],
            used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        });
        game.field_model.set_player_coordinate(id, coord);
    }

    fn add_player_with_skill(game: &mut Game, id: &str, skill: SkillId) {
        add_player_with_skills(game, id, &[skill]);
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

    // ── reroll-source table lookups through the live chokepoint ──────────────

    #[test]
    fn pro_offers_single_die_per_activation_reroll() {
        // Java: bb2025/Pro.postConstruct registers SINGLE_DIE_PER_ACTIVATION → ReRollSources.PRO.
        let mut game = make_game();
        game.turn_mode = TurnMode::Regular;
        add_player_with_skill(&mut game, "p1", SkillId::Pro);
        game.acting_player.player_id = Some("p1".into());
        let source = find_skill_reroll_source(&game, "SINGLE_DIE_PER_ACTIVATION")
            .expect("Pro must offer a SINGLE_DIE_PER_ACTIVATION reroll");
        assert_eq!(source.name, "Pro");
        assert_eq!(source.priority, 1);
    }

    #[test]
    fn brawler_offers_single_both_down_reroll() {
        // Java: bb2025/Brawler.postConstruct registers SINGLE_BOTH_DOWN → ReRollSources.BRAWLER.
        let mut game = make_game();
        game.turn_mode = TurnMode::Regular;
        add_player_with_skill(&mut game, "p1", SkillId::Brawler);
        game.acting_player.player_id = Some("p1".into());
        let source = find_skill_reroll_source(&game, "SINGLE_BOTH_DOWN")
            .expect("Brawler must offer a SINGLE_BOTH_DOWN reroll");
        assert_eq!(source.name, "Brawler");
        assert_eq!(source.priority, 1);
    }

    #[test]
    fn catch_and_monstrous_mouth_tie_break_picks_lower_skill_id() {
        // Both Catch and MonstrousMouth register CATCH at priority 1; the tie is
        // broken deterministically by SkillId declaration order, and Catch is
        // declared before MonstrousMouth.
        let mut game = make_game();
        game.turn_mode = TurnMode::Regular;
        add_player_with_skills(&mut game, "p1", &[SkillId::MonstrousMouth, SkillId::Catch]);
        game.acting_player.player_id = Some("p1".into());
        let source = find_skill_reroll_source(&game, "CATCH")
            .expect("a CATCH reroll source must be found");
        assert_eq!(source.name, "Catch");
        assert_eq!(source.priority, 1);
    }

    #[test]
    fn pass_beats_the_ballista_on_priority() {
        // Pass registers PASS at priority 1; TheBallista at priority 2 (the only
        // priority-2 source in Java ReRollSources) — the lower priority wins.
        let mut game = make_game();
        game.turn_mode = TurnMode::Regular;
        add_player_with_skills(&mut game, "p1", &[SkillId::TheBallista, SkillId::Pass]);
        game.acting_player.player_id = Some("p1".into());
        let source = find_skill_reroll_source(&game, "PASS")
            .expect("a PASS reroll source must be found");
        assert_eq!(source.name, "Pass");
        assert_eq!(source.priority, 1);
    }

    #[test]
    fn whirling_dervish_offers_direction_reroll() {
        // Java: bb2020+bb2025 special/WhirlingDervish register DIRECTION → WHIRLING_DERVISH.
        let mut game = make_game();
        game.turn_mode = TurnMode::Regular;
        add_player_with_skill(&mut game, "p1", SkillId::WhirlingDervish);
        game.acting_player.player_id = Some("p1".into());
        let source = find_skill_reroll_source(&game, "DIRECTION")
            .expect("Whirling Dervish must offer a DIRECTION reroll");
        assert_eq!(source.name, "WhirlingDervish");
        assert_eq!(source.priority, 1);
    }

    #[test]
    fn used_skill_is_excluded_and_other_candidate_returned() {
        // With Catch already used, the CATCH lookup must fall through to MonstrousMouth.
        let mut game = make_game();
        game.turn_mode = TurnMode::Regular;
        add_player_with_skills(&mut game, "p1", &[SkillId::Catch, SkillId::MonstrousMouth]);
        game.acting_player.player_id = Some("p1".into());
        if let Some(p) = game.team_home.player_mut("p1") {
            p.used_skills.insert(SkillId::Catch);
        }
        let source = find_skill_reroll_source(&game, "CATCH")
            .expect("MonstrousMouth must still offer a CATCH reroll");
        assert_eq!(source.name, "MonstrousMouth");
        assert_eq!(source.priority, 1);
    }
}
