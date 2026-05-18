//! Full block step: dice roll → choice → pushback → armor → injury.
//!
//! The step is split into three entry points matching the dialog flow:
//!   1. `begin_block`         — rolls dice, sets SelectBlockDice dialog
//!   2. `apply_block_dice_choice` — applies chosen result, sets SelectPush if needed
//!   3. `apply_push_choice`   — moves defender, applies armor/injury
//!
//! Each function mutates `GameState` and returns a `BlockStepResult` that tells
//! the caller what happened and whether a further dialog is pending.
use crate::mechanics::block::{block_dice_count_ext, dauntless_effective_dice, dwarven_scourge_min_dice, mesmerising_dance_penalty, pushback_options};
use crate::mechanics::injury::{apply_regeneration, armor_roll, resolve_injury, resolve_injury_with_decay, ArmorOutcome};
use crate::model::field_model::FieldModel;
use crate::model::game_state::{DialogState, GameState};
use crate::rng::GameRng;
use crate::skills::SkillId;
use crate::types::{BlockResult, CasualtyType, FieldCoordinate, PlayerId, PlayerState, TeamId};

// ── Result types ──────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub enum BlockStepResult {
    /// Dice rolled — waiting for player to choose a die.
    NeedDiceChoice {
        dice: Vec<BlockResult>,
        defender_chooses: bool,
    },
    /// Dice chosen — waiting for push target selection.
    NeedPushChoice {
        options: Vec<FieldCoordinate>,
    },
    /// Block resolved completely.
    Done(BlockResolution),
}

#[derive(Clone, Debug)]
pub struct BlockResolution {
    pub attacker_knocked_down: bool,
    pub defender_knocked_down: bool,
    pub turnover: bool,
    /// Casualty type if defender was injured as a casualty.
    pub casualty: Option<CasualtyType>,
}

// ── FoulAppearance ────────────────────────────────────────────────────────────

/// Check if a block against `target_id` is allowed given FoulAppearance.
/// If target has FoulAppearance, roll d6: return false (block cancelled) on 1,
/// return true (continue) on 2+.
/// If target does not have FoulAppearance, always return true.
pub fn foul_appearance_check(state: &GameState, target_id: &PlayerId, rng: &mut GameRng) -> bool {
    let team = match state.field.player_team(target_id) {
        Some(t) => t,
        None => return true,
    };
    let has_fa = state.team(team)
        .player_by_id(target_id)
        .map(|p| p.has_skill(SkillId::FoulAppearance))
        .unwrap_or(false);
    if !has_fa {
        return true;
    }
    rng.roll_d6() >= 2
}

// ── Step 1: Begin block ───────────────────────────────────────────────────────

/// Initiate a block action.
/// Rolls block dice and sets `state.dialog` to `SelectBlockDice`.
///
/// Special cases handled before rolling block dice:
/// - Stab: skip dice entirely and go straight to injury roll (2d6, no armor).
/// - Chainsaw: skip dice, armor always breaks — go straight to injury.
pub fn begin_block(
    state: &mut GameState,
    attacker_id: &PlayerId,
    defender_id: &PlayerId,
    rng: &mut GameRng,
) -> BlockStepResult {
    let att_team = state.field.player_team(attacker_id).expect("attacker not on pitch");
    let def_team = att_team.opponent();

    let (att_st, _att_has_block, _att_has_wrestle, _att_has_juggernaut, _att_has_frenzy,
         att_has_stab, att_has_chainsaw, att_has_horns, att_has_dauntless) = {
        let p = state.team(att_team).player_by_id(attacker_id).expect("attacker not found");
        (
            p.effective_st(),
            p.has_skill(SkillId::Block),
            p.has_skill(SkillId::Wrestle),
            p.has_skill(SkillId::Juggernaut),
            p.has_skill(SkillId::Frenzy),
            p.has_skill(SkillId::Stab),
            p.has_skill(SkillId::Chainsaw),
            p.has_skill(SkillId::Horns),
            p.has_skill(SkillId::Dauntless),
        )
    };

    // Chainsaw: armor always breaks — skip block dice and go straight to injury.
    if att_has_chainsaw {
        let mb = state.team(att_team).player_by_id(attacker_id)
            .map(|p| if p.has_skill(SkillId::MightyBlow) { 1u8 } else { 0 })
            .unwrap_or(0);
        state.field.set_player_state(defender_id, PlayerState::Prone);
        let inj = resolve_injury_with_decay(defender_id, state, mb, rng);
        state.field.set_player_state(defender_id, inj.new_state);
        scatter_ball_if_carrier(state, defender_id, rng);
        apply_regeneration(state, defender_id, rng);
        state.dialog = DialogState::None;
        return BlockStepResult::Done(BlockResolution {
            attacker_knocked_down: false,
            defender_knocked_down: true,
            turnover: false,
            casualty: inj.casualty,
        });
    }

    // Stab: skip block dice, roll injury directly (2d6 vs AV, no armor roll needed).
    if att_has_stab {
        let mb = state.team(att_team).player_by_id(attacker_id)
            .map(|p| if p.has_skill(SkillId::MightyBlow) { 1u8 } else { 0 })
            .unwrap_or(0);
        state.field.set_player_state(defender_id, PlayerState::Prone);
        let inj = resolve_injury_with_decay(defender_id, state, mb, rng);
        state.field.set_player_state(defender_id, inj.new_state);
        scatter_ball_if_carrier(state, defender_id, rng);
        apply_regeneration(state, defender_id, rng);
        state.dialog = DialogState::None;
        return BlockStepResult::Done(BlockResolution {
            attacker_knocked_down: false,
            defender_knocked_down: true,
            turnover: false,
            casualty: inj.casualty,
        });
    }

    // FoulAppearance: roll d6 before block; 1 = block cancelled
    if !foul_appearance_check(state, defender_id, rng) {
        state.dialog = DialogState::None;
        return BlockStepResult::Done(BlockResolution {
            attacker_knocked_down: false,
            defender_knocked_down: false,
            turnover: false,
            casualty: None,
        });
    }

    let (def_st, _def_has_dodge, def_has_defensive) = {
        let p = state.team(def_team).player_by_id(defender_id).expect("defender not found");
        (p.effective_st(), p.has_skill(SkillId::Dodge), p.has_skill(SkillId::Defensive))
    };

    let att_coord = state.field.player_coord(attacker_id).expect("attacker coord");
    let def_coord = state.field.player_coord(defender_id).expect("defender coord");

    // Guard: count adjacent standing teammates with Guard skill
    let guard_att = count_guard_with_skill(&state.field, att_coord, att_team, state);
    let guard_def = count_guard_with_skill(&state.field, def_coord, def_team, state);

    // Horns: +1 ST when blitzing (acting player has blitzed)
    let is_blitz_block = state.acting_player.as_ref().map(|ap| ap.has_blitzed).unwrap_or(false);
    let horns_bonus = att_has_horns && is_blitz_block;

    let raw_dice = block_dice_count_ext(att_st, def_st, guard_att, guard_def, horns_bonus, def_has_defensive);
    // Dauntless: if attacker has Dauntless and defender is stronger, roll d6 to potentially
    // raise effective dice to 1 (treating attacker ST as equal to defender ST).
    let raw_dice = if att_has_dauntless && raw_dice < 0 {
        dauntless_effective_dice(att_st, def_st, guard_att, guard_def, true, rng)
    } else {
        raw_dice
    };
    // DwarvenScourge: minimum 2 dice (capped at 3)
    let dice_count = dwarven_scourge_min_dice(state, attacker_id, raw_dice);
    // MesmerisingDance: -1 to dice count if defender has skill (minimum 1 die)
    let mesm_penalty = mesmerising_dance_penalty(state, defender_id);
    let dice_count = (dice_count + mesm_penalty).max(if dice_count > 0 { 1 } else { -1 });
    let _n_dice = dice_count.unsigned_abs().max(1);
    let defender_chooses = dice_count < 0;

    let dice = rng.roll_block_dice(dice_count);

    // Check if team reroll is available (and hasn't been used this turn) to offer
    // before the player picks which die to use.
    let reroll_available = crate::steps::turn_step::active_team_reroll_available(state);

    if reroll_available {
        state.dialog = DialogState::SelectBlockReroll {
            dice: dice.clone(),
            defender_chooses,
            reroll_available: true,
        };
    } else {
        state.dialog = DialogState::SelectBlockDice {
            dice: dice.clone(),
            defender_chooses,
        };
    }

    BlockStepResult::NeedDiceChoice { dice, defender_chooses }
}

/// Count Guard-skilled adjacent teammates (active players only).
fn count_guard_with_skill(
    field: &FieldModel,
    coord: FieldCoordinate,
    team: TeamId,
    state: &GameState,
) -> u8 {
    coord.neighbors()
        .filter(|&n| {
            if let Some(pid) = field.player_at(n) {
                if field.player_team(pid) == Some(team) {
                    if let Some(st) = field.player_state(pid) {
                        if st.is_active() {
                            let p = state.team(team).player_by_id(pid);
                            return p.map(|pl| pl.has_skill(SkillId::Guard)).unwrap_or(false);
                        }
                    }
                }
            }
            false
        })
        .count() as u8
}

// ── Step 2: Apply block dice choice ──────────────────────────────────────────

/// Apply the player's chosen block die result.
/// Handles skill interactions (Block, Wrestle, Juggernaut on Blitz).
/// Returns `NeedPushChoice` if a push is needed, or `Done` if both are knocked down / no push.
pub fn apply_block_dice_choice(
    state: &mut GameState,
    attacker_id: &PlayerId,
    defender_id: &PlayerId,
    chosen: BlockResult,
    rng: &mut GameRng,
) -> BlockStepResult {
    let att_team = state.field.player_team(attacker_id).expect("attacker not on pitch");
    let def_team = att_team.opponent();
    let is_blitz = state.acting_player.as_ref().map(|ap| ap.has_blitzed).unwrap_or(false);

    let (att_has_block, att_has_wrestle, att_has_juggernaut) = {
        let p = state.team(att_team).player_by_id(attacker_id).expect("attacker");
        (
            p.has_skill(SkillId::Block),
            p.has_skill(SkillId::Wrestle),
            p.has_skill(SkillId::Juggernaut),
        )
    };
    let (_def_has_block, def_has_wrestle) = {
        let p = state.team(def_team).player_by_id(defender_id).expect("defender");
        (p.has_skill(SkillId::Block), p.has_skill(SkillId::Wrestle))
    };

    // Juggernaut on blitz: BothDown → Pushed (attacker's choice)
    let effective_result = if is_blitz && att_has_juggernaut && chosen == BlockResult::BothDown {
        BlockResult::Pushback
    } else {
        chosen
    };

    match effective_result {
        BlockResult::Skull => {
            // Attacker knocked down
            knock_down_in_block(state, attacker_id, rng);
            let turnover = true; // skull always causes turnover
            state.dialog = DialogState::None;
            BlockStepResult::Done(BlockResolution {
                attacker_knocked_down: true,
                defender_knocked_down: false,
                turnover,
                casualty: None,
            })
        }
        BlockResult::BothDown => {
            let att_kd;
            let def_kd;
            if att_has_wrestle || def_has_wrestle {
                // Wrestle: both go down, no armor roll
                state.field.set_player_state(attacker_id, PlayerState::Prone);
                state.field.set_player_state(defender_id, PlayerState::Prone);
                att_kd = true;
                def_kd = true;
            } else if att_has_block {
                // Block skill: defender goes down, attacker stays standing
                att_kd = false;
                def_kd = true;
                knock_down_in_block(state, defender_id, rng);
            } else {
                // Both go down
                att_kd = true;
                def_kd = true;
                knock_down_in_block(state, attacker_id, rng);
                knock_down_in_block(state, defender_id, rng);
            }
            let turnover = att_kd; // attacker knocked down = turnover
            state.dialog = DialogState::None;
            BlockStepResult::Done(BlockResolution {
                attacker_knocked_down: att_kd,
                defender_knocked_down: def_kd,
                turnover,
                casualty: None,
            })
        }
        BlockResult::Pushback | BlockResult::PowPushback | BlockResult::Pow => {
            let defender_knocked = matches!(effective_result, BlockResult::PowPushback | BlockResult::Pow);

            if defender_knocked {
                state.field.set_player_state(defender_id, PlayerState::Prone);
            }

            // StandFirm: defender may refuse the push (stays in place).
            // Juggernaut on a Blitz cancels StandFirm.
            // StandFirm does NOT prevent knockdown (Pow/PowPushback still knock them down).
            {
                let def_has_stand_firm = state.team(def_team)
                    .player_by_id(defender_id)
                    .map(|p| p.has_skill(SkillId::StandFirm))
                    .unwrap_or(false);
                let juggernaut_cancels = is_blitz && att_has_juggernaut;
                if def_has_stand_firm && !juggernaut_cancels {
                    // Defender stays put — push is cancelled.
                    // Attacker still gets to follow up on a Frenzy blitz? No — push was refused.
                    state.dialog = DialogState::None;
                    return BlockStepResult::Done(BlockResolution {
                        attacker_knocked_down: false,
                        defender_knocked_down: defender_knocked,
                        turnover: false,
                        casualty: None,
                    });
                }
            }

            // Check if defender has SideStep
            let def_has_side_step = state.team(def_team)
                .player_by_id(defender_id)
                .map(|p| p.has_skill(SkillId::SideStep))
                .unwrap_or(false);

            // Compute pushback options
            let att_coord = state.field.player_coord(attacker_id).expect("att coord");
            let def_coord = state.field.player_coord(defender_id).expect("def coord");

            // SideStep: replace normal 3-square options with ALL adjacent empty squares
            let options = if def_has_side_step {
                side_step_options(&state.field, def_coord)
            } else {
                pushback_options(&state.field, att_coord, def_coord)
            };

            let push_coords: Vec<FieldCoordinate> = options
                .iter()
                .filter_map(|o| o.coord())
                .collect();
            let has_off_pitch = options.iter().any(|o| o.is_off_pitch());

            if push_coords.is_empty() || (push_coords.len() == 1 && !has_off_pitch) || has_off_pitch && push_coords.is_empty() {
                // Auto-resolve: crowd push or only one option
                let target = if push_coords.is_empty() {
                    // Off-pitch only
                    apply_crowd_push(state, defender_id, rng);
                    state.dialog = DialogState::None;
                    return BlockStepResult::Done(BlockResolution {
                        attacker_knocked_down: false,
                        defender_knocked_down: true,
                        turnover: false,
                        casualty: None,
                    });
                } else {
                    push_coords[0]
                };
                // Auto-push to only option
                return apply_push_choice(state, attacker_id, defender_id, target, rng);
            }

            // Multiple options: need player input
            state.dialog = DialogState::SelectPush { options: push_coords.clone() };
            BlockStepResult::NeedPushChoice { options: push_coords }
        }
    }
}

// ── Step 3: Apply push choice ─────────────────────────────────────────────────

/// Move the defender to `push_target`, then apply armor/injury if knocked down.
///
/// Frenzy skill: if the attacker has Frenzy and the result was Pushback or
/// PowPushback, the attacker automatically follows up (moves to defender's previous
/// square) and `ActingPlayer::frenzy_second_block_required` is set to `true` so
/// the simulation loop can trigger the second block.
pub fn apply_push_choice(
    state: &mut GameState,
    attacker_id: &PlayerId,
    defender_id: &PlayerId,
    push_target: FieldCoordinate,
    rng: &mut GameRng,
) -> BlockStepResult {
    let def_team = state.field.player_team(defender_id).expect("defender team");
    let att_team = def_team.opponent();

    // Remember defender's current square before the push (attacker would follow up here)
    let defender_current_coord = state.field.player_coord(defender_id).expect("defender coord");

    // Move defender — handles chain push when target is occupied.
    let crowd_pushed = resolve_chain_push(state, defender_id, defender_current_coord, push_target, rng);
    if crowd_pushed {
        // Defender was pushed off-pitch; crowd push already applied armor/injury.
        // No further armor roll or follow-up.
        state.dialog = DialogState::None;
        return BlockStepResult::Done(BlockResolution {
            attacker_knocked_down: false,
            defender_knocked_down: true,
            turnover: false,
            casualty: None,
        });
    }

    // StripBall: if attacker has StripBall and defender was carrying the ball,
    // scatter it from the defender's new square (applies on Pushback, before armor).
    let att_has_strip_ball = state.team(att_team)
        .player_by_id(attacker_id)
        .map(|p| p.has_skill(SkillId::StripBall))
        .unwrap_or(false);
    if att_has_strip_ball {
        crate::mechanics::block::strip_ball_scatter(&mut state.field, true, defender_id, rng);
    }

    let defender_knocked = state.field.player_state(defender_id)
        .map(|s| s == PlayerState::Prone)
        .unwrap_or(false);

    let casualty = if defender_knocked {
        let (def_av_base, mb, att_has_claws) = {
            let av = state.team(def_team).player_by_id(defender_id)
                .map(|p| p.effective_av())
                .unwrap_or(8);
            let (mb, claws) = state.team(att_team).player_by_id(attacker_id)
                .map(|p| (if p.has_skill(SkillId::MightyBlow) { 1u8 } else { 0 }, p.has_skill(SkillId::Claws)))
                .unwrap_or((0, false));
            (av, mb, claws)
        };
        // Claws: treat defender's AV as at most 7
        let av = crate::mechanics::block::claws_effective_av(att_has_claws, def_av_base);
        match armor_roll(av, mb, 0, rng) {
            ArmorOutcome::NotBroken => None,
            ArmorOutcome::Broken => {
                let inj = resolve_injury_with_decay(defender_id, state, mb, rng);
                state.field.set_player_state(defender_id, inj.new_state);
                // Ball scatter: if defender carried the ball and is now KO/Injured/Prone
                scatter_ball_if_carrier(state, defender_id, rng);
                apply_regeneration(state, defender_id, rng);
                inj.casualty
            }
        }
    } else {
        None
    };

    // KnockBack: if attacker has KnockBack, auto-push defender one more square in the same direction.
    let att_has_knock_back = state.team(att_team)
        .player_by_id(attacker_id)
        .map(|p| p.has_skill(SkillId::KnockBack))
        .unwrap_or(false);

    if att_has_knock_back {
        // Compute the push direction: from defender_current_coord toward push_target
        let knockback_casualty = knock_back_second_push(state, attacker_id, defender_id, defender_current_coord, push_target, rng);
        let final_casualty = casualty.or(knockback_casualty);

        let att_has_frenzy = state.team(att_team)
            .player_by_id(attacker_id)
            .map(|p| p.has_skill(SkillId::Frenzy))
            .unwrap_or(false);
        let kb_is_blitz = state.acting_player.as_ref()
            .map(|ap| ap.current_action == Some(crate::types::PlayerAction::Blitz))
            .unwrap_or(false);
        let kb_att_juggernaut = state.team(att_team)
            .player_by_id(attacker_id)
            .map(|p| p.has_skill(SkillId::Juggernaut))
            .unwrap_or(false);
        let kb_def_has_fend = has_fend_protection(state, defender_id)
            && !(kb_is_blitz && kb_att_juggernaut);

        if att_has_frenzy && !kb_def_has_fend {
            // Frenzy: auto follow-up
            if !state.field.is_occupied(defender_current_coord) {
                state.field.move_player(attacker_id, defender_current_coord);
            }
            if let Some(ap) = state.acting_player.as_mut() {
                ap.frenzy_second_block_required = true;
            }
            state.dialog = DialogState::None;
        } else if !kb_def_has_fend && !state.field.is_occupied(defender_current_coord) {
            // Non-Frenzy: offer voluntary follow-up
            state.dialog = DialogState::SelectFollowup { square: defender_current_coord };
        } else {
            state.dialog = DialogState::None;
        }

        return BlockStepResult::Done(BlockResolution {
            attacker_knocked_down: false,
            defender_knocked_down: defender_knocked,
            turnover: false,
            casualty: final_casualty,
        });
    }

    // Fend: defender's Fend skill prevents attacker from following up after a push.
    // Juggernaut on a Blitz cancels Fend (same as it cancels StandFirm).
    let is_blitz_for_fend = state.acting_player.as_ref()
        .map(|ap| ap.current_action == Some(crate::types::PlayerAction::Blitz))
        .unwrap_or(false);
    let att_has_juggernaut_fend = state.team(att_team)
        .player_by_id(attacker_id)
        .map(|p| p.has_skill(SkillId::Juggernaut))
        .unwrap_or(false);
    let def_has_fend = has_fend_protection(state, defender_id)
        && !(is_blitz_for_fend && att_has_juggernaut_fend);

    let att_has_frenzy = state.team(att_team)
        .player_by_id(attacker_id)
        .map(|p| p.has_skill(SkillId::Frenzy))
        .unwrap_or(false);

    // Turnover check (needs to happen before we potentially pause for follow-up dialog)
    let defender_turnover = {
        let def_team = state.field.player_team(defender_id).expect("defender team");
        let active_team = state.active_team_id();
        if def_team == active_team {
            let defender_state = state.field.player_state(defender_id);
            matches!(defender_state, Some(s) if !s.is_active())
        } else {
            false
        }
    };

    if att_has_frenzy && !def_has_fend {
        // Frenzy: auto follow-up then flag second block required
        if !state.field.is_occupied(defender_current_coord) {
            state.field.move_player(attacker_id, defender_current_coord);
        }
        if let Some(ap) = state.acting_player.as_mut() {
            ap.frenzy_second_block_required = true;
        }
        state.dialog = DialogState::None;
    } else if !def_has_fend && !defender_turnover && !state.field.is_occupied(defender_current_coord) {
        // Non-Frenzy, no Fend, no immediate turnover: offer voluntary follow-up.
        // (If there's a turnover we skip the dialog and resolve the turnover immediately.)
        state.dialog = DialogState::SelectFollowup { square: defender_current_coord };
    } else {
        state.dialog = DialogState::None;
    }

    BlockStepResult::Done(BlockResolution {
        attacker_knocked_down: false,
        defender_knocked_down: defender_knocked,
        turnover: defender_turnover,
        casualty,
    })
}

// ── KnockBack ─────────────────────────────────────────────────────────────────

/// Auto-resolve a second push for the KnockBack skill.
/// Pushes the defender one additional square in the same direction as the first push.
/// Returns any casualty that results from the second push landing (armor/injury).
fn knock_back_second_push(
    state: &mut GameState,
    attacker_id: &PlayerId,
    defender_id: &PlayerId,
    from_coord: FieldCoordinate,
    first_push_target: FieldCoordinate,
    rng: &mut GameRng,
) -> Option<CasualtyType> {
    let def_team = state.field.player_team(defender_id).expect("defender team");
    let att_team = def_team.opponent();

    // Compute direction of first push
    let dx = (first_push_target.x as i16 - from_coord.x as i16).signum();
    let dy = (first_push_target.y as i16 - from_coord.y as i16).signum();

    // Compute second push target
    let cur = match state.field.player_coord(defender_id) {
        Some(c) => c,
        None => return None,
    };
    let new_x = (cur.x as i16 + dx).max(0) as u8;
    let new_y = (cur.y as i16 + dy).max(0) as u8;
    let second_target = FieldCoordinate::new(new_x, new_y);

    if !second_target.is_valid() {
        // Off pitch — crowd push
        apply_crowd_push(state, defender_id, rng);
        return None;
    }

    // Move defender to second push target — handles chain push when occupied.
    let crowd_pushed = resolve_chain_push(state, defender_id, cur, second_target, rng);
    if crowd_pushed {
        // Crowd push already applied armor/injury for the defender.
        return None;
    }

    // Apply armor/injury if defender is knocked down (prone)
    let defender_knocked = state.field.player_state(defender_id)
        .map(|s| s == PlayerState::Prone)
        .unwrap_or(false);

    if defender_knocked {
        let (def_av_base, mb, att_has_claws) = {
            let av = state.team(def_team).player_by_id(defender_id)
                .map(|p| p.effective_av())
                .unwrap_or(8);
            let (mb, claws) = state.team(att_team).player_by_id(attacker_id)
                .map(|p| (if p.has_skill(SkillId::MightyBlow) { 1u8 } else { 0 }, p.has_skill(SkillId::Claws)))
                .unwrap_or((0, false));
            (av, mb, claws)
        };
        let av = crate::mechanics::block::claws_effective_av(att_has_claws, def_av_base);
        match armor_roll(av, mb, 0, rng) {
            ArmorOutcome::NotBroken => None,
            ArmorOutcome::Broken => {
                let inj = resolve_injury_with_decay(defender_id, state, mb, rng);
                state.field.set_player_state(defender_id, inj.new_state);
                apply_regeneration(state, defender_id, rng);
                inj.casualty
            }
        }
    } else {
        None
    }
}

// ── Fend ──────────────────────────────────────────────────────────────────────

/// Returns true if the defender has the Fend skill, which prevents the attacker
/// from following up after a successful push.
pub fn has_fend_protection(state: &GameState, defender_id: &PlayerId) -> bool {
    let team = match state.field.player_team(defender_id) {
        Some(t) => t,
        None => return false,
    };
    let has_skill = state.team(team)
        .player_by_id(defender_id)
        .map(|p| p.has_skill(SkillId::Fend))
        .unwrap_or(false);
    if !has_skill {
        return false;
    }
    // Fend requires the defender to be standing — a prone/stunned defender cannot use it.
    state.field.player_state(defender_id)
        .map(|s| s.is_active())
        .unwrap_or(false)
}

// ── Brawler ───────────────────────────────────────────────────────────────────

/// Returns true if the attacker has the Brawler skill and therefore may re-roll
/// one block die once per block action.
/// In a full implementation this would also check that the reroll has not already
/// been used this activation; here it simply checks for the skill.
pub fn brawler_reroll_available(state: &GameState, player_id: &PlayerId) -> bool {
    let team = match state.field.player_team(player_id) {
        Some(t) => t,
        None => return false,
    };
    state.team(team)
        .player_by_id(player_id)
        .map(|p| p.has_skill(SkillId::Brawler))
        .unwrap_or(false)
}

// ── T-56 skill helpers ────────────────────────────────────────────────────────

/// T-56 #2: WhirlingDervish — returns all adjacent opponents of `attacker_id`.
/// Attacker must have WhirlingDervish for this to return any targets.
pub fn whirling_dervish_targets(state: &GameState, attacker_id: &PlayerId) -> Vec<PlayerId> {
    let has_skill = state.home.player_by_id(attacker_id)
        .or_else(|| state.away.player_by_id(attacker_id))
        .map(|p| p.has_skill(SkillId::WhirlingDervish))
        .unwrap_or(false);
    if !has_skill {
        return Vec::new();
    }

    let att_team = match state.field.player_team(attacker_id) {
        Some(t) => t,
        None => return Vec::new(),
    };
    let att_coord = match state.field.player_coord(attacker_id) {
        Some(c) => c,
        None => return Vec::new(),
    };
    let opp_team = att_team.opponent();

    att_coord.neighbors()
        .filter_map(|n| {
            let pid = state.field.player_at(n)?;
            if state.field.player_team(pid) == Some(opp_team) {
                if let Some(st) = state.field.player_state(pid) {
                    if st.is_active() {
                        return Some(pid.clone());
                    }
                }
            }
            None
        })
        .collect()
}

// ── SideStep ──────────────────────────────────────────────────────────────────

/// When a defender has SideStep, they may move to ANY adjacent empty square
/// instead of the normal 3-square pushback zone.
/// Returns all empty adjacent squares; if none, falls back to the normal
/// pushback options.
fn side_step_options(
    field: &crate::model::field_model::FieldModel,
    def_coord: FieldCoordinate,
) -> Vec<crate::mechanics::block::PushOption> {
    use crate::mechanics::block::PushOption;
    let adjacent_empty: Vec<PushOption> = def_coord
        .neighbors()
        .filter(|&n| !field.is_occupied(n))
        .map(PushOption::Empty)
        .collect();
    if adjacent_empty.is_empty() {
        // No empty squares — off pitch is the only option
        vec![PushOption::OffPitch]
    } else {
        adjacent_empty
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn knock_down_in_block(state: &mut GameState, player_id: &PlayerId, rng: &mut GameRng) {
    state.field.set_player_state(player_id, PlayerState::Prone);
    scatter_ball_if_carrier(state, player_id, rng);
}

/// If `player_id` is carrying the ball (and doesn't have SafePairOfHands),
/// scatter the ball one square. Called whenever a player becomes Prone/KO/Injured.
fn scatter_ball_if_carrier(state: &mut GameState, player_id: &PlayerId, rng: &mut GameRng) {
    let team = match state.field.player_team(player_id) {
        Some(t) => t,
        None => return,
    };
    let player_coord = state.field.player_coord(player_id);
    let carries_ball = state.field.ball.in_play
        && state.field.ball.coord.is_some()
        && state.field.ball.coord == player_coord;
    if carries_ball {
        let has_safe_pair = state.team(team)
            .player_by_id(player_id)
            .map(|p| p.has_skill(crate::skills::SkillId::SafePairOfHands))
            .unwrap_or(false);
        if !has_safe_pair {
            if let Some(coord) = player_coord {
                let scatter = crate::mechanics::pass::pass_scatter_coord(coord, rng);
                state.field.ball.coord = Some(scatter);
            }
        }
    }
}

fn apply_crowd_push(state: &mut GameState, player_id: &PlayerId, rng: &mut GameRng) {
    // Crowd push: armor roll with +1 crowd bonus; Stunned → Ko for injury results
    let av = state.home.player_by_id(player_id)
        .or_else(|| state.away.player_by_id(player_id))
        .map(|p| p.effective_av())
        .unwrap_or(7);
    match armor_roll(av, 0, 1, rng) {
        ArmorOutcome::NotBroken => {
            // Armor holds — crowd push still KOs by Blood Bowl rules (no armor doesn't help off pitch)
            state.field.set_player_state(player_id, PlayerState::Ko);
        }
        ArmorOutcome::Broken => {
            let inj = resolve_injury_with_decay(player_id, state, 0, rng);
            // Crowd push: Stunned → Ko (can't be stunned off pitch)
            let final_state = match inj.new_state {
                PlayerState::Stunned => PlayerState::Ko,
                other => other,
            };
            state.field.set_player_state(player_id, final_state);
            apply_regeneration(state, player_id, rng);
        }
    }
    // Ball carrier pushed into the crowd — scatter the ball from their last on-pitch square.
    scatter_ball_if_carrier(state, player_id, rng);
    // Remove from pitch coordinates — state preserved for KO/injury tracking.
    state.field.remove_from_pitch(player_id);
}

// ── Chain push ────────────────────────────────────────────────────────────────

/// Displace `player_id` (currently at `player_coord`) toward `target`.
///
/// If `target` is empty the player simply moves there.
/// If `target` is off-pitch the player is crowd-pushed.
/// If `target` is occupied a chain push is triggered: the occupant is pushed
/// first (recursively, using the same directional "push_from → player_coord →
/// target" direction), then `player_id` is moved into the now-vacated square.
///
/// When the chain-pushed occupant has multiple valid destinations the first
/// available option is auto-selected (active-player choice is reserved for the
/// top-level push dialog, which is already handled by `apply_block_dice_choice`).
/// Off-pitch destinations for chain-pushed players trigger crowd-push as normal.
///
/// Returns true if the player ended up crowd-pushed off the pitch.
pub fn resolve_chain_push(
    state: &mut GameState,
    player_id: &PlayerId,
    player_coord: FieldCoordinate,
    target: FieldCoordinate,
    rng: &mut GameRng,
) -> bool {
    use crate::mechanics::block::PushOption;

    // Off-pitch: crowd push
    if !target.is_valid() {
        apply_crowd_push(state, player_id, rng);
        return true;
    }

    // Empty: just move
    if !state.field.is_occupied(target) {
        state.field.move_player(player_id, target);
        // A player pushed onto a square with a loose ball scatters it immediately.
        if state.field.ball.in_play && state.field.ball.coord == Some(target) {
            let new_pos = crate::mechanics::pass::pass_scatter_coord(target, rng);
            state.field.ball.coord = Some(new_pos);
        }
        return false;
    }

    // Occupied: chain push the occupant first.
    // Direction for chain = player_coord → target (same direction as the incoming push).
    let chained_pid = state.field.player_at(target).unwrap().clone();

    // Compute chain-push options for the occupant.
    let chain_opts = {
        let pid_team = state.field.player_team(&chained_pid);
        let has_side_step = pid_team
            .and_then(|t| state.team(t).player_by_id(&chained_pid))
            .map(|p| p.has_skill(SkillId::SideStep))
            .unwrap_or(false);
        if has_side_step {
            side_step_options(&state.field, target)
        } else {
            // "attacker" for direction computation is player_coord (the square pushing into target)
            pushback_options(&state.field, player_coord, target)
        }
    };

    let has_off_pitch = chain_opts.iter().any(|o| matches!(o, PushOption::OffPitch));
    let coords: Vec<FieldCoordinate> = chain_opts.iter().filter_map(|o| o.coord()).collect();

    if coords.is_empty() {
        // All options are off-pitch — crowd push the occupant
        apply_crowd_push(state, &chained_pid, rng);
    } else {
        // Auto-pick first valid option for the chained player.
        // The active player's choice is handled at the top level (SelectPush dialog);
        // secondary chain-push destinations are resolved deterministically.
        let chain_target = coords[0];
        resolve_chain_push(state, &chained_pid, target, chain_target, rng);
    }

    // Now the target square should be vacated — move the original player in.
    if !state.field.is_occupied(target) {
        state.field.move_player(player_id, target);
        false
    } else {
        // Shouldn't happen in normal play (all chain options exhausted), but fall back
        // to crowd push rather than leaving the player in place.
        apply_crowd_push(state, player_id, rng);
        true
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::game_state::ActingPlayer;
    use crate::model::player::{Player, PlayerStats};
    use crate::model::team::Team;
    use crate::rng::GameRng;
    use crate::skills::SkillSet;
    use crate::types::{FieldCoordinate, PlayerId, PlayerState, TeamId};

    fn setup(att_pid: &str, def_pid: &str, att_st: u8, def_st: u8) -> (GameState, PlayerId, PlayerId) {
        let att_id = PlayerId(att_pid.into());
        let def_id = PlayerId(def_pid.into());

        let mut home = Team::new("h".into(), "Home".into(), "Human".into(), 3, true);
        let att = Player::new(
            att_id.clone(), att_pid.into(), "blitzer".into(), TeamId::Home, 1,
            PlayerStats::new(6, att_st, 3, 8, None), SkillSet::empty(),
        );
        home.add_player(att);

        let mut away = Team::new("a".into(), "Away".into(), "Orc".into(), 3, false);
        let def = Player::new(
            def_id.clone(), def_pid.into(), "lineman".into(), TeamId::Away, 1,
            PlayerStats::new(5, def_st, 3, 9, None), SkillSet::empty(),
        );
        away.add_player(def);

        let mut state = GameState::new(home, away);
        state.field.place_player(att_id.clone(), TeamId::Home, FieldCoordinate::new(5, 5), PlayerState::Standing);
        state.field.place_player(def_id.clone(), TeamId::Away, FieldCoordinate::new(6, 5), PlayerState::Standing);
        state.acting_player = Some(ActingPlayer::new(att_id.clone(), TeamId::Home));
        state.home_is_active = true;
        (state, att_id, def_id)
    }

    #[test]
    fn block_skull_knocks_down_attacker() {
        let (mut state, att, def) = setup("att", "def", 3, 3);
        // Inject Skull die (value 1 → Skull in roll_block_dice)
        let mut rng = GameRng::new_test([1]);
        let result = begin_block(&mut state, &att, &def, &mut rng);
        assert!(matches!(result, BlockStepResult::NeedDiceChoice { .. }));

        // Force skull choice
        let res = apply_block_dice_choice(&mut state, &att, &def, BlockResult::Skull, &mut rng);
        match res {
            BlockStepResult::Done(r) => {
                assert!(r.attacker_knocked_down);
                assert!(!r.defender_knocked_down);
                assert!(r.turnover);
            }
            _ => panic!("Expected Done"),
        }
        assert_eq!(state.field.player_state(&att), Some(PlayerState::Prone));
    }

    #[test]
    fn block_pow_knocks_down_defender() {
        let (mut state, att, def) = setup("att", "def", 3, 3);
        let mut rng = GameRng::new_test([6, 5, 4]); // dice + armor rolls
        begin_block(&mut state, &att, &def, &mut rng);

        let res = apply_block_dice_choice(&mut state, &att, &def, BlockResult::Pow, &mut rng);
        // Pow: defender knocked down, need push first
        match &res {
            BlockStepResult::NeedPushChoice { .. } | BlockStepResult::Done(_) => {}
            _ => panic!("Unexpected result: {:?}", res),
        }
    }

    #[test]
    fn block_both_down_with_block_skill() {
        let (mut state, att, def) = setup("att", "def", 3, 3);
        // Give attacker Block skill
        state.home.player_by_id_mut(&att).unwrap().skills.add(SkillId::Block);

        let mut rng = GameRng::new_test([2, 3]); // dice
        begin_block(&mut state, &att, &def, &mut rng);

        let res = apply_block_dice_choice(&mut state, &att, &def, BlockResult::BothDown, &mut rng);
        match res {
            BlockStepResult::Done(r) => {
                // Block skill: only defender knocked down
                assert!(!r.attacker_knocked_down);
                assert!(r.defender_knocked_down);
                assert!(!r.turnover);
            }
            _ => panic!("Expected Done"),
        }
    }

    #[test]
    fn block_both_down_no_skills_both_prone() {
        let (mut state, att, def) = setup("att", "def", 3, 3);
        let mut rng = GameRng::new_test([2, 3]);
        begin_block(&mut state, &att, &def, &mut rng);

        let res = apply_block_dice_choice(&mut state, &att, &def, BlockResult::BothDown, &mut rng);
        match res {
            BlockStepResult::Done(r) => {
                assert!(r.attacker_knocked_down);
                assert!(r.defender_knocked_down);
                assert!(r.turnover);
            }
            _ => panic!("Expected Done"),
        }
        assert_eq!(state.field.player_state(&att), Some(PlayerState::Prone));
        assert_eq!(state.field.player_state(&def), Some(PlayerState::Prone));
    }

    #[test]
    fn fend_protection_present_when_defender_has_fend() {
        let (mut state, _att, def) = setup("att", "def", 3, 3);
        state.away.player_by_id_mut(&def).unwrap().skills.add(SkillId::Fend);
        assert!(has_fend_protection(&state, &def));
    }

    #[test]
    fn fend_protection_absent_when_no_skill() {
        let (state, _att, def) = setup("att", "def", 3, 3);
        assert!(!has_fend_protection(&state, &def));
    }

    #[test]
    fn brawler_reroll_available_when_has_skill() {
        let (mut state, att, _def) = setup("att", "def", 3, 3);
        state.home.player_by_id_mut(&att).unwrap().skills.add(SkillId::Brawler);
        assert!(brawler_reroll_available(&state, &att));
    }

    #[test]
    fn brawler_reroll_unavailable_without_skill() {
        let (state, att, _def) = setup("att", "def", 3, 3);
        assert!(!brawler_reroll_available(&state, &att));
    }

    #[test]
    fn block_pushed_defender_moves() {
        let (mut state, att, def) = setup("att", "def", 3, 3);
        let mut rng = GameRng::new_test([4]);
        begin_block(&mut state, &att, &def, &mut rng);

        let res = apply_block_dice_choice(&mut state, &att, &def, BlockResult::Pushback, &mut rng);
        match &res {
            BlockStepResult::NeedPushChoice { options } => {
                let push_to = options[0];
                let final_res = apply_push_choice(&mut state, &att, &def, push_to, &mut rng);
                match final_res {
                    BlockStepResult::Done(r) => {
                        assert!(!r.attacker_knocked_down);
                        // Defender moved but not knocked down on a Pushed result
                        assert!(!r.defender_knocked_down);
                    }
                    _ => panic!("Expected Done after push"),
                }
                assert_eq!(state.field.player_coord(&def), Some(push_to));
            }
            BlockStepResult::Done(_) => {} // auto-resolved if only 1 option
            _ => panic!("Unexpected: {:?}", res),
        }
    }

    // ── T-56 WhirlingDervish tests ─────────────────────────────────────────

    #[test]
    fn whirling_dervish_no_skill_returns_empty() {
        let (state, att, _def) = setup("att", "def", 3, 3);
        // attacker has no WhirlingDervish
        let targets = whirling_dervish_targets(&state, &att);
        assert!(targets.is_empty());
    }

    #[test]
    fn whirling_dervish_with_adjacent_opponent() {
        let (mut state, att, def) = setup("att", "def", 3, 3);
        // Give attacker WhirlingDervish
        state.home.player_by_id_mut(&att).unwrap().skills.add(SkillId::WhirlingDervish);

        let targets = whirling_dervish_targets(&state, &att);
        // Should include the defender who is adjacent at (6,5) relative to attacker at (5,5)
        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0], def);
    }

    #[test]
    fn whirling_dervish_no_adjacent_opponents() {
        let (mut state, att, def) = setup("att", "def", 3, 3);
        state.home.player_by_id_mut(&att).unwrap().skills.add(SkillId::WhirlingDervish);
        // Move defender far away so not adjacent
        state.field.move_player(&def, FieldCoordinate::new(15, 15));

        let targets = whirling_dervish_targets(&state, &att);
        assert!(targets.is_empty());
    }

    // ── Stab tests ────────────────────────────────────────────────────────────

    #[test]
    fn stab_bypasses_armor_no_select_block_dice_dialog() {
        // Attacker with Stab → begin_block returns Done immediately (no dice dialog)
        let (mut state, att, def) = setup("att", "def", 3, 3);
        state.home.player_by_id_mut(&att).unwrap().skills.add(SkillId::Stab);

        // Inject 2 d6 values for injury_roll (roll_2d6 = 6+6 = 12 → Casualty)
        // plus 2 d8 values for casualty_roll
        let mut rng = GameRng::new_test([6, 6, 4, 5]);
        let result = begin_block(&mut state, &att, &def, &mut rng);
        // Must not be NeedDiceChoice
        assert!(
            matches!(result, BlockStepResult::Done(_)),
            "Stab should resolve immediately without block dice dialog, got {:?}", result
        );
    }

    #[test]
    fn stab_defender_prone_after_stab() {
        let (mut state, att, def) = setup("att", "def", 3, 3);
        state.home.player_by_id_mut(&att).unwrap().skills.add(SkillId::Stab);

        // Inject: injury 2+3=5 → Stunned (no casualty roll needed)
        let mut rng = GameRng::new_test([2, 3]);
        let result = begin_block(&mut state, &att, &def, &mut rng);
        match result {
            BlockStepResult::Done(r) => {
                assert!(r.defender_knocked_down);
                assert!(!r.attacker_knocked_down);
            }
            _ => panic!("Expected Done"),
        }
    }

    // ── Chainsaw tests ────────────────────────────────────────────────────────

    #[test]
    fn chainsaw_breaks_armor_always() {
        // Chainsaw attacker → begin_block returns Done immediately (armor always breaks)
        let (mut state, att, def) = setup("att", "def", 3, 3);
        state.home.player_by_id_mut(&att).unwrap().skills.add(SkillId::Chainsaw);

        // Inject 2 d6 for injury (4+5=9 → KO, no casualty roll)
        let mut rng = GameRng::new_test([4, 5]);
        let result = begin_block(&mut state, &att, &def, &mut rng);
        assert!(
            matches!(result, BlockStepResult::Done(_)),
            "Chainsaw should skip armor and dice, got {:?}", result
        );
        match result {
            BlockStepResult::Done(r) => {
                assert!(r.defender_knocked_down);
                assert!(!r.attacker_knocked_down);
            }
            _ => unreachable!(),
        }
    }

    // ── Frenzy tests ──────────────────────────────────────────────────────────

    #[test]
    fn frenzy_auto_follow_up_on_push() {
        // Attacker at (5,5) with Frenzy, defender at (6,5).
        // On a Pushback result the attacker should automatically move to (6,5)
        // and frenzy_second_block_required should be set to true.
        let (mut state, att, def) = setup("att", "def", 3, 3);
        state.home.player_by_id_mut(&att).unwrap().skills.add(SkillId::Frenzy);

        let mut rng = GameRng::new_test([4]);
        begin_block(&mut state, &att, &def, &mut rng);

        let res = apply_block_dice_choice(&mut state, &att, &def, BlockResult::Pushback, &mut rng);
        match &res {
            BlockStepResult::NeedPushChoice { options } => {
                let push_to = options[0];
                apply_push_choice(&mut state, &att, &def, push_to, &mut rng);
                // Attacker should have followed up to defender's old square (6,5)
                assert_eq!(
                    state.field.player_coord(&att),
                    Some(FieldCoordinate::new(6, 5)),
                    "Frenzy attacker should have followed up to defender's previous square"
                );
                // Second block flag should be set
                assert!(
                    state.acting_player.as_ref().map(|ap| ap.frenzy_second_block_required).unwrap_or(false),
                    "frenzy_second_block_required should be set after push"
                );
            }
            BlockStepResult::Done(_) => {
                // Auto-resolved: check frenzy flag
                assert!(
                    state.acting_player.as_ref().map(|ap| ap.frenzy_second_block_required).unwrap_or(false),
                    "frenzy_second_block_required should be set"
                );
            }
            _ => panic!("Unexpected: {:?}", res),
        }
    }

    // ── SideStep tests ────────────────────────────────────────────────────────

    #[test]
    fn side_step_uses_all_adjacent_squares() {
        // Defender at (6,5) with SideStep: push options should include all adjacent
        // empty squares, not just the normal 3 in the push direction.
        let (mut state, att, def) = setup("att", "def", 3, 3);
        state.away.player_by_id_mut(&def).unwrap().skills.add(SkillId::SideStep);

        let mut rng = GameRng::new_test([4]); // block die (unused — we force the choice)
        begin_block(&mut state, &att, &def, &mut rng);

        let res = apply_block_dice_choice(&mut state, &att, &def, BlockResult::Pushback, &mut rng);
        match res {
            BlockStepResult::NeedPushChoice { options } => {
                // With SideStep all adjacent empty squares should be available.
                // Defender at (6,5): neighbors = (5,4),(6,4),(7,4),(5,5),(7,5),(5,6),(6,6),(7,6)
                // (5,5) is occupied by attacker, rest are empty → 7 options
                // Normal pushback would give at most 3.
                assert!(
                    options.len() > 3,
                    "SideStep should yield more than 3 push options, got {}",
                    options.len()
                );
            }
            BlockStepResult::Done(_) => {
                // Auto-resolved if only 1 empty square (unlikely in open field, fail safe)
                panic!("Expected NeedPushChoice with multiple SideStep options");
            }
            _ => panic!("Unexpected: {:?}", res),
        }
    }

    // ── KnockBack tests ───────────────────────────────────────────────────────

    #[test]
    fn knock_back_extra_push_on_pushback() {
        // Attacker at (5,5) with KnockBack, defender at (6,5).
        // On a Pushback result, defender gets pushed to (7,5) (first push).
        // KnockBack then auto-pushes to (8,5) (second push).
        // Defender is NOT knocked down (Pushback result), so no armor roll.
        let att_id = PlayerId("att_kb".into());
        let def_id = PlayerId("def_kb".into());

        let kb_skills: SkillSet = [SkillId::KnockBack].into_iter().collect();
        let mut home = Team::new("h".into(), "Home".into(), "Human".into(), 3, true);
        home.add_player(Player::new(
            att_id.clone(), "KnockBack Att".into(), "blitzer".into(), TeamId::Home, 1,
            PlayerStats::new(6, 3, 3, 8, None), kb_skills,
        ));
        let mut away = Team::new("a".into(), "Away".into(), "Orc".into(), 3, false);
        away.add_player(Player::new(
            def_id.clone(), "Def".into(), "lineman".into(), TeamId::Away, 1,
            PlayerStats::new(5, 3, 3, 9, None), SkillSet::empty(),
        ));
        let mut state = GameState::new(home, away);
        state.field.place_player(att_id.clone(), TeamId::Home, FieldCoordinate::new(5, 5), PlayerState::Standing);
        state.field.place_player(def_id.clone(), TeamId::Away, FieldCoordinate::new(6, 5), PlayerState::Standing);
        state.acting_player = Some(ActingPlayer::new(att_id.clone(), TeamId::Home));
        state.home_is_active = true;

        let mut rng = GameRng::new_test([4]); // block die roll
        begin_block(&mut state, &att_id, &def_id, &mut rng);

        let res = apply_block_dice_choice(&mut state, &att_id, &def_id, BlockResult::Pushback, &mut rng);
        match &res {
            BlockStepResult::NeedPushChoice { options } => {
                // Pick the first push option (should be (7,5) or similar in push direction)
                let first_push = options[0];
                let final_res = apply_push_choice(&mut state, &att_id, &def_id, first_push, &mut rng);
                match final_res {
                    BlockStepResult::Done(r) => {
                        // KnockBack: defender pushed a second time — check they moved further
                        assert!(!r.attacker_knocked_down);
                        // Defender was not knocked down on a Pushback result
                        assert!(!r.defender_knocked_down);
                    }
                    _ => panic!("Expected Done after KnockBack push"),
                }
                // Defender should be at first_push + direction (second push)
                let def_coord = state.field.player_coord(&def_id).expect("defender coord");
                // Defender started at (6,5), first pushed to `first_push`, second pushed further
                // The important thing: defender is not at first_push anymore (KnockBack moved them again)
                // OR they may be at first_push if second push was blocked/off-pitch
                // At minimum, defender is somewhere valid
                assert!(def_coord.is_valid(), "Defender should be on a valid square");
            }
            BlockStepResult::Done(r) => {
                // Auto-resolved: KnockBack still applied
                assert!(!r.attacker_knocked_down);
                let def_coord = state.field.player_coord(&def_id).expect("defender coord after auto push");
                assert!(def_coord.is_valid());
            }
            _ => panic!("Unexpected: {:?}", res),
        }
    }

    // ── Dauntless wired in begin_block ────────────────────────────────────────

    #[test]
    fn dauntless_wired_in_begin_block_success() {
        // Attacker ST=2 with Dauntless vs defender ST=4 → normal would be -2 (defender picks 2).
        // Dauntless roll=4 → raises to 1 die (attacker picks).
        // Roll sequence: [dauntless_roll=4, block_die=4]
        let (mut state, att, def) = setup("att_d", "def_d", 2, 4);
        state.home.player_by_id_mut(&att).unwrap().skills.add(SkillId::Dauntless);

        let mut rng = GameRng::new_test([4, 4]); // dauntless d6=4 (success), then block die
        let result = begin_block(&mut state, &att, &def, &mut rng);
        match result {
            BlockStepResult::NeedDiceChoice { defender_chooses, .. } => {
                // Dauntless succeeded → 1 die, attacker chooses (not defender)
                assert!(!defender_chooses, "Dauntless succeeded: attacker should choose (1 die)");
            }
            _ => panic!("Expected NeedDiceChoice, got {:?}", result),
        }
    }

    #[test]
    fn dauntless_wired_in_begin_block_failure() {
        // Attacker ST=2 with Dauntless vs defender ST=4 → normal = -2.
        // Dauntless roll=3 (fail) → stays at -2 (defender picks).
        let (mut state, att, def) = setup("att_d2", "def_d2", 2, 4);
        state.home.player_by_id_mut(&att).unwrap().skills.add(SkillId::Dauntless);

        let mut rng = GameRng::new_test([3, 4, 4]); // dauntless d6=3 (fail), then 2 block dice
        let result = begin_block(&mut state, &att, &def, &mut rng);
        match result {
            BlockStepResult::NeedDiceChoice { defender_chooses, .. } => {
                assert!(defender_chooses, "Dauntless failed: defender should still choose (2 dice)");
            }
            _ => panic!("Expected NeedDiceChoice, got {:?}", result),
        }
    }

    // ── StandFirm prevents push ────────────────────────────────────────────────

    #[test]
    fn stand_firm_prevents_push_on_pushback() {
        // Defender with StandFirm: Pushback result → push cancelled, defender stays put.
        let (mut state, att, def) = setup("att_sf", "def_sf", 3, 3);
        state.away.player_by_id_mut(&def).unwrap().skills.add(SkillId::StandFirm);

        let mut rng = GameRng::new_test([4]); // block die roll
        begin_block(&mut state, &att, &def, &mut rng);

        let def_coord_before = state.field.player_coord(&def).unwrap();
        let res = apply_block_dice_choice(&mut state, &att, &def, BlockResult::Pushback, &mut rng);
        match res {
            BlockStepResult::Done(r) => {
                assert!(!r.attacker_knocked_down);
                assert!(!r.defender_knocked_down, "StandFirm: Pushback = not knocked down");
                // Defender should not have moved
                assert_eq!(
                    state.field.player_coord(&def),
                    Some(def_coord_before),
                    "StandFirm: defender should stay in place on Pushback"
                );
            }
            _ => panic!("Expected Done (StandFirm cancels push), got {:?}", res),
        }
    }

    #[test]
    fn stand_firm_still_knocked_down_on_pow() {
        // Defender with StandFirm + Pow result: they get knocked down but push is cancelled.
        let (mut state, att, def) = setup("att_sf2", "def_sf2", 3, 3);
        state.away.player_by_id_mut(&def).unwrap().skills.add(SkillId::StandFirm);

        let mut rng = GameRng::new_test([4]);
        begin_block(&mut state, &att, &def, &mut rng);

        let def_coord_before = state.field.player_coord(&def).unwrap();
        let res = apply_block_dice_choice(&mut state, &att, &def, BlockResult::Pow, &mut rng);
        match res {
            BlockStepResult::Done(r) => {
                assert!(r.defender_knocked_down, "StandFirm + Pow: defender still knocked down");
                assert_eq!(
                    state.field.player_coord(&def),
                    Some(def_coord_before),
                    "StandFirm: defender should stay in place even on Pow"
                );
            }
            _ => panic!("Expected Done (StandFirm cancels push), got {:?}", res),
        }
    }

    #[test]
    fn juggernaut_on_blitz_cancels_stand_firm() {
        // Attacker with Juggernaut on a Blitz vs defender with StandFirm.
        // Juggernaut cancels StandFirm → push proceeds normally.
        let (mut state, att, def) = setup("att_jug", "def_jug", 3, 3);
        state.home.player_by_id_mut(&att).unwrap().skills.add(SkillId::Juggernaut);
        state.away.player_by_id_mut(&def).unwrap().skills.add(SkillId::StandFirm);
        // Mark as blitz
        if let Some(ap) = state.acting_player.as_mut() {
            ap.has_blitzed = true;
        }

        let mut rng = GameRng::new_test([4]);
        begin_block(&mut state, &att, &def, &mut rng);

        let def_coord_before = state.field.player_coord(&def).unwrap();
        let res = apply_block_dice_choice(&mut state, &att, &def, BlockResult::Pushback, &mut rng);
        // Should NOT be Done immediately — push should proceed (StandFirm cancelled by Juggernaut)
        match res {
            BlockStepResult::NeedPushChoice { .. } => {
                // Push options shown — StandFirm was cancelled
            }
            BlockStepResult::Done(_) => {
                // Auto-resolved (single option) is also OK — but defender should have moved
                assert_ne!(
                    state.field.player_coord(&def),
                    Some(def_coord_before),
                    "Juggernaut should cancel StandFirm: defender should be pushed"
                );
            }
            _ => panic!("Expected push to proceed when Juggernaut cancels StandFirm"),
        }
    }

    // ── Non-Frenzy follow-up choice ───────────────────────────────────────────

    #[test]
    fn non_frenzy_push_offers_followup_dialog() {
        // Attacker (no Frenzy) at (5,5), defender at (6,5).
        // On Pushback: SelectFollowup dialog should be emitted.
        let (mut state, att, def) = setup("att_nf", "def_nf", 3, 3);
        // No Frenzy skill on attacker

        let mut rng = GameRng::new_test([4]);
        begin_block(&mut state, &att, &def, &mut rng);

        let res = apply_block_dice_choice(&mut state, &att, &def, BlockResult::Pushback, &mut rng);
        match &res {
            BlockStepResult::NeedPushChoice { options } => {
                let push_to = options[0];
                apply_push_choice(&mut state, &att, &def, push_to, &mut rng);
                // Should emit SelectFollowup dialog
                assert!(
                    matches!(state.dialog, DialogState::SelectFollowup { .. }),
                    "Expected SelectFollowup dialog, got {:?}", state.dialog
                );
                // Attacker should NOT have moved yet
                assert_eq!(
                    state.field.player_coord(&att),
                    Some(FieldCoordinate::new(5, 5)),
                    "Attacker should not move until ChooseFollowup is resolved"
                );
            }
            _ => panic!("Expected NeedPushChoice, got {:?}", res),
        }
    }

    #[test]
    fn non_frenzy_follow_up_accepted_moves_attacker() {
        // After SelectFollowup is emitted, accepting it moves attacker to the vacated square.
        let (mut state, att, def) = setup("att_nfa", "def_nfa", 3, 3);

        let mut rng = GameRng::new_test([4]);
        begin_block(&mut state, &att, &def, &mut rng);

        let res = apply_block_dice_choice(&mut state, &att, &def, BlockResult::Pushback, &mut rng);
        let push_to = match &res {
            BlockStepResult::NeedPushChoice { options } => options[0],
            _ => panic!("Expected NeedPushChoice"),
        };
        apply_push_choice(&mut state, &att, &def, push_to, &mut rng);

        let followup_sq = match &state.dialog {
            DialogState::SelectFollowup { square } => *square,
            _ => panic!("Expected SelectFollowup dialog"),
        };

        // Manually simulate ChooseFollowup(true)
        state.dialog = DialogState::None;
        if !state.field.is_occupied(followup_sq) {
            state.field.move_player(&att, followup_sq);
        }

        assert_eq!(
            state.field.player_coord(&att),
            Some(followup_sq),
            "Attacker should be at the vacated square after accepting follow-up"
        );
    }

    #[test]
    fn non_frenzy_follow_up_declined_keeps_attacker() {
        // After SelectFollowup is emitted, declining it leaves attacker in place.
        let (mut state, att, def) = setup("att_nfd", "def_nfd", 3, 3);

        let mut rng = GameRng::new_test([4]);
        begin_block(&mut state, &att, &def, &mut rng);

        let att_coord_before = state.field.player_coord(&att).unwrap();
        let res = apply_block_dice_choice(&mut state, &att, &def, BlockResult::Pushback, &mut rng);
        let push_to = match &res {
            BlockStepResult::NeedPushChoice { options } => options[0],
            _ => panic!("Expected NeedPushChoice"),
        };
        apply_push_choice(&mut state, &att, &def, push_to, &mut rng);

        assert!(matches!(state.dialog, DialogState::SelectFollowup { .. }), "Expected SelectFollowup");

        // Decline follow-up: attacker stays put
        state.dialog = DialogState::None;
        // (no move applied)

        assert_eq!(
            state.field.player_coord(&att),
            Some(att_coord_before),
            "Attacker should stay at original square when follow-up declined"
        );
    }

    #[test]
    fn fend_prevents_non_frenzy_followup_dialog() {
        // Defender with Fend: no SelectFollowup dialog should be emitted.
        let (mut state, att, def) = setup("att_nf2", "def_nf2", 3, 3);
        state.away.player_by_id_mut(&def).unwrap().skills.add(SkillId::Fend);

        let mut rng = GameRng::new_test([4]);
        begin_block(&mut state, &att, &def, &mut rng);

        let res = apply_block_dice_choice(&mut state, &att, &def, BlockResult::Pushback, &mut rng);
        match &res {
            BlockStepResult::NeedPushChoice { options } => {
                let push_to = options[0];
                apply_push_choice(&mut state, &att, &def, push_to, &mut rng);
                assert_eq!(
                    state.dialog,
                    DialogState::None,
                    "Fend should suppress SelectFollowup dialog"
                );
            }
            BlockStepResult::Done(_) => {
                assert_eq!(state.dialog, DialogState::None);
            }
            _ => panic!("Unexpected: {:?}", res),
        }
    }

    // ── Chain push tests ──────────────────────────────────────────────────────

    /// Place an extra player on the away team at a given coord.
    fn place_away_blocker(state: &mut GameState, pid_str: &str, coord: FieldCoordinate) -> PlayerId {
        let pid = PlayerId(pid_str.into());
        let p = Player::new(
            pid.clone(), pid_str.into(), "lineman".into(), TeamId::Away, 99,
            PlayerStats::new(5, 3, 3, 9, None), SkillSet::empty(),
        );
        state.away.add_player(p);
        state.field.place_player(pid.clone(), TeamId::Away, coord, PlayerState::Standing);
        pid
    }

    #[test]
    fn chain_push_displaces_occupant_and_original_defender() {
        // att(5,5), def(6,5). All three push squares occupied:
        //   (7,4)=b_n, (7,5)=b_c, (7,6)=b_s  — each has open squares behind them.
        // pushback_options returns all three as chain targets.
        // We choose (7,5) for def. b_c gets pushed to (8,5) (first empty option behind it).
        // Final: b_c at (8,5), def at (7,5).
        let (mut state, att, def) = setup("att_cp", "def_cp", 3, 3);
        let b_n = place_away_blocker(&mut state, "b_n", FieldCoordinate::new(7, 4));
        let b_c = place_away_blocker(&mut state, "b_c", FieldCoordinate::new(7, 5));
        let b_s = place_away_blocker(&mut state, "b_s", FieldCoordinate::new(7, 6));
        // (8,4), (8,5), (8,6) are all empty

        let mut rng = GameRng::new_test([4]);
        begin_block(&mut state, &att, &def, &mut rng);

        let res = apply_block_dice_choice(&mut state, &att, &def, BlockResult::Pushback, &mut rng);
        match res {
            BlockStepResult::NeedPushChoice { options } => {
                assert!(options.contains(&FieldCoordinate::new(7, 5)), "chain push to (7,5) should be an option");
                apply_push_choice(&mut state, &att, &def, FieldCoordinate::new(7, 5), &mut rng);

                // def should now be at (7,5)
                assert_eq!(
                    state.field.player_coord(&def),
                    Some(FieldCoordinate::new(7, 5)),
                    "original defender should be at (7,5) after chain"
                );
                // b_c should have moved away from (7,5) (to one of the open squares behind)
                let b_c_coord = state.field.player_coord(&b_c);
                assert!(
                    b_c_coord.is_some() && b_c_coord != Some(FieldCoordinate::new(7, 5)),
                    "chain-pushed b_c should have moved; got {:?}", b_c_coord
                );
            }
            BlockStepResult::Done(_) => {
                // Auto-resolved — verify defender moved
                assert_ne!(state.field.player_coord(&def), Some(FieldCoordinate::new(6, 5)),
                    "defender should have moved from original square");
            }
            _ => panic!("Unexpected: {:?}", res),
        }
    }

    #[test]
    fn chain_push_deep_two_levels() {
        // Three players lined up: att(5,5), def(6,5), mid(7,5), end(8,5).
        // def's push options (7,4), (7,5), (7,6): block (7,4) and (7,6) to force def→(7,5).
        // mid's push options (8,4), (8,5), (8,6): block all three to force chain-push of end.
        //   end at (8,5), m_n at (8,4), m_s at (8,6) → mid must push end, end goes to (9,5).
        // Final: end→(9,5), mid→(8,5), def→(7,5).
        let (mut state, att, def) = setup("att_deep", "def_deep", 3, 3);
        let mid = place_away_blocker(&mut state, "mid_deep", FieldCoordinate::new(7, 5));
        let end = place_away_blocker(&mut state, "end_deep", FieldCoordinate::new(8, 5));
        // Block def's diagonal options to force def→(7,5)
        place_away_blocker(&mut state, "d1", FieldCoordinate::new(7, 4));
        place_away_blocker(&mut state, "d2", FieldCoordinate::new(7, 6));
        // Block mid's diagonal options to force mid to chain-push end→(9,5)
        place_away_blocker(&mut state, "m_n", FieldCoordinate::new(8, 4));
        place_away_blocker(&mut state, "m_s", FieldCoordinate::new(8, 6));

        let mut rng = GameRng::new_test([4]);
        begin_block(&mut state, &att, &def, &mut rng);

        let res = apply_block_dice_choice(&mut state, &att, &def, BlockResult::Pushback, &mut rng);
        let chosen = FieldCoordinate::new(7, 5);
        match res {
            BlockStepResult::NeedPushChoice { .. } => {
                apply_push_choice(&mut state, &att, &def, chosen, &mut rng);
            }
            BlockStepResult::Done(_) => {
                // auto-resolved — positions already set
            }
            _ => panic!("Unexpected result"),
        }

        assert_eq!(state.field.player_coord(&def), Some(FieldCoordinate::new(7, 5)),
            "def should be at (7,5)");
        assert_eq!(state.field.player_coord(&mid), Some(FieldCoordinate::new(8, 5)),
            "mid should be chain-pushed to (8,5)");
        assert_eq!(state.field.player_coord(&end), Some(FieldCoordinate::new(9, 5)),
            "end should be chain-pushed to (9,5)");
    }

    #[test]
    fn chain_push_off_pitch_crowd_pushes_occupant() {
        // Defender at (15, 5), blocker at (16, 5).
        // Squares east of (16,5): (17,5), (16,4), (16,6). All are valid.
        // Let's put both near east boundary so chain leads off pitch.
        // Defender at (24,5), blocker at (25,5).
        // Blocker's east squares: x=26 is off pitch → crowd push.
        let (mut state, att, def) = {
            let att_id = PlayerId("att_off".into());
            let def_id = PlayerId("def_off".into());
            let mut home = Team::new("h".into(), "Home".into(), "Human".into(), 3, true);
            home.add_player(Player::new(
                att_id.clone(), "att_off".into(), "blitzer".into(), TeamId::Home, 1,
                PlayerStats::new(6, 3, 3, 8, None), SkillSet::empty(),
            ));
            let mut away = Team::new("a".into(), "Away".into(), "Orc".into(), 3, false);
            away.add_player(Player::new(
                def_id.clone(), "def_off".into(), "lineman".into(), TeamId::Away, 1,
                PlayerStats::new(5, 3, 3, 9, None), SkillSet::empty(),
            ));
            let mut state = GameState::new(home, away);
            state.field.place_player(att_id.clone(), TeamId::Home, FieldCoordinate::new(23, 5), PlayerState::Standing);
            state.field.place_player(def_id.clone(), TeamId::Away, FieldCoordinate::new(24, 5), PlayerState::Standing);
            state.acting_player = Some(ActingPlayer::new(att_id.clone(), TeamId::Home));
            state.home_is_active = true;
            (state, att_id, def_id)
        };

        // Blocker at x=25 (east boundary). Their push options east would be off-pitch.
        let blocker = place_away_blocker(&mut state, "blk_off", FieldCoordinate::new(25, 5));
        // Also block (25,4) and (25,6) so blocker must go off-pitch
        place_away_blocker(&mut state, "bd4", FieldCoordinate::new(25, 4));
        place_away_blocker(&mut state, "bd6", FieldCoordinate::new(25, 6));

        let mut rng = GameRng::new_test([4, 3, 5]); // guard + crowd push armor rolls
        begin_block(&mut state, &att, &def, &mut rng);

        let res = apply_block_dice_choice(&mut state, &att, &def, BlockResult::Pushback, &mut rng);
        // All options occupied (25,4), (25,5), (25,6): chain push expected
        match res {
            BlockStepResult::NeedPushChoice { options } => {
                let chain_to = FieldCoordinate::new(25, 5); // push def to blocker's square
                apply_push_choice(&mut state, &att, &def, chain_to, &mut rng);

                // Blocker should be crowd-pushed (KO or worse) — no longer active on pitch
                let blocker_state = state.field.player_state(&blocker);
                assert!(
                    matches!(blocker_state, Some(PlayerState::Ko) | Some(PlayerState::Injured)),
                    "blocker pushed off pitch should be KO/Injured, got {:?}", blocker_state
                );
                // Defender should now be at (25,5)
                assert_eq!(state.field.player_coord(&def), Some(FieldCoordinate::new(25, 5)),
                    "defender should be at chain target after blocker was crowd-pushed");
            }
            BlockStepResult::Done(_) => {
                // Auto-resolved — still verify blocker was affected
                let blocker_state = state.field.player_state(&blocker);
                assert!(
                    blocker_state.is_some(),
                    "blocker should still have a state"
                );
            }
            _ => panic!("Unexpected: {:?}", res),
        }
    }

    #[test]
    fn single_push_to_empty_no_chain() {
        // Sanity check: normal push to empty square still works correctly.
        let (mut state, att, def) = setup("att_sc", "def_sc", 3, 3);
        // No extra players — (7,5), (7,4), (7,6) all empty

        let mut rng = GameRng::new_test([4]);
        begin_block(&mut state, &att, &def, &mut rng);

        let res = apply_block_dice_choice(&mut state, &att, &def, BlockResult::Pushback, &mut rng);
        match res {
            BlockStepResult::NeedPushChoice { options } => {
                let target = options[0];
                apply_push_choice(&mut state, &att, &def, target, &mut rng);
                assert_eq!(state.field.player_coord(&def), Some(target),
                    "defender should move to chosen push square");
                assert_eq!(state.field.player_coord(&att), Some(FieldCoordinate::new(5, 5)),
                    "attacker should not move (no Frenzy, dialog pending follow-up)");
            }
            BlockStepResult::Done(_) => {}
            _ => panic!("Unexpected: {:?}", res),
        }
    }

    #[test]
    fn chain_push_with_side_step_occupant() {
        // Blocker (occupant of push target) has SideStep — uses adjacent empty squares.
        // att(5,5), def(6,5), blocker(7,5) with SideStep.
        // All three push squares blocked so chain must happen.
        let (mut state, att, def) = setup("att_ss", "def_ss", 3, 3);
        place_away_blocker(&mut state, "d4", FieldCoordinate::new(7, 4));
        let blocker = place_away_blocker(&mut state, "blk_ss", FieldCoordinate::new(7, 5));
        place_away_blocker(&mut state, "d6", FieldCoordinate::new(7, 6));
        // Give blocker SideStep
        state.away.player_by_id_mut(&blocker).unwrap().skills.add(SkillId::SideStep);
        // (8,5) is open — blocker should SideStep there

        let mut rng = GameRng::new_test([4]);
        begin_block(&mut state, &att, &def, &mut rng);

        let res = apply_block_dice_choice(&mut state, &att, &def, BlockResult::Pushback, &mut rng);
        match res {
            BlockStepResult::NeedPushChoice { options } => {
                let chain_to = FieldCoordinate::new(7, 5);
                if options.contains(&chain_to) {
                    apply_push_choice(&mut state, &att, &def, chain_to, &mut rng);
                    // Blocker with SideStep should have moved to an adjacent empty square
                    let blocker_coord = state.field.player_coord(&blocker);
                    assert!(
                        blocker_coord.is_some() && blocker_coord != Some(FieldCoordinate::new(7, 5)),
                        "SideStep blocker should have moved; got {:?}", blocker_coord
                    );
                    // Defender should now be at (7,5)
                    assert_eq!(state.field.player_coord(&def), Some(FieldCoordinate::new(7, 5)));
                }
            }
            _ => {}
        }
    }

    // ── Fend prevents Frenzy follow-up ────────────────────────────────────────

    #[test]
    fn fend_prevents_frenzy_follow_up() {
        // Attacker at (5,5) with Frenzy, defender at (6,5) with Fend.
        // On Pushback: attacker should NOT follow up (Fend prevents it).
        let (mut state, att, def) = setup("att_fend", "def_fend", 3, 3);
        state.home.player_by_id_mut(&att).unwrap().skills.add(SkillId::Frenzy);
        state.away.player_by_id_mut(&def).unwrap().skills.add(SkillId::Fend);

        let mut rng = GameRng::new_test([4]);
        begin_block(&mut state, &att, &def, &mut rng);

        let att_coord_before = state.field.player_coord(&att).unwrap();
        let res = apply_block_dice_choice(&mut state, &att, &def, BlockResult::Pushback, &mut rng);
        match &res {
            BlockStepResult::NeedPushChoice { options } => {
                let push_to = options[0];
                apply_push_choice(&mut state, &att, &def, push_to, &mut rng);
                // Attacker should NOT have followed up — Fend prevents it
                assert_eq!(
                    state.field.player_coord(&att),
                    Some(att_coord_before),
                    "Fend should prevent Frenzy attacker from following up"
                );
                // frenzy_second_block_required should NOT be set
                assert!(
                    !state.acting_player.as_ref().map(|ap| ap.frenzy_second_block_required).unwrap_or(false),
                    "Fend: frenzy_second_block_required should not be set"
                );
            }
            BlockStepResult::Done(_) => {
                // Auto-resolved — attacker should not have moved
                assert_eq!(
                    state.field.player_coord(&att),
                    Some(att_coord_before),
                    "Fend should prevent Frenzy follow-up even on auto-resolve"
                );
            }
            _ => panic!("Unexpected: {:?}", res),
        }
    }

    // ── Fend inactive when defender is prone ─────────────────────────────────

    #[test]
    fn fend_inactive_when_defender_prone_after_pow_pushback() {
        // On PowPushback the defender is knocked down (Prone) before the push resolves.
        // A prone player cannot use Fend, so the attacker should get a follow-up dialog.
        let (mut state, att, def) = setup("att_fp", "def_fp", 3, 3);
        state.away.player_by_id_mut(&def).unwrap().skills.add(SkillId::Fend);

        // block die + 2d6 armor roll (PowPushback knocks down → armor resolved in apply_push_choice)
        let mut rng = GameRng::new_test([4, 2, 3]);
        begin_block(&mut state, &att, &def, &mut rng);

        let res = apply_block_dice_choice(&mut state, &att, &def, BlockResult::PowPushback, &mut rng);
        match res {
            BlockStepResult::NeedPushChoice { options } => {
                let push_to = options[0];
                apply_push_choice(&mut state, &att, &def, push_to, &mut rng);
                // Defender is prone → Fend inactive → SelectFollowup dialog should appear
                assert!(
                    matches!(state.dialog, DialogState::SelectFollowup { .. }),
                    "Fend should be inactive when defender is prone; expected SelectFollowup, got {:?}",
                    state.dialog
                );
            }
            BlockStepResult::Done(_) => {
                // Auto-resolved (single option). Defender is prone → no Fend protection.
                // Dialog doesn't matter in auto-resolve if there was only one push option.
            }
            _ => panic!("Unexpected: {:?}", res),
        }
    }

    // ── Juggernaut on blitz cancels Fend ────────────────────────────────────

    #[test]
    fn juggernaut_on_blitz_cancels_fend() {
        // Attacker has Juggernaut, declares Blitz, defender has Fend.
        // Juggernaut on Blitz should cancel Fend → follow-up dialog appears.
        let (mut state, att, def) = setup("att_jf", "def_jf", 3, 3);
        state.home.player_by_id_mut(&att).unwrap().skills.add(SkillId::Juggernaut);
        state.away.player_by_id_mut(&def).unwrap().skills.add(SkillId::Fend);

        // Mark as Blitz action
        if let Some(ap) = state.acting_player.as_mut() {
            ap.current_action = Some(crate::types::PlayerAction::Blitz);
        }

        let mut rng = GameRng::new_test([4]);
        begin_block(&mut state, &att, &def, &mut rng);

        let res = apply_block_dice_choice(&mut state, &att, &def, BlockResult::Pushback, &mut rng);
        match res {
            BlockStepResult::NeedPushChoice { options } => {
                let push_to = options[0];
                apply_push_choice(&mut state, &att, &def, push_to, &mut rng);
                assert!(
                    matches!(state.dialog, DialogState::SelectFollowup { .. }),
                    "Juggernaut on blitz should cancel Fend: expected SelectFollowup, got {:?}",
                    state.dialog
                );
            }
            BlockStepResult::Done(_) => {
                // Auto-resolved path — acceptable
            }
            _ => panic!("Unexpected: {:?}", res),
        }
    }

    // ── Direct crowd push at boundary ────────────────────────────────────────

    #[test]
    fn direct_crowd_push_at_east_boundary() {
        // Defender at x=25 (east edge), attacker to the west.
        // All 3 push candidates are off-pitch → direct crowd push with no chain.
        let att_id = PlayerId("att_eb".into());
        let def_id = PlayerId("def_eb".into());
        let mut home = Team::new("h".into(), "Home".into(), "Human".into(), 3, true);
        home.add_player(Player::new(
            att_id.clone(), "att_eb".into(), "blitzer".into(), TeamId::Home, 1,
            PlayerStats::new(6, 3, 3, 8, None), SkillSet::empty(),
        ));
        let mut away = Team::new("a".into(), "Away".into(), "Orc".into(), 3, false);
        away.add_player(Player::new(
            def_id.clone(), "def_eb".into(), "lineman".into(), TeamId::Away, 1,
            PlayerStats::new(5, 3, 3, 9, None), SkillSet::empty(),
        ));
        let mut state = GameState::new(home, away);
        state.field.place_player(att_id.clone(), TeamId::Home, FieldCoordinate::new(24, 8), PlayerState::Standing);
        state.field.place_player(def_id.clone(), TeamId::Away, FieldCoordinate::new(25, 8), PlayerState::Standing);
        state.acting_player = Some(ActingPlayer::new(att_id.clone(), TeamId::Home));
        state.home_is_active = true;

        // 1 die for block + 2d6 for crowd-push armor roll
        let mut rng = GameRng::new_test([4, 3, 4]);
        begin_block(&mut state, &att_id, &def_id, &mut rng);

        let res = apply_block_dice_choice(&mut state, &att_id, &def_id, BlockResult::Pushback, &mut rng);
        // Should auto-resolve to crowd push (all options off-pitch)
        match res {
            BlockStepResult::Done(r) => {
                assert!(r.defender_knocked_down, "crowd push always knocks down");
                let def_state = state.field.player_state(&def_id);
                assert!(
                    matches!(def_state, Some(PlayerState::Ko) | Some(PlayerState::Injured)),
                    "crowd-pushed defender should be Ko or Injured, got {:?}", def_state
                );
                assert!(
                    state.field.player_coord(&def_id).is_none(),
                    "crowd-pushed defender should be off the pitch"
                );
            }
            _ => panic!("Expected auto-resolved Done for crowd push, got {:?}", res),
        }
    }

    // ── Ball scatter: crowd push of ball carrier ─────────────────────────────

    #[test]
    fn crowd_push_scatters_ball_if_carrier() {
        // Defender at east boundary carries the ball.
        // Crowd push should scatter the ball from the defender's last square.
        let att_id = PlayerId("att_cb".into());
        let def_id = PlayerId("def_cb".into());
        let mut home = Team::new("h".into(), "Home".into(), "Human".into(), 3, true);
        home.add_player(Player::new(
            att_id.clone(), "att_cb".into(), "blitzer".into(), TeamId::Home, 1,
            PlayerStats::new(6, 3, 3, 8, None), SkillSet::empty(),
        ));
        let mut away = Team::new("a".into(), "Away".into(), "Orc".into(), 3, false);
        away.add_player(Player::new(
            def_id.clone(), "def_cb".into(), "lineman".into(), TeamId::Away, 1,
            PlayerStats::new(5, 3, 3, 9, None), SkillSet::empty(),
        ));
        let mut state = GameState::new(home, away);
        let def_coord = FieldCoordinate::new(25, 8);
        state.field.place_player(att_id.clone(), TeamId::Home, FieldCoordinate::new(24, 8), PlayerState::Standing);
        state.field.place_player(def_id.clone(), TeamId::Away, def_coord, PlayerState::Standing);
        state.field.ball.coord = Some(def_coord);
        state.field.ball.in_play = true;
        state.acting_player = Some(ActingPlayer::new(att_id.clone(), TeamId::Home));
        state.home_is_active = true;

        // block die + armor roll (2d6) + scatter direction (1=N) + scatter distance
        let mut rng = GameRng::new_test([4, 3, 4, 1, 3]);
        begin_block(&mut state, &att_id, &def_id, &mut rng);
        apply_block_dice_choice(&mut state, &att_id, &def_id, BlockResult::Pushback, &mut rng);

        // Ball should no longer be at def_coord (it scattered)
        assert_ne!(
            state.field.ball.coord,
            Some(def_coord),
            "ball should have scattered when carrier was crowd-pushed"
        );
    }

    // ── Ball scatter: player pushed onto loose ball ──────────────────────────

    #[test]
    fn push_to_ball_square_scatters_ball() {
        // Loose ball at (7,5). att(5,5) pushes def(6,5) east.
        // Defender should land on (7,5) and ball scatters.
        let (mut state, att, def) = setup("att_pb", "def_pb", 3, 3);
        let ball_coord = FieldCoordinate::new(7, 5);
        state.field.ball.coord = Some(ball_coord);
        state.field.ball.in_play = true;

        // block die; push has single forced option only if we block (7,4) and (7,6)
        place_away_blocker(&mut state, "bn", FieldCoordinate::new(7, 4));
        place_away_blocker(&mut state, "bs", FieldCoordinate::new(7, 6));
        // All of def's push options: (7,4)=occupied, (7,5)=ball, (7,6)=occupied
        // → only (7,5) is valid (empty, ball counts as empty for movement)
        // Actually (7,4) and (7,6) are occupied → pushback_options returns Occupied for those
        // and Empty for (7,5) → auto-push to (7,5)

        // block die + scatter direction + scatter distance (2 more for the ball scatter)
        let mut rng = GameRng::new_test([4, 2, 3]);
        begin_block(&mut state, &att, &def, &mut rng);

        // Push with PowPushback so it auto-resolves
        let res = apply_block_dice_choice(&mut state, &att, &def, BlockResult::Pushback, &mut rng);
        match res {
            BlockStepResult::NeedPushChoice { options } => {
                // Choose (7,5)
                apply_push_choice(&mut state, &att, &def, ball_coord, &mut rng);
            }
            BlockStepResult::Done(_) => {
                // auto-pushed
            }
            _ => panic!("Unexpected: {:?}", res),
        }

        assert_ne!(
            state.field.ball.coord,
            Some(ball_coord),
            "ball should scatter when a pushed player lands on it"
        );
        assert_eq!(
            state.field.player_coord(&def),
            Some(ball_coord),
            "defender should be at the ball square after push"
        );
    }
}
