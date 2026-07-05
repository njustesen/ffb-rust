/// 1:1 translation of `com.fumbbl.ffb.server.util.UtilServerReRoll` (selected methods).
///
/// The Java version delegates most logic to `RollMechanic.askForReRollIfAvailable` /
/// `RollMechanic.useReRoll`. The Rust translation provides the minimal set needed to
/// complete the BB2025 step re-roll branches without the full mechanic factory hierarchy.
///
/// **Re-roll availability check** (Java `askForReRollIfAvailable`):
/// Returns `true` if a re-roll prompt was issued (step should return `StepOutcome::cont()`
/// to wait for the agent's `Action::UseReRoll` response).
///
/// **Re-roll consumption** (Java `useReRoll`):
/// Returns `true` on success (re-roll token consumed or skill marked used).
use ffb_model::enums::ReRollSource;
use ffb_model::model::game::Game;
use ffb_model::prompts::AgentPrompt;
use crate::step::abstract_step_with_re_roll::find_skill_reroll_source;

/// Java: `UtilServerReRoll.askForReRollIfAvailable(gameState, actingPlayer, reRolledAction,
///         minimumRoll, fumble)`.
///
/// Returns `Some(prompt)` when a re-roll is available and the agent should be prompted.
/// Returns `None` when no re-roll is available.
///
/// Re-roll sources checked (in priority order, mirrors BB2025 RollMechanic):
///   1. Skill re-roll (e.g. Dodge, Sure Feet, Sprint) — single-use
///   2. Team Re-Roll token (TRR) — if not already used this half-turn
///
/// The returned `AgentPrompt` is a `ReRollOffer` that the step embeds in its
/// `StepOutcome::cont()`.  The agent then replies with `Action::UseReRoll { use_reroll }`.
pub fn ask_for_reroll_if_available(
    game: &Game,
    rerolled_action: &str,
    _minimum_roll: i32,
    _fumble: bool,
) -> Option<AgentPrompt> {
    let acting_team_id = if game.home_playing {
        game.team_home.id.clone()
    } else {
        game.team_away.id.clone()
    };

    // Skill re-roll check (highest priority)
    if let Some(source) = find_skill_reroll_source(game, rerolled_action) {
        return Some(AgentPrompt::ReRollOffer {
            source,
            action: rerolled_action.to_owned(),
            team_id: acting_team_id,
        });
    }

    // Team re-roll check
    let td = game.turn_data();
    if td.rerolls > 0 && !td.reroll_used {
        return Some(AgentPrompt::ReRollOffer {
            source: ReRollSource::new("TRR"),
            action: rerolled_action.to_owned(),
            team_id: acting_team_id,
        });
    }

    None
}

/// Java: `UtilServerReRoll.useReRoll(step, reRollSource, player)`.
///
/// Consumes the re-roll (decrements TRR count or marks skill used).
/// Returns `true` if the re-roll was successfully consumed, `false` otherwise.
///
/// The `re_roll_source` name is used to distinguish skill-based from team-based re-rolls.
/// Convention: TRR source name = "TRR"; skill sources = skill enum name.
pub fn use_reroll(game: &mut Game, re_roll_source: &ReRollSource, player_id: &str) -> bool {
    // Check if source is a team re-roll (TRR).
    // Java: ReRollSource.hasProperty(ReRollProperty.TRR) etc.
    if re_roll_source.name == "TRR"
        || re_roll_source.name == "BRILLIANT_COACHING"
        || re_roll_source.name == "MASCOT"
    {
        let td = game.turn_data_mut();
        if td.rerolls > 0 {
            td.rerolls -= 1;
            td.reroll_used = true;
            return true;
        }
        return false;
    }

    // Skill-based re-roll: mark the skill as used.
    use ffb_model::enums::SkillId;
    let skill_name = re_roll_source.name.clone();

    // Try to find the matching SkillId by name (via Debug name or skill id name()).
    let player_opt = game.team_home.player_mut(player_id)
        .or_else(|| game.team_away.player_mut(player_id));
    if let Some(player) = player_opt {
        let skill_ids: Vec<SkillId> = player.all_skill_ids().collect();
        for id in skill_ids {
            if format!("{:?}", id) == skill_name || id.class_name() == skill_name.as_str() {
                player.used_skills.insert(id);
                return true;
            }
        }
    }

    false
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use ffb_model::enums::{Rules, TurnMode};
    use ffb_model::model::player::Player;
    use ffb_model::model::skill_def::SkillWithValue;
    use ffb_model::enums::{SkillId, PlayerType, PlayerGender};
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
    fn no_reroll_available_when_no_trr_and_no_skill() {
        let game = make_game();
        let result = ask_for_reroll_if_available(&game, "DODGE", 3, false);
        assert!(result.is_none());
    }

    #[test]
    fn trr_available_returns_prompt() {
        let mut game = make_game();
        game.turn_mode = TurnMode::Regular;
        game.home_playing = true;
        game.turn_data_home.rerolls = 1;
        let result = ask_for_reroll_if_available(&game, "DODGE", 3, false);
        assert!(result.is_some());
    }

    #[test]
    fn trr_already_used_returns_none() {
        let mut game = make_game();
        game.home_playing = true;
        game.turn_data_home.rerolls = 1;
        game.turn_data_home.reroll_used = true;
        let result = ask_for_reroll_if_available(&game, "DODGE", 3, false);
        assert!(result.is_none());
    }

    #[test]
    fn use_reroll_trr_decrements_count() {
        let mut game = make_game();
        game.home_playing = true;
        game.turn_data_home.rerolls = 2;
        let source = ReRollSource::new("TRR");
        let ok = use_reroll(&mut game, &source, "p1");
        assert!(ok);
        assert_eq!(game.turn_data_home.rerolls, 1);
        assert!(game.turn_data_home.reroll_used);
    }

    #[test]
    fn use_reroll_trr_fails_when_empty() {
        let mut game = make_game();
        game.home_playing = true;
        game.turn_data_home.rerolls = 0;
        let source = ReRollSource::new("TRR");
        let ok = use_reroll(&mut game, &source, "p1");
        assert!(!ok);
    }

    #[test]
    fn use_reroll_skill_marks_used() {
        let mut game = make_game();
        add_player_with_skill(&mut game, "p1", SkillId::Dodge);
        let source = ReRollSource::new("Dodge");
        let ok = use_reroll(&mut game, &source, "p1");
        assert!(ok);
        assert!(game.team_home.player("p1").unwrap().used_skills.contains(&SkillId::Dodge));
    }
}
