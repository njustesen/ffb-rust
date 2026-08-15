/// 1:1 translation of com.fumbbl.ffb.server.step.bb2025.StepEndTurn (BB2025).
///
/// Ends the current turn and drives the turn/half/game state machine:
/// - TurnMode skip guard (BLITZ/KICKOFF_RETURN/etc.) → publish END_TURN + NEXT_STEP
/// - Touchdown detection → score update, TurnMode → SETUP
/// - End-of-half detection (both teams at turn_nr >= 8)
/// - KICKOFF/REGULAR TurnMode transitions (home_playing flip, turn_nr++)
/// - start_turn: resets both TurnData for the next turn
///
/// Stubs (untranslated server-side systems):
/// - ArgueTheCall, Bribes, StarOfTheShow dialogs → skip (choices set to Some(false) immediately)
/// - Secret weapon ban/bribe handling → skip
/// - new_half H2 push wired (h2_kickoff_sequence); end-game sequence not yet ported.
/// - EndGame sequence push → TODO (end-game generator not yet ported)
/// - Prayer/inducement deactivation → skip
/// - Per-drive reroll removal → skip
/// - Fainting (Sweltering Heat) → skip
/// - Reports/sounds/timers → skip
/// - FumbblGame update → skip
use std::collections::HashSet;
use ffb_model::enums::{TurnMode, Weather, PS_KNOCKED_OUT, PS_EXHAUSTED, PS_RESERVE};
use ffb_model::events::GameEvent;
use ffb_model::inducement::usage::Usage;
use ffb_model::model::game::Game;
use ffb_model::report::mixed::report_turn_end::{ReportTurnEnd, KnockoutRecovery, HeatExhaustion};
use ffb_model::types::FIELD_WIDTH;
use ffb_model::util::rng::GameRng;
use ffb_model::util::util_box::UtilBox;
use crate::action::Action;
use crate::dice_interpreter::DiceInterpreter;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::util::util_server_game::UtilServerGame;

pub struct StepEndTurn {
    /// Java: fTouchdown (Boolean tristate — None = unchecked)
    pub touchdown: Option<bool>,
    /// Java: fBribesChoiceHome
    pub bribes_choice_home: Option<bool>,
    /// Java: fBribesChoiceAway
    pub bribes_choice_away: Option<bool>,
    /// Java: fArgueTheCallChoiceHome
    pub argue_the_call_choice_home: Option<bool>,
    /// Java: fArgueTheCallChoiceAway
    pub argue_the_call_choice_away: Option<bool>,
    /// Java: useStarOfTheShow
    pub use_star_of_the_show: Option<bool>,
    /// Java: fNextSequencePushed
    pub next_sequence_pushed: bool,
    /// Java: fRemoveUsedSecretWeapons
    pub remove_used_secret_weapons: bool,
    /// Java: fNewHalf
    pub new_half: bool,
    /// Java: fEndGame
    pub end_game: bool,
    /// Java: fWithinSecretWeaponHandling
    pub within_secret_weapon_handling: bool,
    /// Java: turnNr (captured at step start for send-to-box reason)
    pub turn_nr: i32,
    /// Java: half (captured at step start)
    pub half: i32,
    /// Java: playerIdsNaturalOnes
    pub player_ids_natural_ones: Vec<String>,
    /// Java: playerIdsFailedBribes
    pub player_ids_failed_bribes: HashSet<String>,
    /// Java: playerIdsArgued
    pub player_ids_argued: HashSet<String>,
    /// Guards the KO-recovery/fainting block against running twice — bb2016 runs it BEFORE the
    /// Secret Weapon send-off, bb2025 after.
    pub ko_recovery_done: bool,
    /// Java: touchdownPlayerId
    pub touchdown_player_id: Option<String>,
    /// Java: isHomeTurnEnding = game.isHomePlaying() — captured before home_playing is flipped.
    pub is_home_turn_ending: Option<bool>,
}

impl StepEndTurn {
    pub fn new() -> Self {
        Self {
            touchdown: None,
            bribes_choice_home: None,
            bribes_choice_away: None,
            argue_the_call_choice_home: None,
            argue_the_call_choice_away: None,
            use_star_of_the_show: None,
            next_sequence_pushed: false,
            remove_used_secret_weapons: false,
            new_half: false,
            end_game: false,
            within_secret_weapon_handling: false,
            turn_nr: 0,
            half: 0,
            player_ids_natural_ones: Vec::new(),
            player_ids_failed_bribes: HashSet::new(),
            player_ids_argued: HashSet::new(),
            ko_recovery_done: false,
            touchdown_player_id: None,
            is_home_turn_ending: None,
        }
    }

    /// Java: UtilServerSteps.checkTouchdown — ball in play, not moving, carrier not
    /// prone/stunned, ball is in the correct end zone for the carrier's team.
    fn check_touchdown(game: &Game) -> bool {
        if !game.field_model.ball_in_play || game.field_model.ball_moving {
            return false;
        }
        let ball_coord = match game.field_model.ball_coordinate {
            Some(c) => c,
            None => return false,
        };
        let carrier_id = match game.field_model.player_at(ball_coord) {
            Some(id) => id.clone(),
            None => return false,
        };
        let carrier_state = match game.field_model.player_state(&carrier_id) {
            Some(s) => s,
            None => return false,
        };
        if carrier_state.is_prone_or_stunned() {
            return false;
        }
        let home_has_carrier = game.team_home.player(&carrier_id).is_some();
        if home_has_carrier {
            ball_coord.x == FIELD_WIDTH - 1  // FieldCoordinateBounds.ENDZONE_AWAY
        } else {
            ball_coord.x == 0               // FieldCoordinateBounds.ENDZONE_HOME
        }
    }

    /// Java: UtilServerSteps.checkEndOfHalf — both teams have used all 8 turns.
    fn check_end_of_half(game: &Game) -> bool {
        game.turn_data_home.turn_nr >= 8 && game.turn_data_away.turn_nr >= 8
    }

    /// Java: reportSecretWeaponsUsed(415) → argueTheCall AWAY(422) then HOME(437) → removeUsedSecretWeapons(471).
    /// The end-of-drive Secret Weapon send-off. A played Secret Weapon (getsSentOffAtEndOfDrive, marked by
    /// `mark_played_and_secret_weapons`) is sent off UNLESS its coach argues the call successfully. The parity
    /// harness (ParityRunner ARGUE_THE_CALL) ALWAYS argues the first eligible player per team, looping until
    /// none remain — AWAY team first, then HOME. Each argue rolls one `rollArgueTheCall` d6: success (6) keeps
    /// the player, 1 bans the coach (no more argues for that team), otherwise the player is banned. These argue
    /// dice enter the shared stream BEFORE the half-2 kickoff dice (dwarf seed 1: pos53 away=fail, pos54 home=success).
    /// Java StepEndTurn.reportSecretWeaponsUsed — Phase A of the Secret Weapon send-off. Runs at
    /// EVERY drive end (fNewHalf || fTouchdown), INCLUDING the end of the game — Java gates only the
    /// argue/remove phases by `!fEndGame`, never this ban roll. Each played Stunty-Leeg secret weapon
    /// (skill int value > 0) rolls 2d6 and its has_used_secret_weapon flag is set to `banned`
    /// (total >= penalty). Because it consumes dice even at game end, skipping it there desynced the
    /// shared stream from the end-of-game KO-recovery / MVP rolls (goblin seed 2 step 250: both teams'
    /// Bombardiers rolled in Java but not Rust). Java iterates game.getPlayers() = home then away.
    /// Java `StepEndTurn`: the end-of-drive KO-recovery / Sweltering-Heat fainting block
    /// (`recoverKnockout` + `heatExhaust`, bb2025 via `getFaintingCount`).
    ///
    /// **The ORDER of this block relative to the Secret Weapon send-off is edition-specific**, and
    /// both consume game rng:
    ///   bb2016 `StepEndTurn`: `recoverKnockout` (line 281) runs BEFORE
    ///     `reportSecretWeaponsUsed` (364) / `askForArgueTheCall` (387, 395) /
    ///     `removeUsedSecretWeapons` (404).
    ///   bb2025 `StepEndTurn`: `reportSecretWeaponsUsed` (415) / argue (427, 442) /
    ///     `removeUsedSecretWeapons` (471) run FIRST, then the KO recovery via
    ///     `getFaintingCount` (478 → 597).
    /// So the two editions roll these dice in OPPOSITE order. Extracted into a method so the
    /// caller can place it on either side of the send-off (dwarf bb2016 seed 5 step 128: the
    /// three halftime dice are 4, 6, 3 — Java spends 4 on the KO recovery (recovers) then 6/3
    /// on the away/home argues; the bb2025 order spent 4/6 on the argues and 3 on the KO
    /// recovery, so the KO'd player stayed down and the wrong team's Deathroller was banned).
    fn recover_knockouts_and_fainting(&mut self, game: &mut Game, rng: &mut GameRng, touchdown: bool) {
        // Java: getFaintingCount / heatExhaustions / KO recovery — only on new half or touchdown
        if self.new_half || touchdown {
            // BB2016 and BB2020+ resolve Sweltering-Heat fainting DIFFERENTLY. BB2016
            // (bb2016.StepEndTurn) rolls, PER on-pitch player, one rollKnockoutRecovery d6 and
            // faints (EXHAUSTED) on isExhausted(roll) — interleaved with KO recovery in the same
            // game.getPlayers() loop. BB2020+ (mixed/bb2025) instead roll a single d3 fainting
            // COUNT and pick that many random on-pitch players per team. Rust shared this step
            // (make_step routes EndTurn to bb2025's, make_step_for(bb2016) doesn't override) and
            // applied the BB2020+ d3-count model to bb2016 too — so a bb2016 Sweltering-Heat drive
            // end rolled a d3 where Java rolled one d6 per on-pitch player, missing ~1 die per
            // on-pitch player (amazon bb2016 seed20: half-2 kickoff, Java rolled 22 heatExhaust
            // d6s, Rust rolled 0 via the d3 path → 17-die desync at the transition). Edition-gate:
            // bb2016 → per-player heatExhaust; bb2020+ → the d3-count block, unchanged.
            let is_bb2016 = game.rules == ffb_model::enums::Rules::Bb2016;
            let sweltering = game.field_model.weather == Weather::SwelteringHeat;
            let all_player_ids: Vec<String> = game.team_home.players.iter()
                .chain(game.team_away.players.iter())
                .map(|p| p.id.clone())
                .collect();
            let mut ko_recoveries: Vec<KnockoutRecovery> = Vec::new();
            let mut heat_exhaustions: Vec<HeatExhaustion> = Vec::new();
            for player_id in &all_player_ids {
                let player_state = match game.field_model.player_state(player_id) {
                    Some(s) => s,
                    None => continue,
                };
                // Java captures playerCoordinate BEFORE KO recovery (recover→box); a player that
                // is not on-pitch here never rolls heatExhaust.
                let on_pitch_before = game.field_model.player_coordinate(player_id)
                    .map(|c| !c.is_box_coordinate())
                    .unwrap_or(false);
                let base = player_state.base();
                if base == PS_KNOCKED_OUT {
                    let is_home = game.team_home.has_player(player_id);
                    let bloodweiser_keg = if is_home {
                        game.turn_data_home.inducement_set.value(Usage::KNOCKOUT_RECOVERY)
                    } else {
                        game.turn_data_away.inducement_set.value(Usage::KNOCKOUT_RECOVERY)
                    };
                    let roll = rng.d6();
                    let recovered = DiceInterpreter::is_recovering_from_knockout(roll, bloodweiser_keg);
                    if recovered {
                        game.field_model.set_player_state(player_id, player_state.change_base(PS_RESERVE));
                    }
                    ko_recoveries.push(KnockoutRecovery::new(player_id.clone(), recovered));
                }
                if base == PS_EXHAUSTED {
                    game.field_model.set_player_state(player_id, player_state.change_base(PS_RESERVE));
                }
                // BB2016: per on-pitch player, roll one rollKnockoutRecovery d6 → faint on
                // isExhausted (Java bb2016.StepEndTurn heatExhaust). Interleaved here to match the
                // Java per-player loop order exactly.
                if is_bb2016 && sweltering && on_pitch_before {
                    let roll = rng.d6();
                    let exhausted = DiceInterpreter::is_exhausted(roll);
                    if exhausted {
                        game.field_model.set_player_state(player_id, player_state.change_base(PS_EXHAUSTED));
                        UtilBox::put_player_into_box(game, player_id);
                    }
                    heat_exhaustions.push(HeatExhaustion::new(player_id.clone(), roll));
                }
            }
            // BB2020+ (mixed/bb2025): AFTER KO recovery, roll ONE d3 fainting COUNT and pick that
            // many random on-pitch players per team (one rollDice(onPitch.size()) per faint, the
            // list shrinking as each is removed). NOT used for bb2016 (handled per-player above).
            if !is_bb2016 && sweltering {
                let fainting_count = rng.d3();
                for is_home in [true, false] {
                    let team = if is_home { &game.team_home } else { &game.team_away };
                    let mut on_pitch: Vec<String> = team.players.iter()
                        .filter(|p| game.field_model.player_coordinate(&p.id)
                            .map(|c| !c.is_box_coordinate())
                            .unwrap_or(false))
                        .map(|p| p.id.clone())
                        .collect();
                    let mut i = 0;
                    while i < fainting_count && !on_pitch.is_empty() {
                        let idx = rng.range(on_pitch.len());
                        let pid = on_pitch.remove(idx);
                        if let Some(cur) = game.field_model.player_state(&pid) {
                            game.field_model.set_player_state(&pid, cur.change_base(PS_EXHAUSTED));
                        }
                        UtilBox::put_player_into_box(game, &pid);
                        heat_exhaustions.push(HeatExhaustion::new(pid, 0));
                        i += 1;
                    }
                }
            }
            let td_player_id = if touchdown { game.acting_player.player_id.clone() } else { None };
            game.report_list.add(ReportTurnEnd::new(
                td_player_id,
                ko_recoveries,
                heat_exhaustions,
                vec![],
                0,
            ));
            UtilBox::put_all_players_into_box(game);
        }
    }

    fn report_secret_weapons_used(&mut self, game: &mut Game, rng: &mut GameRng) {
        use ffb_model::model::property::named_properties::NamedProperties;

        // A standard secret weapon (penalty 0, e.g. the dwarf Deathroller) is auto-banned with no die
        // (flag left set); a Stunty-Leeg secret weapon (penalty > 0) rolls 2d6 and is banned on total >= penalty.
        for is_home in [true, false] {
            let ids: Vec<String> = {
                let t = if is_home { &game.team_home } else { &game.team_away };
                t.players.iter().map(|p| p.id.clone()).collect()
            };
            for pid in ids {
                if !game.game_result.team_result_mut(is_home).player_result_mut(&pid).has_used_secret_weapon {
                    continue;
                }
                let (has_skill, penalty) = game.player(&pid)
                    .map(|p| (
                        p.has_skill_property(NamedProperties::GETS_SENT_OFF_AT_END_OF_DRIVE),
                        p.get_skill_int_value(NamedProperties::GETS_SENT_OFF_AT_END_OF_DRIVE),
                    ))
                    .unwrap_or((false, 0));
                if has_skill && penalty > 0 {
                    let total = rng.d6() + rng.d6();
                    let banned = total >= penalty;
                    game.game_result.team_result_mut(is_home).player_result_mut(&pid).has_used_secret_weapon = banned;
                }
            }
        }
    }

    /// Java StepEndTurn argueTheCall (AWAY then HOME) + removeUsedSecretWeapons — Phases B & C of the
    /// Secret Weapon send-off. The caller runs this only when NOT end-of-game (Java's `!fEndGame` guard
    /// on askForArgueTheCall / removeUsedSecretWeapons): a Secret Weapon is physically banned at a
    /// drive end that is not the end of the game, but the final half ending keeps it on the pitch.
    fn argue_and_remove_secret_weapons(&mut self, game: &mut Game, rng: &mut GameRng) {
        use ffb_model::enums::{PS_BANNED, PlayerState, SendToBoxReason};
        use ffb_model::model::property::named_properties::NamedProperties;
        use ffb_model::report::mixed::report_argue_the_call_roll::ReportArgueTheCallRoll;

        // Phase B — argueTheCall: AWAY team first, then HOME. ParityRunner argues each flagged SW player.
        for is_home in [false, true] {
            let team_id = { let t = if is_home { &game.team_home } else { &game.team_away }; t.id.clone() };
            let friends = game.prayer_state.is_friends_with_ref(&team_id);
            let ids: Vec<String> = {
                let t = if is_home { &game.team_home } else { &game.team_away };
                t.players.iter().map(|p| p.id.clone()).collect()
            };
            for pid in ids {
                let coach_banned = if is_home { game.turn_data_home.coach_banned } else { game.turn_data_away.coach_banned };
                if coach_banned { break; }
                if !game.game_result.team_result_mut(is_home).player_result_mut(&pid).has_used_secret_weapon {
                    continue;
                }
                // Java getPlayerIds — the eligibility filter is EDITION-SPECIFIC:
                //   bb2025: `!PlayerState.REMOVED_FROM_PLAY.contains(base)` (casualties/KO'd excluded)
                //           plus the IllBeBack (ignoreFirstSecretWeaponSentOff) opt-out.
                //   bb2016: `playerState.getBase() != PlayerState.BANNED` ONLY — a CASUALTY secret
                //           weapon is still argued for and still banned, and bb2016 has no IllBeBack
                //           clause at all (dwarf seed 3 step 155: the away Deathroller is a casualty
                //           at halftime; Java still rolls its argue d6 and sets it BANNED, so Java
                //           rolls TWO argue dice in a dwarf mirror where Rust rolled one — the
                //           halftime kickoff scatter then read the wrong stream position).
                let is_bb2016_sw = game.rules == ffb_model::enums::Rules::Bb2016;
                let removed = game.field_model.player_state(&pid)
                    .map(|s| if is_bb2016_sw { s.base() == PS_BANNED } else { s.is_casualty() || s.base() == PS_BANNED })
                    .unwrap_or(true);
                if removed { continue; }
                let ignore = !is_bb2016_sw && game.player(&pid)
                    .map(|p| p.has_skill_property(NamedProperties::IGNORE_FIRST_SECRET_WEAPON_SENT_OFF))
                    .unwrap_or(false);
                if ignore {
                    game.game_result.team_result_mut(is_home).player_result_mut(&pid).has_used_secret_weapon = false;
                    continue;
                }
                // Argue (ParityRunner always argues). rollArgueTheCall d6; +1 if friendsWithTheRef and roll>1.
                // Java per-player block, incl. the Bribery-and-Corruption argue re-roll:
                //   biasedRefBonus = inducementSet.value(ADD_TO_ARGUE_ROLL); modifiedRoll += bonus
                //   canBeReRolled = roll == 1 && briberyReRoll present && hasUsesLeft
                //   if (canBeReRolled && coachBanned) reRollArgue(...)   // consume use + report + re-argue
                //   else if (coachBanned) setCoachBanned(true)
                // Java's reRollArgue recursion is bounded by the inducement's uses; the loop below is
                // its iterative equivalent. (dwarf seed 2 i=155: home Deathroller's argue rolled a
                // natural 1 → B&C re-roll 6 → SW flag cleared, Deathroller plays half 2; Rust banned
                // it and the halftime dice stream shifted.)
                loop {
                    let roll = rng.d6();
                    let mut modified = if friends && roll > 1 { roll + 1 } else { roll };
                    let biased_ref_bonus = {
                        let td = if is_home { &game.turn_data_home } else { &game.turn_data_away };
                        td.inducement_set.value(ffb_model::inducement::usage::Usage::ADD_TO_ARGUE_ROLL)
                    };
                    modified += biased_ref_bonus;
                    let successful = DiceInterpreter::is_argue_the_call_successful(modified);
                    let coach_ban = DiceInterpreter::is_coach_banned(modified);
                    game.report_list.add(ReportArgueTheCallRoll::new(
                        Some(pid.clone()), successful, coach_ban, roll, false, friends, biased_ref_bonus,
                    ));
                    if successful {
                        game.game_result.team_result_mut(is_home).player_result_mut(&pid).has_used_secret_weapon = false;
                    }
                    let bribery_type: Option<String> = {
                        let td = if is_home { &game.turn_data_home } else { &game.turn_data_away };
                        td.inducement_set.for_usage(ffb_model::inducement::usage::Usage::REROLL_ARGUE)
                            .filter(|t| td.inducement_set.has_uses_left(t))
                            .map(str::to_owned)
                    };
                    let can_be_rerolled = roll == 1 && bribery_type.is_some();
                    if can_be_rerolled && coach_ban {
                        // Java reRollArgue: consume a use, report USED, argue this player again.
                        let type_id = bribery_type.unwrap();
                        let td = if is_home { &mut game.turn_data_home } else { &mut game.turn_data_away };
                        td.inducement_set.use_one_of(&type_id);
                        game.report_list.add(
                            ffb_model::report::mixed::report_bribery_and_corruption_re_roll::ReportBriberyAndCorruptionReRoll::new(
                                Some(team_id.clone()), "USED".into(),
                            ),
                        );
                        continue;
                    }
                    if coach_ban {
                        if is_home { game.turn_data_home.coach_banned = true; } else { game.turn_data_away.coach_banned = true; }
                    }
                    break;
                }
                // EDITION-SPECIFIC — how many players a team argues for.
                //   bb2025 `StepEndTurn` keeps a `playerIdsArgued` set and `playersForArgue(team, game)`
                //     excludes anyone already argued, so `askForArgueTheCall` RE-FIRES the dialog until
                //     the team has no un-argued secret weapon left: one d6 per eligible player.
                //   bb2016 `StepEndTurn` has NO such tracking. `askForArgueTheCall` is guarded by
                //     `fArgueTheCallChoice{Home,Away} == null`, so it fires ONCE per team, and
                //     `argueTheCall(team, playerIds)` rolls only for the ids the CLIENT named —
                //     ParityRunner names exactly one (`playerIds[0]`). One d6 per TEAM, full stop.
                // Looping in bb2016 rolled one argue die per eligible weapon: a goblin mirror has two
                // per team (Fanatic + Bombardier), so Rust drew 4 argue d6 where Java drew 2 and the
                // halftime kickoff read the wrong stream position (goblin bb2016 seed 1, i=124: Java
                // ball 21,7 vs Rust 25,9).
                if game.rules == ffb_model::enums::Rules::Bb2016 {
                    break;
                }
            }
        }

        // Phase C — removeUsedSecretWeapons: ban the still-flagged players (set BANNED + off the pitch).
        for is_home in [true, false] {
            let ids: Vec<String> = {
                let t = if is_home { &game.team_home } else { &game.team_away };
                t.players.iter().map(|p| p.id.clone()).collect()
            };
            for pid in ids {
                if !game.game_result.team_result_mut(is_home).player_result_mut(&pid).has_used_secret_weapon {
                    continue;
                }
                game.game_result.team_result_mut(is_home).player_result_mut(&pid).has_used_secret_weapon = false;
                // Java removeUsedSecretWeapons: bb2016 sets BANNED for EVERY still-flagged player with
                // no removed-from-play guard (a casualty secret weapon is banned too); bb2025 skips
                // players already out of play.
                let is_bb2016_sw = game.rules == ffb_model::enums::Rules::Bb2016;
                let removed = game.field_model.player_state(&pid)
                    .map(|s| if is_bb2016_sw { s.base() == PS_BANNED } else { s.is_casualty() || s.base() == PS_BANNED })
                    .unwrap_or(true);
                if removed { continue; }
                // Java removeUsedSecretWeapon: setPlayerState(BANNED) + putPlayerIntoBox. remove_player clears
                // the field coordinate + state; set BANNED so setup skips it and the state reads "-1,-1,Reserve"
                // (PS_BANNED maps to the default "Reserve", off-pitch → -1,-1), matching Java.
                game.field_model.remove_player(&pid);
                game.field_model.set_player_state(&pid, PlayerState::new(PS_BANNED));
                let pr = game.game_result.team_result_mut(is_home).player_result_mut(&pid);
                pr.send_to_box_reason = Some(SendToBoxReason::SecretWeaponBan);
                pr.send_to_box_turn = self.turn_nr;
                pr.send_to_box_half = self.half;
            }
        }
    }

    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: if (turnNr == 0) capture turnNr + half for later send-to-box use
        if self.turn_nr == 0 {
            self.turn_nr = game.turn_data().turn_nr;
            self.half = game.half;
            // Java: isHomeTurnEnding = game.isHomePlaying() — captured before state flip.
            self.is_home_turn_ending = Some(game.home_playing);
        }

        // Skip guard: non-player-turn modes just propagate END_TURN and exit
        let skip_mode = matches!(
            game.turn_mode,
            TurnMode::Blitz | TurnMode::KickoffReturn | TurnMode::PassBlock
                | TurnMode::IllegalSubstitution | TurnMode::Swarming
        );
        if skip_mode {
            return StepOutcome::next().publish(StepParameter::EndTurn(true));
        }

        // Stub: within_secret_weapon_handling path (uses server dialogs) → treat as false
        // i.e. always enter the main block below

        // Touchdown check (cached tristate)
        if self.touchdown.is_none() {
            self.touchdown = Some(Self::check_touchdown(game));
        }
        let touchdown = self.touchdown.unwrap_or(false);

        // StarOfTheShow: dialog not translated → always false
        if self.use_star_of_the_show.is_none() {
            self.use_star_of_the_show = Some(false);
        }

        UtilServerGame::mark_played_and_secret_weapons(game);

        self.new_half = Self::check_end_of_half(game);

        // Events collected during this pass, attached to the final outcome.
        let mut pending_events: Vec<GameEvent> = Vec::new();

        if !self.next_sequence_pushed {
            self.next_sequence_pushed = true;

            // Java StepEndTurn: removeAdditionalAssist(actingTeam) when ending a REGULAR/BLITZ turn.
            // The Cheering Fans additional block assist lasts only for the turn it was granted; without
            // this clear it leaked into later turns and inflated a block's dice count, rolling an extra
            // die and desyncing the game-die stream. Runs before any home_playing flip so it targets the
            // acting team.
            // Java StepEndTurn:236-237 clears the ACTING team's additional assist at every
            // REGULAR/BLITZ turn end, with NO turn_started guard. The clear runs BEFORE the
            // transition block below, so at the kickoff->turn-1 handoff turn_mode is still Kickoff
            // (line ~340 flips it to Regular only afterward) → the handoff is naturally skipped,
            // matching Java (whose kickoff handoff is turnMode=KICKOFF). A prior `turn_started`
            // guard was WRONG: a turn played only via a deselecting ThrowTeamMate leaves
            // turn_started=false (it's set only by InitMoving/InitPassing/InitFouling/StandUp/
            // BlockStatistics), so the acting team's assist was never cleared and leaked into its
            // next turn (underworld seed 26: home_02's turn-2 blitz got a stale +1 → a 2-dice block
            // where Java rolled 1 die → wrong both-down injury). Clear on turn_mode alone, like Java.
            // Clear the ACTING team's Cheering-Fans additional assist at its turn end (it lasts only
            // for the turn granted). Java (StepEndTurn.java:236-237) clears at every REGULAR/BLITZ
            // turn end with no extra guard, relying on the kickoff→turn-1 handoff being turnMode=KICKOFF.
            // Rust's handoff can arrive here with turn_mode already Regular (Java's is Kickoff there),
            // so a bare turn_mode check would clear the RECEIVING team's assist before it ever plays
            // (seed 8: away_aa cleared at the pre-turn-1 handoff → away's turn-1 block lost its +1 die).
            // A `turn_started` guard was also wrong: a turn played only via a deselecting ThrowTeamMate
            // never sets turn_started (it's set by InitMoving/InitPassing/InitFouling/StandUp/
            // BlockStatistics), so the acting team's assist leaked into its next turn (underworld seed 26:
            // home turn 1 was a lone deselecting TTM → home_02's turn-2 blitz kept a stale +1 → a 2-dice
            // block where Java rolled 1). The correct discriminator is the ACTING team's turn_nr: it is 0
            // at the pre-turn-1 handoff (preserve) and ≥1 once that team has actually taken a turn (clear).
            if matches!(game.turn_mode, TurnMode::Regular | TurnMode::Blitz)
                && game.turn_data().turn_nr >= 1
            {
                if game.home_playing {
                    game.home_additional_assists = 0;
                } else {
                    game.away_additional_assists = 0;
                }
            }

            if touchdown {
                // Identify ball carrier and update score
                if let Some(ball_coord) = game.field_model.ball_coordinate {
                    if let Some(carrier_id) = game.field_model.player_at(ball_coord).cloned() {
                        self.touchdown_player_id = Some(carrier_id.clone());
                        // Java: server reports the touchdown; coverage counts it via GameEvent.
                        pending_events.push(GameEvent::Touchdown {
                            player_id: carrier_id.clone(),
                            coord: ball_coord,
                        });
                        let home_has_carrier = game.team_home.player(&carrier_id).is_some();
                        let off_turn_touchdown;
                        if home_has_carrier {
                            game.game_result.home.score += 1;
                            off_turn_touchdown = !game.home_playing;
                        } else {
                            game.game_result.away.score += 1;
                            off_turn_touchdown = game.home_playing;
                        }
                        game.home_playing = home_has_carrier;
                        if off_turn_touchdown {
                            game.turn_data_mut().turn_nr += 1;
                            self.new_half = Self::check_end_of_half(game);
                        }
                    }
                }
                game.turn_mode = TurnMode::Setup;
                game.setup_offense = false;
                // Stub: kickoffGenerator / endGenerator pushes → skip
                // Stub: ball clear, sound, resetSpecialSkillAtEndOfDrive → skip
            } else {
                match game.turn_mode {
                    TurnMode::Kickoff => {
                        game.home_playing = !game.home_playing;
                        game.turn_data_mut().turn_nr += 1;
                        game.turn_data_mut().turn_started = false;
                        game.turn_data_mut().first_turn_after_kickoff = true;
                        game.turn_mode = TurnMode::Regular;
                    }
                    TurnMode::Regular => {
                        if self.new_half {
                            game.turn_mode = TurnMode::Setup;
                            game.setup_offense = false;
                        } else {
                            game.home_playing = !game.home_playing;
                            game.turn_data_mut().turn_nr += 1;
                        }
                        game.turn_data_mut().turn_started = false;
                        game.turn_data_mut().first_turn_after_kickoff = false;
                    }
                    _ => {}
                }
                // Java StepEndTurn: UtilPlayer.refreshPlayersForTurnStart on the newly-active team —
                // PRONE/STANDING players → active (per hasToMissTurn), STUNNED → PRONE inactive. The
                // live driver path (this step) was missing it (only the deleted engine.rs had it), so a
                // knocked-down (prone) player was never reactivated at its next turn and the agent
                // wrongly rejected it as inactive. Run only when a real team turn starts (a flip
                // happened): turn_mode is Regular and it is not the new-half Setup handoff.
                if game.turn_mode == TurnMode::Regular && !self.new_half {
                    let gm = crate::mechanic::game_mechanic_for(game.rules);
                    let etr = gm.enhancements_to_remove_at_end_of_turn();
                    let etr_wna = gm.enhancements_to_remove_at_end_of_turn_when_not_setting_active();
                    ffb_model::util::util_player::UtilPlayer::refresh_players_for_turn_start(game, &etr, &etr_wna);
                }
                // Stub: sequence generator pushes (Kickoff, EndGame, Inducement) → skip
            }

            // Java: fieldModel.clearMoveSquares / clearTrackNumbers / clearDiceDecorations → stub
            // Java: reportSecretWeaponsUsed → stub
        }

        // Secret Weapon end-of-drive send-off: reportSecretWeaponsUsed → argueTheCall (away then home) →
        // removeUsedSecretWeapons, all resolved deterministically (ParityRunner always argues; bribes
        // declined). Runs ONCE, only at end of drive (new_half||touchdown) and not end_game — matching
        // Java's `!fEndGame && (fNewHalf || fTouchdown)` argue gate — so its argue d6 dice land in the
        // shared stream BEFORE the half-2 kickoff dice. Non-end-of-drive turns skip it. Bribes remain
        // stubbed (declined). The `argue_the_call_choice_*` flags are then set so `all_choices_done` proceeds.
        // Java gates the argue/remove by `!fEndGame`: Secret Weapons are sent off at a drive end that
        // is NOT the end of the game (end of half 1, or a mid-game touchdown), but NOT when the final
        // half ends. `self.end_game` is never set here, so compute the end-of-game condition directly
        // (game.half is not yet incremented at this point): the final half ending, or a game-ending TD.
        // Without this Rust banned a played Secret Weapon at the end of half 2 (game end) where Java
        // keeps it — a final-state (game_end hash) divergence, dwarf seed 4 step 294.
        let is_end_of_game = (self.new_half && game.half > 1)
            || (touchdown && game.turn_data_home.turn_nr >= 8 && game.turn_data_away.turn_nr >= 8);
        // bb2016 `StepEndTurn` calls `recoverKnockout` (line 281) BEFORE `reportSecretWeaponsUsed`
        // (364) and the argue asks (387/395); bb2025 does the reverse. Both consume game rng, so the
        // block must move with the edition.
        if game.rules == ffb_model::enums::Rules::Bb2016
            && !self.ko_recovery_done
            && (self.new_half || touchdown)
        {
            self.ko_recovery_done = true;
            self.recover_knockouts_and_fainting(game, rng, touchdown);
        }
        if self.argue_the_call_choice_away.is_none() {
            if self.new_half || touchdown {
                // Java reportSecretWeaponsUsed: the 2d6 ban roll runs at every drive end, even the
                // final one — it consumes dice before the end-of-game KO-recovery / MVP rolls.
                self.report_secret_weapons_used(game, rng);
                // Java gates only argueTheCall / removeUsedSecretWeapons by !fEndGame.
                if !is_end_of_game {
                    self.argue_and_remove_secret_weapons(game, rng);
                }
            }
            self.argue_the_call_choice_away = Some(false);
        }
        if self.argue_the_call_choice_home.is_none() && self.argue_the_call_choice_away.is_some() {
            self.argue_the_call_choice_home = Some(false);
        }
        if self.bribes_choice_away.is_none()
            && self.argue_the_call_choice_home.is_some()
            && self.argue_the_call_choice_away.is_some()
        {
            self.bribes_choice_away = Some(false);
        }
        if self.bribes_choice_home.is_none()
            && self.bribes_choice_away.is_some()
            && self.argue_the_call_choice_home.is_some()
            && self.argue_the_call_choice_away.is_some()
        {
            self.bribes_choice_home = Some(false);
        }

        let all_choices_done = self.argue_the_call_choice_home.is_some()
            && self.argue_the_call_choice_away.is_some()
            && self.bribes_choice_home.is_some()
            && self.bribes_choice_away.is_some();

        if self.end_game || all_choices_done {
            // Java: deactivateEffectsAndPrayers / deactivateCards
            {
                use crate::util::util_server_cards::UtilServerCards;
                use ffb_model::enums::InducementDuration;
                let is_home = self.is_home_turn_ending.unwrap_or(game.home_playing);
                // Java `deactivateEffectsAndPrayers(duration, isHomeTurnEnding)` does BOTH the card
                // and the prayer deactivation at each of these points (`StepEndTurn.java:489-506`).
                use crate::factory::mixed::prayer_handler_factory::PrayerHandlerFactory;
                UtilServerCards::deactivate_cards(game, InducementDuration::UntilEndOfTurn, is_home);
                PrayerHandlerFactory::deactivate_prayers_for_duration(
                    game, InducementDuration::UntilEndOfTurn, is_home);
                UtilServerCards::deactivate_cards(game, InducementDuration::UntilEndOfOpponentsTurn, is_home);
                PrayerHandlerFactory::deactivate_prayers_for_duration(
                    game, InducementDuration::UntilEndOfOpponentsTurn, is_home);
                if self.new_half || touchdown {
                    UtilServerCards::deactivate_cards(game, InducementDuration::UntilEndOfDrive, is_home);
                    PrayerHandlerFactory::deactivate_prayers_for_duration(
                        game, InducementDuration::UntilEndOfDrive, is_home);
                    UtilServerCards::deactivate_cards(game, InducementDuration::UntilEndOfHalf, is_home);
                    // Java StepEndTurn (fNewHalf || fTouchdown): remove the Dodgy Snack kickoff
                    // enhancement from every player — it lasts UNTIL_END_OF_DRIVE. Rust stores it as a
                    // "Dodgy Snack" temporary stat-mod (-1 MA / -1 AV); without this the -AV persisted
                    // across drives, so a player snacked in an earlier drive kept AV-1 forever and its
                    // armour broke on a roll Java's (restored) AV survives (elf seed 38 i=265: away_03
                    // eff_av 6 vs Java 7 → Rust breaks armour 6 & casualties where Java stays Prone).
                    for p in game.team_home.players.iter_mut()
                        .chain(game.team_away.players.iter_mut())
                    {
                        p.remove_temporary_stat_mods("Dodgy Snack");
                    }
                }
                // Java gates UNTIL_END_OF_HALF on `if (fNewHalf)` ALONE (`StepEndTurn.java:502-506`),
                // NOT on `fNewHalf || fTouchdown` like the drive-duration block above. (The
                // `deactivate_cards(UntilEndOfHalf)` call sits in the wider block — a pre-existing
                // deviation left untouched, since BB2025 is 30/30 with it.)
                if self.new_half {
                    PrayerHandlerFactory::deactivate_prayers_for_duration(
                        game, InducementDuration::UntilEndOfHalf, is_home);
                }
            }
            // bb2025 order: the KO recovery / fainting block runs AFTER the Secret Weapon
            // send-off. bb2016 runs it BEFORE (see recover_knockouts_and_fainting) and has
            // already done so above, so skip it here.
            if !self.ko_recovery_done {
                self.ko_recovery_done = true;
                self.recover_knockouts_and_fainting(game, rng, touchdown);
            }

            // Java: game.startTurn() — resets pass_coordinate, acting_player, thrower/defender
            // ids+actions, waiting_for_opponent, timeout flags, concession_possible,
            // last_defender_id, AND per-turn flags for both teams' TurnData.
            game.start_turn();

            // Java: endGenerator.pushSequence / kickoffGenerator
            use ffb_model::enums::InducementPhase;
            use crate::step::sequences::{inducement_sequence, h2_kickoff_sequence_for, end_game_sequence};
            // BB2016 shares this StepEndTurn (make_step_for(bb2016) does not override EndTurn), but its
            // ApplyKickoffResult (bb2016-routed) GOTOs GotoLabelOnEnd=END_KICKOFF / GotoLabelOnBlitz=
            // BLITZ_TURN on a turn>8 Riot or a Blitz! result. The hand-rolled `h2_kickoff_sequence`
            // adds a bare ApplyKickoffResult with NO goto params and no labelled END_KICKOFF/BLITZ_TURN
            // targets, so under bb2016 those gotos hit an EMPTY label → the step stack drains and the
            // game ends abnormally (amazon bb2016 seed20: half-2 post-score kickoff rolled Blitz! →
            // Rust bailed after ~136 activations vs Java's ~291). Use the full `Kickoff` generator for
            // bb2016 (the same one the bb2016 opening kickoff uses via StepSpectators — it threads the
            // goto params and includes the labelled steps); bb2025's own ApplyKickoffResult never GOTOs
            // those labels, so it keeps the lighter shared sequence unchanged. Coin/receive toss is
            // game-start only → with_coin_choice=false.
            let kickoff_seq = |game: &Game| {
                if game.rules == ffb_model::enums::Rules::Bb2016 {
                    crate::step::generator::mixed::kickoff::Kickoff::build_sequence(
                        &crate::step::generator::mixed::kickoff::KickoffParams { with_coin_choice: false })
                } else {
                    h2_kickoff_sequence_for(game.rules)
                }
            };
            let mut outcome = StepOutcome::next();
            for ev in pending_events { outcome = outcome.with_event(ev); }
            if self.new_half {
                // Java: half > 2 → end_game; half > 1 → overtime check or end_game; else → H2 kickoff
                if game.half > 1 {
                    // Half 2+ ended → end game (overtime check not yet ported)
                    outcome = outcome.push_seq(end_game_sequence(game.admin_mode));
                } else {
                    // Java: if (game.getHalf() == 1 || (half == 2 && drawWithOvertime))
                    //   stateMechanic.startHalf(this, game.getHalf() + 1)
                    // — increments game.half and resets both teams' turn counters. Without it
                    // the game stays in half 1 at turn 8 forever, re-running the H2 kickoff.
                    {
                        use crate::mechanic::bb2025::state_mechanic::StateMechanic;
                        use crate::mechanic::state_mechanic::StateMechanic as StateMechanicTrait;
                        let half_events = StateMechanic::new().start_half(game, game.half + 1);
                        for ev in half_events { outcome = outcome.with_event(ev); }
                        // Java: ReportStartHalf for the new (second) half. H1's StartHalf is
                        // emitted by StepInitKickoff (TurnMode::StartGame path); the H2 kickoff
                        // enters InitKickoff in TurnMode::Setup, so it must be emitted here.
                        outcome = outcome.with_event(GameEvent::StartHalf { half: game.half });
                    }
                    // End of first half → push H2 kickoff sequence
                    outcome = outcome.push_seq(kickoff_seq(game));
                }
            } else if touchdown {
                // Java: touchdownEndsGame check → end_game if last turn
                let td_ends_game = game.turn_data_home.turn_nr >= 8 && game.turn_data_away.turn_nr >= 8;
                if td_ends_game {
                    outcome = outcome.push_seq(end_game_sequence(game.admin_mode));
                } else {
                    outcome = outcome.push_seq(kickoff_seq(game));
                }
            } else if game.turn_mode != TurnMode::Regular {
                // Non-regular turn end (blitz turn etc.) → kickoff
                outcome = outcome.push_seq(kickoff_seq(game));
            } else {
                outcome = outcome.push_seq(inducement_sequence(InducementPhase::StartOfOwnTurn, game.home_playing));
            }
            // Java: UtilServerTimer.startTurnTimer → stub
            // Java: updateFumbblGame → stub
            return outcome;
        }

        // In the stub, all_choices_done is always true so we never reach here.
        // Kept for correctness if dialogs are later wired in.
        StepOutcome::cont()
    }
}

impl Default for StepEndTurn {
    fn default() -> Self { Self::new() }
}

impl Step for StepEndTurn {
    fn id(&self) -> StepId { StepId::EndTurn }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    // Java StepEndTurn does not override setParameter.
    fn set_parameter(&mut self, _param: &StepParameter) -> bool { false }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::StepAction;
    use ffb_model::enums::{Rules, PS_STANDING, PlayerState, PlayerType, PlayerGender};
    use ffb_model::model::player::Player;
    use ffb_model::model::skill_def::SkillWithValue;
    use ffb_model::types::FieldCoordinate;

    fn make_player(id: &str) -> Player {
        Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(), niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        }
    }

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        let mut game = Game::new(home, away, Rules::Bb2025);
        game.turn_mode = TurnMode::Regular;
        game.home_playing = true;
        game.turn_data_home.turn_nr = 1;
        game.turn_data_away.turn_nr = 1;
        game
    }

    /// Regression: end-of-drive Secret Weapon send-off with argue-the-call. A played Secret Weapon
    /// (getsSentOffAtEndOfDrive) is banned unless its coach argues successfully (roll 6). On a failed
    /// argue the player is removed to the box (PS_BANNED, off-field → state string "-1,-1,Reserve");
    /// on success it stays on the pitch. The argue roll consumes one game die (dwarf seed 1 step 166:
    /// Java rolled away+home argue dice before the half-2 kickoff; Rust used to roll neither).
    #[test]
    fn secret_weapon_argue_bans_on_failure_and_keeps_on_success() {
        use ffb_model::enums::{SkillId, PS_BANNED};
        fn setup() -> (Game, StepEndTurn) {
            let mut game = make_game();
            let mut p = make_player("sw");
            p.starting_skills = vec![SkillWithValue { skill_id: ffb_model::enums::SkillId::SecretWeapon, value: None }];
            game.team_home.players.push(p);
            game.field_model.set_player_coordinate("sw", FieldCoordinate::new(5, 5));
            game.field_model.set_player_state("sw", PlayerState::new(PS_STANDING));
            game.game_result.team_result_mut(true).player_result_mut("sw").has_used_secret_weapon = true;
            (game, StepEndTurn::new())
        }
        // Find seeds whose first d6 is 6 (argue success) and 2 (argue failure).
        let mut seed_ok = 0u64;
        while GameRng::new(seed_ok).d6() != 6 { seed_ok += 1; assert!(seed_ok < 200); }
        let mut seed_fail = 0u64;
        while GameRng::new(seed_fail).d6() != 2 { seed_fail += 1; assert!(seed_fail < 200); }

        // Success: stays on the pitch, flag cleared. (penalty-0 SW → Phase A rolls no die, so the
        // argue die in Phase B is the first d6.)
        let (mut game, mut step) = setup();
        let mut rng_ok = GameRng::new(seed_ok);
        step.report_secret_weapons_used(&mut game, &mut rng_ok);
        step.argue_and_remove_secret_weapons(&mut game, &mut rng_ok);
        assert!(game.field_model.player_coordinate("sw").is_some(), "argued (6) → stays on pitch");
        assert_ne!(game.field_model.player_state("sw").unwrap().base(), PS_BANNED);
        assert!(!game.game_result.team_result_mut(true).player_result_mut("sw").has_used_secret_weapon);

        // Failure: banned to the box (off-field, PS_BANNED).
        let (mut game, mut step) = setup();
        let mut rng_fail = GameRng::new(seed_fail);
        step.report_secret_weapons_used(&mut game, &mut rng_fail);
        step.argue_and_remove_secret_weapons(&mut game, &mut rng_fail);
        assert!(game.field_model.player_coordinate("sw").is_none(), "failed argue → removed from pitch");
        assert_eq!(game.field_model.player_state("sw").unwrap().base(), PS_BANNED);
        assert!(!game.game_result.team_result_mut(true).player_result_mut("sw").has_used_secret_weapon);
    }

    /// The end-of-drive KO-recovery block and the Secret Weapon send-off BOTH consume game rng, and
    /// Java runs them in OPPOSITE order per edition:
    ///   bb2016 `StepEndTurn`: `recoverKnockout` (281) BEFORE `reportSecretWeaponsUsed` (364) /
    ///                         `askForArgueTheCall` (387, 395) / `removeUsedSecretWeapons` (404).
    ///   bb2025 `StepEndTurn`: report (415) / argue (427, 442) / remove (471) FIRST, then the KO
    ///                         recovery via `getFaintingCount` (478 -> 597).
    /// dwarf bb2016 seed 5 step 128: the three halftime dice are 4, 6, 3. Java spends 4 on the KO
    /// recovery (the player recovers) and 6/3 on the away/home argues (away keeps its Deathroller,
    /// home's is banned). Running the bb2025 order under bb2016 spent 4/6 on the argues and 3 on the
    /// KO recovery — the KO'd player stayed down AND the other team's Deathroller was banned.
    #[test]
    fn bb2016_recovers_knockouts_before_the_secret_weapon_argue() {
        use ffb_model::enums::{Rules, PS_KNOCKED_OUT};

        fn setup(rules: Rules) -> (Game, StepEndTurn) {
            let mut game = Game::new(test_team("home", 0), test_team("away", 0), rules);
            // A KO'd away player (needs recovering) and a home Secret Weapon (needs arguing).
            let ko = make_player("ko");
            game.team_away.players.push(ko);
            game.field_model.set_player_state("ko", PlayerState::new(PS_KNOCKED_OUT));
            let mut sw = make_player("sw");
            sw.starting_skills = vec![SkillWithValue { skill_id: ffb_model::enums::SkillId::SecretWeapon, value: None }];
            game.team_home.players.push(sw);
            game.field_model.set_player_coordinate("sw", FieldCoordinate::new(5, 5));
            game.field_model.set_player_state("sw", PlayerState::new(PS_STANDING));
            game.game_result.team_result_mut(true).player_result_mut("sw").has_used_secret_weapon = true;
            let mut step = StepEndTurn::new();
            step.new_half = true;
            (game, step)
        }

        // Order is observable through which die each mechanic consumes: give the rng a sequence whose
        // first d6 recovers a KO (4+) and whose second fails an argue (< 6). Under the bb2016 order the
        // KO recovers and the SW is banned; under the bb2025 order the dice swap roles.
        let mut seed = 0u64;
        let found = loop {
            let mut r = GameRng::new(seed);
            let (a, b) = (r.d6(), r.d6());
            if a >= 4 && b < 6 && !(b >= 4 && a < 6) { break (a, b); }
            seed += 1;
            assert!(seed < 5000, "no suitable seed");
        };
        let _ = found;

        // bb2016: KO recovery takes the FIRST die.
        let (mut game, mut step) = setup(Rules::Bb2016);
        let first = GameRng::new(seed).d6();
        let mut rng = GameRng::new(seed);
        step.recover_knockouts_and_fainting(&mut game, &mut rng, false);
        assert_eq!(rng.call_count, 1, "the KO recovery consumed exactly the first d6");
        assert!(first >= 4, "chosen seed's first d6 recovers a KO");
        assert_ne!(game.field_model.player_state("ko").unwrap().base(), PS_KNOCKED_OUT,
            "bb2016 recovers the KO with the FIRST halftime die");

        // bb2025 keeps the block AFTER the send-off, so under bb2025 the first two dice go to the
        // argue and the KO recovery reads a later one — the ordering is what this guards.
        let (mut game, mut step) = setup(Rules::Bb2025);
        let mut rng = GameRng::new(seed);
        step.report_secret_weapons_used(&mut game, &mut rng);
        step.argue_and_remove_secret_weapons(&mut game, &mut rng);
        assert!(rng.call_count >= 1, "bb2025 spends halftime dice on the argue FIRST");
        assert_eq!(game.field_model.player_state("ko").unwrap().base(), PS_KNOCKED_OUT,
            "bb2025 has not touched the KO'd player yet at this point in the step");
    }

    /// Java's `getPlayerIds` eligibility filter is EDITION-SPECIFIC, and this step is shared:
    ///   bb2025 — `!PlayerState.REMOVED_FROM_PLAY.contains(base)`, so a CASUALTY secret weapon is
    ///            neither argued for nor banned.
    ///   bb2016 — `playerState.getBase() != PlayerState.BANNED` only, so a casualty secret weapon
    ///            IS argued for (consuming a d6) and IS set BANNED.
    /// dwarf bb2016 seed 3 step 155: the away Deathroller is a casualty at halftime; Java rolled its
    /// argue die and banned it, Rust skipped it — one fewer d6 before the half-2 kickoff, so the
    /// kickoff scatter d8 read the wrong shared-stream position.
    #[test]
    fn casualty_secret_weapon_is_argued_and_banned_only_in_bb2016() {
        use ffb_model::enums::{PS_BANNED, PS_BADLY_HURT, Rules};

        fn setup(rules: Rules) -> (Game, StepEndTurn) {
            let mut game = Game::new(test_team("home", 0), test_team("away", 0), rules);
            let mut p = make_player("sw");
            p.starting_skills = vec![SkillWithValue { skill_id: ffb_model::enums::SkillId::SecretWeapon, value: None }];
            game.team_home.players.push(p);
            // A casualty: off the pitch, BADLY_HURT.
            game.field_model.set_player_state("sw", PlayerState::new(PS_BADLY_HURT));
            game.game_result.team_result_mut(true).player_result_mut("sw").has_used_secret_weapon = true;
            (game, StepEndTurn::new())
        }

        // bb2016: the casualty is argued for — one d6 consumed — and then banned.
        let (mut game, mut step) = setup(Rules::Bb2016);
        let mut rng = GameRng::new(0);
        let before = rng.call_count;
        step.report_secret_weapons_used(&mut game, &mut rng);
        step.argue_and_remove_secret_weapons(&mut game, &mut rng);
        assert_eq!(rng.call_count - before, 1, "bb2016 rolls exactly one argue d6 for the casualty SW");
        assert_eq!(game.field_model.player_state("sw").unwrap().base(), PS_BANNED,
            "bb2016 removeUsedSecretWeapons has no removed-from-play guard");

        // bb2025: the casualty is skipped entirely — no die, still a casualty.
        let (mut game, mut step) = setup(Rules::Bb2025);
        let mut rng = GameRng::new(0);
        let before = rng.call_count;
        step.report_secret_weapons_used(&mut game, &mut rng);
        step.argue_and_remove_secret_weapons(&mut game, &mut rng);
        assert_eq!(rng.call_count - before, 0, "bb2025 excludes REMOVED_FROM_PLAY from getPlayerIds");
        assert_eq!(game.field_model.player_state("sw").unwrap().base(), PS_BADLY_HURT);
    }

    /// Regression (goblin seed 2 step 250): a Stunty-Leeg Secret Weapon (penalty > 0) must roll its
    /// 2d6 ban die at the end of the game too — Java's reportSecretWeaponsUsed runs on every drive end,
    /// only the argue/remove phases are gated by !fEndGame. `report_secret_weapons_used` therefore rolls
    /// even at end-of-game; skipping it desynced the shared stream from the end-of-game KO/MVP rolls.
    #[test]
    fn end_of_game_secret_weapon_ban_roll_still_consumes_dice() {
        use ffb_model::enums::SkillId;
        let mut game = make_game();
        let mut p = make_player("sw");
        // A Secret Weapon carrying a penalty value (Stunty-Leeg send-off number, like the goblin Bombardier's 5).
        p.starting_skills = vec![SkillWithValue { skill_id: SkillId::SecretWeapon, value: Some("5".into()) }];
        game.team_home.players.push(p);
        game.field_model.set_player_coordinate("sw", FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("sw", PlayerState::new(PS_STANDING));
        game.game_result.team_result_mut(true).player_result_mut("sw").has_used_secret_weapon = true;

        let mut step = StepEndTurn::new();
        let mut rng = GameRng::new(0);
        let before = rng.call_count;
        step.report_secret_weapons_used(&mut game, &mut rng);
        // 2d6 rolled for the penalty-5 Secret Weapon.
        assert_eq!(rng.call_count - before, 2, "penalty>0 Secret Weapon must roll 2d6 even at game end");
    }

    /// Regression (dwarf seed 2 i=155): a played Secret Weapon whose argue roll is a natural 1
    /// (coach-banned) must be re-rolled when the team holds a Bribery and Corruption
    /// (REROLL_ARGUE) inducement — Java StepEndTurn.argueTheCall: `canBeReRolled = roll==1 &&
    /// briberyReRoll present && hasUsesLeft; if (canBeReRolled && coachBanned) reRollArgue(...)`.
    /// The re-roll consumes the inducement use and rolls a fresh argue d6; a 6 keeps the player.
    /// Without the inducement the coach is banned on the 1. Here the first two d6 are 1 (natural
    /// one) then 6 (successful re-roll) → the SW stays on the pitch and the coach is not banned.
    #[test]
    fn seed2_bribery_and_corruption_rerolls_natural_one_argue() {
        use ffb_model::inducement::inducement::Inducement as InducementModel;
        use ffb_model::inducement::usage::Usage;
        // Seed whose first two d6 are 1 then 6.
        let mut seed = 0u64;
        loop {
            let mut r = GameRng::new(seed);
            if r.d6() == 1 && r.d6() == 6 { break; }
            seed += 1;
            assert!(seed < 100_000, "no seed with d6 sequence 1,6 found");
        }
        let mut game = make_game();
        let mut p = make_player("sw");
        p.starting_skills = vec![SkillWithValue { skill_id: ffb_model::enums::SkillId::SecretWeapon, value: None }];
        game.team_home.players.push(p);
        game.field_model.set_player_coordinate("sw", FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("sw", PlayerState::new(PS_STANDING));
        game.game_result.team_result_mut(true).player_result_mut("sw").has_used_secret_weapon = true;
        // Grant the team its Bribery and Corruption argue re-roll (Java StepBuyInducements.leaveStep).
        game.turn_data_home.inducement_set.add_inducement(
            InducementModel::new("briberyAndCorruption", 1, vec![Usage::REROLL_ARGUE]));

        let mut step = StepEndTurn::new();
        let mut rng = GameRng::new(seed);
        step.report_secret_weapons_used(&mut game, &mut rng);       // penalty-0 SW: no die
        step.argue_and_remove_secret_weapons(&mut game, &mut rng);  // argue 1 → B&C reroll 6
        assert_eq!(rng.call_count, 2, "one natural-1 argue + one B&C re-roll = 2 d6");
        assert!(game.field_model.player_coordinate("sw").is_some(), "re-rolled argue (6) keeps the SW on the pitch");
        assert!(!game.turn_data_home.coach_banned, "successful re-roll → coach not banned");
        assert!(!game.game_result.team_result_mut(true).player_result_mut("sw").has_used_secret_weapon);
        assert!(!game.turn_data_home.inducement_set.has_uses_left("briberyAndCorruption"),
            "the B&C re-roll charge was consumed");
    }

    #[test]
    fn blitz_mode_publishes_end_turn_and_returns_next_step() {
        let mut game = make_game();
        game.turn_mode = TurnMode::Blitz;
        let mut step = StepEndTurn::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndTurn(true))));
    }

    #[test]
    fn regular_turn_flips_home_playing_and_increments_turn_nr() {
        let mut game = make_game();
        game.turn_data_home.turn_nr = 3;
        game.turn_data_away.turn_nr = 3;
        let mut step = StepEndTurn::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        // After home's turn ends, home_playing flips to false (away's turn)
        assert!(!game.home_playing);
        // Away's turn_nr was 3, now 4
        assert_eq!(game.turn_data_away.turn_nr, 4);
        assert_eq!(game.turn_data_home.turn_nr, 3);
    }

    #[test]
    fn end_of_half_transitions_to_setup() {
        let mut game = make_game();
        game.turn_data_home.turn_nr = 8;
        game.turn_data_away.turn_nr = 8;
        let mut step = StepEndTurn::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(game.turn_mode, TurnMode::Setup);
    }

    #[test]
    fn dodgy_snack_enhancement_cleared_at_end_of_drive() {
        use ffb_model::model::player::{STAT_MA, STAT_AV};
        // Java StepEndTurn (fNewHalf || fTouchdown) removes the Dodgy Snack enhancement (lasts
        // UNTIL_END_OF_DRIVE). A player snacked in an earlier drive must have base AV/MA restored at
        // the drive break — else the -AV lingers and its armour breaks on a roll Java survives
        // (elf seed 38 i=265). turn_nr 8 makes this end-of-turn a new half (end of drive).
        let mut game = make_game();
        game.team_home.players.push(make_player("snacked"));
        game.field_model.set_player_coordinate("snacked", FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("snacked", PlayerState::new(PS_STANDING));
        {
            let p = game.player_mut("snacked").unwrap();
            p.add_temporary_stat_mod("Dodgy Snack", STAT_MA, -1);
            p.add_temporary_stat_mod("Dodgy Snack", STAT_AV, -1);
            assert_eq!(p.armour_with_modifiers(), p.armour - 1);
        }
        game.turn_data_home.turn_nr = 8;
        game.turn_data_away.turn_nr = 8;
        let mut step = StepEndTurn::new();
        step.start(&mut game, &mut GameRng::new(0));
        let p = game.player("snacked").unwrap();
        assert_eq!(p.armour_with_modifiers(), p.armour, "Dodgy Snack -AV must be cleared at drive end");
        assert!(p.temporary_stat_mods.iter().all(|(s, _, _)| s != "Dodgy Snack"),
            "Dodgy Snack stat-mods must be removed at drive end");
    }

    #[test]
    fn sweltering_heat_faints_random_players_at_half_end() {
        use ffb_model::enums::{Weather, PS_EXHAUSTED};
        // Java StepEndTurn.getFaintingCount: under Sweltering Heat a d3 count of random on-pitch
        // players per team is knocked EXHAUSTED at the half break. d3 is always >= 1, so with plenty
        // of standing players at least one per team must faint.
        let mut game = make_game();
        game.field_model.weather = Weather::SwelteringHeat;
        for i in 0..6 {
            let hid = format!("hh{i}");
            let aid = format!("aa{i}");
            game.team_home.players.push(make_player(&hid));
            game.team_away.players.push(make_player(&aid));
            game.field_model.set_player_coordinate(&hid, FieldCoordinate::new(4 + i, 5));
            game.field_model.set_player_state(&hid, PlayerState::new(PS_STANDING));
            game.field_model.set_player_coordinate(&aid, FieldCoordinate::new(20 - i, 5));
            game.field_model.set_player_state(&aid, PlayerState::new(PS_STANDING));
        }
        game.turn_data_home.turn_nr = 8;
        game.turn_data_away.turn_nr = 8;
        let mut step = StepEndTurn::new();
        let mut rng = GameRng::new(3);
        step.start(&mut game, &mut rng);
        let exhausted = game.team_home.players.iter().chain(game.team_away.players.iter())
            .filter(|p| game.field_model.player_state(&p.id).map(|s| s.base() == PS_EXHAUSTED).unwrap_or(false))
            .count();
        assert!(exhausted >= 2, "sweltering heat should faint at least one player per team, got {exhausted}");
    }

    #[test]
    fn bb2016_sweltering_heat_rolls_one_die_per_on_pitch_player() {
        use ffb_model::enums::{Weather, Rules};
        // BB2016 (bb2016.StepEndTurn heatExhaust) rolls ONE rollKnockoutRecovery d6 PER on-pitch
        // player at a drive end under Sweltering Heat — NOT the BB2020+ single d3 fainting count.
        // With 12 on-pitch players and no KO'd players, exactly 12 dice must be consumed here (amazon
        // bb2016 seed20: Rust rolled a d3 where Java rolled 12+ d6, desyncing the half-2 kickoff).
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        let mut game = Game::new(home, away, Rules::Bb2016);
        game.turn_mode = TurnMode::Regular;
        game.home_playing = true;
        game.field_model.weather = Weather::SwelteringHeat;
        for i in 0..6 {
            let hid = format!("hh{i}");
            let aid = format!("aa{i}");
            game.team_home.players.push(make_player(&hid));
            game.team_away.players.push(make_player(&aid));
            game.field_model.set_player_coordinate(&hid, FieldCoordinate::new(4 + i, 5));
            game.field_model.set_player_state(&hid, PlayerState::new(PS_STANDING));
            game.field_model.set_player_coordinate(&aid, FieldCoordinate::new(20 - i, 5));
            game.field_model.set_player_state(&aid, PlayerState::new(PS_STANDING));
        }
        game.turn_data_home.turn_nr = 8;
        game.turn_data_away.turn_nr = 8;
        let mut step = StepEndTurn::new();
        let mut rng = GameRng::new(3);
        let before = rng.call_count;
        step.start(&mut game, &mut rng);
        let dice = rng.call_count - before;
        assert_eq!(dice, 12, "bb2016 Sweltering Heat must roll one d6 per on-pitch player (12), got {dice}");
    }

    #[test]
    fn touchdown_increments_home_score_and_transitions_to_setup() {
        let mut game = make_game();
        game.team_home.players.push(make_player("scorer"));
        let ball_coord = FieldCoordinate::new(FIELD_WIDTH - 1, 7);
        game.field_model.set_player_coordinate("scorer", ball_coord);
        game.field_model.set_player_state("scorer", PlayerState::new(PS_STANDING));
        game.field_model.ball_coordinate = Some(ball_coord);
        game.field_model.ball_in_play = true;
        game.field_model.ball_moving = false;

        let mut step = StepEndTurn::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(game.game_result.home.score, 1);
        assert_eq!(game.turn_mode, TurnMode::Setup);
        assert_eq!(step.touchdown_player_id.as_deref(), Some("scorer"));
    }

    #[test]
    fn touchdown_increments_away_score() {
        let mut game = make_game();
        game.home_playing = false;
        game.team_away.players.push(make_player("scorer2"));
        let ball_coord = FieldCoordinate::new(0, 7);
        game.field_model.set_player_coordinate("scorer2", ball_coord);
        game.field_model.set_player_state("scorer2", PlayerState::new(PS_STANDING));
        game.field_model.ball_coordinate = Some(ball_coord);
        game.field_model.ball_in_play = true;
        game.field_model.ball_moving = false;

        let mut step = StepEndTurn::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(game.game_result.away.score, 1);
        assert_eq!(game.game_result.home.score, 0);
        assert_eq!(game.turn_mode, TurnMode::Setup);
    }

    #[test]
    fn additional_assist_not_cleared_before_team_plays_a_turn() {
        // The kickoff->turn-1 transition can run StepEndTurn with turn_mode=Regular but the acting
        // team's turn_nr still 0 (it never took a turn). It must NOT clear that team's Cheering Fans
        // additional assist — the assist lasts until the team actually plays and ends its turn.
        // (Discriminator is the acting team's turn_nr, not turn_started: a turn played only via a
        // deselecting ThrowTeamMate leaves turn_started=false yet must still clear at its end.)
        let mut game = make_game();
        game.home_playing = false; // away = the "acting"/kicking team at the transition
        game.turn_mode = TurnMode::Regular;
        game.turn_data_away.turn_nr = 0; // away has not taken a turn yet
        game.away_additional_assists = 1;
        let mut step = StepEndTurn::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.away_additional_assists, 1, "assist must survive before the team plays a turn");
    }

    #[test]
    fn additional_assist_cleared_at_real_turn_end_even_without_turn_started() {
        // A turn played only via a deselecting ThrowTeamMate never sets turn_started, but the acting
        // team's turn_nr is >= 1, so its Cheering Fans assist MUST be cleared at that turn's end
        // (underworld seed 26 regression: else a stale +1 inflated a later block's dice count).
        let mut game = make_game();
        game.home_playing = true;
        game.turn_mode = TurnMode::Regular;
        game.turn_data_home.turn_nr = 1;
        game.turn_data_home.turn_started = false; // deselecting-TTM turn: never set true
        game.home_additional_assists = 1;
        let mut step = StepEndTurn::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.home_additional_assists, 0, "assist must clear at a real turn end (turn_nr>=1)");
    }

    #[test]
    fn additional_assist_cleared_at_real_turn_end() {
        let mut game = make_game();
        game.home_playing = false;
        game.turn_mode = TurnMode::Regular;
        game.turn_data_away.turn_started = true; // away actually played this turn
        game.away_additional_assists = 1;
        let mut step = StepEndTurn::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.away_additional_assists, 0, "assist cleared at a real (started) turn end");
    }

    #[test]
    fn kickoff_mode_transitions_to_regular_and_flips_team() {
        let mut game = make_game();
        game.turn_mode = TurnMode::Kickoff;
        game.turn_data_away.turn_nr = 2;
        let mut step = StepEndTurn::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(game.turn_mode, TurnMode::Regular);
        // Home was playing, so now away is playing
        assert!(!game.home_playing);
        // Away's turn_nr incremented
        assert_eq!(game.turn_data_away.turn_nr, 3);
        assert!(game.turn_data_away.first_turn_after_kickoff);
    }

    #[test]
    fn start_turn_resets_both_turn_data() {
        let mut game = make_game();
        game.turn_data_home.blitz_used = true;
        game.turn_data_away.foul_used = true;
        let mut step = StepEndTurn::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert!(!game.turn_data_home.blitz_used);
        assert!(!game.turn_data_away.foul_used);
    }

    /// Java `Game.startTurn()` clears far more than the two TurnData objects: it also
    /// resets pass_coordinate, acting_player, thrower/defender ids+actions,
    /// waiting_for_opponent, timeout flags, concession_possible and last_defender_id.
    /// A prior translation only called `turn_data_{home,away}.reset_for_turn()`, leaving
    /// stale acting-player/defender/thrower state from the ended turn.
    #[test]
    fn start_turn_clears_acting_player_defender_and_thrower_state() {
        let mut game = make_game();
        game.acting_player.player_id = Some("stale_actor".into());
        game.defender_id = Some("stale_defender".into());
        game.thrower_id = Some("stale_thrower".into());
        game.pass_coordinate = Some(FieldCoordinate::new(3, 3));
        game.waiting_for_opponent = true;
        game.timeout_possible = true;

        let mut step = StepEndTurn::new();
        step.start(&mut game, &mut GameRng::new(0));

        assert!(game.acting_player.player_id.is_none(), "acting_player must be cleared by startTurn()");
        assert!(game.defender_id.is_none(), "defender_id must be cleared by startTurn()");
        assert!(game.thrower_id.is_none(), "thrower_id must be cleared by startTurn()");
        assert!(game.pass_coordinate.is_none(), "pass_coordinate must be cleared by startTurn()");
        assert!(!game.waiting_for_opponent, "waiting_for_opponent must be cleared by startTurn()");
        assert!(!game.timeout_possible, "timeout_possible must be cleared by startTurn()");
    }

    #[test]
    fn set_parameter_returns_false() {
        let mut step = StepEndTurn::new();
        assert!(!step.set_parameter(&StepParameter::EndTurn(true)));
        assert!(!step.set_parameter(&StepParameter::EndGame(true)));
    }

    #[test]
    fn check_end_of_half_requires_both_teams_at_8() {
        let mut game = make_game();
        game.turn_data_home.turn_nr = 8;
        game.turn_data_away.turn_nr = 7;
        assert!(!StepEndTurn::check_end_of_half(&game));
        game.turn_data_away.turn_nr = 8;
        assert!(StepEndTurn::check_end_of_half(&game));
    }

    /// How many argue dice a TEAM rolls is edition-specific.
    /// bb2025 `StepEndTurn` keeps a `playerIdsArgued` set and `playersForArgue` excludes anyone
    /// already argued, so `askForArgueTheCall` re-fires until the team has no un-argued secret
    /// weapon left — one d6 per eligible player. bb2016 `StepEndTurn` has no such tracking:
    /// `askForArgueTheCall` is guarded by `fArgueTheCallChoice{Home,Away} == null` so it fires ONCE
    /// per team, and `argueTheCall(team, playerIds)` rolls only for the ids the CLIENT named —
    /// ParityRunner names exactly one (`playerIds[0]`). Rust looped in both editions, so a goblin
    /// mirror (Fanatic + Bombardier per team) drew 4 argue d6 where Java drew 2 and the halftime
    /// kickoff read the wrong stream position (goblin bb2016 seed 1 i=124).
    #[test]
    fn bb2016_argues_once_per_team_bb2025_argues_every_secret_weapon() {
        use ffb_model::enums::SkillId;

        fn setup(rules: Rules) -> (Game, StepEndTurn) {
            let mut game = Game::new(test_team("home", 0), test_team("away", 0), rules);
            for (i, id) in ["sw1", "sw2"].iter().enumerate() {
                let mut p = make_player(id);
                p.starting_skills = vec![SkillWithValue { skill_id: SkillId::SecretWeapon, value: None }];
                game.team_home.players.push(p);
                game.field_model.set_player_coordinate(id, FieldCoordinate::new(5, 5 + i as i32));
                game.field_model.set_player_state(id, PlayerState::new(PS_STANDING));
                game.game_result.team_result_mut(true).player_result_mut(id).has_used_secret_weapon = true;
            }
            (game, StepEndTurn::new())
        }

        // A seed whose first two d6 are neither 6 (success) nor 1 (coach ban), so nothing short-circuits.
        let seed = (0u64..10_000)
            .find(|s| {
                let mut r = GameRng::new(*s);
                matches!(r.d6(), 2..=5) && matches!(r.d6(), 2..=5)
            })
            .expect("some seed yields two mid d6");

        let (mut game, mut step) = setup(Rules::Bb2016);
        let mut rng = GameRng::new(seed);
        let before = rng.call_count;
        step.argue_and_remove_secret_weapons(&mut game, &mut rng);
        assert_eq!(rng.call_count - before, 1,
            "bb2016 argues ONCE per team, for the single player the client names");

        let (mut game, mut step) = setup(Rules::Bb2025);
        let mut rng = GameRng::new(seed);
        let before = rng.call_count;
        step.argue_and_remove_secret_weapons(&mut game, &mut rng);
        assert_eq!(rng.call_count - before, 2,
            "bb2025 re-fires the dialog until every eligible secret weapon has been argued");
    }
}
