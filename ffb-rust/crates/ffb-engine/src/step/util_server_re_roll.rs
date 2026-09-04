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
use ffb_model::util::rng::GameRng;

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
/// Java `UtilServerReRoll.askForReRollIfAvailable(gameState, player, ...)` takes the PLAYER who
/// made the roll and reaches `RollMechanic.isTeamReRollAvailable`, whose first condition is
/// `actingTeam.hasPlayer(pPlayer)` — **a team re-roll is only ever offered for a roll made by the
/// team whose turn it is.**
///
/// Rust took no player and therefore skipped that condition. Most callers roll for the acting
/// player, where it is trivially satisfied, so this delegates to the acting player and is a no-op
/// for them. The exceptions are the rolls made by someone on the OTHER team — a catch by the
/// opponent after a scattered ball is the common one — and those must pass the real player through
/// [`ask_for_reroll_if_available_for`].
pub fn ask_for_reroll_if_available(
    game: &Game,
    rerolled_action: &str,
    minimum_roll: i32,
    fumble: bool,
) -> Option<AgentPrompt> {
    // Java `UtilServerReRoll.askForReRollIfAvailable(GameState, ActingPlayer, …)`:
    //   ReRollSource reRollSource = UtilCards.getUnusedRerollSource(actingPlayer, reRolledAction, …);
    //   Skill reRollSkill = reRollSource != null ? reRollSource.getSkill(game) : null;
    //   return askForReRollIfAvailable(gameState, actingPlayer.getPlayer(), …, reRollSkill);
    // The action-keyed skill lookup belongs to THIS overload only — it reads the ACTING player.
    let acting = game.acting_player.player_id.clone();
    let reroll_skill = find_skill_reroll_source(game, rerolled_action);
    ask_for_reroll_if_available_inner(
        game, acting.as_deref(), rerolled_action, minimum_roll, fumble, reroll_skill)
}

/// As [`ask_for_reroll_if_available`], but for a roll made by `player_id` — which may be on the
/// NON-acting team. Mirrors Java's player argument and its `actingTeam.hasPlayer` gate.
///
/// `None` means "no particular player", which skips the membership gate; use it only where Java
/// itself has no player to pass.
pub fn ask_for_reroll_if_available_for(
    game: &Game,
    player_id: Option<&str>,
    rerolled_action: &str,
    minimum_roll: i32,
    fumble: bool,
) -> Option<AgentPrompt> {
    // Java's PLAYER overload does NOT look a re-roll source up by action. Only the ACTING-PLAYER
    // overload does that (`UtilCards.getUnusedRerollSource(actingPlayer, reRolledAction, …)`);
    // reaching `RollMechanic.askForReRollIfAvailable` with `reRollSkill == null` the mechanic
    // considers exactly one skill term, `getUnusedSkillWithProperty(player,
    // canRerollSingleDieOncePerPeriod)` — see `ask_for_reroll_if_available_inner`.
    ask_for_reroll_if_available_inner(game, player_id, rerolled_action, minimum_roll, fumble, None)
}

/// Java: `RollMechanic.askForReRollIfAvailable(GameState, Player, ReRolledAction, int, boolean,
///        Skill modificationSkill, Skill reRollSkill, …)` — the single method both public
/// overloads funnel into. `reroll_skill` is the caller-supplied source: `Some` only from the
/// ACTING-PLAYER overload, which resolves it from the acting player's skills by action.
fn ask_for_reroll_if_available_inner(
    game: &Game,
    player_id: Option<&str>,
    rerolled_action: &str,
    _minimum_roll: i32,
    _fumble: bool,
    reroll_skill: Option<ReRollSource>,
) -> Option<AgentPrompt> {
    // Java `RollMechanic.isTeamReRollAvailable`: `actingTeam.hasPlayer(pPlayer) && ...`.
    // A catch by the opposing team is offered NO team re-roll — Rust offered one and spent it,
    // which showed up as `r1,3` (Java) against `r0,3` (Rust) with the dice otherwise identical
    // (lineman bb2025 seed 45 i=265). Only reachable once an agent accepts re-rolls at all.
    if let Some(pid) = player_id {
        let on_acting_team = if game.home_playing {
            game.team_home.has_player(pid)
        } else {
            game.team_away.has_player(pid)
        };
        if !on_acting_team {
            return None;
        }
    }
    let acting_team_id = if game.home_playing {
        game.team_home.id.clone()
    } else {
        game.team_away.id.clone()
    };

    // Java: `if (reRollSkill == null) { reRollSkill = getUnusedSkillWithProperty(player,
    //        canRerollSingleDieOncePerPeriod).orElse(null); }` — the ONLY skill term the mechanic
    // adds on its own, and it reads the PLAYER it was given, not the acting player.
    //
    // Rust instead called `find_skill_reroll_source` here unconditionally, and that helper reads
    // `game.acting_player` regardless of the `player_id` argument. So a step that correctly passed
    // a non-acting player still had the ACTING player's skills consulted: bb2020 halfling seed 24
    // @1e6 i=41, a Halfling Catcher throws a pass, the (Catch-less) receiver fluffs the catch and
    // Rust raised `ReRollOffer{source: Catch}` off the THROWER's Catch — Java offers nothing (the
    // catcher's own Catch is consumed by `CatchBehaviour`'s hook, and the bank was empty), bounces
    // the ball at d8 pos=104 and the two engines' dice split there.
    let skill_source = reroll_skill.or_else(|| {
        let pid = player_id?;
        let player = game.player(pid)?;
        player.all_skill_ids()
            .filter(|id| !player.used_skills.contains(id))
            .find(|id| id.properties()
                .contains(&ffb_model::model::property::named_properties::NamedProperties::CAN_REROLL_SINGLE_DIE_ONCE_PER_PERIOD))
            .map(|id| ReRollSource::new(format!("{:?}", id)))
    });
    if let Some(source) = skill_source {
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
    // Java RollMechanic.isTeamReRollAvailable, the four bomb-mode terms:
    //   (turnMode != BOMB_HOME || homeHasPlayer) && (turnMode != BOMB_HOME_BLITZ || homeHasPlayer)
    //   && (turnMode != BOMB_AWAY || awayHasPlayer) && (turnMode != BOMB_AWAY_BLITZ || awayHasPlayer)
    // — during a bomb, only a roller on the bomb-OWNING team may be offered the team re-roll.
    // An intercepted-and-re-thrown bomb flips home_playing to the re-thrower, whose team's
    // re-roll Rust then offered for the inaccurate re-throw; Java offers nothing (goblin
    // bb2016 seed 39 k=12: Rust 2 extra sampler draws on a declined PASS TRR offer).
    let bomb_side_ok = {
        use ffb_model::enums::TurnMode;
        match (game.turn_mode, player_id) {
            (TurnMode::BombHome | TurnMode::BombHomeBlitz, Some(pid)) => game.team_home.has_player(pid),
            (TurnMode::BombAway | TurnMode::BombAwayBlitz, Some(pid)) => game.team_away.has_player(pid),
            _ => true,
        }
    };
    if td.rerolls > 0 && !td.reroll_used && team_re_roll_allowed && bomb_side_ok {
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
/// Java `RollMechanic.useReRoll` (all three editions, same shape): a player with **Loner** has to
/// roll for a TEAM re-roll to work at all, and the re-roll is spent either way.
///
/// ```java
/// if (pPlayer.hasSkillProperty(NamedProperties.hasToRollToUseTeamReroll)) {
///     int roll = gameState.getDiceRoller().rollSkill();
///     int minimumRoll = minimumLonerRoll(pPlayer);          // the skill's own X+ value
///     successful = DiceInterpreter.getInstance().isSkillRollSuccessful(roll, minimumRoll);
///     stepResult.addReport(new ReportReRoll(pPlayer.getId(), ReRollSources.LONER, successful, roll));
/// } else {
///     successful = true;
/// }
/// ```
///
/// This was missing entirely: nothing in the engine ever rolled it, and `GameEvent::LonerRoll`
/// had no producer. Two things had to line up before it could be seen — the `reroll` prompt class
/// has to ACCEPT an offer (the random parity contract always declines), and someone has to HAVE
/// Loner, which in a lineman game only happens when a Prayer to Nuffle grants it. bb2020 seed 54
/// is that game: Java rolled the Loner die, Rust did not, and every roll afterwards was one
/// position out of step. The values happened to agree for ten more dice, so the dice-stream diff
/// pointed at a d8 scatter forty positions later.
fn loner_roll(game: &mut Game, player_id: &str, rng: &mut GameRng) -> bool {
    use ffb_model::model::property::named_properties::NamedProperties;
    let Some(player) = game.player(player_id) else { return true };
    if !player.has_skill_property(NamedProperties::HAS_TO_ROLL_TO_USE_TEAM_REROLL) {
        return true;
    }
    // Java: RollMechanic.minimumLonerRoll — bb2016 returns a FIXED 4 (`bb2016/RollMechanic.java:209`,
    // the LRB6 Loner has no printed value); bb2020/25 read the skill's value (Loner (4+) etc.).
    // Rust read the value everywhere, and a valueless bb2016 "Loner" yields 0 → every Loner check
    // auto-succeeded (is_skill_roll_successful(3, 0) = true where Java needs 4+). chaos bb2016
    // seed 10 i=8: the Minotaur's failed GFI re-roll — Java Loner 3 FAILS (min 4), re-roll lost,
    // player falls; Rust "passed", re-rolled the GFI and ran on.
    let minimum_roll = if game.rules == ffb_model::enums::Rules::Bb2016 {
        4
    } else {
        player.get_skill_int_value(NamedProperties::HAS_TO_ROLL_TO_USE_TEAM_REROLL)
    };
    let roll = rng.d6();
    let success = crate::dice_interpreter::DiceInterpreter::is_skill_roll_successful(roll, minimum_roll);
    if std::env::var_os("FFB_DRAWS").is_some() {
        eprintln!("RLONER pid={player_id} roll={roll} min={minimum_roll} success={success}");
    }
    game.report_list.add(ffb_model::report::report_re_roll::ReportReRoll::new(
        Some(player_id.to_string()),
        ReRollSource::new("Loner"),
        success,
        roll,
    ));
    success
}

pub fn use_reroll(
    game: &mut Game,
    re_roll_source: &ReRollSource,
    player_id: &str,
    rng: &mut GameRng,
) -> bool {
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
        // Java `RollMechanic.useTeamReRoll` (all three editions) NEVER re-checks the bank: reaching
        // useReRoll means the OFFER was made (availability is an ask-time check), and
        // `updateTurnDataAfterReRollUsage` decrements unconditionally — the counter can go
        // NEGATIVE. The stale-reRollSource re-entry (ARM_BAR answer into StepMoveDodge) reaches
        // this with the bank already empty, and Java spends anyway and re-rolls (chaos bb2025
        // seed 40 @1e6 i=8: Java's second spend at bank 0 → Loner 6 → fresh dodge 6 → the move
        // continues; Rust's `if rerolls > 0` guard returned false → fall + turnover). The guard
        // was a Rust invention; `rerolls` is i32 and may go negative exactly as Java's does.
        {
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
            return loner_roll(game, player_id, rng);
        }
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
    /// Java `RollMechanic.useTeamReRoll` decrements the bank UNCONDITIONALLY — availability is an
    /// ask-time check only, and the stale-source re-entry (ARM_BAR → StepMoveDodge) legitimately
    /// spends from an empty bank, going negative (chaos bb2025 seed 40 @1e6).
    #[test]
    fn a_team_reroll_spend_at_bank_zero_still_consumes_and_goes_negative() {
        use ffb_model::enums::{Rules, ReRollSource};
        let mut game = ffb_model::model::game::Game::new(
            crate::step::framework::test_team("home", 0),
            crate::step::framework::test_team("away", 0),
            Rules::Bb2025,
        );
        game.turn_data_mut().rerolls = 0;
        let mut rng = ffb_model::util::rng::GameRng::new(0);
        let consumed = use_reroll(&mut game, &ReRollSource::new("TRR"), "nobody", &mut rng);
        assert!(consumed, "the spend is unconditional once the offer was made");
        assert_eq!(game.turn_data_mut().rerolls, -1, "the counter goes negative, as Java's does");
    }

    /// Java bb2016 `RollMechanic.minimumLonerRoll` returns a FIXED 4 (the LRB6 Loner has no
    /// printed value); bb2020/25 read the skill value. A valueless bb2016 Loner must NOT
    /// auto-succeed (chaos bb2016 seed 10 i=8: Loner 3 fails min 4, the GFI re-roll is lost).
    #[test]
    fn bb2016_loner_minimum_is_a_fixed_four() {
        use ffb_model::enums::{Rules, SkillId};
        use ffb_model::model::skill_def::SkillWithValue;
        let mut game = ffb_model::model::game::Game::new(
            crate::step::framework::test_team("home", 0),
            crate::step::framework::test_team("away", 0),
            Rules::Bb2016,
        );
        game.team_home.players.push(ffb_model::model::player::Player {
            id: "p1".into(), name: "p1".into(), nr: 1, position_id: "pos".into(),
            movement: 5, strength: 5, agility: 2, passing: 1, armour: 8,
            starting_skills: vec![SkillWithValue { skill_id: SkillId::Loner, value: None }],
            ..Default::default()
        });
        game.turn_data_mut().rerolls = 1;
        // Seed chosen so the loner d6 is 3: passes a 0-minimum, fails the fixed 4.
        let mut seed = 0u64;
        loop {
            if ffb_model::util::rng::GameRng::new(seed).d6() == 3 { break; }
            seed += 1;
        }
        let mut rng = ffb_model::util::rng::GameRng::new(seed);
        let consumed = use_reroll(&mut game, &ffb_model::enums::ReRollSource::new("TRR"), "p1", &mut rng);
        assert!(!consumed, "a bb2016 Loner roll of 3 must FAIL the fixed minimum of 4");
    }

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

    /// Java's `RollMechanic.isTeamReRollAvailable` opens with `actingTeam.hasPlayer(pPlayer)`: a
    /// team re-roll is only ever offered for a roll made by the team whose turn it is. Rust took no
    /// player and skipped the condition, so a roll by the OPPONENT -- a catch after a scattered
    /// ball is the common one -- was offered a team re-roll Java never offers.
    #[test]
    fn a_team_reroll_is_only_offered_for_the_acting_teams_own_roll() {
        let mut game = make_game();
        game.home_playing = true;
        game.turn_data_home.rerolls = 3;
        game.turn_data_away.rerolls = 3;
        add_player_with_skill(&mut game, "home_01", SkillId::Block);
        // add_player_with_skill pushes onto team_home; place an away player by hand.
        let away = game.team_home.players.last().cloned().expect("just pushed");
        game.team_away.players.push(Player { id: "away_01".into(), ..away });

        // The acting team's own roll is offered a re-roll…
        assert!(
            ask_for_reroll_if_available_for(&game, Some("home_01"), "CATCH", 4, false).is_some(),
            "the acting team's own roll must be offered its team re-roll"
        );
        // …the opponent's is not, even though the away team has re-rolls of its own: the offer is
        // always drawn from the ACTING team's bank, so Java refuses rather than dipping into it.
        assert!(
            ask_for_reroll_if_available_for(&game, Some("away_01"), "CATCH", 4, false).is_none(),
            "a roll by the non-acting team must NOT be offered the acting team's re-roll"
        );
        // `None` means "no particular player" and keeps the old unguarded behaviour.
        assert!(ask_for_reroll_if_available_for(&game, None, "CATCH", 4, false).is_some());
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
    /// A **Loner** spends the team re-roll and then has to roll for it to work.
    ///
    /// `RollMechanic.useReRoll` does this in all three editions and Rust did it in none — nothing
    /// in the engine rolled the die. It takes two coincidences to see: the agent has to ACCEPT a
    /// re-roll (the random parity contract always declines) and the player has to HAVE Loner,
    /// which no lineman does by roster. bb2020 seed 54 supplied both — a Cheering Fans kickoff
    /// rolled the BAD_HABITS prayer, which grants Loner to the opposing team, and the away
    /// lineman that re-rolled a dodge two turns later was one of them.
    #[test]
    fn a_loner_rolls_for_the_team_reroll_and_spends_it_either_way() {
        use ffb_model::model::property::named_properties::NamedProperties;
        use ffb_model::model::skill_def::SkillWithValue;
        use ffb_model::enums::SkillId;
        use ffb_model::model::player::Player;

        let src = ReRollSource::new("TRR");
        let build = || {
            let mut home = test_team("home", 0);
            home.players.push(Player {
                id: "loner_01".into(),
                nr: 1,
                movement: 6,
                strength: 3,
                agility: 3,
                armour: 8,
                extra_skills: vec![SkillWithValue {
                    skill_id: SkillId::Loner,
                    value: Some("4+".into()),
                }],
                ..Default::default()
            });
            let mut game = Game::new(home, test_team("away", 0), Rules::Bb2025);
            game.turn_data_home.rerolls = 3;
            game
        };

        // Loner 4+ registers the property, and its int value IS the minimum roll.
        let g = build();
        let p = g.player("loner_01").expect("the loner");
        assert!(p.has_skill_property(NamedProperties::HAS_TO_ROLL_TO_USE_TEAM_REROLL));
        assert_eq!(p.get_skill_int_value(NamedProperties::HAS_TO_ROLL_TO_USE_TEAM_REROLL), 4);

        // Whatever the die says, the re-roll is GONE. That is the half a naive "return false"
        // would get wrong, and it is the half the bank tracks.
        let mut seen_success = false;
        let mut seen_failure = false;
        for seed in 0..40u64 {
            let mut game = build();
            let ok = use_reroll(&mut game, &src, "loner_01", &mut GameRng::new(seed));
            assert_eq!(game.turn_data_home.rerolls, 2, "seed {seed}: spent either way");
            if ok { seen_success = true } else { seen_failure = true }
        }
        assert!(seen_success && seen_failure, "Loner 4+ must be able to go both ways");

        // A player WITHOUT Loner never rolls, so every other re-roll in the game leaves the die
        // stream exactly where it was — which is what kept this invisible for so long.
        let mut game = build();
        let mut rng = GameRng::new(7);
        assert!(use_reroll(&mut game, &src, "nobody", &mut rng));
        assert_eq!(rng.d6(), GameRng::new(7).d6(), "a non-Loner re-roll must not consume a die");
    }

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
            assert!(use_reroll(&mut game, &src, "nobody", &mut GameRng::new(0)));
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
        assert!(use_reroll(&mut game, &src, "nobody", &mut GameRng::new(0)));
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
        assert!(use_reroll(&mut game, &src, "nobody", &mut GameRng::new(0)));
        assert_eq!(game.turn_data_home.rerolls_pump_up_the_crowd_one_drive, 0);
        assert_eq!(game.turn_data_home.reroll_show_star_one_drive, 1);

        // A plain re-roll with no one-drive grant outstanding touches only the pool.
        let mut game = make_game();
        game.turn_data_home.rerolls = 3;
        assert!(use_reroll(&mut game, &src, "nobody", &mut GameRng::new(0)));
        assert_eq!(game.turn_data_home.rerolls, 2);
        assert_eq!(game.turn_data_home.rerolls_brilliant_coaching_one_drive, 0);

        // Java's updateTurnDataAfterReRollUsage never checks the pool: an "empty" pool still
        // spends (to -1) and still consumes the outstanding one-drive grant.
        let mut game = make_game();
        game.turn_data_home.rerolls = 0;
        game.turn_data_home.rerolls_brilliant_coaching_one_drive = 1;
        assert!(use_reroll(&mut game, &src, "nobody", &mut GameRng::new(0)));
        assert_eq!(game.turn_data_home.rerolls, -1);
        assert_eq!(game.turn_data_home.rerolls_brilliant_coaching_one_drive, 0);
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

    /// Java has TWO `askForReRollIfAvailable` families and they are not the same contract:
    ///
    /// ```java
    /// // ACTING-PLAYER overload (UtilServerReRoll:43-53)
    /// ReRollSource reRollSource = UtilCards.getUnusedRerollSource(actingPlayer, reRolledAction, ignoreSkills);
    /// Skill reRollSkill = reRollSource != null ? reRollSource.getSkill(game) : null;
    /// return askForReRollIfAvailable(gameState, actingPlayer.getPlayer(), …, reRollSkill);
    ///
    /// // PLAYER overload → RollMechanic:239-269 — no action-keyed lookup at all
    /// if (reRollSkill == null) {
    ///     Optional<Skill> reRollOnce = UtilCards.getUnusedSkillWithProperty(player, canRerollSingleDieOncePerPeriod);
    ///     if (reRollOnce.isPresent()) { reRollSkill = reRollOnce.get(); }
    /// }
    /// ```
    ///
    /// `StepCatchScatterThrowIn` calls the PLAYER overload with `state.catcher`. Rust collapsed
    /// both into one function whose skill lookup always read `game.acting_player`, so the THROWER's
    /// Catch was offered for a team-mate's failed catch (bb2020 halfling seed 24 @1e6).
    #[test]
    fn the_player_overload_does_not_read_the_acting_players_skill_reroll() {
        let mut game = make_game();
        game.home_playing = true;
        game.turn_mode = ffb_model::enums::TurnMode::Regular;
        game.turn_data_home.rerolls = 0; // empty bank: only a skill source could open a dialog
        add_player_with_skill(&mut game, "thrower", SkillId::Catch);
        add_player_with_skill(&mut game, "catcher", SkillId::Block); // no Catch
        game.acting_player.player_id = Some("thrower".into());

        // The ACTING-PLAYER overload still finds the acting player's Catch — unchanged.
        assert!(
            matches!(ask_for_reroll_if_available(&game, "CATCH", 4, false),
                     Some(AgentPrompt::ReRollOffer { .. })),
            "the acting-player overload must still resolve the acting player's own Catch"
        );
        // The PLAYER overload, asked about the CATCHER, must offer nothing: the catcher has no
        // Catch, there is no team re-roll left, and Java never consults the acting player here.
        assert!(
            ask_for_reroll_if_available_for(&game, Some("catcher"), "CATCH", 4, false).is_none(),
            "a Catch-less catcher with an empty bank must be offered no re-roll, \
             regardless of what the thrower holds"
        );
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
        let ok = use_reroll(&mut game, &source, "p1", &mut GameRng::new(0));
        assert!(ok);
        assert_eq!(game.turn_data_home.rerolls, 1);
        // `make_game()` is BB2025, and BB2025 does NOT latch `reroll_used` -- Java sets it only in
        // `bb2016/RollMechanic`. This assertion used to require the latch, pinning a deviation that
        // refused the second team re-roll of a turn; see `reroll_used_latch_is_bb2016_only`.
        assert!(!game.turn_data_home.reroll_used);
    }

    /// Java `useTeamReRoll` has no bank check — availability gates the OFFER, never the spend
    /// (the stale-source ARM_BAR re-entry spends from an empty bank; chaos bb2025 seed 40 @1e6).
    #[test]
    fn use_reroll_trr_spends_even_when_empty() {
        let mut game = make_game();
        game.home_playing = true;
        game.turn_data_home.rerolls = 0;
        let source = ReRollSource::new("TRR");
        let ok = use_reroll(&mut game, &source, "p1", &mut GameRng::new(0));
        assert!(ok);
        assert_eq!(game.turn_data_home.rerolls, -1);
    }

    #[test]
    fn use_reroll_skill_marks_used() {
        let mut game = make_game();
        add_player_with_skill(&mut game, "p1", SkillId::Dodge);
        let source = ReRollSource::new("Dodge");
        let ok = use_reroll(&mut game, &source, "p1", &mut GameRng::new(0));
        assert!(ok);
        assert!(game.team_home.player("p1").unwrap().used_skills.contains(&SkillId::Dodge));
    }
}
