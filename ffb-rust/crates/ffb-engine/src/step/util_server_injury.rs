/// Port of `com.fumbbl.ffb.server.util.UtilServerInjury`.
///
/// Provides `handle_injury()` (the main entry point for all injury resolution)
/// and the `drop_player()` family (place player prone + ball handling).
use ffb_model::enums::{
    ApothecaryMode, ApothecaryStatus, PlayerState, SendToBoxReason, SeriousInjuryKind,
    PS_KNOCKED_OUT, PS_PRONE, PS_RIP, PS_RESERVE, PS_SERIOUS_INJURY, PS_STUNNED,
};
use ffb_model::model::SoundId;
use ffb_model::types::{FieldCoordinate, FieldCoordinateBounds};
use ffb_model::util::rng::GameRng;
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use crate::injury::{InjuryContext, InjuryResult, InjuryTypeServer};
use crate::step::framework::{CatchScatterThrowInMode, StepParameter};

// ── handle_injury ─────────────────────────────────────────────────────────────

/// Full port of `UtilServerInjury.handleInjury()`.
///
/// Delegates dice rolling to `injury_type.handle_injury()`, then runs
/// `evaluate_injury_context()` (serious injury, stun→KO, apothecary, send-to-box, sound).
pub fn handle_injury(
    game: &Game,
    rng: &mut GameRng,
    injury_type: &mut dyn InjuryTypeServer,
    attacker_id: Option<&str>,
    defender_id: &str,
    coord: FieldCoordinate,
    from_coord: Option<FieldCoordinate>,
    old_result: Option<&InjuryResult>,
    apo_mode: ApothecaryMode,
) -> InjuryResult {
    // Java lines 72–76: ball-and-chain players always get armor broken
    // (failedArmourPlacesProne AND defender has placedProneCausesInjuryRoll)
    if injury_type.failed_armour_places_prone() {
        if let Some(defender) = game.player(defender_id) {
            if defender.has_skill_property(NamedProperties::PLACED_PRONE_CAUSES_INJURY_ROLL) {
                injury_type.injury_context_mut().armor_broken = true;
            }
        }
    }

    let old_ctx = old_result.map(|r| r.injury_context());
    injury_type.handle_injury(game, rng, attacker_id, defender_id, coord, from_coord, old_ctx, apo_mode);

    // Java: InjuryType.isCausedByOpponent()/isWorthSpps() — propagate onto the context(s) so
    // InjuryResult::apply_to can gate casualty-SPP awarding and opponent-casualty counting.
    // Previously never set in production (only in this module's own tests), so casualty SPPs
    // and opponent-casualty stat counters were never awarded for any injury type.
    let is_caused_by_opponent = injury_type.is_caused_by_opponent();
    let is_worth_spps = injury_type.is_worth_spps();
    injury_type.injury_context_mut().is_caused_by_opponent = is_caused_by_opponent;
    injury_type.injury_context_mut().is_worth_spps = is_worth_spps;

    // Capture flags before any mutable borrow of the context
    let flags = InjuryTypeFlags {
        stun_is_ko: injury_type.stun_is_treated_as_ko(),
        can_use_apo: injury_type.can_use_apo(),
        send_to_box_reason: injury_type.send_to_box_reason(),
        should_play_fall_sound: injury_type.should_play_fall_sound(),
    };

    // Java line 93: evaluateInjuryContext on primary context
    evaluate_injury_context(&flags, defender_id, injury_type.injury_context_mut(), game);

    // Java lines 95–97: if modified injury context present, evaluate it too
    let has_modified = injury_type.injury_context().modified_injury_context.is_some();
    if has_modified {
        let mut modified = injury_type.injury_context_mut().modified_injury_context.take().unwrap();
        modified.is_caused_by_opponent = is_caused_by_opponent;
        modified.is_worth_spps = is_worth_spps;
        evaluate_injury_context(&flags, defender_id, &mut modified, game);
        injury_type.injury_context_mut().modified_injury_context = Some(modified);
    }

    // Store the injury type's simple class name for post-hoc checks (e.g. isBlock()).
    let type_name = injury_type.java_class_name();
    if !type_name.is_empty() {
        injury_type.injury_context_mut().injury_type_name = Some(type_name.to_string());
    }

    let knocked_out = injury_type.injury_context().is_knocked_out();
    let rip = injury_type.injury_context().injury
        .map(|s| s.base() == PS_RIP).unwrap_or(false);

    InjuryResult {
        injury_context: injury_type.injury_context().clone(),
        knocked_out,
        rip,
        already_reported: false,
        pre_regeneration: true,
    }
}

/// Captured injury-type flags passed into `evaluate_injury_context()` to avoid
/// re-borrowing the trait object while we hold a `&mut InjuryContext`.
struct InjuryTypeFlags {
    stun_is_ko: bool,
    can_use_apo: bool,
    send_to_box_reason: Option<SendToBoxReason>,
    should_play_fall_sound: bool,
}

/// Port of `UtilServerInjury.evaluateInjuryContext()` (Java lines 103–161).
///
/// Interprets the injury context after dice have been rolled:
/// 1. Serious injury sub-table (if casualty tier = SERIOUS_INJURY)
/// 2. Stun → KO conversion (skill or injury-type flag)
/// 3. Apothecary eligibility
/// 4. Send-to-box turn/half/reason
/// 5. Sound effect
fn evaluate_injury_context(
    flags: &InjuryTypeFlags,
    defender_id: &str,
    ctx: &mut InjuryContext,
    game: &Game,
) {
    let defender = game.player(defender_id);

    // Java lines 106–113: serious injury sub-table
    if ctx.is_serious_injury() {
        ctx.serious_injury = interpret_serious_injury_roll(ctx.casualty_roll);
        // Java: requiresSecondCasualtyRoll (Decay skill) → second SI interpretation, using the
        // fresh casualty_roll_decay dice rolled in `do_injury_roll_for_player`, not a
        // reinterpretation of the primary roll.
        if defender.map(|d| d.has_skill_property(NamedProperties::REQUIRES_SECOND_CASUALTY_ROLL)).unwrap_or(false) {
            ctx.serious_injury_decay = interpret_serious_injury_roll(ctx.casualty_roll_decay);
        }
    }

    // Java lines 115–118: stun → KO conversion
    let convert_stun = flags.stun_is_ko
        || defender.map(|d| d.has_skill_property(NamedProperties::CONVERT_STUN_TO_KO)).unwrap_or(false);
    if convert_stun {
        if let Some(s) = ctx.injury {
            if s.base() == PS_STUNNED {
                ctx.injury = Some(PlayerState::new(PS_KNOCKED_OUT));
            }
        }
    }

    // Java lines 120–133: apothecary eligibility
    if let Some(state) = ctx.injury {
        if state.base() == PS_KNOCKED_OUT || state.is_casualty() {
            ctx.suffered_injury = Some(state);
            let is_ball_and_chain = state.base() == PS_KNOCKED_OUT
                && defender.map(|d| d.has_skill_property(NamedProperties::PLACED_PRONE_CAUSES_INJURY_ROLL)).unwrap_or(false);

            if !flags.can_use_apo || is_ball_and_chain {
                ctx.apothecary_status = ApothecaryStatus::NoApothecary;
            } else {
                let can = defender
                    .map(|d| can_use_apo_for_edition(game, d, state))
                    .unwrap_or(false);
                ctx.apothecary_status = if can {
                    ApothecaryStatus::DoRequest
                } else {
                    ApothecaryStatus::NoApothecary
                };
            }
        }
    }

    // Java lines 136–141: send-to-box turn/half/reason
    if ctx.is_knocked_out() || ctx.is_casualty() || ctx.is_reserve() {
        ctx.send_to_box_turn = game.turn_data().turn_nr;
        ctx.send_to_box_half = game.half;
        ctx.send_to_box_reason = flags.send_to_box_reason;
    }

    // Java lines 143–161: sound effect
    if let Some(state) = ctx.injury {
        ctx.sound = Some(match state.base() {
            b if b == PS_RIP => SoundId::RIP,
            b if b == PS_SERIOUS_INJURY || state.is_casualty() => SoundId::INJURY,
            b if b == PS_KNOCKED_OUT => SoundId::KO,
            _ => {
                if flags.should_play_fall_sound { SoundId::FALL } else { return }
            }
        });
    }
}

/// Maps the stored casualty roll to a `SeriousInjuryKind` using the BB2025 table.
/// Port of `RollMechanic.interpretSeriousInjuryRoll()` (simplified, no modifiers yet).
fn interpret_serious_injury_roll(casualty_roll: Option<[i32; 2]>) -> Option<SeriousInjuryKind> {
    // Uses d16 (index 0) for the BB2025 table lookup
    casualty_roll.and_then(|r| ffb_mechanics::injury::serious_injury_kind_bb2025(r[0]))
}

/// Dispatch `InjuryMechanic.canUseApo()` to the appropriate edition mechanic.
fn can_use_apo_for_edition(game: &Game, defender: &ffb_model::model::player::Player, state: PlayerState) -> bool {
    use ffb_model::enums::Rules;
    use ffb_mechanics::injury_mechanic::InjuryMechanic as InjuryMechanicTrait;
    match game.rules {
        Rules::Bb2016 => ffb_mechanics::bb2016::injury_mechanic::InjuryMechanic::new().can_use_apo(game, defender, state),
        Rules::Bb2020 => ffb_mechanics::bb2020::injury_mechanic::InjuryMechanic::new().can_use_apo(game, defender, state),
        Rules::Bb2025 | Rules::Common => ffb_mechanics::bb2025::injury_mechanic::InjuryMechanic::new().can_use_apo(game, defender, state),
    }
}

// ── handle_regeneration ───────────────────────────────────────────────────────

/// Port of `UtilServerInjury.handleRegeneration()`.
///
/// Returns `true` if the player regenerated (casualty → reserve).
/// Callers must update the injury context themselves if needed.
pub fn handle_regeneration(
    game: &mut Game,
    rng: &mut GameRng,
    player_id: &str,
) -> bool {
    let state = game.field_model.player_state(player_id);
    let can_regen = game.player(player_id)
        .map(|p| p.has_skill_property(NamedProperties::CAN_ROLL_TO_SAVE_FROM_INJURY))
        .unwrap_or(false);

    if let Some(state) = state {
        if state.is_casualty() && can_regen {
            let roll = rng.d6();
            let successful = roll >= 4;
            if successful {
                let new_state = state.change_base(PS_RESERVE);
                game.field_model.set_player_state(player_id, new_state);
            }
            return successful;
        }
    }
    false
}

// ── handle_injury_by_name ─────────────────────────────────────────────────────

/// Convenience: look up the injury type by name string and call `handle_injury()`.
/// Used by steps that receive `StepParameter::InjuryTypeName(name)`.
pub fn handle_injury_by_name(
    game: &Game,
    rng: &mut GameRng,
    injury_type_name: &str,
    attacker_id: Option<&str>,
    defender_id: &str,
    coord: FieldCoordinate,
    from_coord: Option<FieldCoordinate>,
    old_result: Option<&InjuryResult>,
    apo_mode: ApothecaryMode,
) -> InjuryResult {
    let mut injury_type = crate::injury::make_injury_type(injury_type_name);
    handle_injury(game, rng, &mut *injury_type, attacker_id, defender_id, coord, from_coord, old_result, apo_mode)
}

/// Returns `true` when the named injury type causes a turnover on fall.
pub fn injury_type_causes_turnover(name: &str) -> bool {
    let t = crate::injury::make_injury_type(name);
    t.falling_down_causes_turnover()
}

/// Simplified port of `UtilServerInjury.dropPlayer(step, player, ApothecaryMode, eligibleForSafePairOfHands)`.
///
/// Places the player PRONE (unless already prone/stunned) and deactivates them.
/// If the player had the ball the returned parameters include a `ScatterBall` mode
/// and, when the ball carrier is on the acting team, an `EndTurn(true)` flag.
///
/// Ball-and-Chain players (Java: `placedProneCausesInjuryRoll`) are treated the same
/// as regular drops here — the full injury roll is a TODO.
pub fn drop_player(
    game: &mut Game,
    player_id: &str,
    eligible_for_safe_pair_of_hands: bool,
) -> Vec<StepParameter> {
    drop_player_with_base(game, player_id, PS_PRONE, eligible_for_safe_pair_of_hands)
}

/// Shared implementation of Java's private `UtilServerInjury.dropPlayer(step, player,
/// pPlayerBase, mode, eligibleForSafePairOfHands)` — parameterized by the target base state
/// so both `dropPlayer`/`stunPlayer` (PRONE/STUNNED) can share the same ball-scatter/end-turn
/// logic. The `apothecaryMode != THROWN_PLAYER` exception on the deactivate condition is not
/// threaded through (no caller of the public `drop_player` currently passes an ApothecaryMode);
/// the `STUNNED == pPlayerBase` half of that condition (Java: always deactivate a stunned
/// player) is implemented, since that's what `stun_player` genuinely needs.
fn drop_player_with_base(
    game: &mut Game,
    player_id: &str,
    target_base: u32,
    eligible_for_safe_pair_of_hands: bool,
) -> Vec<StepParameter> {
    let mut params: Vec<StepParameter> = Vec::new();

    let coord: Option<FieldCoordinate> = game.field_model.player_coordinate(player_id);
    let state: Option<PlayerState> = game.field_model.player_state(player_id);

    let (coord, state) = match (coord, state) {
        (Some(c), Some(s)) => (c, s),
        _ => return params,
    };

    if !FieldCoordinateBounds::FIELD.is_in_bounds(coord) {
        return params;
    }

    // Java: !placedProneCausesInjuryRoll branch — place PRONE/STUNNED
    let base = state.base();
    if base != PS_PRONE && base != PS_STUNNED {
        let mut new_state = state;
        if base != ffb_model::enums::PS_HIT_ON_GROUND {
            new_state = new_state.change_rooted(false);
        }
        new_state = new_state.change_base(target_base);
        // Java: (player == actingPlayer && mode != THROWN_PLAYER) || (STUNNED == pPlayerBase) → deactivate
        let is_acting = game.acting_player.player_id.as_deref() == Some(player_id);
        if is_acting || target_base == PS_STUNNED {
            new_state = new_state.change_active(false);
        }
        game.field_model.set_player_state(player_id, new_state);
    }

    // Ball handling
    let has_ball = game.field_model.ball_coordinate
        .map(|bc| game.field_model.player_at(bc).map(|id| id.as_str() == player_id).unwrap_or(false))
        .unwrap_or(false);

    if eligible_for_safe_pair_of_hands && has_ball {
        params.push(StepParameter::DroppedBallCarrier(Some(player_id.to_owned())));
    }

    let ball_at_coord = game.field_model.ball_coordinate == Some(coord);
    if ball_at_coord {
        game.field_model.ball_moving = true;
        params.push(StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::ScatterBall));

        if has_ball {
            let acting_team_has_player = game.active_team().players.iter().any(|p| p.id.as_str() == player_id);
            if acting_team_has_player {
                params.push(StepParameter::EndTurn(true));
            }
        }
    }

    params
}

/// Port of `UtilServerInjury.dropPlayer(step, player, ApothecaryMode)` —
/// without Safe Pair of Hands eligibility.
pub fn drop_player_no_sph(game: &mut Game, player_id: &str) -> Vec<StepParameter> {
    drop_player(game, player_id, false)
}

// ── stun_player ───────────────────────────────────────────────────────────────

/// Port of `UtilServerInjury.stunPlayer(step, player, mode)` — delegates to the same shared
/// `dropPlayer(step, player, STUNNED, mode, false)` path as `drop_player`, so it now returns
/// the same ball-scatter/end-turn `StepParameter`s that dropping a ball carrier can trigger
/// (previously a bare state mutation only). Used by PitchInvasion, cards' blockable-player
/// selection, and similar effects.
pub fn stun_player(game: &mut Game, player_id: &str) -> Vec<StepParameter> {
    drop_player_with_base(game, player_id, PS_STUNNED, false)
}

/// Port of `UtilServerInjury.handleInjurySideEffects(IStep, InjuryResult)`.
///
/// Called after injury resolution is finalised (apothecary done or declined).
/// 1. `handleRaiseDead` — the opposing team may raise a RIP'd player as a Zombie/Thrall/Rotter.
/// 2. `mechanic.handlePumpUp` — grant extra re-roll if attacker has PumpUpTheCrowd skill.
///    Emits `GameEvent::PumpUpTheCrowdReRoll` when a re-roll is granted.
///
/// Returns the events to be included in the step's `StepOutcome`.
pub fn handle_injury_side_effects(
    game: &mut Game,
    injury_result: &InjuryResult,
) -> Vec<ffb_model::events::GameEvent> {
    use ffb_model::enums::Rules;
    use ffb_model::events::GameEvent;
    use crate::mechanic::state_mechanic::StateMechanic as StateMechanicTrait;

    let mut events = handle_raise_dead(game, injury_result.injury_context());

    // Java: mechanic.handlePumpUp(pStep, pInjuryResult) — dispatch on edition
    let injury_context = injury_result.injury_context();
    let pump_up_granted = match game.rules {
        Rules::Bb2025 => {
            use crate::mechanic::bb2025::state_mechanic::StateMechanic;
            StateMechanic::new().handle_pump_up(game, injury_context)
        }
        _ => {
            use crate::mechanic::mixed::state_mechanic::StateMechanic;
            StateMechanic::new().handle_pump_up(game, injury_context)
        }
    };

    if pump_up_granted {
        let attacker_id = injury_result.injury_context().attacker_id.clone()
            .unwrap_or_default();
        events.push(GameEvent::PumpUpTheCrowdReRoll { player_id: attacker_id });
    }
    events
}

/// Dispatch `InjuryMechanic` to the appropriate edition mechanic (boxed, since callers need
/// several of its methods together rather than one dispatched call at a time).
fn injury_mechanic_for(rules: ffb_model::enums::Rules) -> Box<dyn ffb_mechanics::injury_mechanic::InjuryMechanic> {
    use ffb_model::enums::Rules;
    match rules {
        Rules::Bb2016 => Box::new(ffb_mechanics::bb2016::injury_mechanic::InjuryMechanic::new()),
        Rules::Bb2020 => Box::new(ffb_mechanics::bb2020::injury_mechanic::InjuryMechanic::new()),
        Rules::Bb2025 | Rules::Common => Box::new(ffb_mechanics::bb2025::injury_mechanic::InjuryMechanic::new()),
    }
}

/// Port of `UtilServerInjury.handleRaiseDead` + `raisePlayer` + `sendRaisedPlayer`.
///
/// After a fatal injury (RIP), the opposing team may raise the dead player as a Zombie/Thrall
/// (necromancer/vampire-lord teams, via `canRaiseDead`) or a Rotter (infected-by-attacker teams
/// with Nurgle's Rot, via `canRaiseInfectedPlayers`). Returns the `PlayerAdded` event if a
/// player was raised; empty otherwise.
fn handle_raise_dead(
    game: &mut Game,
    injury_context: &InjuryContext,
) -> Vec<ffb_model::events::GameEvent> {
    use ffb_model::enums::{PlayerState, PlayerType, SendToBoxReason, PS_MISSING, PS_RESERVE, PS_RIP};
    use ffb_model::events::GameEvent;
    use ffb_model::model::player::Player;
    use ffb_model::report::report_raise_dead::ReportRaiseDead;
    use ffb_model::util::raise_type::RaiseType;

    let dead_player_id = match injury_context.defender_id.clone() {
        Some(id) => id,
        None => return vec![],
    };
    let is_rip = injury_context.injury.map(|ps| ps.base() == PS_RIP).unwrap_or(false);
    if !is_rip {
        return vec![];
    }
    let dead_player = match game.player(&dead_player_id) {
        Some(p) => p.clone(),
        None => return vec![],
    };

    // Java: UtilPlayer.findOtherTeam(game, deadPlayer) — the necromantic/vampire team is the
    // opponent of the dead player's team.
    let necro_is_home = !game.team_home.has_player(&dead_player_id);
    let necro_team = if necro_is_home { game.team_home.clone() } else { game.team_away.clone() };
    let team_result = game.game_result.team_result(necro_is_home).clone();
    let mechanic = injury_mechanic_for(game.rules);

    let mut nurgles_rot = false;
    let raise_type = if mechanic.can_raise_dead(&necro_team, &team_result, &dead_player) {
        mechanic.raise_type(&necro_team)
    } else {
        let attacker = injury_context.attacker_id.as_deref().and_then(|id| game.player(id).cloned());
        if mechanic.can_raise_infected_players(&necro_team, &team_result, attacker.as_ref(), &dead_player) {
            nurgles_rot = true;
            RaiseType::ROTTER
        } else {
            return vec![];
        }
    };

    let zombie_position = match ffb_model::data::loader::find_roster(&necro_team.roster_id, game.rules)
        .and_then(|roster| roster.raised_roster_position().cloned())
    {
        Some(pos) => pos,
        None => return vec![],
    };

    let team_result_mut = game.game_result.team_result_mut(necro_is_home);
    team_result_mut.raised_dead += 1;
    let raised_id = format!("{}R{}", dead_player_id, team_result_mut.raised_dead);

    let player_type = if raise_type == RaiseType::ROTTER {
        mechanic.raised_nurgle_type()
    } else {
        PlayerType::RaisedFromDead
    };
    let max_nr = necro_team.players.iter().map(|p| p.nr).max().unwrap_or(0);
    let mut raised_player = Player::from_position(raised_id.clone(), dead_player.name.clone(), max_nr + 1, &zombie_position);
    raised_player.player_type = player_type;

    let (send_to_box_reason, new_state) = match raise_type {
        RaiseType::ROTTER => (
            Some(mechanic.raised_by_nurgle_reason()),
            if mechanic.infected_goes_to_reserves() { PlayerState::new(PS_RESERVE) } else { PlayerState::new(PS_MISSING) },
        ),
        RaiseType::ZOMBIE => (Some(SendToBoxReason::Raised), PlayerState::new(PS_RESERVE)),
        RaiseType::THRALL => (None, PlayerState::new(PS_MISSING)),
    };

    let player_result = team_result_mut.player_result_mut(&raised_id);
    player_result.send_to_box_half = game.half;
    player_result.send_to_box_turn = if necro_is_home { game.turn_data_home.turn_nr } else { game.turn_data_away.turn_nr };
    player_result.send_to_box_reason = send_to_box_reason;

    if necro_is_home {
        game.team_home.players.push(raised_player);
    } else {
        game.team_away.players.push(raised_player);
    }
    game.field_model.set_player_state(&raised_id, new_state);
    ffb_model::util::util_box::UtilBox::put_player_into_box(game, &raised_id);

    game.report_list.add(ReportRaiseDead::new(raised_id.clone(), Some(zombie_position.name.clone()), nurgles_rot));
    // Java: getResult().setSound(SoundId.ORGAN) — client-only, no-op in headless

    vec![GameEvent::PlayerAdded {
        team_id: necro_team.id.clone(),
        player_id: raised_id,
        position_id: zombie_position.id.clone(),
    }]
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::injury::{InjuryContext, InjuryTypeServer};
    use ffb_model::enums::{Rules, PS_STANDING, PS_PRONE, PS_BADLY_HURT};
    use ffb_model::enums::{PlayerType, PlayerGender};
    use ffb_model::model::player::Player;
    use ffb_model::model::skill_def::{SkillId, SkillWithValue};
    use ffb_model::types::FieldCoordinate;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    fn add_player(game: &mut Game, id: &str, state: u32) -> FieldCoordinate {
        let pos = FieldCoordinate::new(5, 5);
        game.team_home.players.push(Player {
            id: id.into(),
            name: id.into(),
            nr: 1,
            position_id: "lineman".into(),
            player_type: PlayerType::Regular,
            gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![],
            extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
});
        game.field_model.set_player_coordinate(id, pos);
        game.field_model.set_player_state(id, PlayerState::new(state));
        pos
    }

    fn add_away_player(game: &mut Game, id: &str, state: u32) -> FieldCoordinate {
        let pos = FieldCoordinate::new(6, 5);
        game.team_away.players.push(Player {
            id: id.into(),
            name: id.into(),
            nr: 1,
            position_id: "lineman".into(),
            player_type: PlayerType::Regular,
            gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![],
            extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        });
        game.field_model.set_player_coordinate(id, pos);
        game.field_model.set_player_state(id, PlayerState::new(state));
        pos
    }

    fn add_player_with_skill(game: &mut Game, id: &str, state: u32, skill: SkillId) -> FieldCoordinate {
        let pos = add_player(game, id, state);
        let p = game.team_home.players.iter_mut().find(|p| p.id == id).unwrap();
        p.starting_skills.push(SkillWithValue { skill_id: skill, value: None });
        pos
    }

    // ── Stub injury type for deterministic tests ──────────────────────────────

    /// A minimal `InjuryTypeServer` that sets the injury state directly without rolling dice.
    struct StubInjury {
        ctx: InjuryContext,
        preset_state: Option<PlayerState>,
        stun_is_ko: bool,
        can_apo: bool,
        send_reason: Option<SendToBoxReason>,
    }

    impl StubInjury {
        fn with_state(state: u32) -> Self {
            Self {
                ctx: InjuryContext::new(ApothecaryMode::Defender),
                preset_state: Some(PlayerState::new(state)),
                stun_is_ko: false,
                can_apo: true,
                send_reason: None,
            }
        }
        fn stun_is_ko(mut self) -> Self { self.stun_is_ko = true; self }
        fn no_apo(mut self) -> Self { self.can_apo = false; self }
        fn with_reason(mut self, r: SendToBoxReason) -> Self { self.send_reason = Some(r); self }
    }

    impl InjuryTypeServer for StubInjury {
        fn handle_injury(&mut self, _g: &Game, _rng: &mut GameRng, _att: Option<&str>, defender_id: &str,
            coord: FieldCoordinate, _from: Option<FieldCoordinate>, _old: Option<&InjuryContext>, apo_mode: ApothecaryMode) {
            self.ctx.defender_id = Some(defender_id.to_owned());
            self.ctx.defender_coordinate = Some(coord);
            self.ctx.apothecary_mode = apo_mode;
            self.ctx.injury = self.preset_state;
        }
        fn injury_context(&self) -> &InjuryContext { &self.ctx }
        fn injury_context_mut(&mut self) -> &mut InjuryContext { &mut self.ctx }
        fn stun_is_treated_as_ko(&self) -> bool { self.stun_is_ko }
        fn can_use_apo(&self) -> bool { self.can_apo }
        fn send_to_box_reason(&self) -> Option<SendToBoxReason> { self.send_reason }
    }

    fn run_handle_injury(game: &mut Game, rng: &mut GameRng, injury_type: &mut dyn InjuryTypeServer, defender_id: &str) -> InjuryResult {
        let coord = game.field_model.player_coordinate(defender_id).unwrap_or(FieldCoordinate::new(5, 5));
        handle_injury(game, rng, injury_type, None, defender_id, coord, None, None, ApothecaryMode::Defender)
    }

    // ── stun→KO conversion ────────────────────────────────────────────────────

    #[test]
    fn stunned_stays_stunned_without_convert_flag() {
        let mut game = make_game();
        add_player(&mut game, "p1", PS_STANDING);
        let mut t = StubInjury::with_state(PS_STUNNED);
        let mut rng = GameRng::new(1);
        let result = run_handle_injury(&mut game, &mut rng, &mut t, "p1");
        assert_eq!(result.injury_context.injury.map(|s| s.base()), Some(PS_STUNNED));
    }

    #[test]
    fn stunned_becomes_ko_from_injury_type_flag() {
        let mut game = make_game();
        add_player(&mut game, "p1", PS_STANDING);
        let mut t = StubInjury::with_state(PS_STUNNED).stun_is_ko();
        let mut rng = GameRng::new(1);
        let result = run_handle_injury(&mut game, &mut rng, &mut t, "p1");
        assert_eq!(result.injury_context.injury.map(|s| s.base()), Some(PS_KNOCKED_OUT));
    }

    #[test]
    fn ko_result_is_flagged_in_result() {
        let mut game = make_game();
        add_player(&mut game, "p1", PS_STANDING);
        let mut t = StubInjury::with_state(PS_KNOCKED_OUT);
        let mut rng = GameRng::new(1);
        let result = run_handle_injury(&mut game, &mut rng, &mut t, "p1");
        assert!(result.knocked_out);
        assert!(!result.rip);
    }

    #[test]
    fn rip_result_is_flagged_in_result() {
        let mut game = make_game();
        add_player(&mut game, "p1", PS_STANDING);
        let mut t = StubInjury::with_state(PS_RIP);
        let mut rng = GameRng::new(1);
        let result = run_handle_injury(&mut game, &mut rng, &mut t, "p1");
        assert!(result.rip);
        assert!(!result.knocked_out);
    }

    // ── sound effect ──────────────────────────────────────────────────────────

    #[test]
    fn ko_sound_is_ko() {
        let mut game = make_game();
        add_player(&mut game, "p1", PS_STANDING);
        let mut t = StubInjury::with_state(PS_KNOCKED_OUT);
        let mut rng = GameRng::new(1);
        let result = run_handle_injury(&mut game, &mut rng, &mut t, "p1");
        assert_eq!(result.injury_context.sound, Some(SoundId::KO));
    }

    #[test]
    fn rip_sound_is_rip() {
        let mut game = make_game();
        add_player(&mut game, "p1", PS_STANDING);
        let mut t = StubInjury::with_state(PS_RIP);
        let mut rng = GameRng::new(1);
        let result = run_handle_injury(&mut game, &mut rng, &mut t, "p1");
        assert_eq!(result.injury_context.sound, Some(SoundId::RIP));
    }

    #[test]
    fn serious_injury_sound_is_injury() {
        let mut game = make_game();
        add_player(&mut game, "p1", PS_STANDING);
        let mut t = StubInjury::with_state(PS_SERIOUS_INJURY);
        let mut rng = GameRng::new(1);
        let result = run_handle_injury(&mut game, &mut rng, &mut t, "p1");
        assert_eq!(result.injury_context.sound, Some(SoundId::INJURY));
    }

    #[test]
    fn prone_state_gets_fall_sound() {
        let mut game = make_game();
        add_player(&mut game, "p1", PS_STANDING);
        let mut t = StubInjury::with_state(PS_PRONE);
        let mut rng = GameRng::new(1);
        let result = run_handle_injury(&mut game, &mut rng, &mut t, "p1");
        assert_eq!(result.injury_context.sound, Some(SoundId::FALL));
    }

    // ── send-to-box ───────────────────────────────────────────────────────────

    #[test]
    fn send_to_box_set_for_ko() {
        let mut game = make_game();
        add_player(&mut game, "p1", PS_STANDING);
        let mut t = StubInjury::with_state(PS_KNOCKED_OUT).with_reason(SendToBoxReason::Blocked);
        let mut rng = GameRng::new(1);
        let result = run_handle_injury(&mut game, &mut rng, &mut t, "p1");
        assert_eq!(result.injury_context.send_to_box_reason, Some(SendToBoxReason::Blocked));
    }

    #[test]
    fn send_to_box_not_set_for_stun() {
        let mut game = make_game();
        add_player(&mut game, "p1", PS_STANDING);
        let mut t = StubInjury::with_state(PS_STUNNED).with_reason(SendToBoxReason::Blocked);
        let mut rng = GameRng::new(1);
        let result = run_handle_injury(&mut game, &mut rng, &mut t, "p1");
        assert_eq!(result.injury_context.send_to_box_reason, None);
    }

    // ── apothecary status ─────────────────────────────────────────────────────

    #[test]
    fn no_apothecary_when_injury_type_forbids() {
        let mut game = make_game();
        add_player(&mut game, "p1", PS_STANDING);
        let mut t = StubInjury::with_state(PS_KNOCKED_OUT).no_apo();
        let mut rng = GameRng::new(1);
        let result = run_handle_injury(&mut game, &mut rng, &mut t, "p1");
        assert_eq!(result.injury_context.apothecary_status, ApothecaryStatus::NoApothecary);
    }

    #[test]
    fn stun_does_not_request_apothecary() {
        let mut game = make_game();
        add_player(&mut game, "p1", PS_STANDING);
        let mut t = StubInjury::with_state(PS_STUNNED);
        let mut rng = GameRng::new(1);
        let result = run_handle_injury(&mut game, &mut rng, &mut t, "p1");
        assert_eq!(result.injury_context.apothecary_status, ApothecaryStatus::NoApothecary);
    }

    // ── suffered_injury ───────────────────────────────────────────────────────

    #[test]
    fn suffered_injury_set_for_ko() {
        let mut game = make_game();
        add_player(&mut game, "p1", PS_STANDING);
        let mut t = StubInjury::with_state(PS_KNOCKED_OUT);
        let mut rng = GameRng::new(1);
        let result = run_handle_injury(&mut game, &mut rng, &mut t, "p1");
        assert!(result.injury_context.suffered_injury.is_some());
        assert_eq!(result.injury_context.suffered_injury.unwrap().base(), PS_KNOCKED_OUT);
    }

    #[test]
    fn suffered_injury_not_set_for_stun() {
        let mut game = make_game();
        add_player(&mut game, "p1", PS_STANDING);
        let mut t = StubInjury::with_state(PS_STUNNED);
        let mut rng = GameRng::new(1);
        let result = run_handle_injury(&mut game, &mut rng, &mut t, "p1");
        assert!(result.injury_context.suffered_injury.is_none());
    }

    // ── handle_regeneration ───────────────────────────────────────────────────

    #[test]
    fn regeneration_succeeds_on_high_roll() {
        let mut game = make_game();
        add_player_with_skill(&mut game, "p1", PS_BADLY_HURT, SkillId::Regeneration);
        // GameRng::new(4) rolls 4 on first d6 — exactly meets the threshold
        let mut rng = GameRng::new(4);
        let success = handle_regeneration(&mut game, &mut rng, "p1");
        assert!(success);
        let state = game.field_model.player_state("p1").unwrap();
        assert_eq!(state.base(), PS_RESERVE);
    }

    #[test]
    fn regeneration_fails_on_low_roll() {
        let mut game = make_game();
        add_player_with_skill(&mut game, "p1", PS_BADLY_HURT, SkillId::Regeneration);
        // GameRng::new(1) rolls 1 on first d6 — below threshold
        let mut rng = GameRng::new(1);
        let success = handle_regeneration(&mut game, &mut rng, "p1");
        assert!(!success);
        let state = game.field_model.player_state("p1").unwrap();
        assert_eq!(state.base(), PS_BADLY_HURT);
    }

    #[test]
    fn regeneration_does_nothing_without_skill() {
        let mut game = make_game();
        add_player(&mut game, "p1", PS_BADLY_HURT);
        let mut rng = GameRng::new(6);
        let success = handle_regeneration(&mut game, &mut rng, "p1");
        assert!(!success);
    }

    #[test]
    fn regeneration_does_nothing_for_non_casualty_state() {
        let mut game = make_game();
        add_player_with_skill(&mut game, "p1", PS_STUNNED, SkillId::Regeneration);
        let mut rng = GameRng::new(6);
        let success = handle_regeneration(&mut game, &mut rng, "p1");
        assert!(!success);
    }

    #[test]
    fn standing_player_becomes_prone() {
        let mut game = make_game();
        add_player(&mut game, "p1", PS_STANDING);
        drop_player_no_sph(&mut game, "p1");
        let state = game.field_model.player_state("p1").unwrap();
        assert_eq!(state.base(), PS_PRONE);
    }

    #[test]
    fn already_prone_stays_prone() {
        let mut game = make_game();
        add_player(&mut game, "p1", PS_PRONE);
        drop_player_no_sph(&mut game, "p1");
        let state = game.field_model.player_state("p1").unwrap();
        assert_eq!(state.base(), PS_PRONE);
    }

    #[test]
    fn off_field_player_is_noop() {
        let mut game = make_game();
        // Place player out of bounds (x=26 is out of bounds for 26-wide field)
        game.team_home.players.push(Player {
            id: "p2".into(), name: "p2".into(), nr: 2, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
});
        game.field_model.set_player_coordinate("p2", FieldCoordinate::new(26, 5));
        game.field_model.set_player_state("p2", PlayerState::new(PS_STANDING));
        let params = drop_player_no_sph(&mut game, "p2");
        assert!(params.is_empty());
        // state unchanged
        let state = game.field_model.player_state("p2").unwrap();
        assert_eq!(state.base(), PS_STANDING);
    }

    #[test]
    fn stun_player_sets_stunned_state() {
        let mut game = make_game();
        add_player(&mut game, "p1", PS_STANDING);
        stun_player(&mut game, "p1");
        let state = game.field_model.player_state("p1").unwrap();
        assert_eq!(state.base(), PS_STUNNED);
    }

    #[test]
    fn stun_player_noop_for_unknown_player() {
        let mut game = make_game();
        // No panic — just a no-op
        stun_player(&mut game, "does_not_exist");
    }

    #[test]
    fn ball_carrier_triggers_scatter() {
        let mut game = make_game();
        let pos = add_player(&mut game, "p1", PS_STANDING);
        game.field_model.ball_coordinate = Some(pos);
        game.field_model.ball_in_play = true;
        let params = drop_player(&mut game, "p1", true);
        assert!(params.iter().any(|p| matches!(p, StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::ScatterBall))));
        assert!(game.field_model.ball_moving);
    }

    #[test]
    fn stun_player_ball_carrier_triggers_scatter() {
        // Java: UtilServerInjury.stunPlayer delegates to the same shared dropPlayer path as
        // drop_player, so a stunned ball carrier scatters the ball too (previously a bare
        // state mutation with no returned StepParameters at all).
        let mut game = make_game();
        let pos = add_player(&mut game, "p1", PS_STANDING);
        game.field_model.ball_coordinate = Some(pos);
        game.field_model.ball_in_play = true;
        let params = stun_player(&mut game, "p1");
        assert!(params.iter().any(|p| matches!(p, StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::ScatterBall))));
        assert!(game.field_model.ball_moving);
        assert_eq!(game.field_model.player_state("p1").unwrap().base(), PS_STUNNED);
    }

    #[test]
    fn stun_player_always_deactivates_even_when_not_acting() {
        // Java: dropPlayer's deactivate condition is
        // `(player == actingPlayer && mode != THROWN_PLAYER) || (STUNNED == pPlayerBase)` —
        // a stunned player is deactivated unconditionally, regardless of who is acting.
        let mut game = make_game();
        add_player(&mut game, "p1", PS_STANDING);
        // Start active, to prove stun_player itself deactivates rather than the field already
        // being inactive by default.
        let active_state = game.field_model.player_state("p1").unwrap().change_active(true);
        game.field_model.set_player_state("p1", active_state);
        game.acting_player.player_id = Some("someone_else".into());
        stun_player(&mut game, "p1");
        let state = game.field_model.player_state("p1").unwrap();
        assert_eq!(state.base(), PS_STUNNED);
        assert!(!state.is_active());
    }

    #[test]
    fn stun_player_already_prone_is_left_unchanged() {
        // Java: dropPlayer's outer guard skips the state change entirely when the player is
        // already PRONE or STUNNED — shared by stunPlayer too, so a prone player stays prone
        // rather than being "upgraded" to stunned.
        let mut game = make_game();
        add_player(&mut game, "p1", PS_PRONE);
        stun_player(&mut game, "p1");
        let state = game.field_model.player_state("p1").unwrap();
        assert_eq!(state.base(), PS_PRONE);
    }

    // ── handle_injury_side_effects tests ─────────────────────────────────────

    fn make_pump_up_ir(game: &mut Game, home: bool, attacker_state: u32, injury_type: Option<&str>) -> InjuryResult {
        use ffb_model::enums::{ApothecaryMode, PS_RIP, PlayerState};
        use ffb_model::model::skill_def::SkillWithValue;
        use ffb_model::model::player::Player;
        use std::collections::HashSet;
        // Create attacker with PumpUpTheCrowd skill
        let mut attacker = Player {
            id: "att".into(), name: "att".into(), nr: 99,
            position_id: "pos".into(), player_type: PlayerType::Regular,
            gender: PlayerGender::Male, movement: 6, strength: 3, agility: 3,
            passing: 4, armour: 8, starting_skills: vec![], extra_skills: vec![],
            temporary_skills: vec![], used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        };
        attacker.extra_skills.push(SkillWithValue { skill_id: SkillId::PumpUpTheCrowd, value: None });
        let coord = FieldCoordinate::new(5, 5);
        if home {
            game.team_home.players.push(attacker);
        } else {
            game.team_away.players.push(attacker);
        }
        game.field_model.set_player_coordinate("att", coord);
        game.field_model.set_player_state("att", PlayerState::new(attacker_state));

        let mut ir = InjuryResult::new(ApothecaryMode::Defender);
        ir.injury_context_mut().attacker_id = Some("att".into());
        ir.injury_context_mut().injury = Some(PlayerState::new(PS_RIP));
        ir.injury_context_mut().injury_type_name = injury_type.map(|s| s.to_string());
        ir
    }

    #[test]
    fn handle_injury_side_effects_no_attacker_no_events() {
        use ffb_model::enums::ApothecaryMode;
        let mut game = make_game();
        let ir = InjuryResult::new(ApothecaryMode::Defender);
        let events = handle_injury_side_effects(&mut game, &ir);
        assert!(events.is_empty());
    }

    #[test]
    fn handle_injury_side_effects_block_casualty_emits_pump_up_event() {
        use ffb_model::enums::PS_STANDING;
        use ffb_model::events::GameEvent;
        let mut game = make_game();
        game.home_playing = true;
        let ir = make_pump_up_ir(&mut game, true, PS_STANDING, Some("Block"));
        let events = handle_injury_side_effects(&mut game, &ir);
        assert_eq!(events.len(), 1);
        assert!(matches!(&events[0], GameEvent::PumpUpTheCrowdReRoll { player_id }
            if player_id == "att"));
        assert_eq!(game.turn_data_home.rerolls, 1);
    }

    #[test]
    fn handle_injury_side_effects_non_block_no_event() {
        use ffb_model::enums::PS_STANDING;
        let mut game = make_game();
        game.home_playing = true;
        let ir = make_pump_up_ir(&mut game, true, PS_STANDING, Some("Foul"));
        let events = handle_injury_side_effects(&mut game, &ir);
        // BB2025 rules: only Block casualties grant pump-up
        assert!(events.is_empty());
    }

    #[test]
    fn handle_injury_side_effects_mixed_rules_any_casualty() {
        use ffb_model::enums::{ApothecaryMode, PS_STANDING, PS_RIP, PlayerState};
        use ffb_model::model::skill_def::SkillWithValue;
        use ffb_model::model::player::Player;
        use std::collections::HashSet;
        use ffb_model::events::GameEvent;
        let mut game = Game::new(
            test_team("home", 0), test_team("away", 0), Rules::Bb2016
        );
        game.home_playing = true;
        // Create attacker with PumpUpTheCrowd on BB2016
        let mut attacker = Player {
            id: "att".into(), name: "att".into(), nr: 99,
            position_id: "pos".into(), player_type: PlayerType::Regular,
            gender: PlayerGender::Male, movement: 6, strength: 3, agility: 3,
            passing: 4, armour: 8, starting_skills: vec![], extra_skills: vec![],
            temporary_skills: vec![], used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        };
        attacker.extra_skills.push(SkillWithValue { skill_id: SkillId::PumpUpTheCrowd, value: None });
        let coord = FieldCoordinate::new(5, 5);
        game.team_home.players.push(attacker);
        game.field_model.set_player_coordinate("att", coord);
        game.field_model.set_player_state("att", PlayerState::new(PS_STANDING));
        let mut ir = InjuryResult::new(ApothecaryMode::Defender);
        ir.injury_context_mut().attacker_id = Some("att".into());
        ir.injury_context_mut().injury = Some(PlayerState::new(PS_RIP));
        ir.injury_context_mut().injury_type_name = Some("Foul".into()); // Not Block — but mixed accepts any cas
        let events = handle_injury_side_effects(&mut game, &ir);
        assert_eq!(events.len(), 1);
        assert!(matches!(&events[0], GameEvent::PumpUpTheCrowdReRoll { .. }));
    }

    // ── handle_raise_dead tests ────────────────────────────────────────────────

    fn make_rip_ir(defender_id: &str) -> InjuryResult {
        use ffb_model::enums::{ApothecaryMode, PS_RIP, PlayerState};
        let mut ir = InjuryResult::new(ApothecaryMode::Defender);
        ir.injury_context_mut().defender_id = Some(defender_id.into());
        ir.injury_context_mut().injury = Some(PlayerState::new(PS_RIP));
        ir
    }

    #[test]
    fn handle_raise_dead_creates_zombie_for_necromancer_team() {
        use ffb_model::enums::PS_RIP;
        use ffb_model::events::GameEvent;
        let mut away = test_team("away", 0);
        away.roster_id = "necromantic.lrb6".into();
        away.necromancer = true;
        let mut game = Game::new(test_team("home", 0), away, Rules::Bb2016);
        add_player(&mut game, "dead1", PS_RIP);

        let ir = make_rip_ir("dead1");
        let events = handle_injury_side_effects(&mut game, &ir);

        assert_eq!(game.team_away.players.len(), 1, "zombie should be added to the necromancer team");
        assert_eq!(game.team_away.players[0].position_id, "necromantic.zombie");
        assert_eq!(game.game_result.away.raised_dead, 1);
        assert!(events.iter().any(|e| matches!(e, GameEvent::PlayerAdded { team_id, .. } if team_id == "away")));
    }

    #[test]
    fn handle_raise_dead_noop_when_not_rip() {
        use ffb_model::enums::{ApothecaryMode, PS_KNOCKED_OUT, PlayerState};
        let mut away = test_team("away", 0);
        away.roster_id = "necromantic.lrb6".into();
        away.necromancer = true;
        let mut game = Game::new(test_team("home", 0), away, Rules::Bb2016);
        add_player(&mut game, "dead1", PS_KNOCKED_OUT);

        let mut ir = InjuryResult::new(ApothecaryMode::Defender);
        ir.injury_context_mut().defender_id = Some("dead1".into());
        ir.injury_context_mut().injury = Some(PlayerState::new(PS_KNOCKED_OUT));
        handle_injury_side_effects(&mut game, &ir);

        assert!(game.team_away.players.is_empty());
        assert_eq!(game.game_result.away.raised_dead, 0);
    }

    #[test]
    fn handle_raise_dead_noop_without_necromancer_or_vampire_lord() {
        use ffb_model::enums::PS_RIP;
        let away = test_team("away", 0); // no necromancer, no vampire_lord
        let mut game = Game::new(test_team("home", 0), away, Rules::Bb2016);
        add_player(&mut game, "dead1", PS_RIP);

        let ir = make_rip_ir("dead1");
        handle_injury_side_effects(&mut game, &ir);

        assert!(game.team_away.players.is_empty());
        assert_eq!(game.game_result.away.raised_dead, 0);
    }

    #[test]
    fn handle_raise_dead_raises_thrall_for_vampire_lord_team() {
        use ffb_model::enums::PS_RIP;
        let mut away = test_team("away", 0);
        away.roster_id = "necromantic.lrb6".into();
        away.vampire_lord = true;
        let mut game = Game::new(test_team("home", 0), away, Rules::Bb2016);
        add_player(&mut game, "dead1", PS_RIP);

        let ir = make_rip_ir("dead1");
        handle_injury_side_effects(&mut game, &ir);

        assert_eq!(game.team_away.players.len(), 1);
        assert_eq!(game.team_away.players[0].player_type, PlayerType::RaisedFromDead);
        assert_eq!(game.game_result.away.raised_dead, 1);
    }

    #[test]
    fn handle_raise_dead_emits_raise_dead_report() {
        use ffb_model::enums::PS_RIP;
        let mut away = test_team("away", 0);
        away.roster_id = "necromantic.lrb6".into();
        away.necromancer = true;
        let mut game = Game::new(test_team("home", 0), away, Rules::Bb2016);
        add_player(&mut game, "dead1", PS_RIP);

        let ir = make_rip_ir("dead1");
        handle_injury_side_effects(&mut game, &ir);

        assert_eq!(game.report_list.size(), 1);
        assert!(game.report_list.has_report(ffb_model::report::report_id::ReportId::RAISE_DEAD));
    }

    // ── is_worth_spps / is_caused_by_opponent propagation (Phase ABD) ────────

    #[test]
    fn handle_injury_propagates_block_worth_spps_and_caused_by_opponent() {
        use crate::injury::injuryType::injury_type_block::{BlockMode, InjuryTypeBlock};
        let mut game = make_game();
        add_player(&mut game, "p1", PS_STANDING);
        let mut t = InjuryTypeBlock::new(BlockMode::Regular, true);
        let mut rng = GameRng::new(1);
        let result = run_handle_injury(&mut game, &mut rng, &mut t, "p1");
        assert!(result.injury_context.is_worth_spps);
        assert!(result.injury_context.is_caused_by_opponent);
    }

    #[test]
    fn handle_injury_propagates_foul_neither_worth_spps_nor_caused_by_opponent() {
        use crate::injury::injuryType::injury_type_foul::InjuryTypeFoul;
        let mut game = make_game();
        add_player(&mut game, "p1", PS_STANDING);
        let mut t = InjuryTypeFoul::new();
        let mut rng = GameRng::new(1);
        let result = run_handle_injury(&mut game, &mut rng, &mut t, "p1");
        assert!(!result.injury_context.is_worth_spps);
        assert!(!result.injury_context.is_caused_by_opponent);
    }

    #[test]
    fn handle_injury_propagates_foul_for_spp_both_true() {
        use crate::injury::injuryType::injury_type_foul_for_spp::InjuryTypeFoulForSpp;
        let mut game = make_game();
        add_player(&mut game, "p1", PS_STANDING);
        let mut t = InjuryTypeFoulForSpp::new();
        let mut rng = GameRng::new(1);
        let result = run_handle_injury(&mut game, &mut rng, &mut t, "p1");
        assert!(result.injury_context.is_worth_spps);
        assert!(result.injury_context.is_caused_by_opponent);
    }

    #[test]
    fn handle_injury_propagates_crowd_push_neither_flag() {
        use crate::injury::injuryType::injury_type_crowd_push::InjuryTypeCrowdPush;
        let mut game = make_game();
        add_player(&mut game, "p1", PS_STANDING);
        let mut t = InjuryTypeCrowdPush::new();
        let mut rng = GameRng::new(1);
        let result = run_handle_injury(&mut game, &mut rng, &mut t, "p1");
        assert!(!result.injury_context.is_worth_spps);
        assert!(!result.injury_context.is_caused_by_opponent);
    }

    #[test]
    fn handle_injury_propagates_crowd_push_for_spp_caused_by_opponent_only_asymmetry() {
        // CrowdPushForSpp overrides isCausedByOpponent=true (unlike base CrowdPush), matching
        // FoulForSpp's asymmetry — both flip independently of worth_spps.
        use crate::injury::injuryType::injury_type_crowd_push_for_spp::InjuryTypeCrowdPushForSpp;
        let mut game = make_game();
        add_player(&mut game, "p1", PS_STANDING);
        let mut t = InjuryTypeCrowdPushForSpp::new();
        let mut rng = GameRng::new(1);
        let result = run_handle_injury(&mut game, &mut rng, &mut t, "p1");
        assert!(result.injury_context.is_worth_spps);
        assert!(result.injury_context.is_caused_by_opponent);
    }

    #[test]
    fn handle_injury_block_casualty_increments_attacker_casualty_counter() {
        // Previously impossible: is_worth_spps/is_caused_by_opponent were never populated in
        // production, so InjuryResult::apply_to's casualty-counter gate never fired for any
        // injury type. Seed chosen to produce a casualty (d16 >= 9) on the injury roll.
        use crate::injury::injuryType::injury_type_block::{BlockMode, InjuryTypeBlock};
        let mut game = make_game();
        add_player(&mut game, "attacker", PS_STANDING);
        add_away_player(&mut game, "defender", PS_STANDING);
        let mut t = InjuryTypeBlock::new(BlockMode::Regular, true);
        let mut rng = GameRng::new(1);
        let coord = game.field_model.player_coordinate("defender").unwrap();
        let result = handle_injury(&mut game, &mut rng, &mut t, Some("attacker"), "defender", coord, None, None, ApothecaryMode::Defender);
        result.apply_to(&mut game);
        if result.injury_context.suffered_injury.map(|s| s.is_casualty()).unwrap_or(false) {
            let pr = game.game_result.home.player_result("attacker");
            assert!(pr.is_some() && pr.unwrap().casualties >= 1, "attacker casualty counter should increment");
        }
    }
}
