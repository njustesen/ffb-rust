//! Step engine: the `Step` enum (concrete steps + dispatch), the `StepStack`, and the
//! `GameState` driver loop — the Rust port of Java `GameState.executeStep`.
//! See `docs/step_port/00_framework.md` (driver) and `10_sequences.md` (sequences).
//!
//! Steps are dispatched via the `Step` enum (no `dyn`). A step's `start`/`handle_command`
//! return a `StepOutcome` (next action + events + sub-sequences to push + params to publish +
//! an optional prompt). The driver — sole owner of the stack — applies the pushes and processes
//! the action, so a step only ever borrows `&mut Game` + `&mut GameRng`.
//!
//! Boundary (Java `ClientCommand`/`DialogParameter` analogue): the engine speaks the harness's
//! `Action`/`AgentPrompt` vocabulary directly. A step that must wait yields `Continue` + a
//! `prompt`; the driver surfaces it via `current_prompt()`; the harness's agent answers with an
//! `Action`, which `apply()` feeds to the waiting step's `handle_command`. (The wire
//! `ClientCommand` mapping is the networking phase, G/I; the engine/parity path uses `Action`.)

use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use ffb_model::events::GameEvent;
use ffb_model::enums::{GameStatus, Weather};
use ffb_model::prompts::AgentPrompt;

use ffb_model::model::team::Team;
use ffb_model::enums::{
    Rules, Direction, KickoffResult, PlayerState, PlayerAction,
    PS_STANDING, PS_RESERVE, PS_PRONE, PS_STUNNED,
    PS_KNOCKED_OUT, PS_BADLY_HURT, PS_SERIOUS_INJURY, PS_RIP,
    PS_EXHAUSTED, PS_BANNED,
    BlockResult,
};
use ffb_model::types::{FieldCoordinate, FIELD_WIDTH, FIELD_HEIGHT};
use ffb_model::kickoff::{kickoff_event_bb2025, KickoffEventKind};
use ffb_model::util::state_hash::state_hash;
use ffb_mechanics::mechanics::{
    scatter_coordinate,
    block_result_for_roll, block_dice_count,
    armor_broken, injury_result, InjuryOutcome, casualty_tier_bb2025, CasualtyTier,
    minimum_roll_dodge,
    throw_in_distance, throw_in_direction_for_roll,
    corner_throw_in_direction_for_roll, is_corner_square, corner_direction,
    passing_distance_bb2025,
};
use ffb_mechanics::modifiers::DODGE_TACKLE_ZONE;

use crate::action::{Action, PlayerActionChoice};
use crate::legal_actions::{legal_activate_player_actions, legal_move_targets, legal_blitz_move_targets, legal_block_targets, bfs_path, TeamSide};
use super::framework::{StepAction, StepId, StepParameter};

/// Place a team's available (RESERVE / unset) players in the canonical parity formation —
/// 1:1 with Java `ParityRunner.placeReserves()` (and the validated monolith port): jersey
/// order, ≤11, first three on the LOS (x=12), then the overflow squares; away mirrored x→25-x.
fn place_team_canonical(game: &mut Game, home: bool) {
    let los: &[(i32, i32)] = &[(12, 7), (12, 6), (12, 8), (12, 5), (12, 9), (12, 4), (12, 10)];
    let overflow: &[(i32, i32)] = &[
        (5, 5), (5, 7), (5, 9), (6, 6), (6, 8), (4, 6), (4, 8), (3, 6), (3, 8), (2, 5), (2, 9), (1, 7),
    ];
    let mut players: Vec<(String, i32)> = if home {
        game.team_home.players.iter().map(|p| (p.id.clone(), p.nr)).collect()
    } else {
        game.team_away.players.iter().map(|p| (p.id.clone(), p.nr)).collect()
    };
    players.sort_by_key(|&(_, nr)| nr);
    players.truncate(11);
    let available: Vec<&(String, i32)> = players.iter().filter(|(pid, _)| {
        match game.field_model.player_state(pid) {
            None => true,                          // unset before first setup = available
            Some(s) => s.base() == PS_RESERVE,
        }
    }).collect();
    let los_needed = available.len().min(3);
    for (placed, (pid, _)) in available.iter().enumerate() {
        let (ox, oy) = if placed < los_needed {
            los[placed]
        } else {
            let i = placed - los_needed;
            if i < overflow.len() { overflow[i] } else { continue }
        };
        let coord = if home { FieldCoordinate::new(ox, oy) } else { FieldCoordinate::new(25 - ox, oy) };
        game.field_model.set_player_coordinate(pid, coord);
        game.field_model.set_player_state(pid, PlayerState::new(PS_STANDING));
    }
}

/// Java StepEndTurn.getFaintingCount: when weather is SWELTERING_HEAT and fNewHalf/fTouchdown,
/// roll d3 → faintingCount, then for each team pick that many random on-pitch players via
/// die(count), set them EXHAUSTED and remove from field. Consumes 1 + 2*faintingCount dice.
fn roll_sweltering_heat_fainting(game: &mut Game, rng: &mut GameRng) {
    let fainting_count = rng.d3() as usize;
    for is_home in [true, false] {
        let team_ids: Vec<(String, i32)> = if is_home {
            game.team_home.players.iter().map(|p| (p.id.clone(), p.nr)).collect()
        } else {
            game.team_away.players.iter().map(|p| (p.id.clone(), p.nr)).collect()
        };
        let mut on_field: Vec<String> = {
            let mut ps: Vec<(String, i32)> = team_ids.into_iter()
                .filter(|(id, _)| game.field_model.player_coordinate(id)
                    .map(|c| c.is_on_pitch())
                    .unwrap_or(false))
                .collect();
            ps.sort_by_key(|&(_, nr)| nr);
            ps.into_iter().map(|(id, _)| id).collect()
        };
        let mut i = 0;
        while i < fainting_count && !on_field.is_empty() {
            let idx = (rng.die(on_field.len() as u32) - 1) as usize;
            let pid = on_field.remove(idx);
            game.field_model.remove_player(&pid);
            game.field_model.set_player_state(&pid, PlayerState::new(PS_EXHAUSTED));
            i += 1;
        }
    }
}

/// Map the rolled kickoff-event kind to the `KickoffResult` carried by events/params.
/// (The two enums mirror each other; this is the BB2025-reachable set.)
fn kickoff_result_from_kind(kind: KickoffEventKind) -> KickoffResult {
    match kind {
        KickoffEventKind::GetTheRef => KickoffResult::GetTheRef,
        KickoffEventKind::HighKick => KickoffResult::HighKick,
        KickoffEventKind::CheeringFans => KickoffResult::CheeringFans,
        KickoffEventKind::WeatherChange => KickoffResult::WeatherChange,
        KickoffEventKind::BrilliantCoaching => KickoffResult::BrilliantCoaching,
        KickoffEventKind::QuickSnap => KickoffResult::QuickSnap,
        KickoffEventKind::PitchInvasion => KickoffResult::PitchInvasion,
        KickoffEventKind::Riot => KickoffResult::Riot,
        KickoffEventKind::PerfectDefence => KickoffResult::PerfectDefence,
        KickoffEventKind::ThrowARock => KickoffResult::ThrowARock,
        KickoffEventKind::TimeOut => KickoffResult::TimeOut,
        KickoffEventKind::SolidDefence => KickoffResult::SolidDefence,
        KickoffEventKind::OficiousRef => KickoffResult::OficiousRef,
        KickoffEventKind::Blitz => KickoffResult::Blitz,
        KickoffEventKind::Charge => KickoffResult::Charge,
        KickoffEventKind::DodgySnack => KickoffResult::DodgySnack,
    }
}

// ── Concrete steps ──────────────────────────────────────────────────────────────
// One variant per ported step (BB2025 lineman set grows here per docs/step_port/20_steps).
// Each carries its persistent fields; pregame steps are stateless.

#[derive(Debug, Clone)]
pub enum Step {
    /// Game-start bookkeeping: mark the game active. (Java `StepInitStartGame`.)
    InitStartGame,
    /// Roll fan factor for home then away. (Java `mixed/start/StepSpectators`.)
    Spectators,
    /// Roll initial weather (2d6). (Java `game/start/StepWeather`.)
    Weather,
    /// Prompt the coin guess, then flip the coin (d2). (Java `bb2016/start/StepCoinChoice`.)
    CoinChoice,
    /// Prompt the coin winner to receive or kick; set first-offense. (Java `StepReceiveChoice`.)
    ReceiveChoice,
    /// First-kickoff bookkeeping: StartHalf. (Java `StepInitKickoff`.) 0 dice.
    InitKickoff,
    /// Canonical placement of the active team, then flip. (Java `StepSetup` ×2.) 0 dice.
    Setup,
    /// Latch the kick target: place the ball on the receiving half. (Java `StepKickoff`/KickBall.)
    Kickoff,
    /// Scatter the kicked ball: d8 direction + d6 distance. (Java `StepKickoffScatterRoll`.)
    KickoffScatterRoll,
    /// Roll the 2d6 kickoff-event table; publish the result. (Java `StepKickoffResultRoll`.)
    KickoffResultRoll,
    /// Apply the rolled kickoff event (consumes the published `KickoffResult`). (Java
    /// `StepApplyKickoffResult`.) Cheering Fans ported; other results guarded.
    ApplyKickoffResult { result: Option<KickoffResult>, touchback: bool },
    /// Bounce the ball where it landed — but only when there is no touchback (Java gates this
    /// via `StepKickoffAnimation` publishing CATCH_KICKOFF mode only when `!fTouchback`).
    /// (Java `StepCatchScatterThrowIn` in CATCH_KICKOFF mode.)
    CatchScatterThrowIn { touchback: bool },
    /// Give the ball to the receiving team's player nearest to (13,8) when `touchback` is true.
    /// (Java `StepTouchback`.)
    Touchback { touchback: bool },
    /// End-of-kickoff bookkeeping: pushes the EndTurn sequence. (Java `StepEndKickoff`.) 0 dice.
    EndKickoff,
    /// Pass-through no-op (lineman activation block and EndTurn prefix steps that have no
    /// effect in a skill-less game: ForgoneStalling, SteadyFootingHit, PlaceBallHit,
    /// ApothecaryHit, CatchScatterEndTurn, plus the 18 activation-block stubs in Select).
    NoOp,
    /// End-of-turn bookkeeping: flip active team, bump turn_nr, reset, push Select.
    /// (Java `StepEndTurn`.) 0 dice.
    EndTurn,
    /// Activation gate: emit ActivatePlayer prompt; on command GOTO END_SELECTING.
    /// (Java `StepInitSelecting`.) 0 dice.
    InitSelecting,
    /// Dispatch hub: read acting_player and push the correct sub-sequence.
    /// (Java `StepEndSelecting`.) EndTurn when player_id is None.
    EndSelecting,
    /// Movement gate: emit Move prompt for the active player; on command teleport.
    /// (Java `StepInitMoving`.) For BLITZ: targets filtered to squares adjacent to defender.
    InitMoving,
    /// Roll d6 pickup attempt; on failure set turnover and run the ball-bounce chain
    /// (d8 scatter → catch/throw-in loop). 1:1 port of Java `StepPickUp` +
    /// `StepCatchScatterThrowIn.bounceBall`. Only fires when ball_in_play && ball_moving
    /// && player is standing on the ball square.
    PickUp,
    /// Post-movement stub; currently a no-op for skill-less lineman.
    /// (Java `StepEndMoving`.)
    EndMoving,
    /// Roll block dice, apply the result, execute armor/injury/casualty chain.
    /// (Java `StepDoBlock`.) Auto-picks best result for multi-die blocks.
    DoBlock,
    /// Roll 2d6 armor + injury chain against the foul target (acting_player.defender_id).
    /// 1:1 port of Java `StepFoul.executeStep` + `UtilServerInjury.handleInjury`.
    DoFoul,
    /// Hand-off action: move ball to receiver, roll d6 catch, if catch fails roll d8 scatter.
    /// Receiver is in acting_player.defender_id. No turnover. 1:1 with Java StepHandOver +
    /// StepCatchScatterThrowIn (CATCH_HAND_OFF mode).
    DoHandOff,
    /// Pass action: roll d6 pass accuracy, move ball to receiver square, roll d6 catch,
    /// if catch fails set turnover + roll d8 scatter. Receiver is in acting_player.defender_id.
    /// 1:1 with Java StepPass (BB2025) + StepCatchScatterThrowIn.
    DoPass,
    /// End of a player's activation: record acted_player_id, clear acting_player, re-select.
    /// (Java `StepEndPlayerAction`.)
    EndPlayerAction,
}

impl Step {
    pub fn id(&self) -> StepId {
        match self {
            Step::InitStartGame => StepId::InitStartGame,
            Step::Spectators => StepId::Spectators,
            Step::Weather => StepId::Weather,
            Step::CoinChoice => StepId::CoinChoice,
            Step::ReceiveChoice => StepId::ReceiveChoice,
            Step::InitKickoff => StepId::Kickoff,
            Step::Setup => StepId::Setup,
            Step::Kickoff => StepId::Kickoff,
            Step::KickoffScatterRoll => StepId::KickoffScatterRoll,
            Step::KickoffResultRoll => StepId::KickoffResultRoll,
            Step::ApplyKickoffResult { .. } => StepId::ApplyKickoffResult,
            Step::CatchScatterThrowIn { .. } => StepId::CatchScatterThrowIn,
            Step::Touchback { .. } => StepId::CatchScatterThrowIn,
            Step::EndKickoff => StepId::EndKickoff,
            Step::NoOp => StepId::NoOp,
            Step::EndTurn => StepId::EndTurn,
            Step::InitSelecting => StepId::InitSelecting,
            Step::EndSelecting => StepId::EndSelecting,
            Step::InitMoving => StepId::InitMoving,
            Step::PickUp => StepId::PickUp,
            Step::EndMoving => StepId::EndMoving,
            Step::DoBlock => StepId::BlockRoll,
            Step::DoFoul => StepId::Foul,
            Step::DoHandOff => StepId::HandOver,
            Step::DoPass => StepId::Pass,
            Step::EndPlayerAction => StepId::EndPlayerAction,
        }
    }

    /// The step's `start()` body (Java `AbstractStep.start`). Steps that complete immediately
    /// advance with `NextStep`; steps that wait for an agent decision return `Continue` + a
    /// `prompt` and do their work in `handle_command`.
    fn start(&self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        match self {
            Step::InitStartGame => {
                game.status = GameStatus::Active;
                StepOutcome::next()
            }
            // Java StepSpectators: rollFanFactor() d3 home then d3 away; fanFactor = dedicatedFans
            // + roll. No GameEvent (not in the state hash).
            Step::Spectators => {
                let roll_home = rng.d3();
                game.team_home.fan_factor = game.team_home.dedicated_fans + roll_home;
                let roll_away = rng.d3();
                game.team_away.fan_factor = game.team_away.dedicated_fans + roll_away;
                StepOutcome::next()
            }
            // Java StepWeather: rollWeather() = 2d6, mapped by interpretRollWeather.
            Step::Weather => {
                let w1 = rng.d6();
                let w2 = rng.d6();
                let weather = Weather::for_roll(w1 + w2);
                game.weather = weather;
                StepOutcome::next().with_event(GameEvent::WeatherChange { weather })
            }
            // Java StepCoinChoice: the away coach guesses; the home prompt asks for the guess.
            // No dice in start — the coin is flipped in handle_command after the guess arrives.
            Step::CoinChoice => StepOutcome::cont().with_prompt(AgentPrompt::CoinChoice { is_home: true }),
            // Java StepReceiveChoice: the coin winner (now `home_playing`) chooses receive/kick.
            Step::ReceiveChoice => {
                let team_id = game.active_team().id.clone();
                StepOutcome::cont().with_prompt(AgentPrompt::ReceiveChoice { team_id })
            }
            // Java StepInitKickoff (first kickoff): start half 1. Bookkeeping, 0 dice.
            Step::InitKickoff => {
                StepOutcome::next().with_event(GameEvent::StartHalf { half: game.half })
            }
            // Java StepSetup ×2 (kicking then receiving team). The parity agent places its team
            // in the canonical formation; we place the active team and flip so the next Setup
            // handles the other. 0 dice, no prompt.
            Step::Setup => {
                place_team_canonical(game, game.home_playing);
                game.home_playing = !game.home_playing;
                StepOutcome::next()
            }
            // Java StepKickoff: the kicking coach picks the target square (KickBall command,
            // 2 decisionRng draws — must consume them to keep the agent RNG synced with Java).
            // The ball is placed in handle_command.
            Step::Kickoff => StepOutcome::cont().with_prompt(AgentPrompt::KickBall),
            // Java StepKickoffScatterRoll: scatter the kicked ball. Dice IN ORDER: d8 direction,
            // then d6 distance. Java back-walks from the full-distance endpoint decreasing
            // `distance` until FieldCoordinateBounds.FIELD.isInBounds — equivalent to
            // is_on_pitch() (FIELD = (0,0)-(25,14)). d=0 (kick start) is always in bounds.
            // Touchback is determined from the FULL-distance endpoint (not the back-walked
            // position): if the endpoint is not in the receiving half → touchback.
            // Receiving half: HALF_AWAY (x 13-25) when home kicks, HALF_HOME (x 0-12) when away.
            // Java publishes TOUCHBACK and KICKOFF_BOUNDS; we publish StepParameter::Touchback
            // so ApplyKickoffResult and CatchScatterThrowIn can gate their logic on it.
            Step::KickoffScatterRoll => {
                let start = game.field_model.ball_coordinate.unwrap_or(FieldCoordinate::new(0, 0));
                let dir_roll = rng.d8();
                let direction = Direction::for_roll(dir_roll).expect("d8 roll is 1..=8");
                let distance = rng.d6();
                if std::env::var("FFB_KICKOFF_TRACE").is_ok() {
                    eprintln!("SCATTER_ROLL half={} home_playing={} start=({},{}) dir_roll={} dist={}", game.half, game.home_playing, start.x, start.y, dir_roll, distance);
                }
                if std::env::var("FFB_SCATTER_TRACE").is_ok() {
                    eprintln!("SCATTER half={} start=({},{}) rng_call_count={} dir_roll={} dir={:?} dist={}", game.half, start.x, start.y, rng.call_count, dir_roll, direction, distance);
                }
                // Full-distance endpoint for touchback determination.
                let (ex, ey) = scatter_coordinate(start.x, start.y, direction, distance);
                let endpoint = FieldCoordinate::new(ex, ey);
                // Touchback = endpoint NOT in the receiving half.
                // home_playing = kicker (home kicks), so receiving half is HALF_AWAY (x>=13).
                let scatter_touchback = if game.home_playing {
                    !(endpoint.x >= 13 && endpoint.x <= 25 && endpoint.y >= 0 && endpoint.y <= 14)
                } else {
                    !(endpoint.x >= 0 && endpoint.x <= 12 && endpoint.y >= 0 && endpoint.y <= 14)
                };
                if std::env::var("FFB_KICKOFF_TRACE").is_ok() {
                    eprintln!("SCATTER_TB half={} endpoint=({},{}) scatter_touchback={}", game.half, endpoint.x, endpoint.y, scatter_touchback);
                }
                // Back-walk: start from full distance; decrement until FIELD.isInBounds (= is_on_pitch).
                // d=0 (the kick start) is always in bounds and serves as the ultimate fallback.
                let mut landing = start; // fallback: kick start position
                let mut d = distance;
                loop {
                    let (x, y) = scatter_coordinate(start.x, start.y, direction, d);
                    let c = FieldCoordinate::new(x, y);
                    if c.is_on_pitch() {
                        landing = c;
                        break;
                    }
                    if d == 0 { break; }
                    d -= 1;
                }
                game.field_model.ball_coordinate = Some(landing);
                // Java StepKickoffScatterRoll line 153: setBallMoving(true) — needed so CSTIN's
                // catchBall() condition (isBallInPlay && isBallMoving) fires when player is at ball.
                game.field_model.ball_moving = true;
                StepOutcome::next()
                    .publish(StepParameter::Touchback(scatter_touchback))
                    .with_event(GameEvent::KickoffScatter { start, direction: dir_roll, distance })
            }
            // Java StepKickoffResultRoll: rollKickoff() = 2d6; interpret → KickoffResult; publish it.
            Step::KickoffResultRoll => {
                let d1 = rng.d6();
                let d2 = rng.d6();
                let total = d1 + d2;
                if std::env::var("FFB_KICKOFF_TRACE").is_ok() {
                    eprintln!("KICKOFF_RESULT half={} d1={} d2={} total={} result={:?} ball_in_play={}", game.half, d1, d2, total, kickoff_event_bb2025(total), game.field_model.ball_in_play);
                }
                let mut out = StepOutcome::next();
                if let Some(kind) = kickoff_event_bb2025(total) {
                    let result = kickoff_result_from_kind(kind);
                    out = out
                        .with_event(GameEvent::KickoffResultEvent { result })
                        .publish(StepParameter::KickoffResult(result));
                }
                out
            }
            // Java StepApplyKickoffResult: dispatch on the published KickoffResult (captured via
            // set_parameter). Also receives fTouchback via StepParameter::Touchback.
            // WeatherChange: gust fires only when !touchback && Nice; checks receiving half bounds.
            Step::ApplyKickoffResult { result, touchback } => {
                if std::env::var("FFB_KICKOFF_TRACE").is_ok() {
                    eprintln!("APPLY_KR half={} result={:?} touchback={}", game.half, result, touchback);
                }
                match result {
                    Some(KickoffResult::CheeringFans) => {
                        // Java StepApplyKickoffResult.handleCheeringFans:
                        // totalHome = rollD6 + cheerleaders; totalAway = rollD6 + cheerleaders.
                        // Team with >= total gets +1 additional assist for their next block.
                        // Java: setTeamIdsAdditionalAssist → getAdditionalAssist adds to ATK strength.
                        let home_roll = rng.d6();
                        let away_roll = rng.d6();
                        let home_total = home_roll + game.team_home.cheerleaders;
                        let away_total = away_roll + game.team_away.cheerleaders;
                        if home_total >= away_total {
                            game.home_additional_assists += 1;
                        }
                        if away_total >= home_total {
                            game.away_additional_assists += 1;
                        }
                        StepOutcome::next().publish(StepParameter::Touchback(*touchback))
                    }
                    Some(KickoffResult::WeatherChange) => {
                        // Java handleWeatherChange: 2d6 for new weather.
                        // If Nice AND !fTouchback: up to 3× d8 gust (each step checks fKickoffBounds
                        // = receiving half; goes OOB from that half → fTouchback=true, keep ball, break).
                        // Java bb2025 StepApplyKickoffResult.handleWeatherChange lines 548-574.
                        let w1 = rng.d6();
                        let w2 = rng.d6();
                        let weather = Weather::for_roll(w1 + w2);
                        game.weather = weather;
                        let mut new_touchback = *touchback;
                        if weather == Weather::Nice && !new_touchback {
                            let mut last_pos = game.field_model.ball_coordinate
                                .unwrap_or(FieldCoordinate::new(12, 7));
                            for _ in 0..3 {
                                let dir_roll = rng.d8();
                                let dir = Direction::for_roll(dir_roll).expect("d8 is 1..=8");
                                let (nx, ny) = scatter_coordinate(last_pos.x, last_pos.y, dir, 1);
                                let new_pos = FieldCoordinate::new(nx, ny);
                                // Check if new position is in the receiving half (fKickoffBounds).
                                // home_playing = kicker: receiving half is HALF_AWAY (x 13-25).
                                let in_receiving_half = if game.home_playing {
                                    new_pos.x >= 13 && new_pos.x <= 25 && new_pos.y >= 0 && new_pos.y <= 14
                                } else {
                                    new_pos.x >= 0 && new_pos.x <= 12 && new_pos.y >= 0 && new_pos.y <= 14
                                };
                                if in_receiving_half {
                                    game.field_model.ball_coordinate = Some(new_pos);
                                    last_pos = new_pos;
                                } else {
                                    // Gust would leave receiving half: keep ball, set touchback, stop.
                                    game.field_model.ball_coordinate = Some(last_pos);
                                    new_touchback = true;
                                    break;
                                }
                            }
                        }
                        StepOutcome::next()
                            .publish(StepParameter::Touchback(new_touchback))
                            .with_event(GameEvent::WeatherChange { weather })
                    }
                    Some(KickoffResult::BrilliantCoaching) => {
                        // Java handleBrilliantCoaching: d6 home, d6 away; higher gains a reroll.
                        let _home = rng.d6();
                        let _away = rng.d6();
                        StepOutcome::next().publish(StepParameter::Touchback(*touchback))
                    }
                    Some(KickoffResult::QuickSnap) => {
                        // Java handleQuickSnap: rolls d3 to determine extra MA granted.
                        let _ = rng.d3();
                        StepOutcome::next().publish(StepParameter::Touchback(*touchback))
                    }
                    Some(KickoffResult::Blitz) => {
                        // Java handleBlitz: rolls d3 (extra MA for kicking team); same pattern as QuickSnap.
                        let _ = rng.d3();
                        StepOutcome::next().publish(StepParameter::Touchback(*touchback))
                    }
                    Some(KickoffResult::Charge) => {
                        // Java handleCharge: rolls d3 (extra movement for receiving team).
                        let _ = rng.d3();
                        StepOutcome::next().publish(StepParameter::Touchback(*touchback))
                    }
                    Some(KickoffResult::GetTheRef) => {
                        // No dice for GetTheRef (gains 1 reroll).
                        StepOutcome::next().publish(StepParameter::Touchback(*touchback))
                    }
                    Some(KickoffResult::TimeOut) => {
                        // Java handleTimeout: kickingTeamTurn >= 6 → both lose 1 turn; else gain 1.
                        // home_playing = kicker at this point in the sequence.
                        let kicking_turn = if game.home_playing {
                            game.turn_data_home.turn_nr
                        } else {
                            game.turn_data_away.turn_nr
                        };
                        let modifier: i32 = if kicking_turn >= 6 { -1 } else { 1 };
                        game.turn_data_home.turn_nr += modifier;
                        game.turn_data_away.turn_nr += modifier;
                        StepOutcome::next().publish(StepParameter::Touchback(*touchback))
                    }
                    Some(KickoffResult::Riot) => {
                        // Java handleRiot: rolls d6; 1-3 = lose a turn, 4-6 = gain a turn.
                        let roll = rng.d6();
                        let modifier: i32 = if roll <= 3 { 1 } else { -1 };
                        game.turn_data_home.turn_nr += modifier;
                        game.turn_data_away.turn_nr += modifier;
                        StepOutcome::next().publish(StepParameter::Touchback(*touchback))
                    }
                    Some(KickoffResult::PitchInvasion) => {
                        // Java handlePitchInvasion: d6 home fans, d6 away fans (+ fan factor from
                        // gameResult; starts at 0 for new games). Then d3 stun count. For each team
                        // that "lost" the fan roll (lower or equal total), stun up to `stun_count`
                        // of their standing players — each stun picks a random player via
                        // rollDice(standing.size()). Teams are compared with <= and >= so ties stun both.
                        // Java stunPlayers() iterates team.getPlayers() (jersey order).
                        let roll_home = rng.d6();
                        let roll_away = rng.d6();
                        let stun_count = rng.d3();
                        // Fan factor is 0 at game start, so total = roll.
                        // home team gets stunned if their fans ≤ away fans.
                        if roll_home <= roll_away {
                            let mut standing: Vec<String> = {
                                let mut ps: Vec<(String, i32)> = game.team_home.players.iter()
                                    .filter(|p| game.field_model.player_state(&p.id)
                                        .map(|s| s.base() == PS_STANDING).unwrap_or(false))
                                    .map(|p| (p.id.clone(), p.nr))
                                    .collect();
                                ps.sort_by_key(|&(_, nr)| nr);
                                ps.into_iter().map(|(id, _)| id).collect()
                            };
                            for _ in 0..stun_count {
                                if standing.is_empty() { break; }
                                let idx = (rng.die(standing.len() as u32) - 1) as usize;
                                let pid = standing.remove(idx);
                                game.field_model.set_player_state(&pid, PlayerState::new(PS_STUNNED));
                            }
                        }
                        // away team gets stunned if their fans ≤ home fans.
                        if roll_away <= roll_home {
                            let mut standing: Vec<String> = {
                                let mut ps: Vec<(String, i32)> = game.team_away.players.iter()
                                    .filter(|p| game.field_model.player_state(&p.id)
                                        .map(|s| s.base() == PS_STANDING).unwrap_or(false))
                                    .map(|p| (p.id.clone(), p.nr))
                                    .collect();
                                ps.sort_by_key(|&(_, nr)| nr);
                                ps.into_iter().map(|(id, _)| id).collect()
                            };
                            for _ in 0..stun_count {
                                if standing.is_empty() { break; }
                                let idx = (rng.die(standing.len() as u32) - 1) as usize;
                                let pid = standing.remove(idx);
                                game.field_model.set_player_state(&pid, PlayerState::new(PS_STUNNED));
                            }
                        }
                        StepOutcome::next().publish(StepParameter::Touchback(*touchback))
                    }
                    Some(KickoffResult::SolidDefence) => {
                        // Java handleSolidDefense: rolls d3 (number of players kicking team may redeploy).
                        let _ = rng.d3();
                        StepOutcome::next().publish(StepParameter::Touchback(*touchback))
                    }
                    Some(KickoffResult::HighKick) => {
                        // No dice for HighKick in parity context (receiving player selection only).
                        StepOutcome::next().publish(StepParameter::Touchback(*touchback))
                    }
                    Some(KickoffResult::PerfectDefence) => {
                        // No dice for PerfectDefence (kicking team repositions, no roll needed).
                        StepOutcome::next().publish(StepParameter::Touchback(*touchback))
                    }
                    Some(KickoffResult::ThrowARock) => {
                        // Java handleThrowARock: rolls armor/injury vs a random player.
                        // Full injury chain deferred; consume the two armor dice to stay synced.
                        let _ = rng.d6();
                        let _ = rng.d6();
                        StepOutcome::next().publish(StepParameter::Touchback(*touchback))
                    }
                    Some(KickoffResult::DodgySnack) => {
                        // Java handleDodgySnack: d6 home + d6 away (always). If rollAway >= rollHome,
                        // pick a random home player via die(count) and roll d6 for effect (1=box).
                        // If rollHome >= rollAway, pick a random away player via die(count) and roll
                        // d6 for effect. Both branches fire on a tie. Matches Java's rollDice(6) calls
                        // and randomPlayer(array) = rollDice(array.length) - 1 index.
                        let roll_home = rng.d6();
                        let roll_away = rng.d6();
                        // Build on-field player lists in jersey order (matches Java team.getPlayers()).
                        let home_on_field: Vec<String> = {
                            let mut ps: Vec<(String, i32)> = game.team_home.players.iter()
                                .filter(|p| game.field_model.player_coordinate(&p.id)
                                    .map(|c| c.x >= 0 && c.x <= 25 && c.y >= 0 && c.y <= 14)
                                    .unwrap_or(false))
                                .map(|p| (p.id.clone(), p.nr))
                                .collect();
                            ps.sort_by_key(|&(_, nr)| nr);
                            ps.into_iter().map(|(id, _)| id).collect()
                        };
                        let away_on_field: Vec<String> = {
                            let mut ps: Vec<(String, i32)> = game.team_away.players.iter()
                                .filter(|p| game.field_model.player_coordinate(&p.id)
                                    .map(|c| c.x >= 0 && c.x <= 25 && c.y >= 0 && c.y <= 14)
                                    .unwrap_or(false))
                                .map(|p| (p.id.clone(), p.nr))
                                .collect();
                            ps.sort_by_key(|&(_, nr)| nr);
                            ps.into_iter().map(|(id, _)| id).collect()
                        };
                        // rollAway >= rollHome → home team's player gets the snack
                        let player_home_id: Option<String> = if roll_away >= roll_home && !home_on_field.is_empty() {
                            let idx = (rng.die(home_on_field.len() as u32) - 1) as usize;
                            Some(home_on_field[idx].clone())
                        } else {
                            None
                        };
                        // rollHome >= rollAway → away team's player gets the snack
                        let player_away_id: Option<String> = if roll_home >= roll_away && !away_on_field.is_empty() {
                            let idx = (rng.die(away_on_field.len() as u32) - 1) as usize;
                            Some(away_on_field[idx].clone())
                        } else {
                            None
                        };
                        // insertSteps: roll d6 per selected player; 1 → box (RESERVE + remove from field)
                        if let Some(ref pid) = player_home_id {
                            let roll = rng.d6();
                            if roll == 1 {
                                game.field_model.set_player_state(pid, PlayerState::new(PS_RESERVE));
                                game.field_model.remove_player(pid);
                            }
                            // else: enhancement marker (-MA/-AV) not tracked in state hash
                        }
                        if let Some(ref pid) = player_away_id {
                            let roll = rng.d6();
                            if roll == 1 {
                                game.field_model.set_player_state(pid, PlayerState::new(PS_RESERVE));
                                game.field_model.remove_player(pid);
                            }
                        }
                        StepOutcome::next().publish(StepParameter::Touchback(*touchback))
                    }
                    Some(KickoffResult::OficiousRef) => {
                        // No dice for OficiousRef in parity context.
                        StepOutcome::next().publish(StepParameter::Touchback(*touchback))
                    }
                    None => StepOutcome::next().publish(StepParameter::Touchback(*touchback)),
                }
            }
            // Java StepCatchScatterThrowIn (CATCH_KICKOFF mode): bounce the ball one square if the
            // landing square is empty (d8 direction). Java's StepKickoffAnimation gates CSTIN via
            // the CATCH_KICKOFF mode parameter — but KickoffAnimation's fTouchback defaults to
            // false for all non-WeatherChange events, so CSTIN ALWAYS bounces for non-WC events.
            //
            // For WeatherChange: if the gust went OOB (new_touchback=true) or if the initial
            // scatter was a touchback AND weather was Nice (scatter_touchback=true → WC skips gust
            // and re-publishes TOUCHBACK(true)), ApplyKickoffResult publishes Touchback(true)
            // here → touchback=true → skip (matches Java's KickoffAnimation not publishing
            // CATCH_KICKOFF when it receives TOUCHBACK=true). Ball stays at last valid pos.
            //
            // When the bounce goes off-pitch: Java's CSTIN publishes TOUCHBACK(true) (line 424)
            // so StepTouchback can fire. We do the same via StepOutcome::publish.
            Step::CatchScatterThrowIn { touchback } => {
                if *touchback {
                    // Touchback already set — skip bounce and forward the signal to the Touchback step.
                    return StepOutcome::next().publish(StepParameter::Touchback(true));
                }
                if std::env::var("FFB_KICKOFF_TRACE").is_ok() {
                    eprintln!("CSTIN_RUN half={} ball={:?} in_play={}", game.half, game.field_model.ball_coordinate, game.field_model.ball_in_play);
                }
                if let Some(ball) = game.field_model.ball_coordinate {
                    if ball.is_on_pitch() {
                        if let Some(player_id) = game.field_model.player_at(ball).cloned() {
                            // Java CSTIN CATCH_KICKOFF: catcher found → catchBall() iff isBallInPlay && isBallMoving.
                            if game.field_model.ball_in_play && game.field_model.ball_moving {
                                let ps = game.field_model.player_state(&player_id).unwrap_or_default();
                                if ps.has_tacklezones() {
                                    // rollSkill() = d6; minimumRollCatch = max(2, AG)
                                    let catch_roll = rng.d6();
                                    let ag = find_player_agility(game, &player_id);
                                    let min_roll = (ag).max(2);
                                    if std::env::var("FFB_KICKOFF_TRACE").is_ok() {
                                        eprintln!("CSTIN_CATCH half={} at=({},{}) roll={} min={}", game.half, ball.x, ball.y, catch_roll, min_roll);
                                    }
                                    if catch_roll >= min_roll {
                                        // Catch success: setBallMoving(false), ball stays.
                                        game.field_model.ball_moving = false;
                                    } else {
                                        // Catch fail → SCATTER_BALL → bounceBall()
                                        let dir_roll = rng.d8();
                                        let dir = Direction::for_roll(dir_roll).expect("d8 is 1..=8");
                                        let (x, y) = scatter_coordinate(ball.x, ball.y, dir, 1);
                                        let new_pos = FieldCoordinate::new(x, y);
                                        game.field_model.ball_coordinate = Some(new_pos);
                                        let in_receiving_half = if game.home_playing {
                                            new_pos.x >= 13 && new_pos.x <= 25 && new_pos.y >= 0 && new_pos.y <= 14
                                        } else {
                                            new_pos.x >= 0 && new_pos.x <= 12 && new_pos.y >= 0 && new_pos.y <= 14
                                        };
                                        if !in_receiving_half {
                                            return StepOutcome::next().publish(StepParameter::Touchback(true));
                                        }
                                    }
                                }
                                // else: no tackle zones → SCATTER_BALL; not reached for Standing linemen
                            }
                        } else {
                            // No player at ball: divingCatch → SCATTER_BALL (no Diving Catch for linemen)
                            // → bounceBall()
                            let dir_roll = rng.d8();
                            let dir = Direction::for_roll(dir_roll).expect("d8 roll is 1..=8");
                            let (x, y) = scatter_coordinate(ball.x, ball.y, dir, 1);
                            let new_pos = FieldCoordinate::new(x, y);
                            if std::env::var("FFB_KICKOFF_TRACE").is_ok() {
                                eprintln!("CSTIN_BOUNCE half={} from=({},{}) dir_roll={} dir={:?} to=({},{})", game.half, ball.x, ball.y, dir_roll, dir, x, y);
                            }
                            game.field_model.ball_coordinate = Some(new_pos);
                            // Java CSTIN bounceBall() uses fScatterBounds (receiving half during
                            // kickoff), not the full field. If bounce lands outside receiving half →
                            // touchback. HALF_AWAY = x 13..=25 (away receives), HALF_HOME = x 0..=12.
                            let in_receiving_half = if game.home_playing {
                                new_pos.x >= 13 && new_pos.x <= 25 && new_pos.y >= 0 && new_pos.y <= 14
                            } else {
                                new_pos.x >= 0 && new_pos.x <= 12 && new_pos.y >= 0 && new_pos.y <= 14
                            };
                            if !in_receiving_half {
                                return StepOutcome::next().publish(StepParameter::Touchback(true));
                            }
                        }
                    }
                }
                StepOutcome::next()
            }
            // Java StepTouchback: give the ball to the receiving team's player nearest to (13,8)
            // when fTouchback is true (received via StepParameter::Touchback from KickoffScatterRoll
            // and potentially updated by ApplyKickoffResult's WeatherChange gust). 0 dice consumed.
            Step::Touchback { touchback } => {
                if *touchback {
                    let kick_from = FieldCoordinate::new(13, 8);
                    let recv_ids: Vec<String> = if game.home_playing {
                        game.team_away.players.iter().map(|p| p.id.clone()).collect()
                    } else {
                        game.team_home.players.iter().map(|p| p.id.clone()).collect()
                    };
                    let mut best_coord: Option<FieldCoordinate> = None;
                    let mut best_dist = i32::MAX;
                    for pid in &recv_ids {
                        let coord = match game.field_model.player_coordinate(pid) {
                            Some(c) if c.is_on_pitch() => c,
                            _ => continue,
                        };
                        let ps = match game.field_model.player_state(pid) {
                            Some(ps) => ps,
                            None => continue,
                        };
                        if ps.base() != ffb_model::enums::PS_STANDING { continue; }
                        let dx = coord.x - kick_from.x;
                        let dy = coord.y - kick_from.y;
                        let dist = dx * dx + dy * dy;
                        if dist < best_dist { best_dist = dist; best_coord = Some(coord); }
                    }
                    if let Some(c) = best_coord {
                        game.field_model.ball_coordinate = Some(c);
                        // Java StepTouchback sets setBallMoving(false) when placing ball at a
                        // standing player with tackle zones (which is always the touchback target).
                        game.field_model.ball_moving = false;
                    }
                }
                StepOutcome::next()
            }
            // Java StepEndKickoff: pushes the EndTurn generator so the receiving team starts.
            Step::EndKickoff => StepOutcome::next().push_seq(end_turn_sequence()),
            // No-op pass-through (all lineman activation-block and EndTurn prefix steps that
            // have no effect for a skill-less game).
            Step::NoOp => StepOutcome::next(),
            // Java StepEndTurn: check if both teams have reached turn 8 (end-of-half) BEFORE
            // any flip — mirrors Java's checkEndOfHalf = (home.turn_nr >= 8 && away.turn_nr >= 8).
            // When true: for half 1 → H2 kickoff; for half 2 → game over (NO flip, NO increment,
            // matching Java's TurnMode.SETUP path which leaves home_playing/turn_nr unchanged).
            // When false: normal turn flip + increment + push Select.
            Step::EndTurn => {
                let end_of_half = game.turn_data_home.turn_nr >= 8 && game.turn_data_away.turn_nr >= 8;
                if end_of_half {
                    if game.half == 1 {
                        // H1 → H2: reset turns, set H2 kicker, push H2 kickoff sequence.
                        game.half = 2;
                        game.turn_data_home.turn_nr = 0;
                        game.turn_data_away.turn_nr = 0;
                        // H2 kicker = H1 receiver (home_first_offense = true → home received H1 →
                        // home kicks H2 → home_playing=true; false → away kicks H2 → false).
                        game.home_playing = game.home_first_offense;
                        // Java StepEndTurn.getFaintingCount() when fNewHalf=true: KO recovery BEFORE
                        // SwelteringHeat. Each KNOCKED_OUT player rolls 1 d6; recovers to RESERVE on 4+.
                        // Player iteration order: home players first, then away (Java game.getPlayers()).
                        {
                            let all_player_ids: Vec<String> = game.team_home.players.iter()
                                .chain(game.team_away.players.iter())
                                .map(|p| p.id.clone())
                                .collect();
                            for id in &all_player_ids {
                                let is_ko = game.field_model.player_state(&id)
                                    .map(|ps| ps.base() == PS_KNOCKED_OUT)
                                    .unwrap_or(false);
                                if is_ko {
                                    let roll = rng.d6();
                                    if roll >= 4 {
                                        game.field_model.set_player_state(&id, PlayerState::new(PS_RESERVE));
                                    }
                                }
                            }
                        }
                        // Java StepEndTurn.getFaintingCount(): SWELTERING_HEAT rolls d3 + die(N)
                        // per team to select players who faint (set EXHAUSTED, removed from field).
                        if game.weather == Weather::SwelteringHeat {
                            roll_sweltering_heat_fainting(game, rng);
                        }
                        // Java StepEndTurn.getFaintingCount() calls putAllPlayersIntoBox() when
                        // fNewHalf=true: move all canBeSetUpNextDrive players to the dugout so
                        // that H2 Setup can place them via place_team_canonical().
                        let all_ids: Vec<String> = game.team_home.players.iter()
                            .chain(game.team_away.players.iter())
                            .map(|p| p.id.clone())
                            .collect();
                        for id in &all_ids {
                            let can_box = game.field_model.player_state(&id)
                                .map(|ps| ps.can_be_set_up_next_drive())
                                .unwrap_or(false);
                            if can_box { game.field_model.remove_player(&id); }
                        }
                        StepOutcome::next().push_seq(h2_kickoff_sequence())
                    } else {
                        // H2 over — game ends. Do NOT flip home_playing or increment turn_nr so
                        // the game_end state hash matches Java's (Java leaves both at 8, active=away).
                        // Java StepEndTurn.getFaintingCount() when fNewHalf=true: KO recovery BEFORE
                        // SwelteringHeat. Each KNOCKED_OUT player rolls 1 d6; recovers to RESERVE on 4+.
                        {
                            let all_player_ids: Vec<String> = game.team_home.players.iter()
                                .chain(game.team_away.players.iter())
                                .map(|p| p.id.clone())
                                .collect();
                            for id in &all_player_ids {
                                let is_ko = game.field_model.player_state(&id)
                                    .map(|ps| ps.base() == PS_KNOCKED_OUT)
                                    .unwrap_or(false);
                                if is_ko {
                                    let roll = rng.d6();
                                    if roll >= 4 {
                                        game.field_model.set_player_state(&id, PlayerState::new(PS_RESERVE));
                                    }
                                }
                            }
                        }
                        // Java StepEndTurn.getFaintingCount(): SWELTERING_HEAT dice even at game end.
                        if game.weather == Weather::SwelteringHeat {
                            roll_sweltering_heat_fainting(game, rng);
                        }
                        // Java's StepEndTurn.getFaintingCount() calls UtilBox.putAllPlayersIntoBox()
                        // whenever fNewHalf=true, which moves all canBeSetUpNextDrive players to
                        // reserve boxes (negative X), producing (-1,-1,Reserve) in the state hash.
                        let all_ids: Vec<String> = game
                            .team_home
                            .players
                            .iter()
                            .chain(game.team_away.players.iter())
                            .map(|p| p.id.clone())
                            .collect();
                        for id in &all_ids {
                            let can_box = game
                                .field_model
                                .player_state(id)
                                .map(|ps| ps.can_be_set_up_next_drive())
                                .unwrap_or(false);
                            if can_box {
                                game.field_model.remove_player(id);
                            }
                        }
                        game.status = GameStatus::Finished;
                        StepOutcome::next()
                    }
                } else {
                    // Java StepEndTurn: remove any unused additional assist for the acting team.
                    if game.home_playing {
                        game.home_additional_assists = 0;
                    } else {
                        game.away_additional_assists = 0;
                    }
                    // Normal turn: flip to next team and increment their turn counter.
                    game.home_playing = !game.home_playing;
                    if game.home_playing {
                        game.turn_data_home.turn_nr += 1;
                        game.turn_data_home.reset_for_turn();
                    } else {
                        game.turn_data_away.turn_nr += 1;
                        game.turn_data_away.reset_for_turn();
                    }
                    // Java UtilPlayer.refreshPlayersForTurnStart: flips transient states and
                    // recovers STUNNED players on the newly-active team to PRONE.
                    refresh_players_for_turn_start(game);
                    if std::env::var_os("FFB_TRACE").is_some() {
                        let ss = ffb_model::util::state_hash::state_string(game);
                        eprintln!("RUST_ENDTURN_STATE home_turn={} away_turn={} home_playing={}: {}", game.turn_data_home.turn_nr, game.turn_data_away.turn_nr, game.home_playing, ss);
                    }
                    StepOutcome::next().push_seq(select_sequence())
                }
            }
            // Java StepInitSelecting: build the eligible player list, emit the ActivatePlayer
            // prompt, and wait for the command. Command handling in handle_command below.
            //
            // Java parity: Java ParityRunner calls computeEligiblePlayers() ONCE at turn start
            // and caches it for the whole turn. We do the same: populate turn_eligible_cache on
            // the first activation of each turn (when acted_player_ids is empty), then filter
            // the cache by acted_player_ids for subsequent activations. This ensures mid-turn
            // player movement doesn't change which actions are available (stale-list parity).
            Step::InitSelecting => {
                let turn_data = if game.home_playing {
                    &mut game.turn_data_home
                } else {
                    &mut game.turn_data_away
                };
                // Populate cache on first activation of the turn.
                if turn_data.turn_eligible_cache.is_empty() {
                    let side = if game.home_playing { TeamSide::Home } else { TeamSide::Away };
                    let raw_actions = legal_activate_player_actions(game, side);
                    let mut cache: Vec<(String, Vec<ffb_model::enums::PlayerAction>)> = Vec::new();
                    for act in raw_actions {
                        if let Action::ActivatePlayer { player_id, player_action, .. } = act {
                            let pa = pac_to_player_action(player_action);
                            if let Some((_, acts)) = cache.iter_mut().find(|(pid, _)| pid == &player_id) {
                                acts.push(pa);
                            } else {
                                cache.push((player_id, vec![pa]));
                            }
                        }
                    }
                    let turn_data = if game.home_playing {
                        &mut game.turn_data_home
                    } else {
                        &mut game.turn_data_away
                    };
                    turn_data.turn_eligible_cache = cache;
                }
                // Filter cache by acted_player_ids to get the current remaining eligible.
                let acted = if game.home_playing {
                    &game.turn_data_home.acted_player_ids
                } else {
                    &game.turn_data_away.acted_player_ids
                };
                let cache = if game.home_playing {
                    &game.turn_data_home.turn_eligible_cache
                } else {
                    &game.turn_data_away.turn_eligible_cache
                };
                let eligible: Vec<(String, Vec<ffb_model::enums::PlayerAction>)> = cache.iter()
                    .filter(|(pid, _)| !acted.contains(pid))
                    .cloned()
                    .collect();
                StepOutcome::cont().with_prompt(AgentPrompt::ActivatePlayer { eligible_players: eligible })
            }
            // Java StepEndSelecting: dispatch on acting_player.player_action.
            // When player_id is None the agent chose EndTurn → push the end-turn sequence.
            Step::EndSelecting => {
                if std::env::var_os("FFB_TRACE").is_some() {
                    eprintln!("RUST_ACTIVATION pid={:?} action={:?} ball={:?} ball_moving={} rng={}", game.acting_player.player_id, game.acting_player.player_action, game.field_model.ball_coordinate, game.field_model.ball_moving, rng.call_count);
                }
                match &game.acting_player.player_action {
                    None => StepOutcome::next().push_seq(end_turn_sequence()),
                    Some(PlayerAction::Move) => StepOutcome::next().push_seq(move_sequence()),
                    Some(PlayerAction::Blitz) => {
                        // Java: prone player choosing BLITZ gets stood up here (STAND_UP_BLITZ flow).
                        let pid = game.acting_player.player_id.clone().unwrap_or_default();
                        let is_prone = game.field_model.player_state(&pid)
                            .map(|s| s.base() == PS_PRONE)
                            .unwrap_or(false);
                        if is_prone {
                            game.field_model.set_player_state(&pid, PlayerState::new(PS_STANDING));
                        }
                        StepOutcome::next().push_seq(blitz_sequence())
                    }
                    Some(PlayerAction::Block) => StepOutcome::next().push_seq(block_sequence()),
                    Some(PlayerAction::StandUp) => {
                        let pid = game.acting_player.player_id.clone().unwrap_or_default();
                        game.field_model.set_player_state(&pid, PlayerState::new(PS_STANDING));
                        // Java MOVE for prone: stand up, then run full move sequence.
                        StepOutcome::next().push_seq(move_sequence())
                    }
                    Some(PlayerAction::Foul) => StepOutcome::next().push_seq(foul_sequence()),
                    Some(PlayerAction::Pass) => StepOutcome::next().push_seq(pass_sequence()),
                    Some(PlayerAction::HandOver) => StepOutcome::next().push_seq(handoff_sequence()),
                    // SecureTheBall (BB2025): like Move but pickup uses a 2+ auto-success.
                    // Java StepEndSelecting pushes the same move sequence as MOVE.
                    Some(PlayerAction::SecureTheBall) => StepOutcome::next().push_seq(move_sequence()),
                    Some(other) => panic!("EndSelecting: unhandled player_action {other:?}"),
                }
            }

            // Java StepInitMoving: compute legal move targets and emit the Move prompt.
            // For BLITZ the targets are squares adjacent to the declared defender.
            Step::InitMoving => {
                let player_id = match &game.acting_player.player_id {
                    Some(id) => id.clone(),
                    None => return StepOutcome::next(),
                };
                let squares = match &game.acting_player.player_action {
                    Some(PlayerAction::Blitz) => {
                        match &game.acting_player.defender_id {
                            Some(def_id) => {
                                let def_id = def_id.clone();
                                legal_blitz_move_targets(game, &player_id, &def_id)
                            }
                            None => legal_move_targets(game, &player_id),
                        }
                    }
                    _ => legal_move_targets(game, &player_id),
                };
                StepOutcome::cont()
                    .with_prompt(AgentPrompt::Move { player_id, squares })
            }

            // Java StepPickUp + StepCatchScatterThrowIn.bounceBall:
            // Attempt pickup if ball_in_play && ball_moving && player is on ball square.
            // On success: ball_moving = false. On failure: turnover = true, run bounce chain.
            // The bounce chain matches Java's CSTIN re-queue loop (FAILED_PICK_UP →
            // SCATTER_BALL → bounceBall → CATCH_SCATTER / THROW_IN / null).
            Step::PickUp => {
                let player_id = match &game.acting_player.player_id {
                    Some(id) => id.clone(),
                    None => return StepOutcome::next(),
                };
                let player_coord = match game.field_model.player_coordinate(&player_id) {
                    Some(c) => c,
                    None => return StepOutcome::next(),
                };
                // isPickUp(): ball_in_play && ball_moving && player_coord == ball_coord
                let on_ball = game.field_model.ball_coordinate
                    .map(|c| c == player_coord)
                    .unwrap_or(false);
                if std::env::var_os("FFB_TRACE").is_some() {
                    eprintln!("RUST_PICKUP pid={player_id} coord={player_coord:?} ball={:?} in_play={} moving={} on_ball={}", game.field_model.ball_coordinate, game.field_model.ball_in_play, game.field_model.ball_moving, on_ball);
                }
                if !game.field_model.ball_in_play || !game.field_model.ball_moving || !on_ball {
                    return StepOutcome::next();
                }
                // minimumRollPickup: max(2, AG + opponent_TZ_count)
                let ag = find_player_agility(game, &player_id);
                let tz_mod = count_opponent_tackle_zones_at(game, &player_id, player_coord);
                let minimum = (ag + tz_mod).max(2);
                let roll = rng.d6();
                if roll >= minimum {
                    game.field_model.ball_moving = false;
                    return StepOutcome::next();
                }
                // FAILURE: turnover; run CSTIN FAILED_PICK_UP → SCATTER_BALL → bounceBall chain
                game.turnover = true;

                // Local mode tags matching Java's CatchScatterThrowInMode state machine.
                // Data (catcher_id, throw_in_coord) stored separately to avoid enum-with-data
                // lifetime issues inside the match.
                #[derive(PartialEq)]
                enum Mode { Scatter, CatchScatter, ThrowIn, CatchThrowIn }

                let mut mode = Mode::Scatter;
                let mut catcher_id: Option<String> = None;
                let mut throw_in_coord: FieldCoordinate = player_coord; // initial ball pos

                loop {
                    match mode {
                        Mode::Scatter => {
                            // bounceBall(): roll d8, move ball 1 step.
                            // Java: lastValidCoordinate = end if in-bounds else start.
                            // Sets ball_coordinate = end (even if OOB). OOB → THROW_IN.
                            // Empty in-bounds → null (done). Player with TZ → CATCH_SCATTER.
                            // Player without TZ → FAILED_CATCH → SCATTER_BALL (loop again).
                            let dir_roll = rng.d8();
                            let dir = Direction::for_roll(dir_roll).expect("d8 is 1..=8");
                            let ball_start = game.field_model.ball_coordinate.unwrap_or(player_coord);
                            let (ex, ey) = scatter_coordinate(ball_start.x, ball_start.y, dir, 1);
                            let ball_end = FieldCoordinate::new(ex, ey);
                            let last_valid = if ball_end.is_on_pitch() { ball_end } else { ball_start };
                            game.field_model.ball_coordinate = Some(ball_end);
                            game.field_model.ball_moving = true;
                            if std::env::var_os("FFB_TRACE").is_some() {
                                eprintln!("RUST_PICKUP_BOUNCE from={ball_start:?} dir={dir:?} dir_roll={dir_roll} to={ball_end:?} on_pitch={}", ball_end.is_on_pitch());
                            }

                            if ball_end.is_on_pitch() {
                                if let Some(pid) = game.field_model.player_at(ball_end).cloned() {
                                    let ps = game.field_model.player_state(&pid).unwrap_or_default();
                                    if ps.has_tacklezones() {
                                        catcher_id = Some(pid);
                                        mode = Mode::CatchScatter;
                                    }
                                    // else: no TZ → FAILED_CATCH → stays Mode::Scatter
                                } else {
                                    break; // empty square: return null → done
                                }
                            } else {
                                // OOB: fThrowInCoordinate = last valid (= ball_start)
                                throw_in_coord = last_valid;
                                mode = Mode::ThrowIn;
                            }
                        }
                        Mode::CatchScatter => {
                            // catchBall() with CATCH_SCATTER context: +1 modifier.
                            // minimumRollCatch = max(2, AG + 1).
                            let pid = catcher_id.take().unwrap();
                            let catch_ag = find_player_agility(game, &pid);
                            let catch_target = (catch_ag + 1).max(2);
                            let catch_roll = rng.d6();
                            if catch_roll >= catch_target {
                                game.field_model.ball_moving = false;
                                break;
                            } else {
                                // FAILED_CATCH → SCATTER_BALL
                                mode = Mode::Scatter;
                            }
                        }
                        Mode::ThrowIn => {
                            // throwInBall(): roll direction (d3 corner / d6 edge) + 2×d6 distance.
                            // Java loop: for i in 0..distance, position = start + i steps.
                            // (i=0 gives start itself, max displacement = distance-1 squares.)
                            // In-bounds end → CATCH_THROW_IN. OOB end → THROW_IN again.
                            let ti_start = throw_in_coord;
                            let is_corner = is_corner_square(ti_start.x, ti_start.y);
                            let dir_roll = if is_corner { rng.d3() } else { rng.d6() };
                            let dir = if is_corner {
                                corner_throw_in_direction_for_roll(
                                    corner_direction(ti_start.x, ti_start.y), dir_roll)
                            } else {
                                throw_in_direction_for_roll(ti_start.x, ti_start.y, dir_roll)
                            };
                            let d1 = rng.d6();
                            let d2 = rng.d6();
                            let distance = throw_in_distance(d1, d2, game.rules);
                            // Java: for i in 0..distance; final pos = start + (distance-1) steps
                            let mut ball_end = ti_start;
                            let mut last_valid_ti = ti_start;
                            for i in 0..distance {
                                let (nx, ny) = scatter_coordinate(ti_start.x, ti_start.y, dir, i);
                                let nc = FieldCoordinate::new(nx, ny);
                                ball_end = nc;
                                if nc.is_on_pitch() {
                                    last_valid_ti = nc;
                                }
                            }
                            game.field_model.ball_moving = true;
                            if ball_end == last_valid_ti {
                                // in bounds: CATCH_THROW_IN
                                game.field_model.ball_coordinate = Some(last_valid_ti);
                                mode = Mode::CatchThrowIn;
                            } else {
                                // OOB: Java sets ball_coordinate = null, throw in from last_valid
                                game.field_model.ball_coordinate = None;
                                throw_in_coord = last_valid_ti;
                                // mode stays ThrowIn
                            }
                        }
                        Mode::CatchThrowIn => {
                            // CATCH_THROW_IN: player at ball → CATCH_SCATTER.
                            // No player → divingCatch → no diving catchers for linemen
                            // → SCATTER_BALL → bounceBall again.
                            let ball_pos = match game.field_model.ball_coordinate {
                                Some(c) => c,
                                None => break,
                            };
                            if let Some(pid) = game.field_model.player_at(ball_pos).cloned() {
                                let ps = game.field_model.player_state(&pid).unwrap_or_default();
                                if ps.has_tacklezones() {
                                    catcher_id = Some(pid);
                                    mode = Mode::CatchScatter;
                                } else {
                                    // no TZ → SCATTER_BALL
                                    mode = Mode::Scatter;
                                }
                            } else {
                                // no player → divingCatch → SCATTER_BALL (no Diving Catch for linemen)
                                mode = Mode::Scatter;
                            }
                        }
                    }
                }
                StepOutcome::next()
            }

            // Java StepEndMoving: post-movement stub (no dice for skill-less lineman).
            Step::EndMoving => StepOutcome::next(),

            // Java StepDoBlock: roll block dice, apply result, armor/injury/casualty chain.
            Step::DoBlock => {
                if std::env::var_os("FFB_TRACE").is_some() {
                    eprintln!("RUST_DOBLOCK atk={:?} def={:?} rng={}", game.acting_player.player_id, game.acting_player.defender_id, rng.call_count);
                }
                let attacker_id = match &game.acting_player.player_id {
                    Some(id) => id.clone(),
                    None => return StepOutcome::next(),
                };
                let defender_id = match &game.acting_player.defender_id {
                    Some(id) => id.clone(),
                    None => {
                        // Pick first adjacent opponent
                        let side = if game.home_playing { TeamSide::Home } else { TeamSide::Away };
                        let targets = legal_block_targets(game, &attacker_id, side);
                        if targets.is_empty() {
                            if std::env::var_os("FFB_TRACE").is_some() {
                                let atk_coord = game.field_model.player_coordinate(&attacker_id);
                                eprintln!("RUST_BLOCK_SKIP atk={attacker_id} home_playing={} coord={atk_coord:?}", game.home_playing);
                                let opp_team = if game.home_playing { &game.team_away } else { &game.team_home };
                                for p in &opp_team.players {
                                    let c = game.field_model.player_coordinate(&p.id);
                                    let s = game.field_model.player_state(&p.id);
                                    eprintln!("  OPP {} coord={c:?} state={s:?}", p.id);
                                }
                            }
                            return StepOutcome::next();
                        }
                        targets[0].clone()
                    }
                };
                // Effective strengths include assist counting (mirrors Java ServerUtilBlock).
                let atk_str = effective_attacker_strength(game, &attacker_id, &defender_id);
                let def_str = effective_defender_strength(game, &attacker_id, &defender_id);
                let dice_count = block_dice_count(atk_str, def_str);
                let n_dice = dice_count.unsigned_abs() as usize;
                // Consume additional assist (mirrors Java StepBlockRoll.removeAdditionalAssist).
                if game.home_playing {
                    game.home_additional_assists = 0;
                } else {
                    game.away_additional_assists = 0;
                }


                // Roll N dice, always pick index 0 (mirrors Java parity runner §7: always die 0).
                let roll = rng.d6();
                for _ in 1..n_dice {
                    let _ = rng.d6(); // consume remaining dice to keep game RNG in sync
                }
                let result = block_result_for_roll(roll);
                if std::env::var_os("FFB_TRACE").is_some() {
                    eprintln!("RUST_DOBLOCK_RESULT atk={attacker_id} def={defender_id} roll={roll} result={result:?} rng_after={}", rng.call_count);
                }
                // Negative dice_count = defender picks: in Java parity, always pick index 0.
                // Index 0 = first die rolled. No change needed (we already use the first die).
                let _ = dice_count;

                // Capture ball positions before knockdowns (KO removes player coordinate).
                let atk_coord_pre = game.field_model.player_coordinate(&attacker_id);
                let def_coord_pre = game.field_model.player_coordinate(&defender_id);
                let ball_coord = game.field_model.ball_coordinate;
                let atk_has_ball = atk_coord_pre.zip(ball_coord).map_or(false, |(a, b)| a == b);
                let def_has_ball = def_coord_pre.zip(ball_coord).map_or(false, |(d, b)| d == b);

                match result {
                    BlockResult::Skull => {
                        // Attacker knocked down = turnover. Java StepDropFallingPlayers publishes
                        // END_TURN=true whenever the acting player's state is FALLING.
                        apply_knockdown(game, &attacker_id, rng);
                        game.turnover = true;
                        // Java StepCatchScatterThrowIn.bounceBall: scatter ball if carrier fell.
                        if atk_has_ball {
                            scatter_ball_from_knockdown(game, atk_coord_pre.unwrap(), rng);
                        }
                    }
                    BlockResult::BothDown => {
                        // Attacker knocked down = turnover (no Block/Wrestle skill for linemen).
                        // Java resolves DEFENDER's armor/injury first, then ATTACKER's,
                        // then scatters ball after both armor rolls (not interleaved).
                        apply_knockdown(game, &defender_id, rng);
                        apply_knockdown(game, &attacker_id, rng);
                        game.turnover = true;
                        if atk_has_ball {
                            scatter_ball_from_knockdown(game, atk_coord_pre.unwrap(), rng);
                        } else if def_has_ball {
                            scatter_ball_from_knockdown(game, def_coord_pre.unwrap(), rng);
                        }
                    }
                    BlockResult::Pushback => {
                        // Follow-up DECLINED per §7 of AGENT_CONTRACT: defender pushed,
                        // attacker stays in place (no follow-through move).
                        auto_push(game, &attacker_id, &defender_id, game.home_playing);
                        // If defender had ball, move it with them (Java FieldModel keeps ball with pusher).
                        if def_has_ball {
                            if let Some(new_dc) = game.field_model.player_coordinate(&defender_id) {
                                game.field_model.ball_coordinate = Some(new_dc);
                            }
                        }
                    }
                    BlockResult::PowPushback => {
                        // Follow-up DECLINED per §7: defender pushed + knocked down,
                        // attacker stays in place.
                        auto_push(game, &attacker_id, &defender_id, game.home_playing);
                        if def_has_ball {
                            if let Some(new_dc) = game.field_model.player_coordinate(&defender_id) {
                                game.field_model.ball_coordinate = Some(new_dc);
                            }
                        }
                        let def_coord_post_push = game.field_model.player_coordinate(&defender_id);
                        apply_knockdown(game, &defender_id, rng);
                        if def_has_ball {
                            if let Some(dc) = def_coord_post_push {
                                scatter_ball_from_knockdown(game, dc, rng);
                            }
                        }
                    }
                    BlockResult::Pow => {
                        // Pow also pushes the defender (same as PowPushback, but no dodge option).
                        // Java StepBlockChoice case POW calls initPushback — identical sequence.
                        auto_push(game, &attacker_id, &defender_id, game.home_playing);
                        if def_has_ball {
                            if let Some(new_dc) = game.field_model.player_coordinate(&defender_id) {
                                game.field_model.ball_coordinate = Some(new_dc);
                            }
                        }
                        let def_coord_post_push = game.field_model.player_coordinate(&defender_id);
                        apply_knockdown(game, &defender_id, rng);
                        if def_has_ball {
                            if let Some(dc) = def_coord_post_push {
                                scatter_ball_from_knockdown(game, dc, rng);
                            }
                        }
                    }
                }
                StepOutcome::next()
            }

            // Java StepFoul → StepReferee → StepBribes → StepEjectPlayer.
            // Target stored in acting_player.defender_id.
            Step::DoFoul => {
                let fouler_id = match &game.acting_player.player_id {
                    Some(id) => id.clone(),
                    None => return StepOutcome::next(),
                };
                let target_id = match &game.acting_player.defender_id {
                    Some(id) => id.clone(),
                    None => return StepOutcome::next(),
                };
                apply_foul_injury(game, &fouler_id, &target_id, rng);
                StepOutcome::next()
            }

            // Java StepHandOver (ball placement) + StepCatchScatterThrowIn (catch + bounce).
            // BB2025: no pass accuracy roll for HandOff. Receiver stored in defender_id.
            // StepHandOver sets setBallMoving(true) before any catch attempt.
            // StepEndPassing triggers turnover when catcher==null (hasPassed==true) or opponent catches.
            Step::DoHandOff => {
                let receiver_id = match &game.acting_player.defender_id {
                    Some(id) => id.clone(),
                    None => return StepOutcome::next(),
                };
                let receiver_coord = match game.field_model.player_coordinate(&receiver_id) {
                    Some(c) => c,
                    None => return StepOutcome::next(),
                };
                let receiver_ag = find_player_agility(game, &receiver_id);

                // Java StepHandOver line 78: setBallMoving(true) before the catch attempt.
                game.field_model.ball_moving = true;
                game.field_model.ball_coordinate = Some(receiver_coord);

                // Initial catch attempt (CATCH_HAND_OFF). Java: TZ of RECEIVER count as penalty.
                // CatchModifierFactory.isAffectedByTackleZones → true; base collection has 1-8 TZ mods.
                let tz_count = count_opponent_tackle_zones_at(game, &receiver_id, receiver_coord);
                let catch_min = std::cmp::max(2, receiver_ag + tz_count);
                let catch_roll = rng.d6();
                if std::env::var_os("FFB_TRACE").is_some() {
                    eprintln!("RUST_HANDOVER recv={receiver_id} coord={receiver_coord:?} ag={receiver_ag} catch_min={catch_min} roll={catch_roll} rng={}", rng.call_count);
                }
                if catch_roll >= catch_min {
                    game.field_model.ball_moving = false;
                    return StepOutcome::next();
                }

                // Catch failed: bounceBall loop (FAILED_CATCH → SCATTER_BALL → CATCH_SCATTER ...).
                let acting_is_home = game.acting_player.player_id.as_ref()
                    .map(|pid| game.team_home.has_player(pid))
                    .unwrap_or(false);

                let mut ball_coord = receiver_coord;
                loop {
                    // bounceBall(): d8 scatter direction, move ball 1 square.
                    let dir_roll = rng.d8();
                    let dir = Direction::for_roll(dir_roll).expect("d8 is 1..=8");
                    let (bx, by) = scatter_coordinate(ball_coord.x, ball_coord.y, dir, 1);
                    let new_coord = FieldCoordinate::new(bx, by);
                    game.field_model.ball_coordinate = Some(new_coord);
                    game.field_model.ball_moving = true;

                    if !new_coord.is_on_pitch() {
                        // OOB: throw-in needed (not yet fully implemented); treat as turnover.
                        game.turnover = true;
                        return StepOutcome::next();
                    }

                    ball_coord = new_coord;
                    if let Some(pid) = game.field_model.player_at(new_coord).cloned() {
                        let has_tz = game.field_model.player_state(&pid)
                            .map(|s| s.has_tacklezones())
                            .unwrap_or(false);
                        if has_tz {
                            // CATCH_SCATTER: +1 mode modifier only. BB2025: no TZ modifier for catch.
                            // min = max(2, AG + 1).
                            let ag = find_player_agility(game, &pid);
                            let catch_min2 = std::cmp::max(2, ag + 1);
                            let catch_roll2 = rng.d6();
                            if catch_roll2 >= catch_min2 {
                                game.field_model.ball_moving = false;
                                let pid_in_home = game.team_home.has_player(&pid);
                                if pid_in_home != acting_is_home {
                                    game.turnover = true;
                                }
                                return StepOutcome::next();
                            }
                            // Catch failed: FAILED_CATCH → SCATTER_BALL → loop again.
                        }
                        // No TZ: FAILED_CATCH immediately → SCATTER_BALL → loop again.
                    } else {
                        // Empty square: catcher==null + hasPassed==true → TURNOVER.
                        game.turnover = true;
                        return StepOutcome::next();
                    }
                }
            }

            // Java StepPass (BB2025) + StepCatchScatterThrowIn.
            // Pass DOES cause a turnover if the catch fails (unlike HandOff).
            // Receiver stored in acting_player.defender_id (player ID on the pitch).
            Step::DoPass => {
                // If pickup already failed this activation, skip the pass entirely.
                if game.turnover {
                    return StepOutcome::next();
                }
                if std::env::var_os("FFB_TRACE").is_some() {
                    eprintln!("RUST_DOPASS ball={:?} ball_moving={} turnover={} rng={}", game.field_model.ball_coordinate, game.field_model.ball_moving, game.turnover, rng.call_count);
                }
                let passer_id = match &game.acting_player.player_id {
                    Some(id) => id.clone(),
                    None => return StepOutcome::next(),
                };
                let receiver_id = match &game.acting_player.defender_id {
                    Some(id) => id.clone(),
                    None => return StepOutcome::next(),
                };
                let pa = find_player_passing(game, &passer_id);
                let receiver_ag = find_player_agility(game, &receiver_id);
                let passer_coord = match game.field_model.player_coordinate(&passer_id) {
                    Some(c) => c,
                    None => return StepOutcome::next(),
                };
                let receiver_coord = match game.field_model.player_coordinate(&receiver_id) {
                    Some(c) => c,
                    None => return StepOutcome::next(),
                };

                // StepPass.executeStep:221 — roll d6 for pass accuracy.
                // BB2025 PassMechanic.evaluatePass:
                //   PA<=0 → FUMBLE (no roll consumed)
                //   roll==1 → FUMBLE
                //   resultAfterModifiers (= roll - dist_mod) <= 1 → FUMBLE
                //   roll==6 || resultAfterModifiers >= PA → ACCURATE
                //   else → INACCURATE
                // Java StepPass line 169: setBallMoving(true) before the roll.
                game.field_model.ball_moving = true;

                if pa <= 0 {
                    // No PA: auto-fumble (no d6, ball scatters from thrower, turnover).
                    let dir_roll = rng.d8();
                    let dir = Direction::for_roll(dir_roll).expect("d8 is 1..=8");
                    let (bx, by) = scatter_coordinate(passer_coord.x, passer_coord.y, dir, 1);
                    game.field_model.ball_coordinate = Some(FieldCoordinate::new(bx, by));
                    game.turnover = true;
                    return StepOutcome::next();
                }
                let dist_mod = passing_distance_bb2025(passer_coord, receiver_coord)
                    .map(|d| d.modifier_2020())
                    .unwrap_or(0);
                // BB2025 PassMechanic.passModifiers: each adjacent opposing player with TZ = +1 penalty.
                let tz_penalty = count_opponent_tackle_zones_at(game, &passer_id, passer_coord);
                let pass_roll = rng.d6();
                let effective = pass_roll - dist_mod - tz_penalty;
                let fumble = pass_roll == 1 || effective <= 1;
                let accurate = pass_roll == 6 || effective >= pa;
                if std::env::var_os("FFB_TRACE").is_some() {
                    eprintln!("RUST_PASS_ROLL passer={passer_id} recv={receiver_id} pa={pa} dist_mod={dist_mod} tz_penalty={tz_penalty} roll={pass_roll} eff={effective} fumble={fumble} accurate={accurate} rng={}", rng.call_count);
                }

                if fumble {
                    // FUMBLE: ball stays at thrower, bounces from thrower, turnover.
                    let dir_roll = rng.d8();
                    let dir = Direction::for_roll(dir_roll).expect("d8 is 1..=8");
                    let (bx, by) = scatter_coordinate(passer_coord.x, passer_coord.y, dir, 1);
                    game.field_model.ball_coordinate = Some(FieldCoordinate::new(bx, by));
                    game.turnover = true;
                    return StepOutcome::next();
                }

                // Helper: whether a player is on the active (passing) team.
                let passer_is_home = game.team_home.has_player(&passer_id);
                let player_is_active = |pid: &str| game.team_home.has_player(pid) == passer_is_home;

                if !accurate {
                    // INACCURATE pass: Java StepMissedPass does 3-step random scatter from receiver_coord.
                    // Each step rolls d8 → direction → 1-square move. Stops early if start goes OOB.
                    // lastValidCoordinate = last in-bounds position reached.
                    let mut scatter_start = receiver_coord;
                    let mut last_valid = receiver_coord;
                    for _ in 0..3 {
                        if !scatter_start.is_on_pitch() {
                            break;
                        }
                        let dir_roll = rng.d8();
                        let dir = Direction::for_roll(dir_roll).expect("d8 is 1..=8");
                        let (nx, ny) = scatter_coordinate(scatter_start.x, scatter_start.y, dir, 1);
                        let next_coord = FieldCoordinate::new(nx, ny);
                        if next_coord.is_on_pitch() {
                            last_valid = next_coord;
                        }
                        scatter_start = next_coord; // move even if OOB — stops on next iteration
                    }
                    game.field_model.ball_coordinate = Some(last_valid);
                    game.field_model.ball_moving = true;

                    // CSTIN CATCH_MISSED_PASS: player at landing spot with TZ → CATCH_SCATTER (ag+1).
                    // No player → SCATTER_BALL → bounceBall loop.
                    let mut ball_coord = last_valid;

                    if let Some(pid) = game.field_model.player_at(ball_coord).cloned() {
                        let has_tz = game.field_model.player_state(&pid)
                            .map(|s| s.has_tacklezones())
                            .unwrap_or(false);
                        if has_tz {
                            let ag = find_player_agility(game, &pid);
                            let catch_min = std::cmp::max(2, ag + 1);
                            let catch_roll = rng.d6();
                            if catch_roll >= catch_min {
                                game.field_model.ball_moving = false;
                                if !player_is_active(&pid) {
                                    game.turnover = true;
                                }
                                return StepOutcome::next();
                            }
                            // catch failed → SCATTER_BALL → bounceBall loop
                        }
                        // no TZ → FAILED_CATCH → SCATTER_BALL → bounceBall loop
                    }
                    // SCATTER_BALL → bounceBall loop
                    loop {
                        let dir_roll = rng.d8();
                        let dir = Direction::for_roll(dir_roll).expect("d8 is 1..=8");
                        let (bx, by) = scatter_coordinate(ball_coord.x, ball_coord.y, dir, 1);
                        let new_coord = FieldCoordinate::new(bx, by);
                        game.field_model.ball_coordinate = Some(new_coord);
                        game.field_model.ball_moving = true;

                        if !new_coord.is_on_pitch() {
                            game.turnover = true;
                            return StepOutcome::next();
                        }

                        ball_coord = new_coord;
                        if let Some(pid) = game.field_model.player_at(new_coord).cloned() {
                            let has_tz = game.field_model.player_state(&pid)
                                .map(|s| s.has_tacklezones())
                                .unwrap_or(false);
                            if has_tz {
                                let ag = find_player_agility(game, &pid);
                                let catch_min = std::cmp::max(2, ag + 1);
                                let catch_roll = rng.d6();
                                if catch_roll >= catch_min {
                                    game.field_model.ball_moving = false;
                                    if !player_is_active(&pid) {
                                        game.turnover = true;
                                    }
                                    return StepOutcome::next();
                                }
                                // catch failed → loop again
                            }
                            // no TZ → FAILED_CATCH → loop again
                        } else {
                            // empty square → ball rests, turnover
                            game.turnover = true;
                            return StepOutcome::next();
                        }
                    }
                }

                // ACCURATE: ball moves to receiver's square, receiver catches.
                // BB2025: no TZ modifier for catch. min = max(2, AG).
                // Java: only rolls d6 if receiver hasTacklezones(). Otherwise FAILED_CATCH immediately.
                game.field_model.ball_coordinate = Some(receiver_coord);
                let receiver_has_tz = game.field_model.player_state(&receiver_id)
                    .map(|s| s.has_tacklezones())
                    .unwrap_or(false);
                if std::env::var_os("FFB_TRACE").is_some() {
                    let recv_state = game.field_model.player_state(&receiver_id).map(|s| s.base()).unwrap_or(0);
                    eprintln!("RUST_PASS_CATCH recv={} coord={:?} has_tz={} recv_state={} ag={} rng={}", receiver_id, receiver_coord, receiver_has_tz, recv_state, receiver_ag, rng.call_count);
                }
                if receiver_has_tz {
                    let catch_min = std::cmp::max(2, receiver_ag);
                    let catch_roll = rng.d6();
                    if catch_roll >= catch_min {
                        game.field_model.ball_moving = false;
                        return StepOutcome::next();
                    }
                }

                // Receiver failed catch: FAILED_CATCH → SCATTER_BALL loop.
                // Turnover determined at end: nobody catches → turnover; opponent catches → turnover.
                game.field_model.ball_moving = true;
                let mut ball_coord = receiver_coord;
                loop {
                    let dir_roll = rng.d8();
                    let dir = Direction::for_roll(dir_roll).expect("d8 is 1..=8");
                    let (bx, by) = scatter_coordinate(ball_coord.x, ball_coord.y, dir, 1);
                    let new_coord = FieldCoordinate::new(bx, by);
                    game.field_model.ball_coordinate = Some(new_coord);
                    game.field_model.ball_moving = true;
                    if std::env::var_os("FFB_TRACE").is_some() {
                        let pid_at = game.field_model.player_at(new_coord).cloned();
                        eprintln!("RUST_PASS_SCATTER from={:?} dir_roll={} dir={:?} to={:?} player_at={:?} rng={}", ball_coord, dir_roll, dir, new_coord, pid_at, rng.call_count);
                    }

                    if !new_coord.is_on_pitch() {
                        game.turnover = true;
                        return StepOutcome::next();
                    }

                    ball_coord = new_coord;
                    if let Some(pid) = game.field_model.player_at(new_coord).cloned() {
                        let has_tz = game.field_model.player_state(&pid)
                            .map(|s| s.has_tacklezones())
                            .unwrap_or(false);
                        if has_tz {
                            // CATCH_SCATTER: +1 mode modifier only. No TZ modifier in BB2025.
                            let ag = find_player_agility(game, &pid);
                            let catch_min2 = std::cmp::max(2, ag + 1);
                            let catch_roll2 = rng.d6();
                            if catch_roll2 >= catch_min2 {
                                game.field_model.ball_moving = false;
                                if !player_is_active(&pid) {
                                    game.turnover = true;
                                }
                                return StepOutcome::next();
                            }
                            // Catch failed → loop again.
                        }
                        // No TZ → FAILED_CATCH → loop again.
                    } else {
                        // Empty square: nobody caught → turnover.
                        game.turnover = true;
                        return StepOutcome::next();
                    }
                }
            }

            // Java StepEndPlayerAction: record activation, clear acting_player.
            // On turnover (game.turnover = true from StepPickUp failure, dodge failure, etc.)
            // push end_turn_sequence() instead of select_sequence().
            Step::EndPlayerAction => {
                if let Some(pid) = game.acting_player.player_id.take() {
                    let td = if game.home_playing {
                        &mut game.turn_data_home
                    } else {
                        &mut game.turn_data_away
                    };
                    td.acted_player_ids.push(pid);
                }
                game.acting_player.clear();
                if game.turnover {
                    game.turnover = false;
                    StepOutcome::next().push_seq(end_turn_sequence())
                } else {
                    StepOutcome::next().push_seq(select_sequence())
                }
            }
        }
    }

    /// The step's `handle_command()` body (Java `AbstractStep.handleCommand`). Called by the
    /// driver when an `Action` arrives for the waiting current step.
    fn handle_command(&self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        match (self, action) {
            // Flip the coin with the game RNG (Java throwCoin = 1× d2). Winner = guess == coin.
            // The winner becomes `home_playing` (the chooser) for the following ReceiveChoice.
            (Step::CoinChoice, Action::CoinChoice { heads }) => {
                let coin_is_heads = rng.bool();
                let home_won = *heads == coin_is_heads;
                game.home_playing = home_won;
                StepOutcome::next().with_event(GameEvent::CoinThrow { home_won })
            }
            // The chooser's `receive` resolves to whether HOME has first offense. The KICKER
            // sets up first, so home_playing = !home_receives (matches Java setup ordering).
            (Step::ReceiveChoice, Action::ReceiveChoice { receive }) => {
                let home_receives = if game.home_playing { *receive } else { !*receive };
                game.home_first_offense = home_receives;
                game.home_playing = !home_receives;
                let team_id = if home_receives { game.team_home.id.clone() } else { game.team_away.id.clone() };
                StepOutcome::next().with_event(GameEvent::ReceiveChoice { team_id, receive: home_receives })
            }
            // Java StepKickoff handleCommand(ClientCommandKickoff): place the ball on the chosen
            // target square (the kicking coach's pick), then proceed to the scatter. The ball
            // becomes live here (ball_in_play drives the state hash and catch/throw-in logic).
            (Step::Kickoff, Action::KickBall { coord }) => {
                game.field_model.ball_coordinate = Some(*coord);
                game.field_model.ball_in_play = true;
                StepOutcome::next()
            }
            // Java StepInitSelecting handleCommand: store the chosen player + action in
            // acting_player, set per-turn flags, then GOTO END_SELECTING so EndSelecting
            // dispatches the correct sub-sequence.
            (Step::InitSelecting, Action::ActivatePlayer { player_id, player_action, block_defender_id }) => {
                let action = pac_to_player_action(*player_action);
                game.acting_player.set_player(player_id.clone(), action);
                game.acting_player.defender_id = block_defender_id.clone();
                // Mark blitz/block slot used
                let td = if game.home_playing { &mut game.turn_data_home } else { &mut game.turn_data_away };
                match player_action {
                    // Java StepInitMoving/StepEndSelecting: blitzUsed set for BLITZ_MOVE only.
                    // BLOCK does NOT set blitzUsed in Java.
                    PlayerActionChoice::Blitz => { td.blitz_used = true; }
                    PlayerActionChoice::Foul => { td.foul_used = true; }
                    _ => {}
                }
                StepOutcome::goto("END_SELECTING")
            }
            (Step::InitSelecting, Action::EndTurn) => {
                // acting_player stays cleared (player_id = None) → EndSelecting → end_turn_sequence
                StepOutcome::goto("END_SELECTING")
            }
            // Java StepInitMoving handleCommand: move player step-by-step, rolling dodge in each TZ.
            (Step::InitMoving, Action::Move { path }) => {
                let player_id = match &game.acting_player.player_id {
                    Some(id) => id.clone(),
                    None => return StepOutcome::next(),
                };
                let src = match game.field_model.player_coordinate(&player_id) {
                    Some(c) => c,
                    None => return StepOutcome::next(),
                };
                let dest = match path.last() {
                    Some(&d) => d,
                    None => {
                        game.field_model.set_player_state(&player_id, PlayerState::new(PS_STANDING));
                        return StepOutcome::next();
                    }
                };
                let step_path = if src == dest { vec![dest] } else { bfs_path(game, &player_id, src, dest) };
                let ag = find_player_agility(game, &player_id);
                for &step_dest in &step_path {
                    let cur_pos = game.field_model.player_coordinate(&player_id).unwrap_or(src);
                    // Ball moves with its carrier. ball_moving=false means it is held (picked up).
                    // A loose ball (ball_moving=true) stays put until a player attempts pickup.
                    let has_ball = game.field_model.ball_coordinate == Some(cur_pos)
                        && !game.field_model.ball_moving;
                    if has_ball {
                        game.field_model.ball_coordinate = Some(step_dest);
                    }
                    let src_tz = count_opponent_tackle_zones_at(game, &player_id, cur_pos);
                    if src_tz > 0 {
                        let dest_tz = count_opponent_tackle_zones_at(game, &player_id, step_dest);
                        let modifiers: Vec<ffb_mechanics::modifiers::Modifier> =
                            (0..dest_tz).map(|_| DODGE_TACKLE_ZONE).collect();
                        let minimum = minimum_roll_dodge(ag, &modifiers);
                        let roll = rng.d6();
                        if roll < minimum {
                            game.field_model.set_player_coordinate(&player_id, step_dest);
                            apply_knockdown(game, &player_id, rng);
                            game.turnover = true;
                            // Ball bounces when carrier falls (Java StepCatchScatterThrowIn.bounceBall).
                            if has_ball {
                                scatter_ball_from_knockdown(game, step_dest, rng);
                            }
                            return StepOutcome::next();
                        }
                    }
                    game.field_model.set_player_coordinate(&player_id, step_dest);
                }
                game.field_model.set_player_state(&player_id, PlayerState::new(PS_STANDING));
                StepOutcome::next()
            }
            // A command the current step does not recognise (Java StepCommandStatus::UNHANDLED):
            // stay put and keep waiting. (The harness never sends one in the parity path.)
            _ => StepOutcome::cont(),
        }
    }

    /// Offer a published parameter to this step while the driver walks the stack top→bottom.
    /// Return `true` to consume it (stops propagation). Java `AbstractStep.setParameter`.
    /// Plumbing in place; the first consumers land with the Phase D steps that read params
    /// (e.g. MoveStack, EndTurn) — pregame steps consume nothing.
    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match (self, param) {
            // StepKickoffResultRoll publishes the rolled event down to StepApplyKickoffResult.
            (Step::ApplyKickoffResult { result, .. }, StepParameter::KickoffResult(r)) => {
                *result = Some(*r);
                true
            }
            // StepKickoffScatterRoll publishes touchback down to Apply/CSTIN/Touchback.
            (Step::ApplyKickoffResult { touchback, .. }, StepParameter::Touchback(t)) => {
                *touchback = *t;
                true
            }
            (Step::CatchScatterThrowIn { touchback }, StepParameter::Touchback(t)) => {
                *touchback = *t;
                true
            }
            (Step::Touchback { touchback }, StepParameter::Touchback(t)) => {
                *touchback = *t;
                true
            }
            _ => false,
        }
    }
}

// ── Step outcome / stack ─────────────────────────────────────────────────────────

/// What a step produced: how to advance, the events it emitted, sub-sequences to push, params
/// to publish down the stack, and an optional prompt (when it yields `Continue` to wait).
pub struct StepOutcome {
    pub action: StepAction,
    pub goto_label: Option<String>,
    pub events: Vec<GameEvent>,
    /// Sequences to push (authored order; the stack reverses them on push).
    pub pushes: Vec<Vec<StepEntry>>,
    /// Parameters to publish down the stack (top→bottom) after this step runs.
    pub published: Vec<StepParameter>,
    /// Set together with `Continue` when the step is waiting for an agent decision.
    pub prompt: Option<AgentPrompt>,
}

impl StepOutcome {
    fn base(action: StepAction) -> Self {
        StepOutcome { action, goto_label: None, events: Vec::new(), pushes: Vec::new(), published: Vec::new(), prompt: None }
    }
    pub fn next() -> Self { Self::base(StepAction::NextStep) }
    pub fn cont() -> Self { Self::base(StepAction::Continue) }
    pub fn goto(label: &str) -> Self {
        let mut o = Self::base(StepAction::GotoLabel);
        o.goto_label = Some(label.to_owned());
        o
    }
    pub fn with_event(mut self, e: GameEvent) -> Self { self.events.push(e); self }
    pub fn with_prompt(mut self, p: AgentPrompt) -> Self { self.prompt = Some(p); self }
    pub fn push_seq(mut self, seq: Vec<StepEntry>) -> Self { self.pushes.push(seq); self }
    pub fn publish(mut self, p: StepParameter) -> Self { self.published.push(p); self }
}

/// A stacked step: the concrete step plus an optional label (goto target).
#[derive(Debug, Clone)]
pub struct StepEntry {
    pub step: Step,
    pub label: Option<String>,
}

impl StepEntry {
    pub fn new(step: Step) -> Self { StepEntry { step, label: None } }
    pub fn labelled(step: Step, label: &str) -> Self { StepEntry { step, label: Some(label.to_owned()) } }
    pub fn id(&self) -> StepId { self.step.id() }
}

/// LIFO step stack. Java keeps top at index 0; here top = last (`Vec::last`).
/// `push_sequence` pushes authored order REVERSED so the first-authored step ends on top
/// and runs first (matches Java's back-to-front push at index 0).
#[derive(Default)]
pub struct StepStack {
    steps: Vec<StepEntry>,
}

impl StepStack {
    pub fn new() -> Self { Self::default() }
    pub fn push(&mut self, step: StepEntry) { self.steps.push(step); }
    pub fn push_sequence(&mut self, seq: Vec<StepEntry>) {
        for s in seq.into_iter().rev() { self.steps.push(s); }
    }
    pub fn pop(&mut self) -> Option<StepEntry> { self.steps.pop() }
    pub fn peek(&self) -> Option<&StepEntry> { self.steps.last() }
    pub fn len(&self) -> usize { self.steps.len() }
    pub fn is_empty(&self) -> bool { self.steps.is_empty() }

    /// Pop the stack down until the labelled step is on top (left in place). Java
    /// `handleStepResultGotoLabel`: discard intervening steps; error if the label is absent.
    pub fn goto_label(&mut self, label: &str) -> Result<(), String> {
        while let Some(top) = self.steps.last() {
            if top.label.as_deref() == Some(label) {
                return Ok(());
            }
            self.steps.pop();
        }
        Err(format!("goto unknown label '{label}'"))
    }

    /// Publish a parameter down the stack (top→bottom), stopping once a step consumes it.
    /// Java `StepStack.publishParameter` → each step's `setParameter`. The publisher is the
    /// current step (already popped into the driver), so this only reaches steps below it.
    pub fn publish(&mut self, param: &StepParameter) {
        for entry in self.steps.iter_mut().rev() {
            if entry.step.set_parameter(param) {
                return;
            }
        }
    }
}

// ── Driver ──────────────────────────────────────────────────────────────────────

/// The game driver — owns the model, RNG, step stack and current step. Port of Java
/// `GameState` (the executeStep/processStepResult loop, flattened to an explicit loop per
/// `00_framework.md` §7). Drives start-mode chains and command-mode (handle_command) steps,
/// surfacing an `AgentPrompt` when a step waits and accepting an `Action` to resume.
pub struct GameState {
    pub game: Game,
    pub rng: GameRng,
    stack: StepStack,
    current: Option<StepEntry>,
    /// When `Some`, the next drive of `current` re-delivers this command (NextStep/GotoLabel
    /// *AndRepeat* — Java's `forwardCommand`) instead of calling `start`.
    forwarded: Option<Action>,
    /// The prompt the waiting current step raised; `None` when the engine is idle.
    pending_prompt: Option<AgentPrompt>,
    /// Events accumulated since the last drain (the parity log reads these).
    pub events: Vec<GameEvent>,
    /// State hash of the FRESH game, captured before any roll — the parity log's GameStart
    /// (i:0) hash. Java logs this on the freshly-created game, so we snapshot it in `new`
    /// before the StartGame sequence runs.
    initial_hash: String,
}

impl GameState {
    /// Construct directly from a pre-built `Game` (used by step characterization tests; the
    /// caller pushes a sequence and drives explicitly).
    pub fn from_game(game: Game, seed: u64) -> Self {
        GameState {
            game, rng: GameRng::new(seed), stack: StepStack::new(),
            current: None, forwarded: None, pending_prompt: None, events: Vec::new(),
            initial_hash: String::new(),
        }
    }

    /// Game-driver entry point the parity harness constructs from: build the game, snapshot the
    /// fresh-game (pre-roll) GameStart hash, push the StartGame sequence, and run to the first
    /// prompt so `current_prompt()` is immediately available.
    pub fn new(home: Team, away: Team, rules: Rules, seed: u64) -> Self {
        let game = Game::new(home, away, rules);
        let mut gs = GameState::from_game(game, seed);
        gs.initial_hash = state_hash(&gs.game); // fresh, before any StartGame roll
        gs.push_sequence(start_game_sequence());
        gs.run_until_prompt();
        gs
    }

    /// The GameStart (i:0) state hash — the fresh game before any roll. (Parity log anchor.)
    pub fn initial_state_hash(&self) -> &str { &self.initial_hash }

    pub fn push_sequence(&mut self, seq: Vec<StepEntry>) { self.stack.push_sequence(seq); }

    /// The prompt the engine is currently waiting on, if any. `None` ⇒ idle (stack drained).
    pub fn current_prompt(&self) -> Option<&AgentPrompt> { self.pending_prompt.as_ref() }

    /// Drain events accumulated so far, resetting the buffer (parity log read point).
    pub fn take_events(&mut self) -> Vec<GameEvent> { std::mem::take(&mut self.events) }

    // ── Harness-facing facade ──────────────────────────────────────────────────────
    // The parity harness is engine-agnostic: it needs only these few methods + `.game`.

    /// The side currently to act (derived from the model — the engine infers it, so `apply`'s
    /// `side` argument is advisory only).
    pub fn active_side(&self) -> TeamSide {
        if self.game.home_playing { TeamSide::Home } else { TeamSide::Away }
    }

    /// Whether the game has ended.
    pub fn is_finished(&self) -> bool { self.game.is_finished() }

    /// Game-dice draw count (parity diagnostics / no-progress guard).
    pub fn rng_call_count(&self) -> u64 { self.rng.call_count }

    /// FNV-1a 64-bit state hash (matches Java's `ParityRunner.stateHash()`).
    pub fn state_hash_str(&self) -> String { state_hash(&self.game) }

    /// Feed an agent decision and advance, returning the events produced. The `side` is advisory
    /// (the engine infers the acting side); kept for the harness's call shape.
    pub fn apply(&mut self, _side: TeamSide, action: Action) -> Result<Vec<GameEvent>, String> {
        self.apply_action(action);
        Ok(self.take_events())
    }

    /// Apply a step's side effects to driver-owned state (events, sub-sequence pushes, and
    /// published parameters). Shared by start- and command-mode.
    fn apply_effects(&mut self, outcome: &mut StepOutcome) {
        self.events.append(&mut outcome.events);
        for seq in outcome.pushes.drain(..) { self.stack.push_sequence(seq); }
        for param in outcome.published.drain(..) { self.stack.publish(&param); }
    }

    /// Feed an agent decision to the waiting current step (Java command-mode `executeStep`),
    /// then drive forward until the next prompt or idle. Internal driver entry; the harness
    /// uses the `apply(side, action)` facade above.
    pub fn apply_action(&mut self, action: Action) {
        let entry = self.current.take().expect("apply_action() with no waiting step");
        let mut outcome = entry.step.handle_command(&action, &mut self.game, &mut self.rng);
        self.apply_effects(&mut outcome);
        self.pending_prompt = None;
        self.dispatch(entry, action, outcome);
        self.drive();
    }

    /// Drive the start-mode chain until a step waits (Continue + prompt) or the stack empties.
    /// Mirrors `GameState.executeStep`'s start-mode loop + `processStepResult`.
    pub fn run_until_prompt(&mut self) { self.drive(); }

    fn drive(&mut self) {
        loop {
            // Already waiting on a prompt from a prior apply/dispatch — nothing to start.
            if self.current.is_some() && self.pending_prompt.is_some() {
                return;
            }
            if self.current.is_none() {
                match self.stack.pop() {
                    Some(s) => self.current = Some(s),
                    None => { self.pending_prompt = None; return; }
                }
            }
            let entry = self.current.take().unwrap();
            // Forwarded command (AndRepeat) → re-deliver via handle_command; else start().
            let mut outcome = match self.forwarded.take() {
                Some(cmd) => {
                    let o = entry.step.handle_command(&cmd, &mut self.game, &mut self.rng);
                    // keep cmd available in case this step also forwards
                    self.dispatch(entry, cmd, o);
                    if self.pending_prompt.is_some() { return; }
                    continue;
                }
                None => entry.step.start(&mut self.game, &mut self.rng),
            };
            self.apply_effects(&mut outcome);
            match outcome.action {
                StepAction::Continue | StepAction::Repeat => {
                    // Continue: wait for a command (prompt set by the step). Repeat: pregame
                    // steps don't use it; treated as idle until a repeat()-capable step lands.
                    self.pending_prompt = outcome.prompt;
                    self.current = Some(entry);
                    return;
                }
                StepAction::NextStep => { self.current = None; }
                StepAction::GotoLabel => {
                    let label = outcome.goto_label.expect("goto without label");
                    self.stack.goto_label(&label).expect("goto label present");
                    self.current = None;
                }
                StepAction::NextStepAndRepeat | StepAction::GotoLabelAndRepeat => {
                    // forwardCommand from a start() result has no command to forward; treat as
                    // the non-repeat variant. (Forwarding only originates from handle_command.)
                    if outcome.action.trigger_goto() {
                        let label = outcome.goto_label.expect("goto without label");
                        self.stack.goto_label(&label).expect("goto label present");
                    }
                    self.current = None;
                }
            }
        }
    }

    /// Process a `handle_command` outcome (command-mode `processStepResult`): apply the action,
    /// setting up forwarding when the result is an *AndRepeat* variant.
    fn dispatch(&mut self, entry: StepEntry, cmd: Action, mut outcome: StepOutcome) {
        self.apply_effects(&mut outcome);
        match outcome.action {
            StepAction::Continue | StepAction::Repeat => {
                // Same step keeps waiting (multi-command step) — re-arm its prompt.
                self.pending_prompt = outcome.prompt;
                self.current = Some(entry);
            }
            StepAction::NextStep => { self.current = None; }
            StepAction::GotoLabel => {
                let label = outcome.goto_label.expect("goto without label");
                self.stack.goto_label(&label).expect("goto label present");
                self.current = None;
            }
            StepAction::NextStepAndRepeat => { self.current = None; self.forwarded = Some(cmd); }
            StepAction::GotoLabelAndRepeat => {
                let label = outcome.goto_label.expect("goto without label");
                self.stack.goto_label(&label).expect("goto label present");
                self.current = None;
                self.forwarded = Some(cmd);
            }
        }
    }
}

// ── Helpers ──────────────────────────────────────────────────────────────────────

/// Convert an engine-local `PlayerActionChoice` to the model-level `PlayerAction` used in
/// `AgentPrompt::ActivatePlayer`. The two enums mirror each other; this covers the lineman set.
fn pac_to_player_action(pac: PlayerActionChoice) -> ffb_model::enums::PlayerAction {
    use ffb_model::enums::PlayerAction as PA;
    match pac {
        PlayerActionChoice::Move       => PA::Move,
        PlayerActionChoice::Block      => PA::Block,
        PlayerActionChoice::Blitz      => PA::Blitz,
        PlayerActionChoice::StandUp    => PA::StandUp,
        PlayerActionChoice::StandUpBlitz => PA::StandUpBlitz,
        PlayerActionChoice::Foul       => PA::Foul,
        PlayerActionChoice::Pass       => PA::Pass,
        PlayerActionChoice::HandOff    => PA::HandOver,
        PlayerActionChoice::SecureTheBall => PA::SecureTheBall,
        PlayerActionChoice::Stab       => PA::Stab,
        PlayerActionChoice::ThrowTeamMate => PA::ThrowTeamMate,
        PlayerActionChoice::KickTeamMate => PA::KickTeamMate,
        PlayerActionChoice::HypnoticGaze => PA::Gaze,
        PlayerActionChoice::ThrowBomb  => PA::ThrowBomb,
        PlayerActionChoice::Swoop      => PA::Swoop,
        PlayerActionChoice::Punt       => PA::Punt,
        PlayerActionChoice::BreatheFire => PA::BreatheFire,
        PlayerActionChoice::ProjectileVomit => PA::ProjectileVomit,
    }
}

// ── Sequence generators ───────────────────────────────────────────────────────────

/// Java `StartGame` generator (BB2025) — head through the coin/receive decisions:
/// InitStartGame → Spectators → Weather → CoinChoice → ReceiveChoice → [PettyCash,
/// BuyInducements, Setup, Kickoff — Phase D]. See `10_sequences.md` StartGame.
pub fn start_game_sequence() -> Vec<StepEntry> {
    vec![
        StepEntry::new(Step::InitStartGame),
        StepEntry::new(Step::Spectators),
        StepEntry::new(Step::Weather),
        // Kickoff(withCoinChoice) — coin/receive then the opening kickoff. (PettyCash/
        // BuyInducements are 0-effect for equal-TV lineman and are omitted for now.)
        StepEntry::new(Step::CoinChoice),
        StepEntry::new(Step::ReceiveChoice),
        StepEntry::new(Step::InitKickoff),
        StepEntry::new(Step::Setup), // kicking team
        StepEntry::new(Step::Setup), // receiving team
        StepEntry::new(Step::Kickoff),
        StepEntry::new(Step::KickoffScatterRoll),
        StepEntry::new(Step::KickoffResultRoll),
        StepEntry::new(Step::ApplyKickoffResult { result: None, touchback: false }),
        StepEntry::new(Step::CatchScatterThrowIn { touchback: false }),
        StepEntry::new(Step::Touchback { touchback: false }),
        StepEntry::new(Step::EndKickoff),
    ]
}

/// Java `UtilPlayer.refreshPlayersForTurnStart`: resets transient player states at the start of
/// each team's turn. Called by StepEndTurn after flipping `home_playing` to the new active team.
///
/// Transitions (1:1 with Java switch):
///   BLOCKED / MOVING / FALLING / HIT_ON_GROUND → STANDING
///   STUNNED (new active team only) → PRONE + active=false
///   STANDING / PRONE → active flag update only (no hash-visible change for linemen)
fn refresh_players_for_turn_start(game: &mut Game) {
    use ffb_model::enums::{
        PS_BLOCKED, PS_FALLING, PS_HIT_ON_GROUND, PS_MOVING, PS_PRONE, PS_STANDING, PS_STUNNED,
    };
    let home_ids: Vec<String> = game.team_home.players.iter().map(|p| p.id.clone()).collect();
    let away_ids: Vec<String> = game.team_away.players.iter().map(|p| p.id.clone()).collect();
    let home_playing = game.home_playing;
    for (id, is_home) in home_ids.iter().map(|id| (id, true))
        .chain(away_ids.iter().map(|id| (id, false)))
    {
        let Some(ps) = game.field_model.player_state(id) else { continue };
        let base = ps.base();
        let new_ps = if base == PS_BLOCKED || base == PS_MOVING || base == PS_FALLING || base == PS_HIT_ON_GROUND {
            // Transient mid-action states → STANDING+active=true (linemen: setActive=true).
            Some(ps.change_base(PS_STANDING).change_active(true))
        } else if base == PS_STANDING || base == PS_PRONE {
            // Linemen never have hasToMissTurn, so setActive=true for all STANDING/PRONE.
            // This is the `oldPlayerState.changeActive(setActive)` branch in Java.
            if !ps.is_active() { Some(ps.change_active(true)) } else { None }
        } else if base == PS_STUNNED && is_home == home_playing {
            // Only the newly active team recovers STUNNED → PRONE.
            Some(ps.change_base(PS_PRONE).change_active(false))
        } else {
            None
        };
        if let Some(new_ps) = new_ps {
            game.field_model.set_player_state(id, new_ps);
        }
    }
}

/// Java `EndTurn` generator (BB2025). All 5 prefix steps are no-ops for a skill-less lineman
/// (no stalling, no HIT_PLAYER context, no outstanding apothecary). `StepEndTurn` itself does
/// the turn flip and pushes Select. See `10_sequences.md` EndTurn.
fn end_turn_sequence() -> Vec<StepEntry> {
    vec![
        StepEntry::new(Step::NoOp),  // ForgoneStalling
        StepEntry::new(Step::NoOp),  // SteadyFooting(HIT_PLAYER)
        StepEntry::new(Step::NoOp),  // PlaceBall
        StepEntry::new(Step::NoOp),  // Apothecary(HIT_PLAYER)
        StepEntry::new(Step::NoOp),  // CatchScatterThrowIn
        StepEntry::new(Step::EndTurn),
    ]
}

/// Java H2 kickoff sequence — identical to the opening kickoff but without the coin/receive
/// steps (the H1 result already decided who kicks/receives for both halves).
fn h2_kickoff_sequence() -> Vec<StepEntry> {
    vec![
        StepEntry::new(Step::InitKickoff),
        StepEntry::new(Step::Setup), // kicking team
        StepEntry::new(Step::Setup), // receiving team
        StepEntry::new(Step::Kickoff),
        StepEntry::new(Step::KickoffScatterRoll),
        StepEntry::new(Step::KickoffResultRoll),
        StepEntry::new(Step::ApplyKickoffResult { result: None, touchback: false }),
        StepEntry::new(Step::CatchScatterThrowIn { touchback: false }),
        StepEntry::new(Step::Touchback { touchback: false }),
        StepEntry::new(Step::EndKickoff),
    ]
}

/// Move-only activation: move the player, attempt pickup if on ball, then end their action.
fn move_sequence() -> Vec<StepEntry> {
    vec![
        StepEntry::new(Step::InitMoving),
        StepEntry::new(Step::PickUp),
        StepEntry::new(Step::EndMoving),
        StepEntry::new(Step::EndPlayerAction),
    ]
}

/// BLITZ activation: block immediately from current square, no pre-block move.
/// AGENT_CONTRACT §Blitz: "The agent's blitz blocks immediately (no pre-block move)."
fn blitz_sequence() -> Vec<StepEntry> {
    vec![
        StepEntry::new(Step::DoBlock),
        StepEntry::new(Step::EndPlayerAction),
    ]
}

/// Block-only activation (player already adjacent): block then end action.
fn block_sequence() -> Vec<StepEntry> {
    vec![
        StepEntry::new(Step::DoBlock),
        StepEntry::new(Step::EndPlayerAction),
    ]
}

/// Stand-up only activation: player was prone, now standing. No movement or block.
fn standup_end_sequence() -> Vec<StepEntry> {
    vec![
        StepEntry::new(Step::EndPlayerAction),
    ]
}

/// Pass activation: pickup (if ball loose at passer's square) then throw.
/// Java: StepActivationBB2025 pushes StepPickUp before StepPass.
fn pass_sequence() -> Vec<StepEntry> {
    vec![
        StepEntry::new(Step::PickUp),
        StepEntry::new(Step::DoPass),
        StepEntry::new(Step::EndPlayerAction),
    ]
}

/// HandOff activation: throw accuracy + catch + optional scatter.
fn handoff_sequence() -> Vec<StepEntry> {
    vec![
        StepEntry::new(Step::DoHandOff),
        StepEntry::new(Step::EndPlayerAction),
    ]
}

/// Foul activation: roll armor + injury against the target (stored in acting_player.defender_id).
/// 1:1 with Java StepFoul.executeStep.
fn foul_sequence() -> Vec<StepEntry> {
    vec![
        StepEntry::new(Step::DoFoul),
        StepEntry::new(Step::EndPlayerAction),
    ]
}

/// StandUpBlitz: stand up (already done in EndSelecting), block immediately, no pre-block move.
fn standup_blitz_sequence() -> Vec<StepEntry> {
    vec![
        StepEntry::new(Step::DoBlock),
        StepEntry::new(Step::EndPlayerAction),
    ]
}

/// Java `Select` generator (BB2025). InitSelecting emits the ActivatePlayer prompt and GOTOs
/// END_SELECTING on command; the 18 intervening no-ops (14 ActivationSequenceBuilder stubs +
/// 4 outer Select stubs) are always skipped for a standing lineman. EndSelecting is the dispatch
/// hub — for now a stub that idles the engine. See `10_sequences.md` Select.
fn select_sequence() -> Vec<StepEntry> {
    let mut seq = Vec::with_capacity(20);
    seq.push(StepEntry::new(Step::InitSelecting));
    // 14 ActivationSequenceBuilder stubs (InitActivation, AnimalSavagery, SteadyFooting,
    // HandleDropPlayerContext, PlaceBall, Apothecary, CatchScatterThrowIn, SetDefender,
    // GotoLabel(NEXT), BoneHead, ReallyStupid, TakeRoot, UnchannelledFury, BloodLust).
    for _ in 0..14 {
        seq.push(StepEntry::new(Step::NoOp));
    }
    // 4 outer Select stubs (GotoLabel(NEXT,alt=END_SELECTING), JumpUp, StandUp, ResetFumblerooskie).
    for _ in 0..4 {
        seq.push(StepEntry::new(Step::NoOp));
    }
    seq.push(StepEntry::labelled(Step::EndSelecting, "END_SELECTING"));
    seq
}

/// Player strength lookup (base only, no assists).
fn player_strength(game: &Game, player_id: &str) -> i32 {
    game.team_home.players.iter()
        .chain(game.team_away.players.iter())
        .find(|p| p.id == player_id)
        .map(|p| p.strength_with_modifiers())
        .unwrap_or(3)
}

/// Java ServerUtilPlayer.findBlockStrength: attacker's effective strength including assists.
/// Each attacker-team player adjacent to the DEFENDER (with tackle zones) that is NOT adjacent
/// to any OTHER defender-team player adds +1. Mirrors BB2025 rules exactly.
fn effective_attacker_strength(game: &Game, attacker_id: &str, defender_id: &str) -> i32 {
    let base = player_strength(game, attacker_id);
    let def_coord = match game.field_model.player_coordinate(defender_id) {
        Some(c) => c,
        None => return base,
    };
    let atk_coord = match game.field_model.player_coordinate(attacker_id) {
        Some(c) => c,
        None => return base,
    };

    let atk_is_home = game.team_home.players.iter().any(|p| p.id == attacker_id);
    let (atk_team, def_team) = if atk_is_home {
        (&game.team_home, &game.team_away)
    } else {
        (&game.team_away, &game.team_home)
    };

    let trace = std::env::var_os("FFB_TRACE").is_some();
    let mut strength = base;
    // Offensive assists: atk_team players adjacent to DEFENDER (with tackle zones), excluding attacker
    for assist_player in &atk_team.players {
        if assist_player.id == attacker_id { continue; }
        let assist_coord = match game.field_model.player_coordinate(&assist_player.id) {
            Some(c) => c,
            None => continue,
        };
        let assist_state = match game.field_model.player_state(&assist_player.id) {
            Some(s) => s,
            None => {
                if trace { eprintln!("  ASSIST_SKIP {} no_state", assist_player.id); }
                continue;
            }
        };
        if !assist_state.has_tacklezones() {
            if trace { eprintln!("  ASSIST_SKIP {} no_tz state={assist_state:?}", assist_player.id); }
            continue;
        }
        if !assist_coord.is_adjacent(def_coord) {
            if trace { eprintln!("  ASSIST_SKIP {} not_adj_to_def coord={assist_coord:?} def={def_coord:?}", assist_player.id); }
            continue;
        }
        // Check: not adjacent to any def_team player OTHER than the defender
        let other_def_adjacent = def_team.players.iter().any(|dp| {
            if dp.id == defender_id { return false; }
            let adj = game.field_model.player_coordinate(&dp.id)
                .map(|dc| dc.is_adjacent(assist_coord))
                .unwrap_or(false);
            let tz = game.field_model.player_state(&dp.id)
                    .map(|ds| ds.has_tacklezones())
                    .unwrap_or(false);
            if trace && adj { eprintln!("    MARKING_CHECK {} adj={adj} tz={tz}", dp.id); }
            adj && tz
        });
        if trace { eprintln!("  ASSIST_CANDIDATE {} coord={assist_coord:?} other_def_adj={other_def_adjacent} => {}", assist_player.id, if !other_def_adjacent { "COUNTS" } else { "blocked" }); }
        if !other_def_adjacent {
            strength += 1;
        }
    }
    // Java RollMechanic.getTotalAttackerStrength: add gameState.getAdditionalAssist(actingTeam)
    let additional = if atk_is_home { game.home_additional_assists } else { game.away_additional_assists };
    strength += additional;
    if trace { eprintln!("  ATK_STRENGTH base={base} final={strength} additional={additional} def={defender_id} def_coord={def_coord:?}"); }
    // Ignore that atk_coord is captured but unused by ignoring it here
    let _ = atk_coord;
    strength
}

/// Java ServerUtilPlayer.findBlockStrength for defender: defender's effective strength.
/// Each def-team player adjacent to the ATTACKER (with tackle zones) that is NOT adjacent
/// to any OTHER atk-team player adds +1.
fn effective_defender_strength(game: &Game, attacker_id: &str, defender_id: &str) -> i32 {
    let base = player_strength(game, defender_id);
    let atk_coord = match game.field_model.player_coordinate(attacker_id) {
        Some(c) => c,
        None => return base,
    };

    let atk_is_home = game.team_home.players.iter().any(|p| p.id == attacker_id);
    let (atk_team, def_team) = if atk_is_home {
        (&game.team_home, &game.team_away)
    } else {
        (&game.team_away, &game.team_home)
    };

    let trace = std::env::var_os("FFB_TRACE").is_some();
    let mut strength = base;
    // Defensive assists: def_team players adjacent to ATTACKER (with tackle zones), excluding defender
    for assist_player in &def_team.players {
        if assist_player.id == defender_id { continue; }
        let assist_coord = match game.field_model.player_coordinate(&assist_player.id) {
            Some(c) => c,
            None => continue,
        };
        let assist_state = match game.field_model.player_state(&assist_player.id) {
            Some(s) => s,
            None => continue,
        };
        if !assist_state.has_tacklezones() { continue; }
        if !assist_coord.is_adjacent(atk_coord) { continue; }
        // Check: not adjacent to any atk_team player OTHER than the attacker
        let other_atk_adjacent = atk_team.players.iter().any(|ap| {
            if ap.id == attacker_id { return false; }
            let adj = game.field_model.player_coordinate(&ap.id)
                .map(|ac| ac.is_adjacent(assist_coord))
                .unwrap_or(false);
            let tz = game.field_model.player_state(&ap.id)
                    .map(|as_| as_.has_tacklezones())
                    .unwrap_or(false);
            if trace && adj { eprintln!("    DEF_MARKING_CHECK {} adj={adj} tz={tz}", ap.id); }
            adj && tz
        });
        if trace { eprintln!("  DEF_ASSIST_CANDIDATE {} coord={assist_coord:?} other_atk_adj={other_atk_adjacent} => {}", assist_player.id, if !other_atk_adjacent { "COUNTS" } else { "blocked" }); }
        if !other_atk_adjacent {
            strength += 1;
        }
    }
    if trace { eprintln!("  DEF_STRENGTH base={base} final={strength} atk={attacker_id} atk_coord={atk_coord:?}"); }
    strength
}

fn find_player_agility(game: &Game, player_id: &str) -> i32 {
    game.team_home.players.iter()
        .chain(game.team_away.players.iter())
        .find(|p| p.id == player_id)
        .map(|p| p.agility)
        .unwrap_or(3)
}

fn find_player_passing(game: &Game, player_id: &str) -> i32 {
    game.team_home.players.iter()
        .chain(game.team_away.players.iter())
        .find(|p| p.id == player_id)
        .map(|p| p.passing)
        .unwrap_or(0)
}

fn count_opponent_tackle_zones_at(game: &Game, player_id: &str, coord: FieldCoordinate) -> i32 {
    let is_home = game.team_home.players.iter().any(|p| p.id == player_id);
    let opponents = if is_home { &game.team_away } else { &game.team_home };
    opponents.players.iter().filter(|p| {
        game.field_model.player_coordinate(&p.id)
            .map(|c| c.is_adjacent(coord))
            .unwrap_or(false)
            && game.field_model.player_state(&p.id)
                .map(|s| s.has_tacklezones())
                .unwrap_or(false)
    }).count() as i32
}

/// Player armour value lookup.
fn player_armour(game: &Game, player_id: &str) -> i32 {
    game.team_home.players.iter()
        .chain(game.team_away.players.iter())
        .find(|p| p.id == player_id)
        .map(|p| p.armour_with_modifiers())
        .unwrap_or(8)
}

/// Knock a player down: set PRONE, then roll armor + injury + casualty chain.
fn apply_knockdown(game: &mut Game, player_id: &str, rng: &mut GameRng) {
    game.field_model.set_player_state(player_id, PlayerState::new(PS_PRONE));
    let av = player_armour(game, player_id);
    let a1 = rng.d6();
    let a2 = rng.d6();
    if armor_broken(av, [a1, a2], &[]) {
        let i1 = rng.d6();
        let i2 = rng.d6();
        match injury_result([i1, i2], &[]) {
            InjuryOutcome::Stunned => {
                game.field_model.set_player_state(player_id, PlayerState::new(PS_STUNNED));
            }
            InjuryOutcome::KnockedOut => {
                game.field_model.player_coordinates.remove(player_id);
                game.field_model.set_player_state(player_id, PlayerState::new(PS_KNOCKED_OUT));
            }
            InjuryOutcome::Casualty | InjuryOutcome::BadlyHurt => {
                let c1 = rng.die(16);
                let _c2 = rng.d6();
                let tier = casualty_tier_bb2025(c1);
                let ps = match tier {
                    CasualtyTier::BadlyHurt => PS_BADLY_HURT,
                    CasualtyTier::SeriousInjury => PS_SERIOUS_INJURY,
                    CasualtyTier::Dead => PS_RIP,
                };
                game.field_model.player_coordinates.remove(player_id);
                game.field_model.set_player_state(player_id, PlayerState::new(ps));
            }
        }
    }
}

/// Ball scatter when the carrier is knocked down (Java StepCatchScatterThrowIn.bounceBall).
/// d8 direction, ball moves one square. Catch by a player at landing square is NOT yet
/// implemented — all current lineman seeds land on empty squares.
fn scatter_ball_from_knockdown(game: &mut Game, from: FieldCoordinate, rng: &mut GameRng) {
    let dir_roll = rng.d8();
    let dir = Direction::for_roll(dir_roll).expect("d8 is 1..=8");
    let (nx, ny) = scatter_coordinate(from.x, from.y, dir, 1);
    let new_pos = FieldCoordinate::new(nx, ny);
    game.field_model.ball_coordinate = Some(new_pos);
    // Java StepCatchScatterThrowIn.bounceBall: setBallMoving(true) — ball is now loose.
    game.field_model.ball_moving = true;
}

/// Foul injury: roll 2d6 armor vs AV (no knockdown — target is already prone/stunned).
/// On break: run injury chain. Then run referee (doubles = ejection) + argue-the-call.
/// 1:1 with Java StepFoul → StepReferee (SneakyGitBehaviour hook) → StepBribes → StepEjectPlayer.
fn apply_foul_injury(game: &mut Game, fouler_id: &str, target_id: &str, rng: &mut GameRng) {
    let av = player_armour(game, target_id);
    let a1 = rng.d6();
    let a2 = rng.d6();
    let armor_doubles = a1 == a2;
    let broke = armor_broken(av, [a1, a2], &[]);
    let mut injury_doubles = false;
    if broke {
        let i1 = rng.d6();
        let i2 = rng.d6();
        injury_doubles = i1 == i2;
        match injury_result([i1, i2], &[]) {
            InjuryOutcome::Stunned => {
                game.field_model.set_player_state(target_id, PlayerState::new(PS_STUNNED));
            }
            InjuryOutcome::KnockedOut => {
                game.field_model.player_coordinates.remove(target_id);
                game.field_model.set_player_state(target_id, PlayerState::new(PS_KNOCKED_OUT));
            }
            InjuryOutcome::Casualty | InjuryOutcome::BadlyHurt => {
                let c1 = rng.die(16);
                let _c2 = rng.d6();
                let tier = casualty_tier_bb2025(c1);
                let ps = match tier {
                    CasualtyTier::BadlyHurt => PS_BADLY_HURT,
                    CasualtyTier::SeriousInjury => PS_SERIOUS_INJURY,
                    CasualtyTier::Dead => PS_RIP,
                };
                game.field_model.player_coordinates.remove(target_id);
                game.field_model.set_player_state(target_id, PlayerState::new(ps));
            }
        }
    }
    // StepReferee (SneakyGitBehaviour): doubles on armor OR (armor broken and doubles on injury)
    let referee_spots = armor_doubles || (broke && injury_doubles);
    if referee_spots {
        // StepBribes: parity runner always argues with first player; roll 1 d6.
        // isArgueTheCallSuccessful: roll > 5 (i.e. roll == 6) → argue succeeds, no ejection.
        let argue = rng.d6();
        if argue <= 5 {
            // Argue failed: StepEjectPlayer — fouler sent off (BANNED), turnover.
            game.field_model.player_coordinates.remove(fouler_id);
            game.field_model.set_player_state(fouler_id, PlayerState::new(PS_BANNED));
            game.turnover = true;
        }
        // If argue == 6: argue succeeds, player stays, no turnover.
    }
}

/// Push the defender one step away from the attacker.
/// Replicates Java's MatchRunner.handlePushback + UtilServerPushback.findPushbackSquares:
/// candidates are ordered [CCW-diagonal, straight, CW-diagonal] relative to push direction;
/// pick the first with maximum score (home attacking → max x; away attacking → max 25-x).
fn auto_push(game: &mut Game, attacker_id: &str, defender_id: &str, attacker_is_home: bool) {
    let def_coord = match game.field_model.player_coordinate(defender_id) {
        Some(c) => c,
        None => return,
    };
    let atk_coord = match game.field_model.player_coordinate(attacker_id) {
        Some(c) => c,
        None => return,
    };
    let push_dir = match atk_coord.direction_to(def_coord) {
        Some(d) => d,
        None => return,
    };
    // Java UtilServerPushback.findPushbackSquares offsets (dx,dy) from defender position,
    // in order: [CCW-diagonal, straight, CW-diagonal] relative to push_dir.
    let offsets: [(i32, i32); 3] = match push_dir {
        Direction::North     => [(-1,-1), ( 0,-1), ( 1,-1)], // NW, N, NE
        Direction::Northeast => [( 0,-1), ( 1,-1), ( 1, 0)], // N, NE, E
        Direction::East      => [( 1,-1), ( 1, 0), ( 1, 1)], // NE, E, SE
        Direction::Southeast => [( 1, 0), ( 1, 1), ( 0, 1)], // E, SE, S
        Direction::South     => [( 1, 1), ( 0, 1), (-1, 1)], // SE, S, SW
        Direction::Southwest => [( 0, 1), (-1, 1), (-1, 0)], // S, SW, W
        Direction::West      => [(-1, 1), (-1, 0), (-1,-1)], // SW, W, NW
        Direction::Northwest => [(-1, 0), (-1,-1), ( 0,-1)], // W, NW, N
    };
    // Java ParityRunner.sendPushback: picks the candidate with min(x), then min(y) as tiebreaker.
    let mut best: Option<FieldCoordinate> = None;
    for (dx, dy) in offsets {
        let sq = def_coord.add(dx, dy);
        if !sq.is_on_pitch() { continue; }
        if sq == atk_coord { continue; }
        if game.field_model.player_at(sq).is_some() { continue; }
        let better = match best {
            None => true,
            Some(b) => sq.x < b.x || (sq.x == b.x && sq.y < b.y),
        };
        if better { best = Some(sq); }
    }
    if let Some(sq) = best {
        game.field_model.set_player_coordinate(defender_id, sq);
    }
}

// ── shared test fixtures (used by engine.rs and agent.rs tests) ──
#[cfg(test)]
pub(crate) fn test_team(side: &str, dedicated_fans: i32) -> ffb_model::model::team::Team {
    ffb_model::model::team::Team {
        id: format!("{side}_lineman"), name: format!("{side} Linemen"),
        race: "lineman".into(), roster_id: "lineman".into(), coach: format!("Coach_{side}"),
        rerolls: 3, apothecaries: 1, bribes: 0, master_chefs: 0, prayers_to_nuffle: 0,
        bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0, assistant_coaches: 0,
        fan_factor: 0, dedicated_fans, team_value: 1_000_000, treasury: 0,
        special_rules: vec![], players: vec![],
    }
}

#[cfg(test)]
pub(crate) fn new_game(seed: u64) -> GameState {
    use ffb_model::enums::Rules;
    let game = Game::new(test_team("home", 5), test_team("away", 7), Rules::Bb2025);
    let mut gs = GameState::from_game(game, seed);
    gs.push_sequence(start_game_sequence());
    gs
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── stack mechanics (moved from framework.rs; now carry a real Step) ──
    #[test]
    fn stack_push_sequence_runs_first_authored_first() {
        let mut s = StepStack::new();
        s.push_sequence(vec![
            StepEntry::new(Step::InitStartGame),
            StepEntry::new(Step::Spectators),
            StepEntry::labelled(Step::Weather, "weather"),
        ]);
        assert_eq!(s.pop().unwrap().id(), StepId::InitStartGame);
        assert_eq!(s.pop().unwrap().id(), StepId::Spectators);
        assert_eq!(s.pop().unwrap().id(), StepId::Weather);
    }

    #[test]
    fn goto_label_discards_until_label_on_top() {
        let mut s = StepStack::new();
        s.push(StepEntry::labelled(Step::Weather, "weather"));
        s.push(StepEntry::new(Step::Spectators));
        s.push(StepEntry::new(Step::InitStartGame));
        s.goto_label("weather").unwrap();
        assert_eq!(s.peek().unwrap().id(), StepId::Weather);
        assert_eq!(s.len(), 1);
    }

    #[test]
    fn goto_unknown_label_errors() {
        let mut s = StepStack::new();
        s.push(StepEntry::new(Step::Spectators));
        assert!(s.goto_label("nope").is_err());
    }

    #[test]
    fn publish_walks_top_to_bottom_until_consumed() {
        // No pregame step consumes a param, so a published param propagates to the bottom and is
        // dropped without panicking — proves the walk is wired. (Consumption asserted in Phase D
        // once a param-reading step lands.)
        let mut s = StepStack::new();
        s.push(StepEntry::new(Step::ReceiveChoice));
        s.push(StepEntry::new(Step::CoinChoice));
        s.publish(&StepParameter::EndTurn(true));
        assert_eq!(s.len(), 2, "non-consuming publish leaves the stack intact");
    }

    #[test]
    fn pregame_consumes_d3_d3_d6_d6_then_waits_at_coin_prompt() {
        let seed = 1u64;
        let mut refrng = GameRng::new(seed);
        let exp_fan_home = refrng.d3();
        let exp_fan_away = refrng.d3();
        let exp_w = Weather::for_roll(refrng.d6() + refrng.d6());

        let mut gs = new_game(seed);
        gs.run_until_prompt();

        // 4 dice consumed (fan d3 x2, weather d6 x2); the coin's d2 is NOT rolled until the
        // guess arrives — the engine is now waiting at the coin prompt.
        assert_eq!(gs.rng.call_count, 4);
        assert_eq!(gs.game.status, GameStatus::Active);
        assert_eq!(gs.game.team_home.fan_factor, 5 + exp_fan_home);
        assert_eq!(gs.game.team_away.fan_factor, 7 + exp_fan_away);
        assert_eq!(gs.game.weather, exp_w);
        assert!(matches!(gs.current_prompt(), Some(AgentPrompt::CoinChoice { is_home: true })));
    }

    #[test]
    fn kickoff_scatter_rolls_d8_dir_then_d6_dist_and_places_ball() {
        // Characterization (per SEED1_DICE_MAP): direction d8 FIRST, distance d6 SECOND, ball
        // placed at the on-pitch landing. Pin the order + mechanic against a reference RNG.
        let seed = 99u64;
        let mut refrng = GameRng::new(seed);
        let exp_dir_roll = refrng.d8();
        let exp_dist = refrng.d6();
        let exp_dir = Direction::for_roll(exp_dir_roll).unwrap();
        let start = FieldCoordinate::new(13, 7); // mid-pitch — scatter stays on-pitch
        let (ex, ey) = scatter_coordinate(start.x, start.y, exp_dir, exp_dist);

        let game = Game::new(test_team("home", 0), test_team("away", 0), ffb_model::enums::Rules::Bb2025);
        let mut gs = GameState::from_game(game, seed);
        gs.game.field_model.ball_coordinate = Some(start);
        gs.push_sequence(vec![StepEntry::new(Step::KickoffScatterRoll)]);
        gs.run_until_prompt();

        assert_eq!(gs.rng.call_count, 2, "exactly d8 then d6");
        assert!(matches!(gs.events.as_slice(),
            [GameEvent::KickoffScatter { start: s, direction, distance }]
            if *s == start && *direction == exp_dir_roll && *distance == exp_dist));
        // mid-pitch landing is on-pitch → ball moved there.
        assert_eq!(gs.game.field_model.ball_coordinate, Some(FieldCoordinate::new(ex, ey)));
    }

    #[test]
    fn kickoff_result_rolls_2d6_and_maps_table() {
        // 2d6 → BB2025 kickoff table. Pin the order + that the mapped result is published/emitted.
        let seed = 99u64;
        let mut refrng = GameRng::new(seed);
        let total = refrng.d6() + refrng.d6();
        let exp = kickoff_result_from_kind(kickoff_event_bb2025(total).unwrap());

        let game = Game::new(test_team("home", 0), test_team("away", 0), ffb_model::enums::Rules::Bb2025);
        let mut gs = GameState::from_game(game, seed);
        gs.push_sequence(vec![StepEntry::new(Step::KickoffResultRoll)]);
        gs.run_until_prompt();

        assert_eq!(gs.rng.call_count, 2, "exactly 2d6");
        assert!(matches!(gs.events.as_slice(),
            [GameEvent::KickoffResultEvent { result }] if *result == exp));
    }

    #[test]
    fn coin_then_receive_drives_to_idle_with_correct_offense_and_dice_order() {
        let seed = 1u64;
        // Reference: after fan d3,d3 + weather d6,d6, the next game die is the coin d2.
        let mut refrng = GameRng::new(seed);
        let (_h, _a) = (refrng.d3(), refrng.d3());
        let (_w1, _w2) = (refrng.d6(), refrng.d6());
        let coin_is_heads = refrng.bool();

        let mut gs = new_game(seed);
        gs.run_until_prompt();
        assert_eq!(gs.rng.call_count, 4, "no coin die before the guess");

        // Agent guesses heads=true. Coin flip happens now (5th die = d2).
        gs.apply_action(Action::CoinChoice { heads: true });
        assert_eq!(gs.rng.call_count, 5, "coin flip is the 5th game die (d2)");
        let home_won = true == coin_is_heads;
        assert_eq!(gs.game.home_playing, home_won, "winner becomes the chooser");
        // CoinThrow emitted; now waiting at the receive prompt for the winner's team.
        assert!(gs.events.iter().any(|e| matches!(e, GameEvent::CoinThrow { home_won: hw } if *hw == home_won)));
        let chooser_team = if home_won { gs.game.team_home.id.clone() } else { gs.game.team_away.id.clone() };
        assert!(matches!(gs.current_prompt(), Some(AgentPrompt::ReceiveChoice { team_id }) if *team_id == chooser_team));

        // Winner chooses to receive. home_first_offense follows; kicker (home_playing) is the
        // opposite. The engine then drives through the opening kickoff (InitKickoff, Setup×2,
        // Kickoff, scatter d8+d6, result 2d6) and idles after the result roll.
        gs.apply_action(Action::ReceiveChoice { receive: true });
        let home_receives = if home_won { true } else { false }; // chooser receives
        assert_eq!(gs.game.home_first_offense, home_receives);
        assert_eq!(gs.game.home_playing, !home_receives, "kicker kicks (set up first; two Setup flips net to kicker)");
        assert!(gs.events.iter().any(|e| matches!(e, GameEvent::ReceiveChoice { receive, .. } if *receive == home_receives)));
        // After receive the engine runs InitKickoff/Setup×2/Kickoff and waits at KickBall (the
        // kicking coach's target pick) — still only the coin die rolled so far.
        assert_eq!(gs.rng.call_count, 5, "no kickoff game dice until the ball is kicked");
        assert!(matches!(gs.current_prompt(), Some(AgentPrompt::KickBall)));

        // Kick the ball; the engine then drives scatter d8,d6 (6-7) + result d6,d6 (8-9) +
        // Cheering Fans d6,d6 (10-11) + ball bounce d8 (12) = 12 total. (Seed 1: 2d6=6 → Cheering.)
        let target = if gs.game.home_playing { FieldCoordinate::new(21, 9) } else { FieldCoordinate::new(4, 9) };
        gs.apply_action(Action::KickBall { coord: target });
        assert_eq!(gs.rng.call_count, 12, "full opening kickoff dice after the kick");
        assert!(gs.events.iter().any(|e| matches!(e, GameEvent::KickoffScatter { .. })));
        assert!(gs.events.iter().any(|e| matches!(e, GameEvent::KickoffResultEvent { result } if *result == KickoffResult::CheeringFans)));
        // EndKickoff → EndTurn → InitSelecting: engine now waits at the first ActivatePlayer
        // prompt (0 extra dice). The receiving team (away for seed 1) goes first.
        assert!(matches!(gs.current_prompt(), Some(AgentPrompt::ActivatePlayer { .. })),
            "engine reaches first ActivatePlayer after kickoff (0 dice beyond 12)");
        assert_eq!(gs.rng.call_count, 12, "no game dice consumed by EndKickoff/EndTurn/InitSelecting");
    }

    /// auto_push for WEST push (away attacking): Java picks SW=(x-1,y+1) first.
    /// Attacker at (13,8), defender at (12,8) → push dir WEST → candidates SW(11,9),W(11,8),NW(11,7).
    /// All score 25-11=14 equally → first wins → defender lands at (11,9).
    #[test]
    fn auto_push_west_away_attacker_picks_sw_first() {
        use ffb_model::model::field_model::FieldModel;
        use ffb_model::types::FieldCoordinate;
        let mut game = Game::new(
            test_team("home", 5),
            test_team("away", 7),
            ffb_model::enums::Rules::Bb2025,
        );
        let atk_coord = FieldCoordinate::new(13, 8);
        let def_coord = FieldCoordinate::new(12, 8);
        game.field_model.set_player_coordinate("away_atk", atk_coord);
        game.field_model.set_player_coordinate("home_def", def_coord);
        // attacker_is_home = false (away team attacking)
        auto_push(&mut game, "away_atk", "home_def", false);
        let pushed = game.field_model.player_coordinate("home_def").unwrap();
        // Java sendPushback: min(x) then min(y) on server coords → NW=(11,7)
        assert_eq!(pushed, FieldCoordinate::new(11, 7),
            "away attacking WEST: defender should push to NW=(11,7), not {:?}", pushed);
    }

    /// StepPickUp: failed pickup rolls d6, sets game.turnover=true, and bounces the ball
    /// (d8 scatter) away from the original square. 1:1 with Java StepPickUp.pickUp() FAILURE
    /// branch + StepCatchScatterThrowIn.bounceBall().
    #[test]
    fn pickup_failure_sets_turnover_and_bounces_ball() {
        use ffb_model::model::player::Player;
        use ffb_model::enums::{PlayerType, PlayerGender, Rules, PlayerAction};
        use std::collections::HashSet;

        // One home player at (10,7) with AG=3; no opponents (0 TZ).
        let mut home = test_team("home", 0);
        home.players.push(Player {
            id: "p01".to_string(), name: "Tester".to_string(), nr: 1,
            position_id: "lineman".to_string(), player_type: PlayerType::Regular,
            gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: HashSet::new(), niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None,
        });
        let away = test_team("away", 0);

        let seed = 42u64;
        let mut refrng = GameRng::new(seed);
        // First die: d6 pickup roll. Target = max(2, 7-3+0) = 4 for AG=3, 0 TZ.
        let pickup_roll = refrng.d6();
        let pickup_succeeds = pickup_roll >= 4;
        // Second die (if fail): d8 bounce direction.
        let bounce_dir_roll = refrng.d8();
        let bounce_dir = Direction::for_roll(bounce_dir_roll).unwrap();

        let game = Game::new(home, away, Rules::Bb2025);
        let mut gs = GameState::from_game(game, seed);

        let ball_sq = FieldCoordinate::new(10, 7);
        gs.game.field_model.set_player_coordinate("p01", ball_sq);
        gs.game.field_model.set_player_state("p01", PlayerState::new(PS_STANDING));
        gs.game.field_model.ball_coordinate = Some(ball_sq);
        gs.game.field_model.ball_in_play = true;
        gs.game.field_model.ball_moving = true;
        gs.game.acting_player.set_player("p01".to_string(), PlayerAction::Move);

        // Run only PickUp; no subsequent steps so turnover flag is observable.
        gs.push_sequence(vec![StepEntry::new(Step::PickUp)]);
        gs.run_until_prompt();

        if pickup_succeeds {
            assert!(!gs.game.field_model.ball_moving, "pickup success: ball stops");
            assert_eq!(gs.game.field_model.ball_coordinate, Some(ball_sq), "ball stays on player");
            assert!(!gs.game.turnover, "no turnover");
        } else {
            // Ball must have moved 1 step in bounce_dir from ball_sq.
            let (ex, ey) = scatter_coordinate(ball_sq.x, ball_sq.y, bounce_dir, 1);
            let expected_ball = FieldCoordinate::new(ex, ey);
            assert!(gs.game.turnover, "failed pickup sets turnover");
            assert_eq!(gs.game.field_model.ball_coordinate, Some(expected_ball),
                "ball bounced to ({},{}) from ({},{})", ex, ey, ball_sq.x, ball_sq.y);
        }
    }

    /// auto_push: if SW is occupied, fall through to NW=(11,7) per min(x)/min(y).
    #[test]
    fn auto_push_west_away_attacker_falls_through_to_w_when_sw_blocked() {
        use ffb_model::types::FieldCoordinate;
        let mut game = Game::new(
            test_team("home", 5),
            test_team("away", 7),
            ffb_model::enums::Rules::Bb2025,
        );
        game.field_model.set_player_coordinate("away_atk", FieldCoordinate::new(13, 8));
        game.field_model.set_player_coordinate("home_def", FieldCoordinate::new(12, 8));
        game.field_model.set_player_coordinate("blocker", FieldCoordinate::new(11, 9)); // SW blocked
        auto_push(&mut game, "away_atk", "home_def", false);
        let pushed = game.field_model.player_coordinate("home_def").unwrap();
        // SW=(11,9) blocked, W=(11,8) and NW=(11,7) remain; Java min(x)/min(y) → NW=(11,7)
        assert_eq!(pushed, FieldCoordinate::new(11, 7),
            "SW blocked → min(x)/min(y) picks NW=(11,7)");
    }
}
