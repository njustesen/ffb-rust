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

    // Team re-roll check. Java RollMechanic.isTeamReRollAvailable additionally gates on
    // allowsTeamReRoll(turnMode): KICKOFF / PASS_BLOCK / DUMP_OFF (edition-specific set) prohibit
    // team re-rolls. Without this gate a failed catch of a scattered/bouncing ball DURING the
    // kickoff wrongly consumed a team re-roll (bb2016 amazon seed2 i=149: a receiving-team catch
    // on the kickoff — Java bounces the ball, Rust rerolled the catch → desync).
    let td = game.turn_data();
    let team_re_roll_allowed = crate::mechanic::roll_mechanic_for(game.rules)
        .allows_team_re_roll(game.turn_mode);
    if td.rerolls > 0 && !td.reroll_used && team_re_roll_allowed {
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
        // `reroll_used` is set by Java in exactly ONE place -- `bb2016/RollMechanic`'s
        // `updateTurnDataAfterReRollUsage`, which is a two-line method whose whole body is
        // `setReRollUsed(true); setReRolls(reRolls - 1)`. BB2020 and BB2025 override that method
        // (`bb2025:464-488`) and never touch the flag, so in those editions it stays false and a
        // team may spend MORE THAN ONE team re-roll in a turn, bounded only by the bank.
        //
        // The availability CHECK is shared (Java's `RollMechanic.isTeamReRollAvailable` tests
        // `!isReRollUsed()` for every edition) -- it is only the SET that is bb2016-only. Rust set
        // it unconditionally, so BB2020/BB2025 refused the SECOND re-roll of a turn: lineman bb2025
        // seed 16, home fails a dodge (die 14), re-rolls it (15, success), fails the next dodge
        // (16) and Java re-rolls again (17, success) while Rust fell over and rolled armour+injury.
        // Structurally invisible while the parity contract declined every offer.
        let is_bb2016 = game.rules == ffb_model::enums::Rules::Bb2016;
        let td = game.turn_data_mut();
        if td.rerolls > 0 {
            td.rerolls -= 1;
            if is_bb2016 {
                td.reroll_used = true;
            }
            // Java `RollMechanic.updateTurnDataAfterReRollUsage` (`bb2025:464-488`) spends the
            // ONE-DRIVE re-rolls FIRST: alongside the pool it decrements the first non-zero
            // one-drive counter, in this order, and returns that source for the report.
            //
            // Rust decremented only the pool. The counter therefore still read 1 at the end of the
            // drive, where `StepEndTurn::remove_rerolls_lasting_for_drive` subtracts the sum back
            // out again -- so a re-roll that had already been SPENT was clawed back a second time,
            // costing the team a PERMANENT re-roll. Measured on lineman bb2025 seed 1 with the
            // `reroll` rung on: home granted Brilliant Coaching at the half-2 kickoff (3 -> 4),
            // spent it (4 -> 3), then lost another at the final whistle (3 -> 2) where Java ends
            // on 3. Structurally invisible until an agent actually ACCEPTS a re-roll -- under the
            // random parity contract every offer is declined, so the counters never move and the
            // double-subtraction has nothing to bite.
            if td.rerolls_brilliant_coaching_one_drive > 0 {
                td.rerolls_brilliant_coaching_one_drive -= 1;
            } else if td.rerolls_pump_up_the_crowd_one_drive > 0 {
                td.rerolls_pump_up_the_crowd_one_drive -= 1;
            } else if td.reroll_show_star_one_drive > 0 {
                td.reroll_show_star_one_drive -= 1;
            }
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

    /// `reroll_used` -- the "one team re-roll per turn" latch -- is set by Java in exactly ONE
    /// place, `bb2016/RollMechanic.updateTurnDataAfterReRollUsage`. BB2020 and BB2025 override that
    /// method and never touch the flag, so those editions allow MORE THAN ONE team re-roll per
    /// turn, bounded only by the bank. The availability CHECK is shared; only the SET is
    /// edition-specific.
    ///
    /// Rust set it for every edition, so BB2020/BB2025 refused the second re-roll of a turn
    /// (lineman bb2025 seed 16: Java re-rolls two failed dodges in one turn, Rust re-rolled the
    /// first and fell over on the second).
    #[test]
    fn reroll_used_latch_is_bb2016_only() {
        let src = ReRollSource::new("TRR");

        for (rules, expect_latch) in [
            (Rules::Bb2016, true),
            (Rules::Bb2020, false),
            (Rules::Bb2025, false),
        ] {
            let mut game = Game::new(test_team("home", 0), test_team("away", 0), rules);
            game.turn_data_home.rerolls = 3;
            assert!(use_reroll(&mut game, &src, "nobody"));
            assert_eq!(game.turn_data_home.rerolls, 2, "{rules:?} always spends from the bank");
            assert_eq!(
                game.turn_data_home.reroll_used, expect_latch,
                "{rules:?}: only bb2016 latches reroll_used"
            );

            // …and the consequence: a SECOND team re-roll in the same turn is offered in
            // bb2020/bb2025 and refused in bb2016. `ask_for_reroll_if_available` is the caller
            // that matters, and it gates on `!reroll_used` for every edition.
            let offer = ask_for_reroll_if_available(&game, "DODGE", 4, false);
            assert_eq!(
                offer.is_some(), !expect_latch,
                "{rules:?}: a second team re-roll must be offered iff the latch is unset"
            );
        }
    }

    /// Java `RollMechanic.updateTurnDataAfterReRollUsage` spends the ONE-DRIVE re-rolls first:
    /// consuming a team re-roll decrements the pool AND the first non-zero one-drive counter, in
    /// the order Brilliant Coaching -> Pump Up The Crowd -> Show Star.
    ///
    /// Without that, the counter still reads 1 at the end of the drive, where
    /// `StepEndTurn::remove_rerolls_lasting_for_drive` subtracts it back out -- clawing back a
    /// re-roll that was already spent and costing the team a PERMANENT one (lineman bb2025 seed 1
    /// with the `reroll` rung on: Java ends `r3,3`, Rust ended `r2,3`).
    #[test]
    fn spending_a_team_reroll_also_spends_a_one_drive_reroll() {
        let src = ReRollSource::new("TRR");

        // Brilliant Coaching is spent first.
        let mut game = make_game();
        game.turn_data_home.rerolls = 4;
        game.turn_data_home.rerolls_brilliant_coaching_one_drive = 1;
        game.turn_data_home.rerolls_pump_up_the_crowd_one_drive = 1;
        assert!(use_reroll(&mut game, &src, "nobody"));
        assert_eq!(game.turn_data_home.rerolls, 3);
        assert_eq!(game.turn_data_home.rerolls_brilliant_coaching_one_drive, 0);
        assert_eq!(
            game.turn_data_home.rerolls_pump_up_the_crowd_one_drive, 1,
            "only the FIRST non-zero counter is spent, matching Java's if/else-if chain"
        );

        // Then Pump Up The Crowd, then Show Star.
        let mut game = make_game();
        game.turn_data_home.rerolls = 3;
        game.turn_data_home.rerolls_pump_up_the_crowd_one_drive = 1;
        game.turn_data_home.reroll_show_star_one_drive = 1;
        assert!(use_reroll(&mut game, &src, "nobody"));
        assert_eq!(game.turn_data_home.rerolls_pump_up_the_crowd_one_drive, 0);
        assert_eq!(game.turn_data_home.reroll_show_star_one_drive, 1);

        // A plain re-roll with no one-drive grant outstanding touches only the pool.
        let mut game = make_game();
        game.turn_data_home.rerolls = 3;
        assert!(use_reroll(&mut game, &src, "nobody"));
        assert_eq!(game.turn_data_home.rerolls, 2);
        assert_eq!(game.turn_data_home.rerolls_brilliant_coaching_one_drive, 0);

        // An empty pool consumes nothing at all.
        let mut game = make_game();
        game.turn_data_home.rerolls = 0;
        game.turn_data_home.rerolls_brilliant_coaching_one_drive = 1;
        assert!(!use_reroll(&mut game, &src, "nobody"));
        assert_eq!(game.turn_data_home.rerolls_brilliant_coaching_one_drive, 1);
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
            is_big_guy: false,
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
    fn trr_not_offered_during_kickoff() {
        // A team re-roll must NOT be offered while turn_mode prohibits it (KICKOFF): Java
        // RollMechanic.isTeamReRollAvailable gates on allowsTeamReRoll(turnMode). Regression for
        // bb2016 seed2 i=149: a receiving-team catch on the kickoff wrongly used a team re-roll.
        let mut game = make_game();
        game.home_playing = true;
        game.turn_data_home.rerolls = 1;
        game.turn_mode = TurnMode::Kickoff;
        assert!(ask_for_reroll_if_available(&game, "CATCH", 3, false).is_none(),
            "no team re-roll may be offered during KICKOFF");
        // Sanity: the same TRR IS offered on a regular turn.
        game.turn_mode = TurnMode::Regular;
        assert!(ask_for_reroll_if_available(&game, "CATCH", 3, false).is_some(),
            "a team re-roll IS available on a regular turn");
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
        // `make_game()` is BB2025, and BB2025 does NOT latch `reroll_used` -- Java sets it only in
        // `bb2016/RollMechanic`. This assertion used to require the latch, pinning a deviation that
        // refused the second team re-roll of a turn; see `reroll_used_latch_is_bb2016_only`.
        assert!(!game.turn_data_home.reroll_used);
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
