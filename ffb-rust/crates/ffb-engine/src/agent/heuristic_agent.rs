//! `HeuristicAgent` — the probabilistic policy specified in `docs/HEURISTIC_AGENT.md`.
//!
//! Every decision follows the same pipeline: **enumerate → score → softmax → sample**. Scoring is
//! pure and RNG-free; only `act` touches the RNG.
//!
//! The `temp_scale` constructor argument multiplies every temperature in the table. It exists so
//! the *same* agent — the same enumeration, the same option sets, the same code path — can be run
//! as a uniform sampler by setting it very large, or as true argmax by setting it to zero. That
//! makes "heuristics vs random over an identical action space" a one-parameter A/B rather than a
//! comparison between two different programs.
//!
//! Long-tail prompts (inducements, kickoff events, apothecary, the star specials) fall through to
//! `UniformAgent`, which is identical in both arms of that A/B, so the comparison isolates exactly
//! the decisions this agent scores.
//!
//! # Performance structure (docs §20)
//!
//! Measurement put 84–89% of agent time in `ActivatePlayer` and 10–16% in `Move`, with the other
//! thirteen prompt classes under a tenth of one percent combined. Everything below is shaped by
//! that, and lives **entirely inside the agent** — the engine's own pathfinder, `legal_actions`
//! and `util` are read but never modified:
//!
//! - **§20.1** a plain move never needs a second `Move` prompt: moving twice reaches the same
//!   square as moving once, so the activation ends for free unless the state genuinely changed.
//! - **§20.2** the activation decision *is* the plan; the prompts that follow replay it.
//! - **§20.3** two-tier activation scoring: a search-free proxy for every eligible player, the
//!   real search only for the best few.
//! - **§20.4/§20.5** the mover-independent parts of the value model are rasterised once per
//!   position change — exposure, lane, the support intents.
//! - **§20.6** flat arrays, a binary heap and back-pointers instead of a `HashMap`, a linear
//!   frontier scan and a path clone per improvement.
//! - **§20.7** the whole feature block is cached on a positions stamp.
//! - **§20.8** block strength is memoised per (attacker, defender) pair.
//! - **§20.9** the action space is whole *plans*, which is also what makes a blitz actually block
//!   and a pass reachable after moving.
//! - **§20.10** an admissible bound prunes the destination set before full scoring.

use std::cell::RefCell;
use std::collections::{BinaryHeap, HashMap, HashSet};

use rand_core::{RngCore, SeedableRng};
use rand_xoshiro::Xoshiro256StarStar;

use ffb_model::enums::{PlayerAction, Rules, SkillId, Weather};
use ffb_model::model::game::Game;
use ffb_model::prompts::AgentPrompt;
use ffb_model::types::FieldCoordinate;

use crate::action::{Action, PlayerActionChoice};
use crate::legal_actions::{
    canonical_setup_action, legal_block_targets, legal_foul_targets, legal_handoff_receivers,
    legal_pass_receivers, TeamSide,
};
use crate::step::GameState;

use super::random_agent::player_action_to_pac;
use super::det_math::{exp_f32, ln_f32};
use super::{Agent, RandomAgent, UniformAgent};

/// `mechanics/movement.rs: STAND_UP_COST`.
const STAND_UP_COST: i32 = 3;
const EPS: f32 = 0.02;
/// Residual worth of advancing a carrier who can no longer reach the endzone in time. Was read
/// from FFB_HOPELESS_DAMP on every `value_at` call while it was being fitted; frozen because
/// env-dependent POLICY cannot be mirrored by the Java agent, and the per-call `env::var` was a
/// hot-path allocation besides. See AGENT_CONTRACT.md section 10.
const HOPELESS_DAMP: f32 = 0.25;
const XMAX: i32 = 25;
const YMAX: i32 = 14;
const W: usize = 26;
const H: usize = 15;
const CELLS: usize = W * H;
/// §20.3 — how many players get the real search rather than the proxy.
///
/// Chosen as 3 when one ActivatePlayer cost 4136 µs. After §20's rewrite it costs ~192, and the
/// cap had become a distortion rather than an economy: a player outside it contributes ONE
/// placeholder option standing in for its whole destination space, so it collects the probability
/// mass that should have been spread over a dozen squares and gets over-sampled against players
/// whose destinations are enumerated. 16 covers every player who can ever be eligible.
const TIER2: usize = 16;
/// The declaration a real Java client sends for a give: `HAND_OVER_MOVE` / `PASS_MOVE`.
///
/// `SelectLogicModule` never sends the immediate `HAND_OVER` / `PASS` forms — those exist for the
/// parity harness, which declares an action and resolves it with no movement in between. The MOVE
/// variants open a movement phase *before* the give, which is what makes carrier-move + give +
/// receiver-move reachable in a single turn: the whole point of a hand-off chain. They are strictly
/// more general — the movement phase may consume none of the player's move — so the immediate form
/// is never the better declaration for this agent. The parity agents are untouched and keep
/// declaring the immediate form, so their dice streams are unchanged.
fn move_variant(pac: PlayerActionChoice) -> PlayerActionChoice {
    match pac {
        PlayerActionChoice::HandOff => PlayerActionChoice::HandOffMove,
        PlayerActionChoice::Pass => PlayerActionChoice::PassMove,
        other => other,
    }
}

/// Run-up squares a PassMove considers throwing from, besides standing still.
const THROW_SPOTS: usize = 6;
/// Squares next to a receiver a HandOverMove considers giving from.
const GIVE_SPOTS: usize = 2;
/// How many destinations get a formatted note. Every reachable square is an OPTION (§1: consider
/// all actions); this only bounds how many carry the human-readable arithmetic, because a note is
/// a `format!` and nobody reads the two-thousandth one.
const DEST_NOTES: usize = 10;

#[inline]
fn ix(x: i32, y: i32) -> usize {
    (y as usize) * W + (x as usize)
}
#[inline]
fn ixc(c: FieldCoordinate) -> usize {
    ix(c.x, c.y)
}
#[inline]
fn on_pitch(x: i32, y: i32) -> bool {
    x >= 0 && x <= XMAX && y >= 0 && y <= YMAX
}
#[inline]
fn coord_of(i: usize) -> FieldCoordinate {
    FieldCoordinate::new((i % W) as i32, (i / W) as i32)
}

// ───────────────────────────────────────────────────────────────── primitives

#[inline]
fn p_roll(target: i32) -> f32 {
    ((7 - target) as f32 / 6.0).clamp(1.0 / 6.0, 5.0 / 6.0)
}

#[inline]
fn p_with_reroll(p: f32, p_rr: f32) -> f32 {
    p + (1.0 - p) * p_rr * p
}

/// BB2025/BB2020 use the AG-target scale; BB2016 the old `7 − AG` one.
/// (`DodgeModifierFactory::minimum_roll_edition`, with the TACKLEZONE modifier == the count.)
#[inline]
fn dodge_target(rules: Rules, ag: i32, tz_on_dest: i32) -> i32 {
    match rules {
        Rules::Bb2016 => ((7 - ag.min(6)) - 1 + tz_on_dest).max(2),
        _ => (ag + tz_on_dest).max(2),
    }
}

/// `GoForItModifierFactory::minimum_roll_going_for_it` — base 2, **Blizzard +1 in every edition**.
#[inline]
fn gfi_target(weather: Weather) -> i32 {
    if weather == Weather::Blizzard {
        3
    } else {
        2
    }
}

#[inline]
fn strength_factor(att: i32, def: i32) -> f32 {
    if att > 2 * def {
        1.4
    } else if att > def {
        1.2
    } else if 2 * att < def {
        0.5
    } else if att < def {
        0.7
    } else {
        1.0
    }
}

/// P(2d6 >= need).
fn p_2d6_at_least(need: i32) -> f32 {
    if need <= 2 {
        return 1.0;
    }
    if need > 12 {
        return 0.0;
    }
    let mut ways = 0;
    for a in 1..=6 {
        for b in 1..=6 {
            if a + b >= need {
                ways += 1;
            }
        }
    }
    ways as f32 / 36.0
}

/// §5.3 — the expected cost of failing, which ends the team turn.
#[inline]
fn c_turnover(unactivated: f32, gfi: i32, carries_ball: bool) -> f32 {
    (0.20 + 0.55 * unactivated)
        * if carries_ball { 1.4 } else { 1.0 }
        * (1.0 + 0.15 * gfi as f32)
}

/// Risk aversion on rushes, over and above the expectation in `c_turnover`. A turnover forfeits the
/// rest of the drive, not one square, and that compounding is invisible to a single-step mean.
#[inline]
fn rush_penalty(gfi: i32, carries_ball: bool) -> f32 {
    if gfi <= 0 {
        return 0.0;
    }
    (if carries_ball { 0.10 } else { 0.40 }) * gfi as f32
}

#[inline]
fn endzone_x(home: bool) -> i32 {
    if home {
        XMAX
    } else {
        0
    }
}
#[inline]
fn endzone_distance(c: FieldCoordinate, home: bool) -> i32 {
    (c.x - endzone_x(home)).abs()
}
#[inline]
fn side_idx(home: bool) -> usize {
    if home {
        0
    } else {
        1
    }
}
#[inline]
fn weather_of(g: &Game) -> Weather {
    g.field_model.weather
}

// ─────────────────────────────────────────────────────────────── option scoring

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rule {
    ScoreTouchdown,
    ScoreAdvance,
    Pickup,
    Support,
    EndActivation,
    DiceCount,
    Face,
    Reroll,
    Skill,
    Flat,
}

/// One enumerated option together with the probability the sampler actually gave it.
///
/// Only populated when `FFB_HEUR_DUMP` is set. A visualiser needs the whole distribution, not the
/// pick — the interesting picture is *what the agent considered and how strongly*, which the chosen
/// action alone cannot show. Off by default so it costs nothing in a measurement run.
#[derive(Clone, serde::Serialize)]
pub struct ScoredOption {
    pub action: Action,
    /// The signed weight from §5.3, before any softmax.
    pub w: f32,
    /// The probability this option had of being sampled, at the temperature actually used.
    pub p: f32,
    /// Which §6 rule produced the weight.
    pub why: String,
    /// What this option means, where the `Action` alone does not say. Six destinations for one
    /// Move declaration all serialise to the same `activatePlayer`; a block die choice serialises
    /// as a bare index. The note carries the difference.
    pub note: String,
}

pub struct Weighted {
    pub action: Action,
    /// SIGNED desirability — §5.3 subtracts an expected turnover cost, so a bad option is negative.
    pub weight: f32,
    pub why: Rule,
    pub why_value: f32,
    /// See `ScoredOption::note`. Empty when the action speaks for itself.
    pub note: String,
}

#[derive(Default)]
pub struct Scored {
    pub options: Vec<Weighted>,
    /// Set whenever the candidate set was capped (§20.10). Never silently truncate.
    pub truncated: bool,
}

impl Scored {
    #[inline]
    fn push(&mut self, action: Action, weight: f32, why: Rule, why_value: f32) {
        self.options.push(Weighted { action, weight, why, why_value, note: String::new() });
    }
    #[inline]
    fn push_note(&mut self, action: Action, weight: f32, why: Rule, why_value: f32, note: String) {
        self.options.push(Weighted { action, weight, why, why_value, note });
    }
    fn clear(&mut self) {
        self.options.clear();
        self.truncated = false;
    }
}

// ────────────────────────────────────────── cached board features (§20.4–§20.8)

/// Occupancy: 0 = empty, 1 = home, 2 = away; the high bit marks "has tackle zones".
const OCC_NONE: u8 = 0;
const OCC_HOME: u8 = 1;
const OCC_AWAY: u8 = 2;
const OCC_TZ: u8 = 0x80;

/// Everything about a board position that more than one prompt wants to know. Built once per
/// position (§20.7) and read-only afterwards — the one mutable part is the block-strength memo,
/// which sits behind a `RefCell` so every reader can hold a shared borrow.
struct Features {
    stamp: u64,
    /// Opposing tackle zones on each square. Index 0 = zones threatening a HOME player.
    tz: [Vec<u8>; 2],
    occ: Vec<u8>,
    /// Opponents-of-`side` in row y at column < x, prefix-summed. Index 0 = opponents of home.
    row_prefix: [Vec<u16>; 2],
    /// §20.5 rasterised threat, split so the mover's own strength applies at read time with no
    /// loss of exactness: the best single blitzer's reach factor, that blitzer's strength, and the
    /// small marking term from the next two.
    threat_reach: [Vec<f32>; 2],
    threat_str: [Vec<i8>; 2],
    threat_mark: [Vec<f32>; 2],
    /// §20.4 value of a square to a NON-carrier: max(Cage, Mark, Screen, Retreat). Entirely
    /// mover-independent, so it is a raster rather than a per-square computation.
    support: [Vec<f32>; 2],
    /// §20.4 corridor openness toward that side's endzone.
    lane: [Vec<f32>; 2],
    ball: Option<FieldCoordinate>,
    ball_loose: bool,
    ball_carried: bool,
    carrier: Option<String>,
    /// Where OUR carrier stands, per side. The receiver intent needs it and `value_at` has no
    /// `Game` to look it up from.
    carrier_at: [Option<FieldCoordinate>; 2],
    /// Per side: players still able to act this turn, as a fraction of 11.
    unactivated: [f32; 2],
    /// §20.8 — `find_block_strength` runs a nested player×player loop for the Guard-cancel rule.
    block_memo: RefCell<HashMap<(u32, u32), i32>>,
    /// Whether `threat`, `lane` and `support` were actually computed. Only `Move` and
    /// `ActivatePlayer` read them, and they are almost all of the build cost, so the other prompt
    /// classes get a cheap core and leave the rasters at their neutral fill (§20.7).
    heavy: bool,
}

/// Order-independent position hash. `player_coordinates` is a `HashMap`, so this must not depend on
/// iteration order — a wrapping sum of per-player hashes does not.
fn positions_stamp(g: &Game) -> u64 {
    let mut acc: u64 = 0;
    for (id, &c) in &g.field_model.player_coordinates {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        for b in id.as_bytes() {
            h ^= *b as u64;
            h = h.wrapping_mul(0x100_0000_01b3);
        }
        h ^= ((c.x as u64) << 8) ^ (c.y as u64);
        let st = g.field_model.player_state(id).map(|s| s.id() as u64).unwrap_or(0);
        h = h.wrapping_mul(31).wrapping_add(st);
        acc = acc.wrapping_add(h);
    }
    let b = g
        .field_model
        .ball_coordinate
        .map(|c| ((c.x as u64) << 16) | c.y as u64)
        .unwrap_or(0xffff);
    acc ^= b << 20;
    acc = acc.wrapping_mul(31).wrapping_add(g.field_model.ball_moving as u64);
    acc = acc.wrapping_mul(31).wrapping_add(g.field_model.ball_in_play as u64);
    acc = acc.wrapping_mul(31).wrapping_add(g.half as u64);
    acc = acc.wrapping_mul(31).wrapping_add(g.turn_data_home.turn_nr as u64);
    acc = acc.wrapping_mul(31).wrapping_add(g.turn_data_away.turn_nr as u64);
    acc = acc.wrapping_mul(31).wrapping_add(g.home_playing as u64);
    acc
}

/// The ONE ordering key that exists in both engines: (side, jersey nr). This is already the key
/// the state hash uses on both sides -- `state_hash.rs::collect_player_parts` sorts by `p.nr` and
/// labels positionally, and Java's `ParityRunner.addPlayersFromTeam` sorts by
/// `Comparator.comparingInt(Player::getNr)`.
///
/// Player IDS MUST NEVER ENTER AN ORDERING (AGENT_CONTRACT.md section 6): Rust ids are
/// `home_01..home_11` (and a star carries its own, e.g. `morgNThorg`) while Java's are
/// `teamLinemanParityHome1..11` -- they sort differently, so an id sort cannot be mirrored.
/// For `home_NN`-style ids this key reproduces the old lexicographic id order exactly, which is
/// why swapping to it is a no-op for every non-star roster.
fn canon_key(g: &Game, pid: &str) -> (u8, i32) {
    let side = if g.team_home.has_player(pid) { 0u8 } else { 1u8 };
    let nr = g.player(pid).map(|p| p.nr).unwrap_or(i32::MAX);
    (side, nr)
}

/// On-pitch players in canonical `(side, nr)` order.
///
/// `field_model.player_coordinates` is a `std::collections::HashMap` with the default RANDOMLY
/// SEEDED hasher, so iterating it directly is only safe where the accumulation is commutative.
/// Audited, as of this commit:
///
/// - `positions_stamp` -- safe: `wrapping_add` into one accumulator, and it is only ever a cache
///   key, so its value need not even agree across the two engines.
/// - `Features::build` -- safe: `occ` writes one distinct square per player, `tz` and
///   `row_prefix` are integer increments, and `unact` sums a CONSTANT addend so its f32 result
///   depends only on the count, not the order.
/// - `build_support` -- safe: `screen_tot`/`screen_hits` are integer counters and `support` is a
///   `max`, both commutative.
/// - `build_threat` -- **NOT safe**, which is why this exists. It writes `threat_str` under a
///   strict `>` against `threat_reach`, so two opponents that reach a square equally (`reach ==
///   1.0` whenever both stand adjacent -- the common case) tie, and whichever the hasher happened
///   to yield first records ITS strength. `threat_str` feeds `strength_factor` -> `exposure` ->
///   every arrival weight, so the agent was non-deterministic run-to-run on any roster with mixed
///   ST. Inert for the all-ST3 lineman fixture, which is why it never showed up. The
///   `second`/`third` reach tracking in the same loop has the same shape.
fn canon_players(g: &Game) -> Vec<(String, FieldCoordinate)> {
    let mut v: Vec<(String, FieldCoordinate)> = g
        .field_model
        .player_coordinates
        .iter()
        .map(|(id, &c)| (id.clone(), c))
        .collect();
    v.sort_by_key(|(id, _)| canon_key(g, id));
    v
}

/// `canon_key` packed into a `u32` for use as a map key. Replaces an FNV hash of the player id:
/// the memo is only ever a keyed lookup, never an ordering, but a hash collision that resolved
/// differently in the two languages would be a silent divergence for free.
fn canon_pack(g: &Game, pid: &str) -> u32 {
    let (side, nr) = canon_key(g, pid);
    ((side as u32) << 16) | (nr.clamp(0, 0xffff) as u32)
}

impl Features {
    fn build(g: &Game, stamp: u64, heavy: bool) -> Features {
        let mut occ = vec![OCC_NONE; CELLS];
        let mut tz = [vec![0u8; CELLS], vec![0u8; CELLS]];
        let mut row_prefix = [vec![0u16; H * (W + 1)], vec![0u16; H * (W + 1)]];
        let mut unact = [0f32; 2];

        for (id, &c) in &g.field_model.player_coordinates {
            if !on_pitch(c.x, c.y) {
                continue;
            }
            let is_home = g.team_home.has_player(id);
            let st = g.field_model.player_state(id);
            let standing = st.map(|s| s.has_tacklezones()).unwrap_or(false);
            let s = side_idx(is_home);
            occ[ixc(c)] =
                (if is_home { OCC_HOME } else { OCC_AWAY }) | if standing { OCC_TZ } else { 0 };
            if standing {
                for n in c.neighbours() {
                    if on_pitch(n.x, n.y) {
                        tz[1 - s][ixc(n)] += 1;
                    }
                }
            }
            // Opponents-of-home are the away players, so they fill row_prefix[0].
            let opp_of = 1 - s;
            for x in (c.x + 1)..=(W as i32) {
                row_prefix[opp_of][(c.y as usize) * (W + 1) + x as usize] += 1;
            }
            if st.map(|s| s.is_active()).unwrap_or(false) {
                unact[s] += 1.0 / 11.0;
            }
        }

        let ball = g.field_model.ball_coordinate;
        let in_play = g.field_model.ball_in_play && ball.map(|c| on_pitch(c.x, c.y)).unwrap_or(false);
        // `ball_moving` means LOOSE ON THE GROUND, not in flight.
        let ball_loose = in_play && g.field_model.ball_moving;
        let ball_carried = in_play && !g.field_model.ball_moving;
        let carrier = ball.filter(|_| ball_carried).and_then(|b| {
            g.field_model
                .player_at(b)
                .filter(|id| g.field_model.player_coordinate(id) == Some(b))
                .cloned()
        });

        let mut f = Features {
            stamp,
            tz,
            occ,
            row_prefix,
            threat_reach: [vec![0.0; CELLS], vec![0.0; CELLS]],
            threat_str: [vec![3; CELLS], vec![3; CELLS]],
            threat_mark: [vec![0.0; CELLS], vec![0.0; CELLS]],
            support: [vec![0.10; CELLS], vec![0.10; CELLS]],
            lane: [vec![1.0; CELLS], vec![1.0; CELLS]],
            ball,
            ball_loose,
            ball_carried,
            carrier,
            carrier_at: [None, None],
            unactivated: [unact[0].min(1.0), unact[1].min(1.0)],
            block_memo: RefCell::new(HashMap::new()),
            heavy,
        };
        // Cheap, and the receiver intent needs it, so it is not gated behind `heavy`.
        for side in 0..2 {
            let my_home = side == 0;
            f.carrier_at[side] = f
                .carrier
                .as_ref()
                .filter(|c| g.team_home.has_player(c) == my_home)
                .and_then(|c| g.field_model.player_coordinate(c));
        }
        if heavy {
            f.build_threat(g);
            f.build_lane();
            f.build_support(g);
        }
        f
    }

    #[inline]
    fn tz_against(&self, c: FieldCoordinate, home: bool) -> i32 {
        self.tz[side_idx(home)][ixc(c)] as i32
    }

    #[inline]
    fn occupied(&self, i: usize) -> bool {
        self.occ[i] & 0x7f != OCC_NONE
    }

    fn opponents_between(&self, home: bool, y: i32, x0: i32, x1: i32) -> i32 {
        if y < 0 || y > YMAX {
            return 0;
        }
        let (lo, hi) = if x0 <= x1 { (x0, x1) } else { (x1, x0) };
        let p = &self.row_prefix[side_idx(home)];
        let row = (y as usize) * (W + 1);
        let a = p[row + lo.clamp(0, W as i32) as usize] as i32;
        let b = p[row + hi.clamp(0, W as i32) as usize] as i32;
        (b - a).max(0)
    }

    /// §20.5 / P1. One pass over the players fills the raster; every read afterwards is O(1).
    /// Only ONE opponent can blitz per turn, so the block term is a `max` over opponents and
    /// everyone else contributes only a small marking term.
    fn build_threat(&mut self, g: &Game) {
        for s in 0..2 {
            let victim_home = s == 0;
            let opp_blitz_spent = if victim_home {
                g.turn_data_away.blitz_used
            } else {
                g.turn_data_home.blitz_used
            };
            let mut second = vec![0.0f32; CELLS];
            let mut third = vec![0.0f32; CELLS];

            // Canonical order, NOT hash order -- the `threat_str` write below is a strict-`>`
            // tie-break. See `canon_players`.
            for (id, c) in canon_players(g) {
                let id = &id;
                if !on_pitch(c.x, c.y) || g.team_home.has_player(id) == victim_home {
                    continue;
                }
                if !g.field_model.player_state(id).map(|st| st.has_tacklezones()).unwrap_or(false) {
                    continue;
                }
                let opp = match g.player(id) {
                    Some(p) => p,
                    None => continue,
                };
                let ma = opp.movement_with_modifiers();
                let ostr = opp.strength_with_modifiers();
                let marked_now = self.tz[1 - s][ixc(c)] > 0;
                let r = ma + 3;
                for y in (c.y - r).max(0)..=(c.y + r).min(YMAX) {
                    for x in (c.x - r).max(0)..=(c.x + r).min(XMAX) {
                        let d = c.distance_in_steps(FieldCoordinate::new(x, y));
                        let steps = (d - 1).max(0);
                        let reach = if d == 1 {
                            1.0
                        } else if steps <= ma {
                            // a player who is himself marked is unlikely to leave freely
                            if marked_now {
                                0.55
                            } else {
                                1.0
                            }
                        } else if steps <= ma + 2 {
                            0.25
                        } else {
                            continue;
                        };
                        let i = ix(x, y);
                        // The block term needs a blitz unless the opponent already stands adjacent.
                        if (d == 1 || !opp_blitz_spent) && reach > self.threat_reach[s][i] {
                            self.threat_reach[s][i] = reach;
                            self.threat_str[s][i] = ostr as i8;
                        }
                        if reach > second[i] {
                            third[i] = second[i];
                            second[i] = reach;
                        } else if reach > third[i] {
                            third[i] = reach;
                        }
                    }
                }
            }
            for i in 0..CELLS {
                self.threat_mark[s][i] = 0.18 * (second[i] + third[i]);
            }
        }
    }

    /// §20.4 corridor openness: opponents within ±2 rows between the square and the endzone.
    fn build_lane(&mut self) {
        for s in 0..2 {
            let home = s == 0;
            let ez = endzone_x(home);
            for y in 0..H as i32 {
                for x in 0..W as i32 {
                    let mut corridor = 0;
                    for dy in -2..=2 {
                        corridor += self.opponents_between(home, y + dy, x, ez);
                    }
                    self.lane[s][ix(x, y)] = 1.0 / (1.0 + 0.35 * corridor as f32);
                }
            }
        }
    }

    /// §20.4 the mover-independent support intents: Cage, Mark, Screen, Retreat.
    fn build_support(&mut self, g: &Game) {
        // Screen: how many of the opponent's straight approaches to our ball pass near a square.
        let mut screen_hits = [vec![0u16; CELLS], vec![0u16; CELLS]];
        let mut screen_tot = [0u16; 2];
        for s in 0..2 {
            let my_home = s == 0;
            let target = self
                .carrier
                .as_ref()
                .filter(|c| g.team_home.has_player(c) == my_home)
                .and_then(|c| g.field_model.player_coordinate(c))
                .or(self.ball.filter(|_| self.ball_loose));
            let target = match target {
                Some(t) => t,
                None => continue,
            };
            for (id, &c) in &g.field_model.player_coordinates {
                if !on_pitch(c.x, c.y) || g.team_home.has_player(id) == my_home {
                    continue;
                }
                if !g.field_model.player_state(id).map(|st| st.has_tacklezones()).unwrap_or(false) {
                    continue;
                }
                let d_ot = c.distance_in_steps(target);
                if d_ot == 0 || d_ot > 12 {
                    continue;
                }
                screen_tot[s] += 1;
                // Squares on a shortest-ish approach: going via them costs at most one extra step.
                for y in 0..H as i32 {
                    for x in 0..W as i32 {
                        let sq = FieldCoordinate::new(x, y);
                        let d_os = c.distance_in_steps(sq);
                        let d_st = sq.distance_in_steps(target);
                        if d_st >= 1 && d_os + d_st <= d_ot + 1 {
                            screen_hits[s][ix(x, y)] += 1;
                        }
                    }
                }
            }
        }

        for s in 0..2 {
            let my_home = s == 0;
            let own_carrier = self
                .carrier
                .as_ref()
                .filter(|c| g.team_home.has_player(c) == my_home)
                .and_then(|c| g.field_model.player_coordinate(c));
            let opp_occ = if my_home { OCC_AWAY } else { OCC_HOME };

            for y in 0..H as i32 {
                for x in 0..W as i32 {
                    let i = ix(x, y);
                    let sq = FieldCoordinate::new(x, y);
                    let mut best = 0.10f32; // Retreat floor

                    // Cage — weighted by which side the threat is actually on (A9).
                    if let Some(cc) = own_carrier {
                        let dx = (x - cc.x).abs();
                        let dy = (y - cc.y).abs();
                        if dx <= 1 && dy <= 1 && dx + dy > 0 {
                            if dx == 1 && dy == 1 {
                                let t = self.threat_reach[s][i].min(2.0) / 2.0;
                                best = best.max(0.35 + 0.40 * t);
                            } else {
                                best = best.max(0.35);
                            }
                        }
                    }

                    // Mark — the best adjacent opposing player worth standing next to.
                    let mut mark_best = 0.0f32;
                    for n in sq.neighbours() {
                        if !on_pitch(n.x, n.y) {
                            continue;
                        }
                        let o = self.occ[ixc(n)];
                        if o & 0x7f != opp_occ || o & OCC_TZ == 0 {
                            continue;
                        }
                        let is_carrier = self.ball_carried && Some(n) == self.ball;
                        let mut mv: f32 = if is_carrier { 1.0 } else { 0.30 };
                        if let Some(oid) = g.field_model.player_at(n) {
                            let spent = !g
                                .field_model
                                .player_state(oid)
                                .map(|st| st.is_active())
                                .unwrap_or(true);
                            if spent {
                                mv = mv.max(0.45);
                            }
                        }
                        mark_best = mark_best.max(mv);
                    }
                    if mark_best > 0.0 {
                        best = best.max(0.50 * mark_best);
                    }

                    // Screen — a line between the ball and the threat, not a huddle (P3).
                    if screen_tot[s] > 0 {
                        let share = screen_hits[s][i] as f32 / screen_tot[s] as f32;
                        if share > 0.0 {
                            best = best.max(0.45 * share);
                        }
                    }

                    self.support[s][i] = best;
                }
            }
        }
    }

    /// Exposure at a square for a mover of the given strength. Exact despite being rasterised: the
    /// reach factor and the blitzer's strength are stored apart, so `strength_factor` applies here.
    #[inline]
    fn exposure(&self, i: usize, home: bool, mover_str: i32) -> f32 {
        let s = side_idx(home);
        let block =
            self.threat_reach[s][i] * strength_factor(self.threat_str[s][i] as i32, mover_str);
        1.0 / (1.0 + block + self.threat_mark[s][i])
    }

    /// §20.8 — memoised on the pair for the lifetime of this position.
    fn block_strength(
        &self,
        g: &Game,
        att: &str,
        att_c: FieldCoordinate,
        att_str: i32,
        def: &str,
        def_c: FieldCoordinate,
    ) -> i32 {
        let k = (canon_pack(g, att), canon_pack(g, def));
        if let Some(v) = self.block_memo.borrow().get(&k) {
            return *v;
        }
        let v = crate::util::server_util_player::ServerUtilPlayer::find_block_strength(
            g, att_c, att_str, def_c,
        );
        self.block_memo.borrow_mut().insert(k, v);
        v
    }
}

// ────────────────────────────────────────────── reachability (§4.2, §20.6)

#[derive(Clone, Copy)]
struct ReachCell {
    /// −log(p_arrive), quantised so the heap ordering is integral and therefore deterministic.
    key: u32,
    cost: u8,
    gfi: u8,
    prev: u16,
    seen: bool,
}

const KEY_SCALE: f32 = 4096.0;
const NO_PREV: u16 = u16::MAX;
const UNREACHED: ReachCell =
    ReachCell { key: u32::MAX, cost: 0, gfi: 0, prev: NO_PREV, seen: false };

/// Reusable working memory. §20.10 — allocated once and handed back after every search rather than
/// building a fresh `HashMap` and path `Vec` per call.
#[derive(Default)]
struct Scratch {
    cell: Vec<ReachCell>,
    order: Vec<u16>,
    heap: BinaryHeap<HeapItem>,
    path: Vec<FieldCoordinate>,
}

struct Reach {
    cell: Vec<ReachCell>,
    order: Vec<u16>,
    start: usize,
    gate: f32,
}

impl Reach {
    #[inline]
    fn p_arrive(&self, i: usize) -> f32 {
        exp_f32(-(self.cell[i].key as f32) / KEY_SCALE) * self.gate
    }
    #[inline]
    fn reached(&self, i: usize) -> bool {
        self.cell[i].key != u32::MAX
    }
    /// Back-pointer walk. §20.6: done exactly once, for the destination actually chosen, instead of
    /// cloning a path `Vec` on every improvement.
    fn path_to(&self, mut i: usize, out: &mut Vec<FieldCoordinate>) {
        out.clear();
        while i != self.start {
            out.push(coord_of(i));
            let p = self.cell[i].prev;
            if p == NO_PREV {
                out.clear();
                return;
            }
            i = p as usize;
        }
        out.reverse();
    }
}

#[derive(PartialEq, Eq)]
struct HeapItem {
    key: u32,
    cost: u8,
    idx: u16,
}
impl Ord for HeapItem {
    fn cmp(&self, o: &Self) -> std::cmp::Ordering {
        // min-heap on (key, cost, idx); idx keeps ties deterministic
        o.key.cmp(&self.key).then(o.cost.cmp(&self.cost)).then(o.idx.cmp(&self.idx))
    }
}
impl PartialOrd for HeapItem {
    fn partial_cmp(&self, o: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(o))
    }
}

/// Where a search starts and how much movement it has left.
///
/// Split out of `reach` so a second leg can start somewhere the player is not yet standing — the
/// loose ball, say — with the movement the first leg did not spend. `spent` is what makes the GFI
/// test right across a multi-leg route: leg two's step counter restarts at zero, so leg one's cost
/// has to be carried in.
#[derive(Clone, Copy)]
struct Budget {
    start: FieldCoordinate,
    ma: i32,
    spent: i32,
    cap: i32,
    gate: f32,
}

fn budget_of(g: &Game, player_id: &str) -> Option<Budget> {
    let start = g.field_model.player_coordinate(player_id)?;
    if !on_pitch(start.x, start.y) {
        return None;
    }
    let player = g.player(player_id)?;
    let ma_base = player.movement_with_modifiers();
    let prone = g.field_model.player_state(player_id).map(|s| s.is_prone()).unwrap_or(false);
    let (ma, gate) = if prone {
        if ma_base <= STAND_UP_COST {
            (0, p_roll(4))
        } else {
            (ma_base - STAND_UP_COST, 1.0)
        }
    } else {
        (ma_base, 1.0)
    };
    let spent = if g.acting_player.player_id.as_deref() == Some(player_id) {
        g.acting_player.current_move.max(0)
    } else {
        0
    };
    let cap = (ma + 2 - spent).max(0);
    Some(Budget { start, ma, spent, cap, gate })
}

/// Dijkstra over −log(p_step), capped at the player's REAL remaining budget: a prone player has
/// already spent `STAND_UP_COST`, and at MA ≤ 3 the whole activation is gated behind a roll.
///
/// Flat arrays plus a binary heap (§20.6). The engine's own pathfinder is untouched — this is the
/// agent's own model of where it could go and how likely each arrival is.
fn reach(f: &Features, g: &Game, player_id: &str, team_rr: bool, sc: &mut Scratch) -> Option<Reach> {
    let b = budget_of(g, player_id)?;
    reach_with(f, g, player_id, &b, team_rr, sc)
}

fn reach_with(
    f: &Features,
    g: &Game,
    player_id: &str,
    b: &Budget,
    team_rr: bool,
    sc: &mut Scratch,
) -> Option<Reach> {
    let (start, ma, spent, cap, gate) = (b.start, b.ma, b.spent, b.cap, b.gate);
    if cap <= 0 || !on_pitch(start.x, start.y) {
        return None;
    }
    let player = g.player(player_id)?;
    let home = g.team_home.has_player(player_id);

    let ag = player.agility_with_modifiers();
    let gt = gfi_target(weather_of(g));
    let has_dodge = player.has_skill(SkillId::Dodge);
    let has_sure_feet = player.has_skill(SkillId::SureFeet);
    let s = side_idx(home);

    let mut cell = std::mem::take(&mut sc.cell);
    let mut order = std::mem::take(&mut sc.order);
    let mut heap = std::mem::take(&mut sc.heap);
    cell.clear();
    cell.resize(CELLS, UNREACHED);
    order.clear();
    heap.clear();

    let si = ixc(start);
    cell[si] = ReachCell { key: 0, cost: 0, gfi: 0, prev: NO_PREV, seen: false };
    heap.push(HeapItem { key: 0, cost: 0, idx: si as u16 });

    while let Some(HeapItem { key, cost, idx }) = heap.pop() {
        let i = idx as usize;
        if cell[i].seen || key > cell[i].key {
            continue;
        }
        cell[i].seen = true;
        if i != si {
            order.push(idx);
        }
        if cost as i32 >= cap {
            continue;
        }
        let c = coord_of(i);
        let leaving_tz = f.tz[s][i] > 0;
        // The team re-roll is worth its full value on the FIRST roll of a path and nothing after.
        let first_roll = cell[i].key == 0;

        for n in c.neighbours() {
            if !on_pitch(n.x, n.y) {
                continue;
            }
            let j = ixc(n);
            if f.occupied(j) {
                continue;
            }
            let ncost = cost + 1;
            if ncost as i32 > cap {
                continue;
            }
            let mut p_step = 1.0f32;
            let mut used_rr = false;
            if leaving_tz {
                let t = dodge_target(g.rules, ag, f.tz[s][j] as i32);
                let raw = p_roll(t);
                if has_dodge {
                    p_step *= p_with_reroll(raw, 1.0);
                } else if team_rr && first_roll {
                    p_step *= p_with_reroll(raw, 1.0);
                    used_rr = true;
                } else {
                    p_step *= raw;
                }
            }
            let gfi_here = ncost as i32 + spent > ma;
            if gfi_here {
                let raw = p_roll(gt);
                if has_sure_feet || (team_rr && first_roll && !used_rr) {
                    p_step *= p_with_reroll(raw, 1.0);
                } else {
                    p_step *= raw;
                }
            }
            // `as u32` saturates in Rust but not in Java, so clamp explicitly and go through
            // i64: the increment is provably in [0, 56_600] (p_step >= 1e-6, so -ln <= 13.82,
            // times KEY_SCALE = 4096) and the accumulated key cannot exceed ~453k over a path of
            // at most MA+2 steps -- but stating it here means the Java twin needs no reasoning
            // about out-of-range float-to-int conversion, which the two languages define
            // differently. See AGENT_CONTRACT.md section 10.
            let inc = (-ln_f32(p_step.max(1e-6)) * KEY_SCALE).clamp(0.0, 1.0e9) as i64 as u32;
            let nkey = key + inc;
            if nkey < cell[j].key {
                cell[j] = ReachCell {
                    key: nkey,
                    cost: ncost,
                    gfi: cell[i].gfi + gfi_here as u8,
                    prev: idx,
                    seen: false,
                };
                heap.push(HeapItem { key: nkey, cost: ncost, idx: j as u16 });
            }
        }
    }

    // §9's determinism rule: the output order must not depend on heap tie-breaking.
    order.sort_unstable();
    sc.heap = heap;
    Some(Reach { cell, order, start: si, gate })
}

/// Hand the buffers back so the next search reuses the allocation (§20.10).
fn recycle(sc: &mut Scratch, r: Reach) {
    sc.cell = r.cell;
    sc.order = r.order;
}

// ────────────────────────────────────────────────────── value model (§5, §20.4)

struct Mover {
    home: bool,
    is_carrier: bool,
    ma: i32,
    ag: i32,
    str_: i32,
    sure_hands: bool,
    side_step: bool,
    has_catch: bool,
    d_now: i32,
    turns_left: i32,
    unactivated: f32,
}

#[inline]
fn urgency(d_sq: i32, ma: i32, turns_left: i32) -> f32 {
    let tts = ((d_sq as f32) / (ma.max(1) as f32)).ceil() as i32;
    (1.0 - (turns_left - tts) as f32 / 3.0).clamp(0.0, 1.0)
}

/// §20.4 — the mover-dependent part only. Everything else is an array read.
fn value_at(f: &Features, i: usize, m: &Mover) -> (f32, Rule) {
    let sq = coord_of(i);
    let d_sq = endzone_distance(sq, m.home);
    let s = side_idx(m.home);

    let sideline = if sq.y == 0 || sq.y == YMAX {
        if m.side_step {
            1.0
        } else {
            0.25
        }
    } else if m.is_carrier && (sq.y == 1 || sq.y == YMAX - 1) {
        0.6
    } else {
        1.0
    };
    let exposure = f.exposure(i, m.home, m.str_);

    if m.is_carrier {
        let base = if d_sq == 0 {
            1.0
        } else {
            // A3: measure the gain against what THIS activation could reach, not the whole pitch.
            let max_gain = m.d_now.min(m.ma + 2).max(1);
            let advance = ((m.d_now - d_sq) as f32 / max_gain as f32).clamp(0.0, 1.0);
            // If the endzone is out of reach in the turns the half has left, running there cannot
            // score and must not be priced as though it could. `urgency` alone gets this backwards:
            // it saturates at 1.0 exactly when the score becomes impossible, INCREASING the value
            // of a pointless advance. The ground still has some worth (field position, the next
            // drive), which is what the residual is for. This is also what lets a rescue pass win -
            // it now competes against a correctly-valued run rather than an inflated one.
            let tts = ((d_sq as f32) / (m.ma.max(1) as f32)).ceil() as i32;
            let reachable_in_time = if tts <= m.turns_left {
                1.0
            } else {
                HOPELESS_DAMP
            };
            (0.15 + 0.85 * advance)
                * (0.75 + 0.5 * urgency(d_sq, m.ma, m.turns_left))
                * reachable_in_time
        };
        let v = base * sideline * exposure * f.lane[s][i];
        return (v, if d_sq == 0 { Rule::ScoreTouchdown } else { Rule::ScoreAdvance });
    }

    // Pickup — the single highest-value thing on the board while the ball is loose.
    if f.ball_loose && Some(sq) == f.ball {
        let tgt = (m.ag + f.tz[s][i] as i32).max(2);
        let raw = p_roll(tgt);
        let p = if m.sure_hands { p_with_reroll(raw, 1.0) } else { raw };
        let v = (0.55 + 0.45 * p) * sideline * exposure * f.lane[s][i];
        return (v, Rule::Pickup);
    }

    // RECEIVER: could he catch here, and then run it in next turn? That is worth far more than
    // standing in a screen, and it is the only thing that ever gives a pass somewhere to go.
    // Scaled by how well he actually catches — an accurate pass is caught on AG−1.
    let mut support = f.support[s][i];
    if let Some(cc) = f.carrier_at[s] {
        let dx = (sq.x - cc.x).abs();
        let dy = (sq.y - cc.y).abs();
        // The range table tops out below 14 in each axis; anything beyond cannot be thrown at all.
        let throwable = dx < 14 && dy < 14 && (dx.max(dy) > 0);
        // Could he cover the remaining ground himself on the following turn?
        let can_run_it_in = d_sq <= m.ma + 2;
        // And is this actually a RECEIVER position - ahead of the ball, not beside it? Without
        // this the intent fired on half the pitch and pulled the whole team off the cage:
        // measured, touchdowns fell 2.26 -> 1.94 while passes rose 0.00 -> 0.11.
        let ahead_of_ball = d_sq < endzone_distance(cc, m.home);
        if throwable && can_run_it_in && ahead_of_ball {
            let raw = p_roll((m.ag - 1 + f.tz[s][i] as i32).max(2));
            let catch_q = if m.has_catch { p_with_reroll(raw, 1.0) } else { raw };
            let closeness =
                1.0 - (d_sq as f32 / (m.ma + 2).max(1) as f32).clamp(0.0, 1.0) * 0.35;
            // Deliberately below a threatened cage (0.75): escorting the ball beats running a
            // route when the carrier is under pressure.
            support = support.max(0.30 * catch_q + 0.20 * closeness);
        }
    }
    (support * sideline * exposure, Rule::Support)
}

/// The signed weight of arriving at `i`, and the terms behind it.
///
/// The parts are returned as well as the total because "why that square?" is the first question
/// anyone asks of a movement decision, and the total alone cannot answer it.
struct Arrival {
    w: f32,
    p_arrive: f32,
    v: f32,
    gfi: i32,
}

fn arrival_parts(f: &Features, r: &Reach, i: usize, m: &Mover) -> Arrival {
    let pa = r.p_arrive(i);
    let gfi = r.cell[i].gfi as i32;
    if m.is_carrier && endzone_distance(coord_of(i), m.home) == 0 {
        // A touchdown ends the drive: there is no "after" to lose, so only the rush is priced.
        return Arrival { w: pa - rush_penalty(gfi, true), p_arrive: pa, v: 1.0, gfi };
    }
    let (v, _) = value_at(f, i, m);
    let w = pa * v
        - (1.0 - pa) * c_turnover(m.unactivated, gfi, m.is_carrier)
        - rush_penalty(gfi, m.is_carrier);
    Arrival { w, p_arrive: pa, v, gfi }
}

/// §5.3's `p·V − (1−p)·c`, plus the rush aversion.
#[inline]
fn arrival_weight(f: &Features, r: &Reach, i: usize, m: &Mover) -> f32 {
    arrival_parts(f, r, i, m).w
}

/// Best destination for a plain move.
///
/// §20.10 originally pruned this to the `TOPK_DEST` destinations with the highest `p_arrive`, on the
/// theory that `V ≤ 1` makes `p_arrive` an admissible bound. **It measured a disaster** — 1.76
/// touchdowns per game fell to 0.19 — and the reason is that the bound is admissible but the
/// *ranking* is not: a one-square shuffle arrives with p = 1.0 while a six-square scoring run
/// arrives with p ≈ 0.3, so the top-K by arrival probability is almost exactly the set of moves that
/// go nowhere. The long runs that score were cut before they were ever scored.
///
/// After §20.4's rasters, the full value computation is a handful of multiplies, so the whole
/// reachable set — around ninety squares — costs a couple of microseconds to score properly. The
/// pruning was buying nothing and paying for it in touchdowns.
fn best_move(f: &Features, r: &Reach, m: &Mover) -> (f32, Option<usize>) {
    let mut best = (f32::MIN, None);
    for &idx in &r.order {
        let i = idx as usize;
        let w = arrival_weight(f, r, i, m);
        if w > best.0 {
            best = (w, Some(i));
        }
    }
    match best.1 {
        Some(_) => (best.0, best.1),
        None => (0.0, None),
    }
}

/// The best K destinations, weight-ordered.
///
/// `best_move` returns only the argmax, which is right for a forced re-plan but wrong for the
/// activation menu: it made the destination invisible to sampling and to any reader. Ties break on
/// the flat index so the order is deterministic (§9).
/// The board facts `replay_plan` needs, gathered once by the caller.
///
/// Pulled out as a struct so the decision below is a pure function of them: it is a state machine
/// with seven exits and four engine guards, and the only way to pin it against the Java twin is to
/// be able to call it with made-up inputs.
#[derive(Clone, Copy)]
pub(crate) struct ReplayFacts {
    /// The action the ENGINE currently has on the acting player, which is what gates every
    /// terminal dispatch. `StepInitMoving` falls THROUGH when its guard fails and re-emits the
    /// same prompt, so a rejected action resent would spin forever.
    pub(crate) pa_now: Option<PlayerAction>,
    pub(crate) has_blocked: bool,
    pub(crate) has_fouled: bool,
    /// Whether the plan's victim/receiver is adjacent right now.
    pub(crate) target_adjacent: bool,
    /// Whether the plan's receiver is still on the pitch.
    pub(crate) target_on_pitch: bool,
    /// Whether the offered squares include the next step of the planned path.
    pub(crate) squares_include_next: bool,
    pub(crate) squares_empty: bool,
}

/// What `handle_move` decided, before it is turned into an `Action`.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) enum Replay {
    /// Deliver the whole remaining path in one answer; the engine walks it.
    DeliverPath,
    /// Send the plan's terminal action.
    FireTerminal,
    EndPlayerAction,
    /// No usable plan — re-decide from scratch.
    Replan,
}

/// The plan-replay state machine, as a pure function.
///
/// Seven exits, and the ordering between them is the whole content:
///
/// - an EMPTY square list means no MOVEMENT is left, **not** that there is nothing to do. A pending
///   give, throw, blitz or foul still has to be sent; bailing here threw away every give whose
///   run-up used the carrier's whole move, which is most of the good ones.
/// - a path is only delivered when the offered squares actually contain its next step. If the board
///   moved under the plan, fall through and re-decide rather than insisting.
/// - every terminal action is gated on the engine's OWN condition and latched with `fired`, because
///   `StepInitMoving` re-emits this prompt when its guard fails, so resending would spin.
/// - `Move`/`Immediate` that has been delivered ends: moving twice reaches the same square.
///   A plan that never carried a path (a tier-1 proxy pick) still has to decide, so it falls
///   through instead.
/// - once `fired`, only a `Blitz` has anything legitimate left (its post-block movement); a
///   `Pickup` genuinely changed the value model and re-decides.
pub(crate) fn replay_plan(
    kind: &PlanKind,
    plan_is_for_this_player: bool,
    path_empty: bool,
    delivered: bool,
    fired: bool,
    f: &ReplayFacts,
) -> Replay {
    let terminal_pending = matches!(
        kind,
        PlanKind::HandOff { .. } | PlanKind::Pass { .. } | PlanKind::Blitz { .. }
            | PlanKind::Foul { .. }
    );
    // NOTE the asymmetry, and it is the original's: `terminal_pending` is read off the plan
    // WITHOUT checking whose plan it is. A pending give belonging to another player still
    // suppresses the early exit here. Tightening it to "this player's plan" is the obvious
    // cleanup and would change behaviour.
    if f.squares_empty && !terminal_pending {
        return Replay::EndPlayerAction;
    }
    if !plan_is_for_this_player {
        return Replay::Replan;
    }
    if !path_empty {
        if f.squares_include_next {
            return Replay::DeliverPath;
        }
        return Replay::Replan;
    }
    if !fired {
        let dispatchable = match kind {
            PlanKind::Blitz { .. } => {
                matches!(
                    f.pa_now,
                    Some(PlayerAction::BlitzMove) | Some(PlayerAction::KickEmBlitz)
                ) && !f.has_blocked
                    && f.target_adjacent
            }
            PlanKind::Foul { .. } => {
                f.pa_now == Some(PlayerAction::FoulMove) && !f.has_fouled && f.target_adjacent
            }
            PlanKind::Pass { .. } => {
                matches!(
                    f.pa_now,
                    Some(PlayerAction::PassMove)
                        | Some(PlayerAction::Pass)
                        | Some(PlayerAction::HailMaryPass)
                ) && f.target_on_pitch
            }
            PlanKind::HandOff { .. } => {
                matches!(
                    f.pa_now,
                    Some(PlayerAction::HandOverMove) | Some(PlayerAction::HandOver)
                ) && f.target_on_pitch
            }
            PlanKind::Move | PlanKind::Immediate | PlanKind::Pickup => false,
        };
        if dispatchable {
            return Replay::FireTerminal;
        }
        if matches!(kind, PlanKind::Move | PlanKind::Immediate) && delivered {
            return Replay::EndPlayerAction;
        }
        if !matches!(kind, PlanKind::Pickup | PlanKind::Move) && delivered {
            return Replay::EndPlayerAction;
        }
        return Replay::Replan;
    }
    // Already fired: the activation is over, except for a blitz's post-block movement and a
    // pickup, which genuinely changed the value model.
    if !matches!(kind, PlanKind::Pickup) && !matches!(kind, PlanKind::Blitz { .. }) {
        return Replay::EndPlayerAction;
    }
    Replay::Replan
}

/// Group the candidate list by DECLARATION/// Group the candidate list by DECLARATION — the `(player, action)` pair the engine actually
/// receives.
///
/// `build_plans` emits a player's options one action at a time, so a declaration's candidates are a
/// CONTIGUOUS RUN and detecting runs is linear. The obvious keyed lookup was `O(groups)` of string
/// comparison per candidate and cost 30 ms a game on its own.
///
/// Extracted so the golden emitter and the live path cannot drift: a fixture that reimplements the
/// grouping is pinning its own copy.
fn group_declarations(cands: &[Candidate]) -> Vec<Vec<usize>> {
    let mut groups: Vec<Vec<usize>> = Vec::new();
    for (i, c) in cands.iter().enumerate() {
        let same = i > 0 && cands[i - 1].pac == c.pac && cands[i - 1].player == c.player;
        if same {
            groups.last_mut().expect("run started").push(i);
        } else {
            groups.push(vec![i]);
        }
    }
    groups
}

fn top_moves(f: &Features, r: &Reach, m: &Mover, k: usize) -> Vec<(f32, usize)> {
    let mut v: Vec<(f32, usize)> = r
        .order
        .iter()
        .map(|&idx| (arrival_weight(f, r, idx as usize, m), idx as usize))
        .collect();
    v.sort_by(|a, b| {
        b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal).then(a.1.cmp(&b.1))
    });
    v.truncate(k);
    v
}

/// Fold the chance of never arriving into a plan's weight. NOT `w * p`: that is right only for a
/// positive weight and turns a bad plan reached by a risky route into a better-looking one.
fn risked(w: f32, p_arrive: f32, m: &Mover) -> f32 {
    p_arrive * w - (1.0 - p_arrive) * c_turnover(m.unactivated, 0, true)
}

/// Where a player stands.
fn m_coord(g: &Game, pid: &str) -> FieldCoordinate {
    g.field_model.player_coordinate(pid).unwrap_or(FieldCoordinate::new(0, 0))
}

/// Squares a carrier might throw from: where he stands, plus the best a run-up could reach.
/// Standing still is always first, so "use none of my move" is never lost.
fn run_up_squares(r: Option<&Reach>, m: &Mover, here: FieldCoordinate) -> Vec<usize> {
    let mut out = vec![ixc(here)];
    if let Some(r) = r {
        let mut v: Vec<(f32, usize)> = r
            .order
            .iter()
            .map(|&idx| {
                let i = idx as usize;
                let fwd = (m.d_now - endzone_distance(coord_of(i), m.home)) as f32;
                (r.p_arrive(i) * (1.0 + 0.25 * fwd.max(0.0)), i)
            })
            .collect();
        v.sort_by(|a, b| {
            b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal).then(a.1.cmp(&b.1))
        });
        for (_, i) in v.into_iter().take(THROW_SPOTS) {
            if !out.contains(&i) {
                out.push(i);
            }
        }
    }
    out
}

/// §20.3 tier-1 proxy: a search-free estimate of what a player could be worth this activation.
/// No Dijkstra — the eight adjacent squares exactly, plus an admissible ceiling over everything
/// inside MA+2 read straight off the rasters.
fn proxy_value(f: &Features, g: &Game, pid: &str, m: &Mover) -> f32 {
    let c = match g.field_model.player_coordinate(pid) {
        Some(c) => c,
        None => return 0.0,
    };
    let mut best = 0.0f32;
    for n in c.neighbours() {
        if on_pitch(n.x, n.y) {
            let i = ixc(n);
            if !f.occupied(i) {
                best = best.max(value_at(f, i, m).0);
            }
        }
    }
    let r = m.ma + 2;
    let s = side_idx(m.home);
    let mut ceiling = 0.0f32;
    for y in (c.y - r).max(0)..=(c.y + r).min(YMAX) {
        for x in (c.x - r).max(0)..=(c.x + r).min(XMAX) {
            let i = ix(x, y);
            if f.occupied(i) {
                continue;
            }
            let v: f32 = if m.is_carrier {
                let d_sq = endzone_distance(FieldCoordinate::new(x, y), m.home);
                let max_gain = m.d_now.min(m.ma + 2).max(1);
                let adv = ((m.d_now - d_sq) as f32 / max_gain as f32).clamp(0.0, 1.0);
                (0.15 + 0.85 * adv) * f.lane[s][i]
            } else if f.ball_loose && f.ball == Some(FieldCoordinate::new(x, y)) {
                0.9
            } else {
                f.support[s][i]
            };
            if v > ceiling {
                ceiling = v;
            }
        }
    }
    // The ceiling is optimistic by construction, so discount it rather than trust it.
    best.max(0.55 * ceiling)
}

// ────────────────────────────────────────────────────────── plans (§20.9, §20.1)

/// What the activation is actually FOR. The engine's follow-up prompts replay this instead of
/// re-deciding, which is both the speed win and the reason a blitz now lands.
#[derive(Clone, Debug, PartialEq)]
enum PlanKind {
    /// Plain movement. Once the path is delivered the activation is DONE — moving twice reaches
    /// the same square as moving once (§20.1).
    Move,
    /// Movement onto the loose ball. Picking it up changes the value model, so the activation may
    /// legitimately continue afterwards.
    Pickup,
    /// Move adjacent to the victim, then send the block that dispatches the blitz. Without the
    /// second half the blitz is declared and abandoned: 7.8 per game, 0% follow-through (§19.2).
    Blitz { victim: String },
    /// Move adjacent to the victim, then foul it.
    Foul { victim: String },
    /// Move, then throw. `Move → Pass` is the case the fragmented action space could not express
    /// at all (§19.3).
    Pass { receiver: String },
    HandOff { receiver: String },
    /// Declared and resolved without a movement phase.
    Immediate,
}

#[derive(Clone, Debug)]
struct Plan {
    player: String,
    kind: PlanKind,
    /// Remaining squares of the planned path, in order.
    path: Vec<FieldCoordinate>,
    /// Set once the path has been handed to the engine.
    delivered: bool,
    /// Set once the plan's terminal action has been attempted, so it is never resent.
    fired: bool,
}

/// One enumerated (player, plan) candidate — §20.9's action space.
struct Candidate {
    weight: f32,
    player: String,
    pac: PlayerActionChoice,
    /// Folded into the declaration so the declaration and the follow-up prompt agree (§19.2).
    target: Option<String>,
    kind: PlanKind,
    /// An explicit route, for plans that need one (the two-leg blitz).
    path: Vec<FieldCoordinate>,
    /// A plain move's destination, as a flat cell index. The path is walked back only for the
    /// option that is actually sampled — materialising two thousand of them would cost far more
    /// than the search that produced them.
    dest: Option<usize>,
    why: Rule,
    /// Shown to a reader; the engine never sees it.
    note: String,
}

// ─────────────────────────────────────────────────────────────────── the agent

/// How the agent searches the action space. Same weights either way — see the module note.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    /// One draw from the full joint action space: every player × action × target × destination.
    Wide,
    /// Wide, with passing and hand-offs switched off. Exists so "did the ball-move logic actually
    /// make the agent stronger?" can be answered head-to-head instead of by comparing self-play
    /// touchdown rates, which measure the pair rather than the policy (§23.3).
    WideNoBall,
    /// Wide, with PASSING switched off but hand-offs left on. The `WideNoBall` control answers
    /// "do ball moves help?" as one question, and hand-offs outnumber passes better than ten to
    /// one — so a positive reading there is compatible with passing being worthless or actively
    /// harmful, hidden under the hand-offs carrying it. This isolates the throw: A/B `Wide`
    /// against `WideNoPass` and the only difference left is passing.
    WideNoPass,
    /// Wide, with HAND-OFFS switched off but passing left on — the mirror of `WideNoPass`, so each
    /// half of the ball game can be read on its own. `WideNoBall` answers only "do ball moves
    /// help?", and hand-offs outnumber passes sixteen to one, so it cannot say which half earned
    /// the reading.
    WideNoHandOff,
    /// A chain of small draws: player, then action-and-target, then one movement square at a time.
    Deep,
}

/// The prompt classes the heuristic agent can score, one bit each in a [`ClassMask`].
///
/// This exists for the Java-parity ladder (`docs/PARITY_HEURISTIC_CAMPAIGN.md`). Porting 3,600
/// lines of scorer and only then finding out whether the two engines agree would mean debugging
/// every prompt class at once. Instead the mask lets exactly one class at a time be switched from
/// "answered by the parity `RandomAgent` contract" to "scored by the heuristic", so the 100-seed
/// gate is green at rung 0 by construction and each rung adds one well-defined thing to blame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum PromptClass {
    CoinChoice = 0,
    ReceiveChoice = 1,
    KickBall = 2,
    Touchback = 3,
    TeamSetup = 4,
    FollowUp = 5,
    ReRollOffer = 6,
    SkillUse = 7,
    Interception = 8,
    BlockChoice = 9,
    Pushback = 10,
    BlockTarget = 11,
    BlitzTarget = 12,
    ActivatePlayer = 13,
    Move = 14,
    /// Everything the heuristic does not score itself; always delegated.
    Other = 15,
}

impl PromptClass {
    /// The CLI spelling, and the name used in `--heur-classes`.
    pub fn name(self) -> &'static str {
        match self {
            PromptClass::CoinChoice => "coin",
            PromptClass::ReceiveChoice => "receive",
            PromptClass::KickBall => "kick",
            PromptClass::Touchback => "touchback",
            PromptClass::TeamSetup => "setup",
            PromptClass::FollowUp => "followup",
            PromptClass::ReRollOffer => "reroll",
            PromptClass::SkillUse => "skill",
            PromptClass::Interception => "intercept",
            PromptClass::BlockChoice => "blockchoice",
            PromptClass::Pushback => "pushback",
            PromptClass::BlockTarget => "blocktarget",
            PromptClass::BlitzTarget => "blitztarget",
            PromptClass::ActivatePlayer => "activate",
            PromptClass::Move => "move",
            PromptClass::Other => "other",
        }
    }

    pub const ALL: [PromptClass; 16] = [
        PromptClass::CoinChoice,
        PromptClass::ReceiveChoice,
        PromptClass::KickBall,
        PromptClass::Touchback,
        PromptClass::TeamSetup,
        PromptClass::FollowUp,
        PromptClass::ReRollOffer,
        PromptClass::SkillUse,
        PromptClass::Interception,
        PromptClass::BlockChoice,
        PromptClass::Pushback,
        PromptClass::BlockTarget,
        PromptClass::BlitzTarget,
        PromptClass::ActivatePlayer,
        PromptClass::Move,
        PromptClass::Other,
    ];
}

/// Which [`PromptClass`]es the heuristic scores. Everything else is answered by the embedded
/// parity `RandomAgent`, byte-for-byte as `AGENT_CONTRACT.md` specifies.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClassMask(u32);

impl Default for ClassMask {
    fn default() -> Self {
        ClassMask::ALL
    }
}

impl ClassMask {
    /// Delegate everything — rung 0. Must reproduce the random-agent gate exactly.
    pub const NONE: ClassMask = ClassMask(0);
    /// Score everything the agent knows how to score. The normal, non-parity configuration.
    pub const ALL: ClassMask = ClassMask(u32::MAX);

    pub fn has(self, c: PromptClass) -> bool {
        self.0 & (1u32 << (c as u8)) != 0
    }

    pub fn with(self, c: PromptClass) -> ClassMask {
        ClassMask(self.0 | (1u32 << (c as u8)))
    }

    /// The canonical spelling of this mask, round-tripping through [`ClassMask::parse`]. Used to
    /// hand the Java side the IDENTICAL mask: passing through the user's raw string instead would
    /// let `--heur-classes coin,coin` or a differently-ordered list mean two different things.
    pub fn to_spec(self) -> String {
        if self == ClassMask::NONE {
            return "none".to_string();
        }
        let on: Vec<&str> =
            PromptClass::ALL.iter().filter(|c| self.has(**c)).map(|c| c.name()).collect();
        if on.len() == PromptClass::ALL.len() {
            return "all".to_string();
        }
        on.join(",")
    }

    /// `all`, `none`, or a comma-separated list of [`PromptClass::name`]s.
    pub fn parse(spec: &str) -> Result<ClassMask, String> {
        let spec = spec.trim();
        if spec.eq_ignore_ascii_case("all") {
            return Ok(ClassMask::ALL);
        }
        if spec.is_empty() || spec.eq_ignore_ascii_case("none") {
            return Ok(ClassMask::NONE);
        }
        let mut m = ClassMask::NONE;
        for tok in spec.split(',') {
            let tok = tok.trim();
            if tok.is_empty() {
                continue;
            }
            match PromptClass::ALL.iter().find(|c| c.name() == tok) {
                Some(c) => m = m.with(*c),
                None => {
                    let known: Vec<&str> = PromptClass::ALL.iter().map(|c| c.name()).collect();
                    return Err(format!("unknown prompt class '{tok}'; known: {}", known.join(",")));
                }
            }
        }
        Ok(m)
    }
}

/// Which class a prompt belongs to, for [`ClassMask`].
pub fn prompt_class_of(p: &AgentPrompt) -> PromptClass {
    match p {
        AgentPrompt::CoinChoice { .. } => PromptClass::CoinChoice,
        AgentPrompt::ReceiveChoice { .. } => PromptClass::ReceiveChoice,
        AgentPrompt::KickBall => PromptClass::KickBall,
        AgentPrompt::Touchback { .. } => PromptClass::Touchback,
        AgentPrompt::TeamSetup { .. } => PromptClass::TeamSetup,
        AgentPrompt::FollowUp { .. } => PromptClass::FollowUp,
        AgentPrompt::ReRollOffer { .. } => PromptClass::ReRollOffer,
        AgentPrompt::SkillUse { .. } => PromptClass::SkillUse,
        AgentPrompt::Interception { .. } => PromptClass::Interception,
        AgentPrompt::BlockChoice { .. } => PromptClass::BlockChoice,
        AgentPrompt::Pushback { .. } => PromptClass::Pushback,
        AgentPrompt::BlockTarget { .. } => PromptClass::BlockTarget,
        AgentPrompt::BlitzTarget { .. } => PromptClass::BlitzTarget,
        AgentPrompt::ActivatePlayer { .. } => PromptClass::ActivatePlayer,
        AgentPrompt::Move { .. } => PromptClass::Move,
        _ => PromptClass::Other,
    }
}

pub struct HeuristicAgent {
    mode: Mode,
    /// Which prompt classes this agent scores; the rest go to `parity`. `ALL` outside the
    /// Java-port ladder.
    classes: ClassMask,
    /// The parity `RandomAgent`, for every class not in `classes`. Only ever consulted when the
    /// mask is partial, so its RNG streams stay untouched in the normal `ALL` configuration.
    parity: Box<RandomAgent>,
    rng: Xoshiro256StarStar,
    /// Multiplies every temperature. 1.0 = the §8 table; a very large value = uniform sampling over
    /// the identical option set; 0.0 = true argmax with no RNG consumed at all.
    temp_scale: f32,
    fallback: UniformAgent,
    buf: Scored,
    feat: Option<Features>,
    plan: Option<Plan>,
    sc: Scratch,
    seen_action: HashMap<String, u32>,
    seen_bucket: HashMap<u64, u32>,
    last_turn_key: Option<(i32, i32, bool)>,
    used_this_turn: HashSet<String>,
    /// Java `ParityRunner.justDeselected`. Set when a non-REGULAR window ends a turn; it then ends
    /// the FOLLOWING turn too, because in Java that window's activation was the original team's
    /// last processed one. `RandomAgent` carries the same flag.
    just_deselected: bool,
    /// Who just received the ball. The second half of a ball-move plan: he has to be the next one
    /// activated, or the throw bought nothing.
    awaiting_run: Option<String>,
    /// Counts activation decisions, so `FFB_CAND=<k>` can name one. Diagnostics only.
    probe_act: u32,
    /// Total sampler draws consumed. The Java `Sampler` keeps the same counter, and comparing the
    /// two is how a desynchronised RNG stream is localised to the prompt that caused it.
    probe_draws: u64,
    /// `FFB_HEUR_DUMP` — record every decision's full distribution for the visualiser.
    dump_enabled: bool,
    /// The option set of the most recent decision, with probabilities. Empty unless dumping.
    pub last_options: Vec<ScoredOption>,
    /// Index into `last_options` of the option that was actually taken.
    pub last_chosen: usize,
}

impl HeuristicAgent {
    pub fn new(seed: u64, temp_scale: f32) -> Self {
        Self::with_mode(seed, temp_scale, Mode::Wide)
    }

    pub fn with_mode(seed: u64, temp_scale: f32, mode: Mode) -> Self {
        HeuristicAgent {
            mode,
            rng: Xoshiro256StarStar::seed_from_u64(seed ^ 0x4845_5552_4953_5449),
            temp_scale,
            classes: ClassMask::ALL,
            parity: Box::new(RandomAgent::new_parity(seed)),
            fallback: UniformAgent::new(seed),
            buf: Scored::default(),
            feat: None,
            plan: None,
            sc: Scratch::default(),
            seen_action: HashMap::new(),
            seen_bucket: HashMap::new(),
            last_turn_key: None,
            used_this_turn: HashSet::new(),
            just_deselected: false,
            awaiting_run: None,
            probe_act: 0,
            probe_draws: 0,
            dump_enabled: std::env::var_os("FFB_HEUR_DUMP").is_some(),
            last_options: Vec::new(),
            last_chosen: 0,
        }
    }

    /// Restrict the agent to scoring only `classes`, delegating everything else to the parity
    /// `RandomAgent`. This is the Java-port ladder's rung selector; see [`ClassMask`].
    pub fn with_classes(seed: u64, temp_scale: f32, mode: Mode, classes: ClassMask) -> Self {
        let mut a = Self::with_mode(seed, temp_scale, mode);
        a.classes = classes;
        a
    }

    /// Uniform sampling over whatever the scorer enumerated.
    pub fn new_uniform(seed: u64) -> Self {
        Self::new(seed, 1.0e6)
    }
    /// Pure argmax — no RNG consumed, fully deterministic for a given board.
    pub fn new_argmax(seed: u64) -> Self {
        Self::new(seed, 0.0)
    }

    // ── sampling ───────────────────────────────────────────────────────────

    fn unit(&mut self) -> f32 {
        self.probe_draws += 1;
        (self.rng.next_u64() >> 11) as f32 / (1u64 << 53) as f32
    }

    fn argmax(&self) -> usize {
        let mut bi = 0usize;
        let mut bw = f32::MIN;
        for (i, o) in self.buf.options.iter().enumerate() {
            if o.weight > bw {
                bw = o.weight;
                bi = i;
            }
        }
        bi
    }

    fn sample(&mut self, t_base: f32) -> usize {
        let idx = self.pick(t_base);
        if self.dump_enabled {
            self.record_distribution(t_base, idx);
        }
        self.last_chosen = idx;
        idx
    }

    /// The distribution the sampler used, recorded for the visualiser. Mirrors `pick` exactly:
    /// argmax puts all the mass on one option, otherwise it is the softmax at the same temperature.
    fn record_distribution(&mut self, t_base: f32, chosen: usize) {
        let n = self.buf.options.len();
        if n == 0 {
            self.last_options.clear();
            return;
        }
        let mut ps = vec![0.0f32; n];
        if n == 1 || self.temp_scale <= 0.0 {
            ps[chosen.min(n - 1)] = 1.0;
        } else {
            let t = (t_base * self.temp_scale).max(1e-6);
            let max = self.buf.options.iter().map(|o| o.weight).fold(f32::MIN, f32::max);
            let mut acc = 0.0f32;
            for (i, o) in self.buf.options.iter().enumerate() {
                ps[i] = exp_f32((o.weight - max) / t);
                acc += ps[i];
            }
            if acc > 0.0 {
                for v in ps.iter_mut() {
                    *v /= acc;
                }
            }
        }
        self.record_probs(&ps, chosen);
    }

    fn record_probs(&mut self, ps: &[f32], chosen: usize) {
        self.last_options.clear();
        for (i, o) in self.buf.options.iter().enumerate() {
            self.last_options.push(ScoredOption {
                action: o.action.clone(),
                w: o.weight,
                p: ps.get(i).copied().unwrap_or(0.0),
                why: format!("{:?}", o.why),
                note: o.note.clone(),
            });
        }
        self.last_chosen = chosen;
    }

    /// Softmax over `w` at temperature `t`, returning the pick and the full distribution.
    /// `temp_scale <= 0` degenerates to argmax with all the mass on one entry, as everywhere else.
    fn softmax_pick(&mut self, w: &[f32], t_base: f32) -> (usize, Vec<f32>) {
        let n = w.len();
        if n == 0 {
            return (0, Vec::new());
        }
        if n == 1 {
            return (0, vec![1.0]);
        }
        if self.temp_scale <= 0.0 {
            let mut bi = 0;
            for i in 1..n {
                if w[i] > w[bi] {
                    bi = i;
                }
            }
            let mut ps = vec![0.0f32; n];
            ps[bi] = 1.0;
            return (bi, ps);
        }
        let t = (t_base * self.temp_scale).max(1e-6);
        let max = w.iter().copied().fold(f32::MIN, f32::max);
        let mut ps: Vec<f32> = w.iter().map(|v| exp_f32((v - max) / t)).collect();
        let acc: f32 = ps.iter().sum();
        if acc > 0.0 {
            for v in ps.iter_mut() {
                *v /= acc;
            }
        }
        let r = self.unit();
        let mut c = 0.0f32;
        let mut pick = n - 1;
        for (i, v) in ps.iter().enumerate() {
            c += *v;
            if r < c {
                pick = i;
                break;
            }
        }
        (pick, ps)
    }

    fn pick(&mut self, t_base: f32) -> usize {
        let n = self.buf.options.len();
        // §20.10 — nothing to decide, so do not pay for a softmax or consume a draw.
        if n <= 1 {
            return 0;
        }
        if self.temp_scale <= 0.0 {
            return self.argmax();
        }
        let t = (t_base * self.temp_scale).max(1e-6);
        let max = self.buf.options.iter().map(|o| o.weight).fold(f32::MIN, f32::max);
        let eps = if self.temp_scale < 0.1 { 0.0 } else { EPS };
        if eps > 0.0 && self.unit() < eps {
            // This second draw does not go through unit(), and Java counts it. Count it
            // here too or the two probe totals disagree by one on every epsilon hit -- which is
            // exactly the false 'divergence' the bb2016 measurement first showed.
            self.probe_draws += 1;
            return ((self.rng.next_u64() as usize) % n).min(n - 1);
        }
        let mut acc = 0.0f32;
        let mut cum: Vec<f32> = Vec::with_capacity(n);
        for o in &self.buf.options {
            acc += exp_f32((o.weight - max) / t);
            cum.push(acc);
        }
        let r = self.unit() * acc;
        cum.partition_point(|&c| c < r).min(n - 1)
    }

    fn take(&mut self, i: usize) -> Action {
        self.buf.options.swap_remove(i).action
    }

    // ── bookkeeping ────────────────────────────────────────────────────────

    /// Returns true when this call started a NEW turn, so the caller can take the turn-start
    /// snapshot the parity contract is defined on.
    fn refresh_turn(&mut self, g: &Game) -> bool {
        let turn_nr = if g.home_playing {
            g.turn_data_home.turn_nr
        } else {
            g.turn_data_away.turn_nr
        };
        let key = (g.half, turn_nr, g.home_playing);
        if self.last_turn_key != Some(key) {
            self.last_turn_key = Some(key);
            self.used_this_turn.clear();
            self.awaiting_run = None;
            return true;
        }
        false
    }

    /// `FFB_ENDTURN`: every EndTurn this agent returns, with the branch that produced it. The
    /// turn boundary is where the amazon reds live, and "which branch ended the turn" is the one
    /// fact neither the state hash nor the candidate summary can show.
    fn probe_endturn(&self, g: &Game, turn_nr: i32, why: &str) {
        if std::env::var_os("FFB_ENDTURN").is_some() {
            eprintln!(
                "RET k={} turn={} side={} mode={:?} why={} used={} latch={}",
                self.probe_act, turn_nr,
                if g.home_playing { "home" } else { "away" },
                g.turn_mode, why, self.used_this_turn.len(), self.just_deselected
            );
        }
    }

    fn bucket(&self, f: &Features, g: &Game) -> u64 {
        let ballz = f.ball.map(|c| (c.x / 5) as u64 * 4 + (c.y / 4) as u64).unwrap_or(31);
        let carried = f.carrier.is_some() as u64;
        let turn = (g.turn_data_home.turn_nr.max(g.turn_data_away.turn_nr) / 3) as u64;
        ballz | (carried << 6) | (turn << 8) | ((weather_of(g) as u64) << 12) | ((g.half as u64) << 16)
    }

    /// §6.5.2's live coverage floor — a coverage tool that costs play strength by construction, so
    /// it is off in the sharp arms and on in the sampling arms.
    fn coverage_floor(&self, key: &str) -> f32 {
        if self.temp_scale < 0.1 {
            return 0.0;
        }
        let seen = *self.seen_action.get(key).unwrap_or(&0);
        0.35 * (1.0 - (seen as f32 / 4.0).min(1.0))
    }

    fn mover_of(&self, g: &Game, f: &Features, pid: &str) -> Option<Mover> {
        let p = g.player(pid)?;
        let c = g.field_model.player_coordinate(pid)?;
        let home = g.team_home.has_player(pid);
        let td = if home { &g.turn_data_home } else { &g.turn_data_away };
        Some(Mover {
            home,
            is_carrier: f.ball_carried && f.ball == Some(c),
            ma: p.movement_with_modifiers(),
            ag: p.agility_with_modifiers(),
            str_: p.strength_with_modifiers(),
            sure_hands: p.has_skill(SkillId::SureHands),
            side_step: p.has_skill(SkillId::SideStep),
            has_catch: p.has_skill(SkillId::Catch),
            d_now: endzone_distance(c, home),
            turns_left: (8 - td.turn_nr).max(0),
            unactivated: f.unactivated[side_idx(home)],
        })
    }

    // ── Move: replay the plan (§20.1, §20.2, §20.9) ────────────────────────

    fn handle_move(
        &mut self,
        g: &Game,
        f: &Features,
        player_id: String,
        squares: Vec<FieldCoordinate>,
    ) -> Action {
        // Java: the engine flow never re-presents INIT_SELECTING phase 2 for a pass-block window
        // mover, so `ParityRunner`'s INIT_MOVING handler deselects immediately -- the mover
        // activates but never moves, and no target is drawn. `RandomAgent` mirrors it (its comment
        // names amazon seeds 8/11, where the On-the-Ball defender stays put in Java); the
        // heuristic knew nothing about pass-block windows at all, which is exactly the roster
        // where they fire: On the Ball is the Amazon Thrower's skill in bb2020 and bb2025.
        // Java `ParityRunner` INIT_SELECTING **phase 2** -- shared by BOTH agent paths (phase 1
        // picks the player and injects CLIENT_ACTING_PLAYER; the NEXT harness iteration, with the
        // acting player set, is phase 2):
        //
        //   if (tier <= 2 || game.getTurnMode() != TurnMode.REGULAR) {
        //       justDeselected = true;
        //       inject(new ClientCommandActingPlayer(null, null, false));   // deselect
        //   } else { sendConcreteAction(game, gameState); }
        //
        // So in ANY non-REGULAR window the picked player is recorded and then deselected without a
        // Move sequence ever being pushed. `FFB_STEPTRACE` on bb2025 seed 33 shows it directly:
        // `JSTATE ... mode=KICKOFF_RETURN ap=H6 act=MOVE cm=0` and the very next iteration
        // `ap=null`, with no INIT_MOVING in between and no `JMOVEP` line for the player.
        //
        // Rust's prompt model merges Java's two commands into ActivatePlayer + Move, so the
        // phase-2 deselect lands HERE, at the Move prompt. ITER21 had this right (`!= Regular`).
        // ITER25 narrowed it to PASS_BLOCK on the strength of a probe that only made sense
        // together with the `push_self` fix landing in the same gate -- and the kickoff-return
        // mover then walked four squares Java never walked (seed 33 `home_06`, (5,9) -> (9,10)),
        // which was every remaining amazon red in both editions.
        if g.turn_mode != ffb_model::enums::TurnMode::Regular {
            self.just_deselected = true;
            return Action::EndPlayerAction;
        }
        // The whole state machine lives in `replay_plan`, so it can be pinned against the Java
        // twin with made-up inputs. What is left here is gathering the board facts it reads and
        // turning its verdict into an `Action`.
        let facts = {
            let here = g.field_model.player_coordinate(&player_id);
            let target = self.plan.as_ref().and_then(|pl| match &pl.kind {
                PlanKind::Blitz { victim } | PlanKind::Foul { victim } => Some(victim.clone()),
                PlanKind::Pass { receiver } | PlanKind::HandOff { receiver } => {
                    Some(receiver.clone())
                }
                _ => None,
            });
            let tc = target.as_deref().and_then(|t| g.field_model.player_coordinate(t));
            ReplayFacts {
                pa_now: g.acting_player.player_action,
                has_blocked: g.acting_player.has_blocked,
                has_fouled: g.acting_player.has_fouled,
                target_adjacent: here
                    .zip(tc)
                    .map(|(a, b)| a.distance_in_steps(b) == 1)
                    .unwrap_or(false),
                target_on_pitch: tc.is_some(),
                squares_include_next: self
                    .plan
                    .as_ref()
                    .and_then(|pl| pl.path.first())
                    .map(|n| squares.contains(n))
                    .unwrap_or(false),
                squares_empty: squares.is_empty(),
            }
        };

        if let Some(mut pl) = self.plan.take() {
            let verdict = replay_plan(
                &pl.kind,
                pl.player == player_id,
                pl.path.is_empty(),
                pl.delivered,
                pl.fired,
                &facts,
            );
            match verdict {
                Replay::DeliverPath => {
                    let path = std::mem::take(&mut pl.path);
                    pl.delivered = true;
                    self.plan = Some(pl);
                    return Action::Move { path };
                }
                Replay::FireTerminal => {
                    let kind = pl.kind.clone();
                    pl.fired = true;
                    self.plan = Some(pl);
                    return match kind {
                        PlanKind::Blitz { victim } => Action::Block { defender_id: victim },
                        PlanKind::Foul { victim } => Action::Foul { target_id: victim },
                        PlanKind::Pass { receiver } => {
                            match g.field_model.player_coordinate(&receiver) {
                                Some(coord) => Action::Pass { coord },
                                None => Action::EndPlayerAction,
                            }
                        }
                        PlanKind::HandOff { receiver } => {
                            Action::HandOff { receiver_id: receiver }
                        }
                        _ => Action::EndPlayerAction,
                    };
                }
                Replay::EndPlayerAction => {
                    // The plan is dropped here exactly as the original did: `take()` above already
                    // removed it and this arm does not put it back.
                    return Action::EndPlayerAction;
                }
                Replay::Replan => {
                    // Keep the plan; the fall-through below re-decides and may overwrite it.
                    self.plan = Some(pl);
                }
            }
        } else if facts.squares_empty {
            return Action::EndPlayerAction;
        }

        // No usable plan: interrupted, pushed, or a prompt the activation did not predict.
        let m = match self.mover_of(g, f, &player_id) {
            Some(m) => m,
            None => return Action::EndPlayerAction,
        };
        let td = if m.home { &g.turn_data_home } else { &g.turn_data_away };
        let team_rr = td.rerolls > 0 && !td.reroll_used;
        let r = match reach(f, g, &player_id, team_rr, &mut self.sc) {
            Some(r) => r,
            None => return Action::EndPlayerAction,
        };
        let (w, dest) = best_move(f, &r, &m);
        let mut path = Vec::new();
        if let Some(i) = dest {
            if w > 0.0 {
                r.path_to(i, &mut self.sc.path);
                path = self.sc.path.clone();
            }
        }
        recycle(&mut self.sc, r);
        if path.first().map(|c| squares.contains(c)).unwrap_or(false) {
            // A re-plan that ends on the loose ball is a PICKUP, not a plain move: the value model
            // changes the moment the player has the ball, so the activation must be allowed to
            // continue rather than end (§20.1's exception).
            let ends_on_ball = path.last().map(|c| f.ball_loose && f.ball == Some(*c)).unwrap_or(false);
            self.plan = Some(Plan {
                player: player_id,
                kind: if ends_on_ball { PlanKind::Pickup } else { PlanKind::Move },
                path: Vec::new(),
                delivered: true,
                fired: false,
            });
            return Action::Move { path };
        }
        Action::EndPlayerAction
    }

    // ── ActivatePlayer: the joint (player, plan) choice ─────────────────────

    fn handle_activate(
        &mut self,
        g: &Game,
        f: &Features,
        eligible: Vec<(String, Vec<PlayerAction>)>,
    ) -> Action {
        // Two rules the random contract has always applied and the heuristic never inherited.
        // Both live in `ParityRunner`'s INIT_SELECTING arm, ahead of the branch that reaches the
        // agent at all, so the Java side obeys them no matter which agent is driving -- and Rust's
        // heuristic, which replaced the whole pick loop, obeyed neither.
        //
        // 1. `if (turn < 1) { EndTurn }`, BEFORE the turn-key update. A team whose turn counter is
        //    still 0 has not started a turn this half; a window that opens for it closes with zero
        //    movers and zero draws.
        // 2. A non-REGULAR mode -- a Blitz! or Quick Snap kickoff, a pass-block window -- allows
        //    exactly ONE activation and then ends.
        //
        // Missing them, a bb2016 Blitz! kickoff let the heuristic keep activating during a turn
        // Java had already ended, and the game ran off the rails immediately: seed 4 finished in
        // 5 ms with eight players still in the box, and the parity log diverged at its very first
        // recorded step.
        let turn_nr = if g.home_playing {
            g.turn_data_home.turn_nr
        } else {
            g.turn_data_away.turn_nr
        };
        if turn_nr < 1 {
            return Action::EndTurn;
        }
        // The kickoff-return window is ANSWERED, not played: letting the agent activate inside it
        // livelocks the driver (measured -- a bb2020 20-seed run never terminated). Java does
        // record one activation inside the window on some seeds, so this is not the whole truth
        // yet; see docs/PARITY_AMAZON_CAMPAIGN.md ITER19.
        // The window is PLAYED, not answered away: Java records one activation inside it (bb2020
        // seed 1 step 142, `Activate(Away6, MOVE)` in KICKOFF_RETURN mode). The phase-2 deselect
        // above plus its `just_deselected` latch is what ends it after that one pick.
        if std::env::var_os("FFB_KRLOOP").is_some()
            && g.turn_mode == ffb_model::enums::TurnMode::KickoffReturn
        {
            eprintln!(
                "KRACT used={} elig={} just_desel={} turn={}",
                self.used_this_turn.len(), eligible.len(), self.just_deselected, turn_nr
            );
        }
        // Java freezes the eligible list for the whole turn
        // (`eligibleThisTurn = computeEligiblePlayers(game)`) and then, at each activation, drops
        // the entries that turn stale (`filterStaleActions`). Both halves, or neither.
        //
        // A bare freeze was measured worse THREE times (ITER4, ITER5, ITER18) and the standing
        // note here concluded the engine's list must simply differ from the harness's. It does
        // not: `RandomAgent` snapshots the SAME engine list and is 100/100 against Java in all
        // three editions for both matchups. What those three attempts were missing is the second
        // half -- a frozen list still offering a Blitz the team has already spent makes the
        // POSITIONAL action pick read a longer list than Java's, which is worse than being live.
        //
        // Live-vs-frozen is measurable directly, and it is what the remaining reds are made of:
        // bb2025 seed 1 k=19, Java offers `Home6:Move` while Rust offers `home_06:Move|Foul`,
        // because an opponent went prone next to him AFTER the turn began (n=319 vs 318).
        // Java `ParityRunner`, HEURISTIC branch: it does NOT reuse `eligibleThisTurn`. It
        // recomputes, and says why -- "a turn-start snapshot cannot see [what the engine allows];
        // `computeEligiblePlayers` is a pure function of the current game state, so recomputing it
        // is exactly what the engine would report." The live list is therefore correct, which is
        // the answer to four separate attempts (ITER4, ITER5, ITER18, ITER22) at freezing it: the
        // frozen list measures worse because it IS worse, not because half of it was missing.
        //
        // What the live list still owes Java is the filter. `eligibleFor` runs
        // `filterStaleActions` over the recomputed entries before the agent scores them, and on a
        // live list that is not a no-op: it shrinks a PASS_BLOCK window to MOVE + the UseSkill
        // specials, and it applies the edition rules for Throw/Kick Team-Mate that the engine's
        // own list does not encode.
        self.refresh_turn(g);
        let eligible: Vec<(String, Vec<PlayerAction>)> = eligible
            .iter()
            .map(|(pid, acts)| (pid.clone(), crate::agent::filter_stale_actions(g, acts)))
            .filter(|(_, acts)| !acts.is_empty())
            .collect();
        if g.turn_mode != ffb_model::enums::TurnMode::Regular && !self.used_this_turn.is_empty() {
            self.just_deselected = true;
            self.probe_endturn(g, turn_nr, "window-closed");
            return Action::EndTurn;
        }
        if eligible.is_empty() {
            self.probe_endturn(g, turn_nr, "eligible-empty");
            return Action::EndTurn;
        }
        let any_unused = eligible.iter().any(|(pid, _)| !self.used_this_turn.contains(pid));
        // Every eligible player has already had its activation decided this turn. Re-offering them
        // is how the driver livelocks: an activation that ends without moving leaves the engine's
        // eligible list unchanged, so `used_this_turn` is the only thing that makes progress.
        // Java: `if (remaining.isEmpty() || justDeselected) { justDeselected = false;
        // usedThisTurn.clear(); EndTurn }` -- one exit, and it CLEARS rather than waiting for the
        // next turn key to do it.
        if !any_unused || self.just_deselected {
            let why = if self.just_deselected { "latch" } else { "all-used" };
            self.just_deselected = false;
            self.used_this_turn.clear();
            self.probe_endturn(g, turn_nr, why);
            return Action::EndTurn;
        }
        let home = g.home_playing;
        let side = if home { TeamSide::Home } else { TeamSide::Away };
        let td = if home { &g.turn_data_home } else { &g.turn_data_away };
        let team_rr = td.rerolls > 0 && !td.reroll_used;
        let bucket = self.bucket(f, g);
        let novelty = if self.temp_scale >= 0.1
            && self.seen_bucket.get(&bucket).copied().unwrap_or(0) == 0
        {
            0.08
        } else {
            0.0
        };

        // ---- tier 1: search-free proxy for every eligible player (§20.3) ----
        struct Cand1 {
            pid: String,
            live: Vec<PlayerAction>,
            m: Mover,
            w_player: f32,
            proxy: f32,
        }
        let mut c1: Vec<Cand1> = Vec::new();
        for (pid, actions) in &eligible {
            if self.used_this_turn.contains(pid) {
                continue;
            }
            let st = match g.field_model.player_state(pid) {
                Some(s) => s,
                None => continue,
            };
            // SKIP_INACTIVE (AGENT_CONTRACT.md §2.4). The engine's eligible list does not carry
            // the ACTIVE bit, and Java's `StepInitSelecting` guards its whole CLIENT_ACTING_PLAYER
            // branch on `playerState.isActive()` -- an activation for an inactive player is
            // silently IGNORED there, leaving the acting player null. Rust's engine has no such
            // guard and executes it, so the two only agree if the agent never asks. The random
            // agent already honours this (it skips the pick and burns the decisionRng call); the
            // heuristic replaced that pick loop wholesale and inherited nothing, so it activated
            // players Java would not move -- a just-unstunned player, or a team-mate thrown this
            // turn who lands STANDING but inactive.
            if !st.is_active() {
                continue;
            }
            let live: Vec<PlayerAction> =
                actions.iter().filter(|a| action_is_live(a, td, g.rules)).cloned().collect();
            if live.is_empty() {
                continue;
            }
            let m = match self.mover_of(g, f, pid) {
                Some(m) => m,
                None => continue,
            };
            let c = match g.field_model.player_coordinate(pid) {
                Some(c) => c,
                None => continue,
            };
            let proxy = proxy_value(f, g, pid, &m);
            let marked = f.tz[side_idx(home)][ixc(c)] > 0;
            let can_fetch = f.ball_loose
                && f.ball.map(|b| c.distance_in_steps(b) <= m.ma + 2).unwrap_or(false);
            let mut w_player = if m.is_carrier && marked {
                0.95
            } else if can_fetch {
                0.92
            } else if m.is_carrier {
                0.88
            } else if st.is_prone() && marked {
                0.70
            } else if proxy > 0.25 {
                0.45
            } else {
                0.30
            };
            if g.player(pid).map(has_negatrait).unwrap_or(false) {
                w_player *= 0.55;
            }
            // He was just thrown the ball. Running it on is the reason the throw was made.
            if self.awaiting_run.as_deref() == Some(pid.as_str()) {
                w_player = 1.0;
            }
            c1.push(Cand1 { pid: pid.clone(), live, m, w_player, proxy });
        }
        if c1.is_empty() {
            return Action::EndTurn;
        }
        // Deterministic order first, then rank by the proxy so ties never depend on hash order.
        c1.sort_by_key(|a| canon_key(g, &a.pid));
        let mut rank: Vec<usize> = (0..c1.len()).collect();
        rank.sort_by(|&a, &b| {
            (c1[b].w_player * c1[b].proxy.max(0.05))
                .partial_cmp(&(c1[a].w_player * c1[a].proxy.max(0.05)))
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(canon_key(g, &c1[a].pid).cmp(&canon_key(g, &c1[b].pid)))
        });
        let tier2: HashSet<usize> = rank.iter().take(TIER2).copied().collect();

        // ---- tier 2: one search per top player, shared by every plan (§20.2, §20.3) ----
        let mut cands: Vec<Candidate> = Vec::new();
        for (ci, c) in c1.iter().enumerate() {
            let r = if tier2.contains(&ci) {
                reach(f, g, &c.pid, team_rr, &mut self.sc)
            } else {
                None
            };
            self.build_plans(
                g,
                f,
                side,
                (&c.pid, &c.live, &c.m, c.w_player, c.proxy),
                r.as_ref(),
                novelty,
                team_rr,
                &mut cands,
            );
            if let Some(r) = r {
                recycle(&mut self.sc, r);
            }
        }

        if cands.is_empty() {
            return Action::EndTurn;
        }
        for c in &cands {
            self.buf.push_note(
                Action::ActivatePlayer {
                    player_id: c.player.clone(),
                    // Declare the MOVE-variant for a ball action, as the real Java client does:
                    // HAND_OVER_MOVE / PASS_MOVE give a movement phase before the give, which is
                    // what makes carrier-move + give + receiver-move possible in one turn. The
                    // parity agents keep declaring the immediate form, so their streams are
                    // untouched.
                    player_action: move_variant(c.pac),
                    block_defender_id: c.target.clone(),
                },
                c.weight,
                c.why,
                c.weight,
                c.note.clone(),
            );
        }
        // Ending the turn banks what the team has: zero gain, zero risk.
        self.buf.push(Action::EndTurn, 0.0, Rule::EndActivation, 0.0);

        // Agent diagnostics, all env-gated and off by default. Diffing the two agents' candidate
        // lists is the campaign's highest-yield tool, and the draw ACCOUNTING is what caught
        // ITER2: identical lists, identical weights, and a different pick, because one side had
        // silently spent two draws the other never spent.
        //
        //   FFB_CANDSUM=1  one line per activation: size, running draw total, per-declaration counts
        //   FFB_CAND=<k>   the k-th activation's full candidate list with raw float weights
        //   FFB_DRAWS=1    one line per prompt: its class and the running draw total
        {
            self.probe_act += 1;
            if std::env::var_os("FFB_CANDSUM").is_some() {
                let mut n = std::collections::BTreeMap::new();
                for c in cands.iter() {
                    *n.entry((c.player.clone(), format!("{:?}", c.pac))).or_insert(0u32) += 1;
                }
                let parts: Vec<String> =
                    n.iter().map(|((p, a), c)| format!("{p}/{a}:{c}")).collect();
                eprintln!(
                    "RSUM k={} n={} draws={} {}",
                    self.probe_act, cands.len(), self.probe_draws, parts.join(" ")
                );
                let elig: Vec<String> = eligible
                    .iter()
                    .map(|(pid, acts)| format!("{pid}:{}", acts.iter()
                        .map(|a| format!("{a:?}")).collect::<Vec<_>>().join("|")))
                    .collect();
                eprintln!(
                    "RELIG k={} turn={} blitz={} pass={} hand={} foul={} {}",
                    self.probe_act, turn_nr, td.blitz_used, td.pass_used, td.hand_over_used,
                    td.foul_used, elig.join(" ")
                );
            }
            if let Ok(want) = std::env::var("FFB_CAND") {
                if want.parse::<u32>().ok() == Some(self.probe_act) {
                    // Every occupancy source Features::build reads, with whether the id is
                    // actually on a roster. Java's `Features.snapshot` walks the TEAM ROSTERS and
                    // looks the coordinate up; Rust walks `field_model.player_coordinates`. An
                    // entry in that map for a player no longer on a roster is a phantom occupant
                    // that only Rust can see.
                    let mut ids: Vec<(&String, &FieldCoordinate)> =
                        g.field_model.player_coordinates.iter().collect();
                    ids.sort_by_key(|(id, _)| (*id).clone());
                    for (id, c) in ids {
                        if !on_pitch(c.x, c.y) { continue; }
                        let roster = g.team_home.has_player(id) || g.team_away.has_player(id);
                        eprintln!(
                            "RPOS k={} id={} at={},{} roster={} state={:?}",
                            self.probe_act, id, c.x, c.y, roster,
                            g.field_model.player_state(id).map(|s| s.base())
                        );
                    }
                    for (pid, acts) in eligible.iter() {
                        eprintln!(
                            "RELIG k={} pid={} at={:?} ma={:?} actions={:?}",  // (board dump below)
                            self.probe_act, pid,
                            g.field_model.player_coordinate(pid).map(|c| (c.x, c.y)),
                            g.player(pid).map(|p| p.movement),
                            acts
                        );
                        // Every adjacent OPPONENT with its coordinate and full state. The foul and
                        // block predicates are pure functions of exactly this, so when the two
                        // agents' action lists disagree while their flags and positions agree, the
                        // disagreement has to be visible here (`JNBR` is the Java mirror).
                        if let Some(c) = g.field_model.player_coordinate(pid) {
                            let opp = if g.home_playing { &g.team_away } else { &g.team_home };
                            for o in opp.players.iter() {
                                let Some(oc) = g.field_model.player_coordinate(&o.id) else { continue };
                                if !oc.is_adjacent(c) { continue; }
                                let os = g.field_model.player_state(&o.id);
                                eprintln!(
                                    "RNBR k={} pid={} opp={} nr={} at={},{} base={:?} active={:?} tz={:?}",
                                    self.probe_act, pid, o.id, o.nr, oc.x, oc.y,
                                    os.map(|s| s.base()), os.map(|s| s.is_active()),
                                    os.map(|s| s.has_tacklezones())
                                );
                            }
                        }
                    }
                    for (i, c) in cands.iter().enumerate() {
                        eprintln!(
                            "RCAND k={} i={} pid={} pac={:?} tgt={:?} dest={:?} w={:08x}",
                            self.probe_act, i, c.player, c.pac, c.target, c.dest,
                            c.weight.to_bits()
                        );
                    }
                }
            }
        }
        // Two-level draw. Group by DECLARATION — the (player, action) pair the engine actually
        // receives — and score each group by its best child, which keeps argmax identical to a flat
        // draw while stopping a branch with two thousand destinations from drowning one with nine.
        // build_plans emits a player's options one action at a time, so a declaration's candidates
        // are a CONTIGUOUS RUN. Detecting runs is linear; the obvious keyed lookup was O(groups) of
        // string comparison per candidate and cost 30 ms a game on its own.
        let mut groups = group_declarations(&cands);
        // EndTurn is its own group, and sits last in `buf`.
        let end_idx = self.buf.options.len() - 1;
        groups.push(vec![end_idx]);

        let gw: Vec<f32> = groups
            .iter()
            .map(|g| {
                g.iter()
                    .map(|&j| self.buf.options[j].weight)
                    .fold(f32::MIN, f32::max)
            })
            .collect();
        let (gi, gp) = self.softmax_pick(&gw, 0.18);
        let cw: Vec<f32> = groups[gi].iter().map(|&j| self.buf.options[j].weight).collect();
        let (ci, _) = self.softmax_pick(&cw, 0.10);
        let i = groups[gi][ci];

        if self.dump_enabled {
            // Joint probability, so the panel shows what the sampler really did.
            let mut ps = vec![0.0f32; self.buf.options.len()];
            for (g, idxs) in groups.iter().enumerate() {
                let w: Vec<f32> = idxs.iter().map(|&j| self.buf.options[j].weight).collect();
                let inner = if w.len() == 1 || self.temp_scale <= 0.0 {
                    let mut v = vec![0.0f32; w.len()];
                    let mut bi = 0;
                    for k in 1..w.len() {
                        if w[k] > w[bi] {
                            bi = k;
                        }
                    }
                    if !v.is_empty() {
                        v[bi] = 1.0;
                    }
                    v
                } else {
                    let t = (0.10 * self.temp_scale).max(1e-6);
                    let mx = w.iter().copied().fold(f32::MIN, f32::max);
                    let mut v: Vec<f32> = w.iter().map(|x| exp_f32((x - mx) / t)).collect();
                    let acc: f32 = v.iter().sum();
                    if acc > 0.0 {
                        for x in v.iter_mut() {
                            *x /= acc;
                        }
                    }
                    v
                };
                for (k, &j) in idxs.iter().enumerate() {
                    ps[j] = gp.get(g).copied().unwrap_or(0.0) * inner[k];
                }
            }
            self.record_probs(&ps, i);
        }
        if i < cands.len() {
            let c = cands.swap_remove(i);
            // The path is walked back HERE, once, for the option that won — see `Candidate::dest`.
            let path = if !c.path.is_empty() {
                c.path
            } else if let Some(d) = c.dest {
                match reach(f, g, &c.player, team_rr, &mut self.sc) {
                    Some(r) => {
                        let mut p = Vec::new();
                        r.path_to(d, &mut p);
                        recycle(&mut self.sc, r);
                        p
                    }
                    None => Vec::new(),
                }
            } else {
                Vec::new()
            };
            self.plan = Some(Plan {
                player: c.player.clone(),
                kind: c.kind,
                path,
                delivered: false,
                fired: false,
            });
            if std::env::var_os("FFB_BALLMOVE").is_some()
                && matches!(
                    c.pac,
                    PlayerActionChoice::Pass | PlayerActionChoice::HandOff
                )
            {
                // Accumulates across seeds, unlike FFB_HEUR_DUMP which overwrites per game.
                eprintln!("BM {:?} {}", c.pac, c.note);
            }
            if matches!(c.pac, PlayerActionChoice::Pass | PlayerActionChoice::HandOff) {
                self.awaiting_run = c.target.clone();
            }
            self.used_this_turn.insert(c.player);
            *self.seen_action.entry(format!("{:?}", c.pac)).or_insert(0) += 1;
            *self.seen_bucket.entry(bucket).or_insert(0) += 1;
        }
        // Java's heuristic driver declares a TTM/KTM and then, at phase 2, picks the thrown
        // player with the harness's random rule (sendThrowTeamMateAction: coord-sorted adjacent
        // throwable teammates, ONE actionRng draw). The candidate here — built by the same
        // default arm as Java's chooser — carries NO target, and a targetless declaration
        // deselects instantly, so no TTM ever resolved under the heuristic (chaos_pact bb2020
        // seeds 6/7/10/19, bb2016 seed 4). Fold the identical pick, from the identical
        // action_rng stream (the embedded parity agent's), into the taken answer.
        let mut taken = self.take(i);
        if let Action::ActivatePlayer {
            ref player_id,
            player_action: PlayerActionChoice::ThrowTeamMate | PlayerActionChoice::KickTeamMate,
            ref mut block_defender_id,
        } = taken
        {
            if block_defender_id.is_none() {
                let pid = player_id.clone();
                let t = self.parity.fold_ttm_target(g, &pid);
                if std::env::var_os("FFB_TRACE").is_some() {
                    eprintln!("RTTMFOLD pid={pid} target={t:?}");
                }
                *block_defender_id = t;
            }
        }
        taken
    }

    // ── DEEP mode ───────────────────────────────────────────────────────────

    /// Stage 1 picks the player, stage 2 picks that player's action and target.
    ///
    /// No pathfinding runs here at all: the player is scored by the search-free proxy and the
    /// action set is built with `r = None`, the same tier-1 path §20.3 already had. The destination
    /// is decided later, by `handle_move_deep`, from the single search deep mode pays for — which is
    /// the whole economy of the mode: one Dijkstra per activation instead of TIER2 per prompt.
    fn handle_activate_deep(
        &mut self,
        g: &Game,
        f: &Features,
        eligible: Vec<(String, Vec<PlayerAction>)>,
    ) -> Action {
        // Two rules the random contract has always applied and the heuristic never inherited.
        // Both live in `ParityRunner`'s INIT_SELECTING arm, ahead of the branch that reaches the
        // agent at all, so the Java side obeys them no matter which agent is driving -- and Rust's
        // heuristic, which replaced the whole pick loop, obeyed neither.
        //
        // 1. `if (turn < 1) { EndTurn }`, BEFORE the turn-key update. A team whose turn counter is
        //    still 0 has not started a turn this half; a window that opens for it closes with zero
        //    movers and zero draws.
        // 2. A non-REGULAR mode -- a Blitz! or Quick Snap kickoff, a pass-block window -- allows
        //    exactly ONE activation and then ends.
        //
        // Missing them, a bb2016 Blitz! kickoff let the heuristic keep activating during a turn
        // Java had already ended, and the game ran off the rails immediately: seed 4 finished in
        // 5 ms with eight players still in the box, and the parity log diverged at its very first
        // recorded step.
        let turn_nr = if g.home_playing {
            g.turn_data_home.turn_nr
        } else {
            g.turn_data_away.turn_nr
        };
        if turn_nr < 1 {
            return Action::EndTurn;
        }
        // The kickoff-return window is ANSWERED, not played: letting the agent activate inside it
        // livelocks the driver (measured -- a bb2020 20-seed run never terminated). Java does
        // record one activation inside the window on some seeds, so this is not the whole truth
        // yet; see docs/PARITY_AMAZON_CAMPAIGN.md ITER19.
        // The window is PLAYED, not answered away: Java records one activation inside it (bb2020
        // seed 1 step 142, `Activate(Away6, MOVE)` in KICKOFF_RETURN mode). The phase-2 deselect
        // above plus its `just_deselected` latch is what ends it after that one pick.
        if std::env::var_os("FFB_KRLOOP").is_some()
            && g.turn_mode == ffb_model::enums::TurnMode::KickoffReturn
        {
            eprintln!(
                "KRACT used={} elig={} just_desel={} turn={}",
                self.used_this_turn.len(), eligible.len(), self.just_deselected, turn_nr
            );
        }
        // Same live list + stale filter as `handle_activate`; kept in step with it so the two
        // activation paths cannot drift. (Mode::Deep is not covered by the parity gate.)
        self.refresh_turn(g);
        let eligible: Vec<(String, Vec<PlayerAction>)> = eligible
            .iter()
            .map(|(pid, acts)| (pid.clone(), crate::agent::filter_stale_actions(g, acts)))
            .filter(|(_, acts)| !acts.is_empty())
            .collect();
        if g.turn_mode != ffb_model::enums::TurnMode::Regular && !self.used_this_turn.is_empty() {
            self.just_deselected = true;
            return Action::EndTurn;
        }
        if eligible.is_empty() {
            return Action::EndTurn;
        }
        if !eligible.iter().any(|(pid, _)| !self.used_this_turn.contains(pid))
            || self.just_deselected
        {
            self.just_deselected = false;
            self.used_this_turn.clear();
            return Action::EndTurn;
        }
        let home = g.home_playing;
        let side = if home { TeamSide::Home } else { TeamSide::Away };
        let td = if home { &g.turn_data_home } else { &g.turn_data_away };
        let team_rr = td.rerolls > 0 && !td.reroll_used;

        // ---- stage 1: which player ----
        struct P {
            pid: String,
            live: Vec<PlayerAction>,
            m: Mover,
            w: f32,
            proxy: f32,
        }
        let mut ps: Vec<P> = Vec::new();
        for (pid, actions) in &eligible {
            if self.used_this_turn.contains(pid) {
                continue;
            }
            let st = match g.field_model.player_state(pid) {
                Some(s) => s,
                None => continue,
            };
            // SKIP_INACTIVE (AGENT_CONTRACT.md §2.4). The engine's eligible list does not carry
            // the ACTIVE bit, and Java's `StepInitSelecting` guards its whole CLIENT_ACTING_PLAYER
            // branch on `playerState.isActive()` -- an activation for an inactive player is
            // silently IGNORED there, leaving the acting player null. Rust's engine has no such
            // guard and executes it, so the two only agree if the agent never asks. The random
            // agent already honours this (it skips the pick and burns the decisionRng call); the
            // heuristic replaced that pick loop wholesale and inherited nothing, so it activated
            // players Java would not move -- a just-unstunned player, or a team-mate thrown this
            // turn who lands STANDING but inactive.
            if !st.is_active() {
                continue;
            }
            let live: Vec<PlayerAction> =
                actions.iter().filter(|a| action_is_live(a, td, g.rules)).cloned().collect();
            if live.is_empty() {
                continue;
            }
            let m = match self.mover_of(g, f, pid) {
                Some(m) => m,
                None => continue,
            };
            let c = match g.field_model.player_coordinate(pid) {
                Some(c) => c,
                None => continue,
            };
            let proxy = proxy_value(f, g, pid, &m);
            let marked = f.tz[side_idx(home)][ixc(c)] > 0;
            let can_fetch = f.ball_loose
                && f.ball.map(|b| c.distance_in_steps(b) <= m.ma + 2).unwrap_or(false);
            let mut w = if m.is_carrier && marked {
                0.95
            } else if can_fetch {
                0.92
            } else if m.is_carrier {
                0.88
            } else if st.is_prone() && marked {
                0.70
            } else if proxy > 0.25 {
                0.45
            } else {
                0.30
            };
            if g.player(pid).map(has_negatrait).unwrap_or(false) {
                w *= 0.55;
            }
            ps.push(P { pid: pid.clone(), live, m, w: w * proxy.max(0.05), proxy });
        }
        if ps.is_empty() {
            return Action::EndTurn;
        }
        ps.sort_by_key(|a| canon_key(g, &a.pid));

        self.buf.clear();
        for q in &ps {
            self.buf.push_note(
                Action::ActivatePlayer {
                    player_id: q.pid.clone(),
                    player_action: PlayerActionChoice::Move,
                    block_defender_id: None,
                },
                q.w,
                Rule::ScoreAdvance,
                q.w,
                format!("stage 1 · activate {} · proxy {:.2}", q.pid, q.proxy),
            );
        }
        // Banking the turn competes with the players, exactly as in wide mode.
        self.buf.push(Action::EndTurn, 0.0, Rule::EndActivation, 0.0);
        let pick = self.pick(0.18);
        // Stage 1 has to be stashed: stage 2 records over it, and reporting only the second stage
        // would understate deep mode's branching factor by hiding the player draw entirely.
        let stage1: Vec<ScoredOption> = if self.dump_enabled {
            self.record_distribution(0.18, pick);
            self.last_options.clone()
        } else {
            Vec::new()
        };
        if pick >= ps.len() {
            return Action::EndTurn;
        }
        let chosen = &ps[pick];

        // ---- stage 2: which action, and against whom ----
        let mut cands: Vec<Candidate> = Vec::new();
        self.build_plans(
            g,
            f,
            side,
            (&chosen.pid, &chosen.live, &chosen.m, 1.0, chosen.proxy),
            None,
            0.0,
            team_rr,
            &mut cands,
        );
        if cands.is_empty() {
            self.used_this_turn.insert(chosen.pid.clone());
            return Action::EndTurn;
        }
        self.buf.clear();
        for c in &cands {
            self.buf.push_note(
                Action::ActivatePlayer {
                    player_id: c.player.clone(),
                    player_action: move_variant(c.pac),
                    block_defender_id: c.target.clone(),
                },
                c.weight,
                c.why,
                c.weight,
                format!("stage 2 · {}", if c.note.is_empty() { "declare" } else { &c.note }),
            );
        }
        let a2 = self.pick(0.14);
        if self.dump_enabled {
            self.record_distribution(0.14, a2);
            let n1 = stage1.len();
            self.last_options.splice(0..0, stage1);
            self.last_chosen += n1;
        }
        let c = cands.swap_remove(a2.min(cands.len() - 1));
        self.used_this_turn.insert(c.player.clone());
        *self.seen_action.entry(format!("{:?}", c.pac)).or_insert(0) += 1;
        self.plan = Some(Plan {
            player: c.player.clone(),
            kind: c.kind,
            // Deep mode leaves the route to stage 3, once the player is settled.
            path: Vec::new(),
            delivered: false,
            fired: false,
        });
        // Java's heuristic driver declares a TTM/KTM and then, at phase 2, picks the thrown
        // player with the harness's random rule (sendThrowTeamMateAction: coord-sorted adjacent
        // throwable teammates, ONE actionRng). The Rust candidate (built by the same default arm
        // as Java's chooser) carries NO target, and a targetless declaration deselects instantly
        // — so fold the identical pick, from the identical action_rng stream, into the answer.
        let target = if matches!(c.pac, PlayerActionChoice::ThrowTeamMate | PlayerActionChoice::KickTeamMate)
            && c.target.is_none()
        {
            let t = self.parity.fold_ttm_target(g, &c.player);
            if std::env::var_os("FFB_TRACE").is_some() {
                eprintln!("RTTMFOLD pid={} pac={:?} target={:?}", c.player, c.pac, t);
            }
            t
        } else {
            if std::env::var_os("FFB_TRACE").is_some()
                && matches!(c.pac, PlayerActionChoice::ThrowTeamMate | PlayerActionChoice::KickTeamMate)
            {
                eprintln!("RTTMFOLD-SKIP pid={} target_was={:?}", c.player, c.target);
            }
            c.target
        };
        Action::ActivatePlayer {
            player_id: c.player,
            player_action: move_variant(c.pac),
            block_defender_id: target,
        }
    }

    /// Stage 3: the destination, as a full path, from the ONE search deep mode pays for.
    ///
    /// Identical scoring to wide mode's — every reachable square, weight-ordered — over a single
    /// Dijkstra for the player already chosen, rather than one per candidate player. The path is
    /// built only for the square that wins.
    fn handle_move_deep(
        &mut self,
        g: &Game,
        f: &Features,
        player_id: String,
        squares: Vec<FieldCoordinate>,
    ) -> Action {
        if squares.is_empty() {
            self.plan = None;
            return Action::EndPlayerAction;
        }

        // The plan's terminal action first, on exactly the engine's own conditions.
        if let Some(mut pl) = self.plan.take() {
            if pl.player == player_id && !pl.fired {
                let here = g.field_model.player_coordinate(&player_id);
                let pa_now = g.acting_player.player_action;
                let adj = |v: &str| {
                    here.zip(g.field_model.player_coordinate(v))
                        .map(|(a, b)| a.distance_in_steps(b) == 1)
                        .unwrap_or(false)
                };
                match &pl.kind {
                    PlanKind::Blitz { victim } => {
                        let ok = matches!(
                            pa_now,
                            Some(PlayerAction::BlitzMove) | Some(PlayerAction::KickEmBlitz)
                        ) && !g.acting_player.has_blocked;
                        if ok && adj(victim) {
                            let defender_id = victim.clone();
                            pl.fired = true;
                            self.plan = Some(pl);
                            return Action::Block { defender_id };
                        }
                    }
                    PlanKind::Foul { victim } => {
                        if pa_now == Some(PlayerAction::FoulMove)
                            && !g.acting_player.has_fouled
                            && adj(victim)
                        {
                            let target_id = victim.clone();
                            pl.fired = true;
                            self.plan = Some(pl);
                            return Action::Foul { target_id };
                        }
                    }
                    PlanKind::Pass { receiver } => {
                        let ok = matches!(
                            pa_now,
                            Some(PlayerAction::PassMove)
                                | Some(PlayerAction::Pass)
                                | Some(PlayerAction::HailMaryPass)
                        );
                        if let (true, Some(coord)) = (ok, g.field_model.player_coordinate(receiver))
                        {
                            pl.fired = true;
                            self.plan = Some(pl);
                            return Action::Pass { coord };
                        }
                    }
                    PlanKind::HandOff { receiver } => {
                        let ok = matches!(
                            pa_now,
                            Some(PlayerAction::HandOverMove) | Some(PlayerAction::HandOver)
                        );
                        if ok && g.field_model.player_coordinate(receiver).is_some() {
                            let receiver_id = receiver.clone();
                            pl.fired = true;
                            self.plan = Some(pl);
                            return Action::HandOff { receiver_id };
                        }
                    }
                    _ => {}
                }
            }
            // §20.1 — a delivered plain move has no options it did not already have. A pickup
            // does: the value model changed the moment the player got the ball.
            if pl.player == player_id
                && pl.delivered
                && matches!(pl.kind, PlanKind::Move | PlanKind::Immediate)
            {
                return Action::EndPlayerAction;
            }
        }

        let m = match self.mover_of(g, f, &player_id) {
            Some(m) => m,
            None => return Action::EndPlayerAction,
        };
        let td = if m.home { &g.turn_data_home } else { &g.turn_data_away };
        let team_rr = td.rerolls > 0 && !td.reroll_used;
        let r = match reach(f, g, &player_id, team_rr, &mut self.sc) {
            Some(r) => r,
            None => return Action::EndPlayerAction,
        };

        let tops = top_moves(f, &r, &m, usize::MAX);
        let reachable = tops.len();
        self.buf.clear();
        let mut dests: Vec<usize> = Vec::with_capacity(reachable);
        for (rank, (w, i)) in tops.iter().enumerate() {
            let c = coord_of(*i);
            // NOTE: `squares` is the set of legal NEXT squares - adjacent ones - not destinations.
            // Filtering destinations against it left only one-square moves, which is why an earlier
            // cut of this scored zero touchdowns. The path's FIRST square is what has to be legal,
            // and that is checked once the winner is known.
            let onto_ball = f.ball_loose && f.ball == Some(c);
            let note = if rank < DEST_NOTES || onto_ball {
                let ar = arrival_parts(f, &r, *i, &m);
                format!(
                    "{}{},{} · {} sq · arrive {:.0}% · V {:.2}{} · w {:+.3} · #{} of {}",
                    if onto_ball { "PICK UP at " } else { "to " },
                    c.x,
                    c.y,
                    r.cell[*i].cost,
                    100.0 * ar.p_arrive,
                    ar.v,
                    if ar.gfi > 0 { format!(" · {} rush", ar.gfi) } else { String::new() },
                    ar.w,
                    rank + 1,
                    reachable
                )
            } else {
                format!("to {},{} · #{} of {}", c.x, c.y, rank + 1, reachable)
            };
            // A placeholder action: the real path is built for the winner only, below.
            self.buf.push_note(Action::Move { path: vec![c] }, *w, Rule::ScoreAdvance, *w, note);
            dests.push(*i);
        }
        // Stopping here is always available and costs nothing.
        self.buf.push_note(
            Action::EndPlayerAction,
            0.0,
            Rule::EndActivation,
            0.0,
            "stop".to_string(),
        );
        let pick = self.pick(0.12);
        if self.dump_enabled {
            self.record_distribution(0.12, pick);
        }
        let act = if pick < dests.len() {
            let mut path = Vec::new();
            r.path_to(dests[pick], &mut path);
            if !path.first().map(|c| squares.contains(c)).unwrap_or(false) {
                recycle(&mut self.sc, r);
                self.plan = None;
                return Action::EndPlayerAction;
            }
            let onto_ball = path
                .last()
                .map(|c| f.ball_loose && f.ball == Some(*c))
                .unwrap_or(false);
            self.plan = Some(Plan {
                player: player_id,
                kind: if onto_ball { PlanKind::Pickup } else { PlanKind::Move },
                path: Vec::new(),
                delivered: true,
                fired: false,
            });
            Action::Move { path }
        } else {
            self.plan = None;
            Action::EndPlayerAction
        };
        recycle(&mut self.sc, r);
        act
    }

    #[allow(clippy::too_many_arguments)]
    fn build_plans(
        &self,
        g: &Game,
        f: &Features,
        side: TeamSide,
        parts: (&str, &[PlayerAction], &Mover, f32, f32),
        r: Option<&Reach>,
        novelty: f32,
        team_rr: bool,
        out: &mut Vec<Candidate>,
    ) {
        let (pid, live, m, w_player, proxy) = parts;
        for pa in live {
            let pac = player_action_to_pac(pa);
            let floor = self.coverage_floor(&format!("{:?}", pac));
            let mut push = |w: f32,
                            target: Option<String>,
                            kind: PlanKind,
                            path: Vec<FieldCoordinate>,
                            dest: Option<usize>,
                            why,
                            note: String| {
                out.push(Candidate {
                    weight: w_player * w.max(floor) + novelty,
                    player: pid.to_string(),
                    pac,
                    target,
                    kind,
                    path,
                    dest,
                    why,
                    note,
                });
            };

            match pac {
                PlayerActionChoice::Move | PlayerActionChoice::StandUp => {
                    match r {
                        Some(r) => {
                            // EVERY reachable square, weight-ordered. They are already scored —
                            // `top_moves` computes an arrival weight for all of them — so offering
                            // them costs a Vec entry each, not a search.
                            let tops = top_moves(f, r, m, usize::MAX);
                            let reachable = tops.len();
                            if tops.is_empty() {
                                push(
                                    0.02,
                                    None,
                                    PlanKind::Move,
                                    Vec::new(),
                                    None,
                                    Rule::Support,
                                    String::new(),
                                );
                            }
                            for (rank, (w, i)) in tops.into_iter().enumerate() {
                                let c = coord_of(i);
                                let onto_ball = f.ball_loose && f.ball == Some(c);
                                let kind =
                                    if onto_ball { PlanKind::Pickup } else { PlanKind::Move };
                                let note = if rank < DEST_NOTES || onto_ball {
                                    let ar = arrival_parts(f, r, i, m);
                                    format!(
                                        "{}{},{} · {} sq · arrive {:.0}% · V {:.2}{} · w {:+.3} · #{} of {} reachable",
                                        if onto_ball { "PICK UP at " } else { "to " },
                                        c.x,
                                        c.y,
                                        r.cell[i].cost,
                                        100.0 * ar.p_arrive,
                                        ar.v,
                                        if ar.gfi > 0 {
                                            format!(" · {} rush", ar.gfi)
                                        } else {
                                            String::new()
                                        },
                                        ar.w,
                                        rank + 1,
                                        reachable
                                    )
                                } else {
                                    format!("to {},{} · #{} of {}", c.x, c.y, rank + 1, reachable)
                                };
                                push(
                                    w,
                                    None,
                                    kind,
                                    Vec::new(),
                                    Some(i),
                                    Rule::ScoreAdvance,
                                    note,
                                );
                            }
                        }
                        // tier 1: the proxy stands in, discounted for being optimistic
                        None => push(
                            proxy * 0.8,
                            None,
                            PlanKind::Move,
                            Vec::new(),
                            None,
                            Rule::ScoreAdvance,
                            "no search ran for this player (§20.3 tier 1)".to_string(),
                        ),
                    }
                }
                PlayerActionChoice::Block => {
                    // Java's board for the heuristic is `ActivationDriver.foes(..., adjacentOnly)`,
                    // which keeps an opponent only while `os.hasTacklezones()` --
                    // `(STANDING || MOVING || BLOCKED) && !confused && !hypnotized`.
                    // `legal_block_targets` answers the RANDOM contract's question instead
                    // (`can_be_blocked`, i.e. `STANDING || MOVING`) and must stay that way: making
                    // it Java-faithful took `--agent random` bb2025 amazon from 100 to 93/100.
                    //
                    // The gap that matters is CONFUSION. A hexed player is STANDING and confused,
                    // so Java drops him as a target and Rust kept him -- amazon bb2025 seed 9,
                    // where both engines hex `home_01` and Rust then offers blocks on `home_01`
                    // and `home_03` against Java's `home_03` alone.
                    for t in legal_block_targets(g, pid, side).into_iter().filter(|t| {
                        g.field_model.player_state(t).map(|s| s.has_tacklezones()).unwrap_or(false)
                    }) {
                        let w = block_weight(f, g, pid, &t, m.str_);
                        push(
                            w,
                            Some(t.clone()),
                            PlanKind::Immediate,
                            Vec::new(),
                            None,
                            Rule::DiceCount,
                            format!("block {}", t),
                        );
                    }
                }
                PlayerActionChoice::Blitz | PlayerActionChoice::StandUpBlitz => {
                    // A blitz can only be declared against a victim ALREADY adjacent: see below
                    // and docs §24. Movement after the block still works, so an adjacent blitz is
                    // a block plus a free reposition.
                    let mut foes: Vec<String> = g
                        .field_model
                        .player_coordinates
                        .iter()
                        .filter(|(oid, &oc)| {
                            on_pitch(oc.x, oc.y)
                                && g.team_home.has_player(oid) != m.home
                                && f.occ[ixc(oc)] & OCC_TZ != 0
                        })
                        .map(|(oid, _)| oid.clone())
                        .collect();
                    foes.sort_by_key(|id| canon_key(g, id));
                    let here = g.field_model.player_coordinate(pid);
                    for oid in foes {
                        let oc = match g.field_model.player_coordinate(&oid) {
                            Some(c) => c,
                            None => continue,
                        };
                        let dice = block_weight(f, g, pid, &oid, m.str_);
                        if here.map(|h| h.distance_in_steps(oc) == 1).unwrap_or(false) {
                            // Discounted against a plain Block because it spends the team's
                            // once-per-turn blitz, which another player might use better.
                            push(
                                dice * 0.85,
                                Some(oid.clone()),
                                PlanKind::Blitz { victim: oid.clone() },
                                Vec::new(),
                                None,
                                Rule::DiceCount,
                                format!("blitz {} (already adjacent)", oid),
                            );
                            continue;
                        }
                        // Everything past this point required MOVEMENT to reach the victim,
                        // and a move-then-blitz does not dispatch in this engine build (0.2% of
                        // 505 measured attempts). Offering it wastes the once-per-turn blitz, so
                        // the branch stops at adjacency. See docs §24.
                    }
                }
                PlayerActionChoice::Foul => {
                    if std::env::var_os("FFB_TRACE").is_some() {
                        eprintln!("RFOULCAND pid={pid} targets={:?}", legal_foul_targets(g, pid, side));
                    }
                    for t in legal_foul_targets(g, pid, side) {
                        let w = foul_weight(f, g, pid, &t, m);
                        push(
                            w,
                            Some(t.clone()),
                            PlanKind::Foul { victim: t.clone() },
                            Vec::new(),
                            None,
                            Rule::Flat,
                            format!("foul {}", t),
                        );
                    }
                }
                // HandOverMove: the carrier moves FIRST and gives the ball at the end of it, so
                // every team-mate he can get NEXT TO is a candidate — not just the ones he is
                // already touching, which is all `legal_handoff_receivers` reports.
                PlayerActionChoice::HandOff
                    if !matches!(self.mode, Mode::WideNoBall | Mode::WideNoHandOff) =>
                {
                    let r = match r {
                        Some(r) => r,
                        None => continue,
                    };
                    let here = m_coord(g, pid);
                    let mut mates: Vec<String> = g
                        .field_model
                        .player_coordinates
                        .iter()
                        .filter(|(oid, &oc)| {
                            oid.as_str() != pid
                                && on_pitch(oc.x, oc.y)
                                && g.team_home.has_player(oid) == m.home
                        })
                        .map(|(oid, _)| oid.clone())
                        .collect();
                    mates.sort_by_key(|id| canon_key(g, id));
                    for rcv in mates {
                        let rc = match g.field_model.player_coordinate(&rcv) {
                            Some(c) => c,
                            None => continue,
                        };
                        let mut spots: Vec<(f32, usize)> = Vec::new();
                        for n in rc.neighbours() {
                            if !on_pitch(n.x, n.y) {
                                continue;
                            }
                            let j = ixc(n);
                            if !r.reached(j) || (f.occupied(j) && n != here) {
                                continue;
                            }
                            if let Some(w) = handoff_weight(f, g, coord_of(j), &rcv, m) {
                                spots.push((risked(w, r.p_arrive(j), m), j));
                            }
                        }
                        spots.sort_by(|a, b| {
                            b.0.partial_cmp(&a.0)
                                .unwrap_or(std::cmp::Ordering::Equal)
                                .then(a.1.cmp(&b.1))
                        });
                        for (w, j) in spots.into_iter().take(GIVE_SPOTS) {
                            let from = coord_of(j);
                            let mut path = Vec::new();
                            r.path_to(j, &mut path);
                            push(
                                w,
                                Some(rcv.clone()),
                                PlanKind::HandOff { receiver: rcv.clone() },
                                path,
                                None,
                                Rule::Flat,
                                format!(
                                    "{} hand off to {}{}",
                                    if from == here {
                                        "stand and".to_string()
                                    } else {
                                        format!("run to {},{} then", from.x, from.y)
                                    },
                                    rcv,
                                    pass_note(g, from, &rcv, m)
                                ),
                            );
                        }
                    }
                }
                // PassMove: same shape, with the throw at the end of the run-up.
                PlayerActionChoice::Pass
                | PlayerActionChoice::ThrowBomb
                | PlayerActionChoice::HailMaryPass
                | PlayerActionChoice::AllYouCanEat
                    if !matches!(self.mode, Mode::WideNoBall | Mode::WideNoPass) =>
                {
                    let here = m_coord(g, pid);
                    let spots = run_up_squares(r, m, here);
                    for rcv in legal_pass_receivers(g, pid, side) {
                        for &j in &spots {
                            let from = coord_of(j);
                            let w = match pass_weight(f, g, pid, from, &rcv, m) {
                                Some(w) => w,
                                None => continue,
                            };
                            let (w, path) = match r {
                                Some(r) if from != here => {
                                    let mut path = Vec::new();
                                    r.path_to(j, &mut path);
                                    (risked(w, r.p_arrive(j), m), path)
                                }
                                _ => (w, Vec::new()),
                            };
                            push(
                                w,
                                Some(rcv.clone()),
                                PlanKind::Pass { receiver: rcv.clone() },
                                path,
                                None,
                                Rule::Flat,
                                format!(
                                    "{} pass to {}{}",
                                    if from == here {
                                        "stand and".to_string()
                                    } else {
                                        format!("run to {},{} then", from.x, from.y)
                                    },
                                    rcv,
                                    pass_note(g, from, &rcv, m)
                                ),
                            );
                        }
                    }
                }
                // ThrowTeamMate / KickTeamMate: Java's ActivationChoice has NO arm for these —
                // they land in its `default:` (ONE immediate candidate, weight wPlayer*max(0.40,
                // floor)+novelty, no target). A bespoke Rust arm built one candidate PER TARGET at
                // 0.35, so a chaos_pact big guy with N throwable goblins put N-1 extra candidates
                // into the lottery and every draw after the first activation read a different
                // stream (chaos_pact baseline 0/100 in ALL THREE editions; bb2025 seed 2 k=1:
                // Java n=2000, Rust n=1998 — exactly the two big guys' phantom arms).
                _ => push(
                    0.40,
                    None,
                    PlanKind::Immediate,
                    Vec::new(),
                    None,
                    Rule::Flat,
                    String::new(),
                ),
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────── act()

impl Agent for HeuristicAgent {
    fn act(&mut self, gs: &GameState) -> Action {
        let g = &gs.game;
        self.buf.clear();
        // A prompt answered from the activation plan (20.1/20.2) or by a fixed rule makes no
        // decision at all, so it must not appear to have made the previous one. Clearing here means
        // an empty option set in the dump reads as 'replayed, nothing scored' - which is true, and
        // is most Move prompts.
        if self.dump_enabled {
            self.last_options.clear();
            self.last_chosen = 0;
        }

        let prompt = match gs.current_prompt() {
            Some(p) => p.clone(),
            None => return Action::Acknowledge,
        };

        // The Java-port ladder (docs/PARITY_HEURISTIC_CAMPAIGN.md). Any class not switched on is
        // answered by the parity RandomAgent contract instead, so a partially-ported Java side can
        // still be gated at 100/100: rung 0 delegates everything and must reproduce the
        // random-agent gate exactly. `ClassMask::ALL` (the default) never takes this branch.
        if !self.classes.has(prompt_class_of(&prompt)) {
            return self.parity.act(gs);
        }
        // Per-prompt draw accounting (`FFB_DRAWS`). Printing the RUNNING TOTAL on entry
        // gives the previous prompt's cost by differencing, which works despite act()'s many
        // early returns.
        if std::env::var_os("FFB_DRAWS").is_some() {
            let what = match &prompt {
                AgentPrompt::SkillUse { skill_name, player_id, .. } => {
                    format!(" skill={skill_name} pid={player_id}")
                }
                AgentPrompt::ReRollOffer { source, action, .. } => {
                    format!(" src={source:?} action={action}")
                }
                _ => String::new(),
            };
            eprintln!(
                "RDRAW cls={} total={}{}",
                prompt_class_of(&prompt).name(),
                self.probe_draws,
                what
            );
        }

        // §20.7 — build the feature block at most once per board position, then take it out of
        // `self` for the duration of the decision so the borrow checker stays satisfied without
        // any unsafe aliasing.
        // Three tiers, cheapest first: prompts that read nothing about the board, prompts that
        // read only the cheap core (tackle zones, occupancy, who has the ball), and the two that
        // read the value rasters. Measurement is what forced the middle tier — building the rasters
        // for a `BlockChoice` took it from 11 µs to 86 µs, and it does not read one of them.
        let needs_features = matches!(
            prompt,
            AgentPrompt::Move { .. }
                | AgentPrompt::ActivatePlayer { .. }
                | AgentPrompt::BlitzTarget { .. }
                | AgentPrompt::BlockChoice { .. }
                | AgentPrompt::Pushback { .. }
                | AgentPrompt::FollowUp { .. }
                | AgentPrompt::ReRollOffer { .. }
        );
        if !needs_features {
            return self.act_boardless(gs, g, prompt);
        }
        let needs_heavy =
            matches!(prompt, AgentPrompt::Move { .. } | AgentPrompt::ActivatePlayer { .. });
        let stamp = positions_stamp(g);
        let usable = self
            .feat
            .as_ref()
            .map(|f| f.stamp == stamp && (f.heavy || !needs_heavy))
            .unwrap_or(false);
        if !usable {
            self.feat = Some(Features::build(g, stamp, needs_heavy));
        }
        let f = self.feat.take().expect("features just built");
        let action = self.act_with_features(gs, g, &f, prompt);
        self.feat = Some(f);
        action
    }
}

impl HeuristicAgent {
    fn act_with_features(
        &mut self,
        gs: &GameState,
        g: &Game,
        f: &Features,
        prompt: AgentPrompt,
    ) -> Action {
        match prompt {
            AgentPrompt::Move { ref player_id, ref squares } if std::env::var_os("FFB_MOVEP").is_some() => {
                // `FFB_MOVEP`: what the agent is OFFERED and what it answers at every move prompt.
                // The amazon reds bottom out in an activation that declares the same action from
                // the same state and reaches a different state with no dice rolled, which can only
                // be the submitted path (`JMOVEP` is the Java mirror).
                let at = g.field_model.player_coordinate(player_id).map(|c| (c.x, c.y));
                let offered: Vec<String> =
                    squares.iter().map(|c| format!("{},{}", c.x, c.y)).collect();
                let pid = player_id.clone();
                let sq = squares.clone();
                let a = match self.mode {
                    Mode::Deep => self.handle_move_deep(&gs.game, &f, pid, sq),
                    _ => self.handle_move(&gs.game, &f, pid, sq),
                };
                eprintln!(
                    "RMOVEP k={} pid={} at={:?} n={} offered=[{}] ans={:?}",
                    self.probe_act, player_id, at, squares.len(), offered.join(" "), a
                );
                a
            }
            AgentPrompt::Move { player_id, squares } => match self.mode {
                Mode::Wide | Mode::WideNoBall | Mode::WideNoPass | Mode::WideNoHandOff => self.handle_move(g, f, player_id, squares),
                Mode::Deep => self.handle_move_deep(g, f, player_id, squares),
            },

            AgentPrompt::ActivatePlayer { eligible_players } => match self.mode {
                Mode::Wide | Mode::WideNoBall | Mode::WideNoPass | Mode::WideNoHandOff => self.handle_activate(g, f, eligible_players),
                Mode::Deep => self.handle_activate_deep(g, f, eligible_players),
            },

            // §19.2 defect 1 — the declaration and this prompt must agree.
            //
            // `eligible_players` is adjacency-based and the prompt fires BEFORE movement, so a
            // ranged blitz's victim is legitimately absent from it. The engine's `handle_command`
            // accepts any player id, and constrains the following Move prompt with
            // `legal_blitz_move_targets`, so naming the planned victim is correct even when it is
            // not on the list. Deferring to the list instead was what left 42% of blitzes
            // walking toward a victim they had never selected.
            AgentPrompt::BlitzTarget { attacker_id, eligible_players } => {
                if let Some(pl) = &self.plan {
                    if pl.player == attacker_id {
                        if let PlanKind::Blitz { victim } = &pl.kind {
                            let alive = g
                                .field_model
                                .player_coordinate(victim)
                                .map(|c| on_pitch(c.x, c.y))
                                .unwrap_or(false);
                            if alive {
                                return Action::SelectPlayer { player_id: victim.clone() };
                            }
                        }
                    }
                }
                if eligible_players.is_empty() {
                    // NOT EndPlayerAction. The engine raises this prompt whenever ANY in-bounds
                    // opponent can be blocked, but the candidates are only the ADJACENT ones, so a
                    // blitzer with no neighbour gets an empty list -- and
                    // `ParityRunner.sendBlitzTargetSelection` answers exactly that case with
                    // ClientCommandEndTurn. `RandomAgent` mirrors it; this arm did not, and
                    // EndPlayerAction leaves `StepSelectBlitzTarget` waiting on a target that never
                    // comes: bb2025 seed 8 stalled with 50 unchanged iterations and aborted the game
                    // mid-drive.
                    return Action::EndTurn;
                }
                let astr = g.player(&attacker_id).map(|p| p.strength_with_modifiers()).unwrap_or(3);
                let mut ep = eligible_players;
                ep.sort_by_key(|id| canon_key(g, id));
                for did in &ep {
                    let w = block_weight(f, g, &attacker_id, did, astr);
                    self.buf.push_note(
                        Action::SelectPlayer { player_id: did.clone() },
                        w,
                        Rule::DiceCount,
                        w,
                        format!("blitz target {}", did),
                    );
                }
                let i = self.sample(0.15);
                let act = self.take(i);
                // Falling back means the plan's victim is gone. Re-point the plan at what was
                // actually selected, or the walk heads somewhere with nothing to hit.
                if let Action::SelectPlayer { player_id } = &act {
                    if let Some(pl) = self.plan.as_mut() {
                        if pl.player == attacker_id {
                            if let PlanKind::Blitz { .. } = &pl.kind {
                                pl.kind = PlanKind::Blitz { victim: player_id.clone() };
                                pl.path.clear();
                            }
                        }
                    }
                }
                act
            }

            AgentPrompt::BlockTarget { .. } => Action::EndPlayerAction,

            // ── block dice (§6.3) ───────────────────────────────────────────
            AgentPrompt::BlockChoice { attacker_id, defender_id, dice, own_choice, .. } => {
                let def_has_ball =
                    f.ball_carried && f.ball == g.field_model.player_coordinate(&defender_id);
                let att = g.player(&attacker_id);
                let def = g.player(&defender_id);
                let att_block = att.map(|p| p.has_skill(SkillId::Block)).unwrap_or(false);
                let att_wrestle = att.map(|p| p.has_skill(SkillId::Wrestle)).unwrap_or(false);
                let att_tackle = att.map(|p| p.has_skill(SkillId::Tackle)).unwrap_or(false);
                let def_block = def.map(|p| p.has_skill(SkillId::Block)).unwrap_or(false);
                let def_dodge = def.map(|p| p.has_skill(SkillId::Dodge)).unwrap_or(false);
                let surf = can_surf(g, &attacker_id, &defender_id);
                for (i, d) in dice.iter().enumerate() {
                    let mut w = match d {
                        6 => 0.90,
                        5 => {
                            if def_dodge && !att_tackle {
                                0.30
                            } else if surf {
                                0.95
                            } else {
                                0.80
                            }
                        }
                        2 => {
                            let att_down = !att_block && !att_wrestle;
                            let def_down = !def_block;
                            match (att_down, def_down) {
                                (false, true) => 0.70,
                                (true, true) => {
                                    if def_has_ball {
                                        0.50
                                    } else {
                                        0.30
                                    }
                                }
                                (true, false) => 0.10,
                                (false, false) => 0.35,
                            }
                        }
                        1 => 0.05,
                        _ => {
                            if surf {
                                0.80
                            } else {
                                0.40
                            }
                        }
                    };
                    if !own_choice {
                        w = 1.0 - w;
                    }
                    self.buf.push_note(
                        Action::BlockChoice { die_index: i, target_id: None },
                        w,
                        Rule::Face,
                        *d as f32,
                        format!("{} ({})", block_die_name(*d as i32), if own_choice { "ours" } else { "theirs" }),
                    );
                }
                if self.buf.options.is_empty() {
                    return Action::BlockChoice { die_index: 0, target_id: None };
                }
                let i = self.sample(0.12);
                self.take(i)
            }

            // ── pushback (§6.13) ────────────────────────────────────────────
            AgentPrompt::Pushback { attacker_id, defender_id, squares } => {
                if squares.is_empty() {
                    return Action::Acknowledge;
                }
                let def_home = g.team_home.has_player(&defender_id);
                let def_has_ball =
                    f.ball_carried && f.ball == g.field_model.player_coordinate(&defender_id);
                let att_c = g.field_model.player_coordinate(&attacker_id);
                let mut sorted = squares;
                sorted.sort_by_key(|c| (c.x, c.y));
                for sq in &sorted {
                    let off = !on_pitch(sq.x, sq.y);
                    let mut w = if off {
                        if def_has_ball {
                            1.0
                        } else {
                            0.95
                        }
                    } else if sq.y == 0 || sq.y == YMAX {
                        0.55
                    } else {
                        0.20
                    };
                    if let Some(a) = att_c {
                        if endzone_distance(*sq, !def_home) > endzone_distance(a, !def_home) {
                            w *= 1.3;
                        }
                    }
                    self.buf.push(Action::PushTo { coord: *sq }, w, Rule::Flat, w);
                }
                let i = self.sample(0.15);
                self.take(i)
            }

            // ── follow-up (§6.10) ───────────────────────────────────────────
            AgentPrompt::FollowUp { attacker_id, target_coord } => {
                let home = g.team_home.has_player(&attacker_id);
                let cur = g.field_model.player_coordinate(&attacker_id);
                let carries = f.ball_carried && f.ball == cur;
                let mut w: f32 = 0.5;
                if carries {
                    w -= 0.45;
                }
                if let Some(c) = cur {
                    if f.tz_against(target_coord, home) > f.tz_against(c, home) {
                        w -= 0.35;
                    }
                }
                if target_coord.y == 0 || target_coord.y == YMAX {
                    w -= 0.30;
                }
                let wf = w.clamp(0.02, 0.98);
                self.buf.push(Action::FollowUp { follow_up: true }, wf, Rule::Flat, wf);
                self.buf.push(Action::FollowUp { follow_up: false }, 1.0 - wf, Rule::Flat, 1.0 - wf);
                let i = self.sample(0.30);
                self.take(i)
            }

            // ── re-roll (§6.14) ─────────────────────────────────────────────
            AgentPrompt::ReRollOffer { action, .. } => {
                let home = g.home_playing;
                let td = if home { &g.turn_data_home } else { &g.turn_data_away };
                let we_carry = f
                    .carrier
                    .as_ref()
                    .map(|c| g.team_home.has_player(c) == home)
                    .unwrap_or(false);
                let consequence = match action.as_str() {
                    "GFI" | "DODGE" | "PICKUP" | "CATCH" | "JUMP" | "ESCAPE" => {
                        if we_carry {
                            0.85
                        } else {
                            0.55
                        }
                    }
                    "STAND_UP" | "TENTACLES" | "ALWAYS_HUNGRY" | "RIGHT_STUFF" => 0.35,
                    "FOUL_APPEARANCE" | "HYPNOTIC_GAZE" => 0.20,
                    _ => 0.45,
                };
                let scarcity = if td.rerolls > 0 {
                    let base = 0.45 + 0.55 * (td.rerolls as f32 / 3.0).min(1.0);
                    if td.turn_nr >= 7 {
                        base * 1.35
                    } else {
                        base
                    }
                } else {
                    0.0
                };
                let w_use = (consequence * 0.833 * scarcity).clamp(0.0, 1.0);
                self.buf.push(Action::UseReRoll { use_reroll: true }, w_use, Rule::Reroll, w_use);
                self.buf.push(Action::UseReRoll { use_reroll: false }, 1.0 - w_use, Rule::Reroll, 0.0);
                let i = self.sample(0.20);
                self.take(i)
            }

            other => self.act_boardless(gs, g, other),
        }
    }

    /// Prompts that read nothing about the board, so §20.7 never builds it for them.
    fn act_boardless(&mut self, gs: &GameState, g: &Game, prompt: AgentPrompt) -> Action {
        match prompt {
            // ── skill use (§6.16) ───────────────────────────────────────────
            AgentPrompt::SkillUse { skill_name, .. } => {
                let skill_id = SkillId::from_class_name(&skill_name).unwrap_or(SkillId::Block);
                let w_use = match skill_name.as_str() {
                    "Dodge" => 0.95,
                    "Juggernaut" => 0.80,
                    "HitAndRun" => 0.70,
                    "Fend" => 0.85,
                    "Wrestle" => 0.55,
                    "QuickBite" | "AnimalSavagery" => 0.85,
                    // The four skills whose USE path no harness can drive (DumpOff enters an
                    // undriveable INIT_PASSING, PrimalSavagery/Swoop open target dialogs,
                    // SafePairOfHands a PLACE_BALL coach dialog): pinned to DECLINE, still
                    // spending the sampler draws. Mirrored in HeuristicDriver.useSkill.
                    "DumpOff" | "PrimalSavagery" | "SafePairOfHands" | "Swoop" => 0.0,
                    _ => 0.50,
                };
                self.buf.push(
                    Action::UseSkill { skill_id, use_skill: true },
                    w_use,
                    Rule::Skill,
                    w_use,
                );
                self.buf.push(
                    Action::UseSkill { skill_id, use_skill: false },
                    1.0 - w_use,
                    Rule::Skill,
                    0.0,
                );
                let i = self.sample(0.20);
                if std::env::var_os("FFB_DRAWS").is_some() {
                    // `RSKILL`: the ANSWER to a SkillUse prompt (index 0 = use), so the two sides'
                    // skill decisions can be diffed and not just their draw totals (`JSKILL`).
                    eprintln!("RSKILL skill={skill_name} w_use={w_use} idx={i} draws={}", self.probe_draws);
                }
                self.take(i)
            }

            // ── interception (§6.19) ────────────────────────────────────────
            AgentPrompt::Interception { target_number, .. } => {
                let w = p_roll(target_number) * 1.5;
                self.buf.push(Action::Intercept { attempt: true }, w, Rule::Flat, w);
                self.buf.push(Action::Intercept { attempt: false }, 0.20, Rule::Flat, 0.20);
                let i = self.sample(0.20);
                self.take(i)
            }

            // ── kickoff placement (§6.27) ───────────────────────────────────
            //
            // A touchback hands the ball straight to the receivers, so the dominant term is the
            // chance of avoiding one — and that is exactly computable rather than a guess.
            // `StepKickoffScatterRoll` rolls a d8 direction and a d6 distance (a **d3** when the
            // kicker has Kick), and tests the touchback on the RAW endpoint before walking the ball
            // back onto the pitch. So enumerating the 48 (or 24) equally likely outcomes gives the
            // true probability for every candidate square.
            AgentPrompt::KickBall => {
                // Home kicking → the ball must land in HALF_AWAY (x 13..25); away → HALF_HOME.
                let home_kicking = g.home_playing;
                let (x0, x1) = if home_kicking { (13, XMAX) } else { (0, 12) };
                // The receiving team's own endzone — the far end of their half.
                let ez_x = if home_kicking { XMAX } else { 0 };

                // Kick halves the scatter, which is what makes a deep corner kick affordable.
                let has_kick = g
                    .field_model
                    .player_coordinates
                    .iter()
                    .filter(|(id, &c)| {
                        on_pitch(c.x, c.y) && g.team_home.has_player(id) == home_kicking
                    })
                    .any(|(id, _)| g.player(id).map(|p| p.has_skill(SkillId::Kick)).unwrap_or(false));
                let dmax = if has_kick { 3 } else { 6 };

                // Aim: the middle of the receiving half. With Kick, three squares toward an endzone
                // corner — both corners are offered, they are mirror images.
                let cx = (x0 + x1) / 2;
                let cy = YMAX / 2;
                let aims: Vec<(i32, i32)> = if has_kick {
                    let ax = cx + 3 * (ez_x - cx).signum();
                    vec![(ax, cy - 3), (ax, cy + 3)]
                } else {
                    vec![(cx, cy)]
                };

                const DIRS: [(i32, i32); 8] =
                    [(-1, -1), (0, -1), (1, -1), (-1, 0), (1, 0), (-1, 1), (0, 1), (1, 1)];

                // The scatter is not the only way to give the ball away. On a **Weather Change**
                // kickoff result (2d6 = 8, so 5/36) that rolls **Nice** (4–10, so 30/36), the ball
                // takes THREE further single-square d8 scatters and `StepApplyKickoffResult` re-tests
                // the half bounds after each one — any step that leaves the half is a touchback. So
                // the real risk of a square is its scatter risk plus 25/216 of a three-step random
                // walk falling out, which is why a square that looks perfectly safe is not.
                const P_GUST: f32 = 25.0 / 216.0;
                // q[k] = probability a k-step uniform-d8 walk from this square stays inside the
                // half at every step. Built by iteration rather than enumerating 8^3 paths per
                // candidate square.
                let inside = |x: i32, y: i32| x >= x0 && x <= x1 && y >= 0 && y <= YMAX;
                let mut q = vec![1.0f32; CELLS];
                for _ in 0..3 {
                    let mut next = vec![0.0f32; CELLS];
                    for x in x0..=x1 {
                        for y in 0..=YMAX {
                            let mut acc = 0.0f32;
                            for (dx, dy) in DIRS {
                                let (nx, ny) = (x + dx, y + dy);
                                if inside(nx, ny) {
                                    acc += q[ix(nx, ny)];
                                }
                            }
                            next[ix(x, y)] = acc / 8.0;
                        }
                    }
                    q = next;
                }

                for x in x0..=x1 {
                    for y in 0..=YMAX {
                        let mut acc = 0.0f32;
                        let mut total = 0;
                        for (dx, dy) in DIRS {
                            for d in 1..=dmax {
                                total += 1;
                                let (ex, ey) = (x + dx * d, y + dy * d);
                                if inside(ex, ey) {
                                    // survived the scatter; now survive a possible gust
                                    acc += (1.0 - P_GUST) + P_GUST * q[ix(ex, ey)];
                                }
                            }
                        }
                        let p_safe = acc / total as f32;
                        // Squared: a touchback does not merely waste the kick, it gifts possession.
                        let mut best = f32::MAX;
                        for (ax, ay) in &aims {
                            let dx = (x - ax) as f32;
                            let dy = (y - ay) as f32;
                            best = best.min(dx * dx / 16.0 + dy * dy / 9.0);
                        }
                        let w = p_safe * p_safe * exp_f32(-best);
                        // Scoring is done in the SERVER frame, but the action is a client command:
                        // `StepKickoff` mirrors an away coach's coordinate back, so an away kick has
                        // to be pre-transformed or it lands in the KICKING half and touchbacks.
                        // `RandomAgent`'s KickBall arm does the same, and says so; this arm did not,
                        // and every away kick in the game was mirrored (bb2025 seed 2 diverged at the
                        // very first activation with the two engines' picks byte-identical).
                        let target = FieldCoordinate::new(x, y);
                        let coord = if home_kicking { target } else { target.transform() };
                        self.buf.push_note(
                            Action::KickBall { coord },
                            w,
                            Rule::Flat,
                            p_safe,
                            format!(
                                "{},{} · {:.0}% no touchback (scatter + gust)",
                                x, y, 100.0 * p_safe
                            ),
                        );
                    }
                }
                let i = self.sample(0.10);
                self.take(i)
            }

            AgentPrompt::Touchback { eligible_players } => {
                if eligible_players.is_empty() {
                    return Action::Acknowledge;
                }
                let mut sorted = eligible_players;
                sorted.sort_by_key(|(pid, _)| canon_key(g, pid));
                let los = if g.home_playing { 12 } else { 13 };
                for (pid, coord) in &sorted {
                    let p = g.player(pid);
                    let ma = p.map(|q| q.movement_with_modifiers()).unwrap_or(6);
                    let mut w = 0.3 + 0.4 * (ma as f32 / 9.0).min(1.0);
                    if p.map(|q| q.has_skill(SkillId::SureHands)).unwrap_or(false) {
                        w += 0.3;
                    }
                    if (coord.x - los).abs() <= 1 {
                        w -= 0.5;
                    }
                    self.buf.push(Action::Touchback { player_id: pid.clone() }, w, Rule::Flat, w);
                }
                let i = self.sample(0.20);
                self.take(i)
            }

            AgentPrompt::CoinChoice { .. } => {
                self.buf.push(Action::CoinChoice { heads: true }, 0.5, Rule::Flat, 0.5);
                self.buf.push(Action::CoinChoice { heads: false }, 0.5, Rule::Flat, 0.5);
                let i = self.sample(1.0);
                self.take(i)
            }

            AgentPrompt::ReceiveChoice { .. } => {
                let w = if g.half == 1 { 0.65 } else { 0.85 };
                self.buf.push(Action::ReceiveChoice { receive: true }, w, Rule::Flat, w);
                self.buf.push(Action::ReceiveChoice { receive: false }, 1.0 - w, Rule::Flat, 0.0);
                let i = self.sample(0.30);
                self.take(i)
            }

            AgentPrompt::TeamSetup { team_id, .. } => canonical_setup_action(g, &team_id),

            // Everything else: the long tail this agent does not model.
            //
            // This used to fall through to `UniformAgent`, and that was wrong for a reason that has
            // nothing to do with the heuristic: `UniformAgent`'s `PlayerChoice` arm sorts the
            // candidates BY PLAYER ID, and the two engines generate different ids (`home_06` vs
            // `teamLinemanParityHome6`), so it cannot agree with anything on the Java side by
            // construction. `RandomAgent` -- the byte-matched parity contract -- has a dozen
            // reason-specific arms for exactly this prompt, every one of them coordinate-sorted for
            // exactly that reason.
            //
            // Delegating here makes `PromptClass::Other` a true no-op: on or off, an unmodelled
            // prompt gets the same answer, which is what "the agent does not model this" should
            // mean. bb2020 seed 26 is the case that showed it -- one PlayerChoice in the whole
            // game, from a prayer, answered `home_06` by the uniform tail and something else by
            // the contract.
            _ => self.parity.act(gs),
        }
    }
}

// ─────────────────────────────────────────────────────────────────── helpers

/// The block die faces, by the names a player uses. `BlockChoice { die_index }` carries only an
/// index, which tells a reader nothing about what was actually on the table.
/// Is `c` already next to the victim? If so there is no second leg to plan — the straight-shot
/// route above already covers it.
fn adjacent_to_victim(_f: &Features, g: &Game, c: FieldCoordinate, victim: &str) -> bool {
    g.field_model
        .player_coordinate(victim)
        .map(|v| c.distance_in_steps(v) == 1)
        .unwrap_or(false)
}

fn block_die_name(d: i32) -> &'static str {
    match d {
        1 => "Skull",
        2 => "Both Down",
        3 | 4 => "Push",
        5 => "Defender Stumbles",
        6 => "Defender Down",
        _ => "?",
    }
}

fn has_negatrait(p: &ffb_model::model::player::Player) -> bool {
    p.has_skill(SkillId::BoneHead)
        || p.has_skill(SkillId::ReallyStupid)
        || p.has_skill(SkillId::WildAnimal)
        || p.has_skill(SkillId::TakeRoot)
        || p.has_skill(SkillId::BloodLust)
}

fn action_is_live(a: &PlayerAction, td: &ffb_model::model::turn_data::TurnData, rules: Rules) -> bool {
    match a {
        PlayerAction::Block | PlayerAction::Blitz | PlayerAction::StandUpBlitz => !td.blitz_used,
        PlayerAction::Pass | PlayerAction::HailMaryPass => !td.pass_used,
        PlayerAction::HandOver => !td.hand_over_used,
        PlayerAction::Foul => !td.foul_used,
        PlayerAction::ThrowTeamMate => {
            if rules == Rules::Bb2025 {
                !td.ttm_used
            } else {
                !td.ttm_used && !td.pass_used
            }
        }
        PlayerAction::KickTeamMate => {
            if rules == Rules::Bb2016 {
                !td.blitz_used
            } else {
                !td.ktm_used
            }
        }
        _ => true,
    }
}

fn push_squares(g: &Game, att: &str, def: &str) -> Vec<FieldCoordinate> {
    let (a, d) = match (
        g.field_model.player_coordinate(att),
        g.field_model.player_coordinate(def),
    ) {
        (Some(a), Some(d)) => (a, d),
        _ => return vec![],
    };
    let dx = (d.x - a.x).signum();
    let dy = (d.y - a.y).signum();
    let base = FieldCoordinate::new(d.x + dx, d.y + dy);
    let mut out = vec![base];
    if dx != 0 && dy != 0 {
        out.push(FieldCoordinate::new(d.x + dx, d.y));
        out.push(FieldCoordinate::new(d.x, d.y + dy));
    } else if dx != 0 {
        out.push(FieldCoordinate::new(base.x, base.y - 1));
        out.push(FieldCoordinate::new(base.x, base.y + 1));
    } else {
        out.push(FieldCoordinate::new(base.x - 1, base.y));
        out.push(FieldCoordinate::new(base.x + 1, base.y));
    }
    out
}

fn can_surf(g: &Game, att: &str, def: &str) -> bool {
    push_squares(g, att, def).iter().any(|c| !on_pitch(c.x, c.y))
}

/// Assist-resolved dice count through the §2.4 table, plus context. §20.8 memoises the two
/// `find_block_strength` calls, each of which runs a nested player×player loop.
fn block_weight(f: &Features, g: &Game, att: &str, def: &str, att_str: i32) -> f32 {
    let (ac, dc) = match (
        g.field_model.player_coordinate(att),
        g.field_model.player_coordinate(def),
    ) {
        (Some(a), Some(d)) => (a, d),
        _ => return 0.05,
    };
    let (ap, dp) = match (g.player(att), g.player(def)) {
        (Some(a), Some(d)) => (a, d),
        _ => return 0.05,
    };
    let a_str = f.block_strength(g, att, ac, att_str, def, dc);
    let d_str = f.block_strength(g, def, dc, dp.strength_with_modifiers(), att, ac);
    let n = if a_str > 2 * d_str {
        3
    } else if a_str > d_str {
        2
    } else if 2 * a_str < d_str {
        -3
    } else if a_str < d_str {
        -2
    } else {
        1
    };
    let mut w: f32 = match n {
        3 => 0.90,
        2 => 0.60,
        1 => {
            if ap.has_skill(SkillId::Block) {
                0.40
            } else {
                0.25
            }
        }
        -2 => 0.10,
        _ => 0.025,
    };
    let has_ball = f.ball_carried && f.ball == Some(dc);
    if has_ball {
        w *= 1.35;
    }
    if can_surf(g, att, def) {
        w *= if has_ball { 1.9 } else { 1.5 };
    }
    if dp.has_skill(SkillId::Block)
        && !ap.has_skill(SkillId::Block)
        && !ap.has_skill(SkillId::Wrestle)
    {
        w *= 0.70;
    }
    w.clamp(0.01, 1.0)
}

/// An expectation, not a stack of multipliers: what the foul buys minus what it risks. Foul assists
/// modify the ARMOUR roll (+1 per net offensive assist), which is the whole reason to prefer one
/// victim over another — and the reason to foul at all.
fn foul_weight(f: &Features, g: &Game, att: &str, def: &str, m: &Mover) -> f32 {
    let av = g.player(def).map(|q| q.armour_with_modifiers()).unwrap_or(8);
    let off =
        ffb_model::util::util_player::UtilPlayer::find_offensive_foul_assists(g, att, def) as i32;
    let dfn =
        ffb_model::util::util_player::UtilPlayer::find_defensive_foul_assists(g, att, def) as i32;
    let p_break = p_2d6_at_least(av - (off - dfn) + 1).clamp(0.03, 0.97);
    let victim = if f.ball_carried && f.ball == g.field_model.player_coordinate(def) {
        1.0
    } else if f
        .ball
        .zip(g.field_model.player_coordinate(def))
        .map(|(b, t)| b.distance_in_steps(t) <= 1)
        .unwrap_or(false)
    {
        0.7
    } else {
        0.35
    };
    // The team's remaining Bribe the Ref, read from the TURN DATA's inducement set -- which is
    // where the engine actually keeps it. `team.bribes` is a separate field that only the
    // inducement-PURCHASE step writes, so it stays 0 for a bribe granted mid-game: the "Get the
    // Ref" kickoff hands +1 to BOTH teams (`StepApplyKickoffResult::handle_get_the_ref`, which
    // Rust implements faithfully -- into the inducement set). Reading the wrong field left every
    // foul after that kickoff priced with the 0.45 ejection cost instead of 0.07 (bb2025 seed 20
    // step 114: Java 0.064770 against Rust 0.003657, and Java fouled where Rust moved). The
    // purchase step writes BOTH, so the set covers bought bribes too.
    let ind_set = if m.home { &g.turn_data_home.inducement_set } else { &g.turn_data_away.inducement_set };
    let bribes = ind_set
        .for_usage(ffb_model::inducement::usage::Usage::AVOID_BAN)
        .and_then(|t| ind_set.get(t))
        .map(|i| i.value - i.uses)
        .unwrap_or(0);
    let eject_cost = if bribes > 0 { 0.07 } else { 0.45 };
    // The referee spots a foul on DOUBLES — armour, then injury if the armour broke. Fixed at about
    // 1/6, rising with the chance of hurting the victim, and nothing the agent chooses lowers it.
    let p_eject = 0.167 + p_break * (5.0 / 6.0) * 0.167;
    let timing = if m.unactivated <= 3.0 / 11.0 { 1.0 } else { 0.85 };
    (p_break * victim - p_eject * eject_cost) * timing
}

/// A fumble is a turnover on the spot, so the pass has to be an expectation.
///
/// The throw is priced with the ENGINE's own ruler rather than a distance band of my own: an
/// asymmetric `throwing_range_table` per edition, which also rules Long and LongBomb out entirely
/// in a Blizzard. `None` means the pass is not a legal throw at all, so the option should not
/// exist — the caller drops it.
///
/// The one approximation left is the modifier list: the engine assembles real `PassModifier`s from
/// tackle zones, weather and skills, and this passes an empty list and then charges +1 per opposing
/// tackle zone on the thrower, which is the dominant term.
/// Everything about a receiver that matters to a throw or a hand-off.
struct Receiver {
    /// Catch chance, at the accurate-pass/hand-off target of AG−1 plus tackle zones.
    p_catch: f32,
    /// Value of the square once HE is the carrier.
    v: f32,
    /// He still has his activation, so he can catch and then run it in on THIS turn.
    scores_now: bool,
    /// Turns he needs to reach the endzone himself.
    tts: i32,
    /// Turns he actually has, which is one fewer once he has already been activated.
    turns: i32,
}

fn receiver_of(f: &Features, g: &Game, rcv: &str, from: FieldCoordinate, m: &Mover) -> Option<Receiver> {
    let rc = g.field_model.player_coordinate(rcv)?;
    let rp = g.player(rcv)?;
    let i = ixc(rc);
    let tz = f.tz[side_idx(m.home)][i] as i32;
    let ag = rp.agility_with_modifiers();
    let raw = p_roll((ag - 1 + tz).max(2));
    let p_catch = if rp.has_skill(SkillId::Catch) { p_with_reroll(raw, 1.0) } else { raw };

    let td = if m.home { &g.turn_data_home } else { &g.turn_data_away };
    let rm = Mover {
        home: m.home,
        is_carrier: true,
        ma: rp.movement_with_modifiers(),
        ag,
        str_: rp.strength_with_modifiers(),
        sure_hands: rp.has_skill(SkillId::SureHands),
        side_step: rp.has_skill(SkillId::SideStep),
        has_catch: rp.has_skill(SkillId::Catch),
        // Measured from where the ball starts, so the advance term reads as ground the ball gained.
        d_now: endzone_distance(from, m.home),
        turns_left: (8 - td.turn_nr).max(0),
        unactivated: m.unactivated,
    };
    let d_rcv = endzone_distance(rc, m.home);
    let ma = rm.ma.max(1);
    // Still active = still holds his activation, so he can run after catching, this turn.
    let active = g.field_model.player_state(rcv).map(|st| st.is_active()).unwrap_or(false);
    let reach_after = if active { rm.ma + 2 } else { 0 };
    // P(he actually gets it in | he catches it). ~0.85 covers the dodges a run through traffic
    // needs; each rush beyond MA costs its own roll on top.
    let p_gfi = p_roll(gfi_target(weather_of(g)));
    let p_run_in = if !active {
        0.0
    } else if d_rcv <= (rm.ma - 2).max(0) {
        0.95
    } else if d_rcv <= rm.ma {
        0.85
    } else if d_rcv <= rm.ma + 1 {
        0.85 * p_gfi
    } else if d_rcv <= rm.ma + 2 {
        0.85 * p_gfi * p_gfi
    } else {
        // Out of scoring reach this turn, but he can still run: the delivery discount floors this.
        0.0
    };
    // Where the ball ENDS UP, not where the receiver is standing. This is the tempo the throw buys.
    let effective_d = (d_rcv - reach_after).max(0);
    let carrier_scores_now = m.d_now <= m.ma + 2;
    let scores_now = effective_d == 0 && active && !carrier_scores_now;

    // ABSOLUTE, on the same scale `value_at` uses for a carrier's own move: how far up the pitch
    // is the ball when the turn ends? That is the only way a ball-move and a run can be compared.
    let max_gain = m.d_now.min(m.ma + 2).max(1) as f32;
    let advance = ((m.d_now - effective_d) as f32 / max_gain).clamp(0.0, 1.0);
    let exposure = f.exposure(i, m.home, rm.str_);
    let lane = f.lane[side_idx(m.home)][i];
    // Getting the ball out of trouble is the other half of why teams hand off, and it is a margin
    // too: zero when the receiver is in as much danger as the carrier.
    let relief = (exposure - f.exposure(ixc(from), m.home, m.str_)).max(0.0);
    // MARGIN over running, not absolute position value. Zero gain is a catch roll bought for
    // nothing, and must price out that way.
    let v = if scores_now {
        // The one case worth paying a catch roll for, and it needs no lookahead to value.
        p_run_in
    } else {
        // A token positional credit only. Crediting the ground a throw buys, or the safety it
        // buys, measured -2.55 SE over 3200 games: this agent cannot collect on either.
        (0.12 * advance * exposure * lane + 0.10 * relief).min(0.20)
    };
    if std::env::var_os("FFB_RCV_TRACE").is_some() {
        eprintln!(
            "RCV {rcv} pCatch={p_catch:.2} adv={advance:.2} relief={relief:.2} v={v:.2}              dNow={} dRcv={d_rcv} reach={reach_after} effD={effective_d} active={active} now={scores_now}",
            m.d_now
        );
    }
    Some(Receiver {
        p_catch,
        v,
        scores_now,
        tts: (effective_d + ma - 1) / ma,
        turns: if active { m.turns_left } else { (m.turns_left - 1).max(0) },
    })
}

/// A short tag saying why a pass is, or is not, worth throwing — the arithmetic that decides it is
/// the turns-to-score comparison, and it is invisible in the weight alone.
fn pass_note(g: &Game, tc: FieldCoordinate, rcv: &str, m: &Mover) -> String {
    let rc = match g.field_model.player_coordinate(rcv) {
        Some(b) => b,
        None => return String::new(),
    };
    let ma_t = m.ma.max(1);
    let own = (endzone_distance(tc, m.home) + ma_t - 1) / ma_t;
    let rcv_ma = g.player(rcv).map(|p| p.movement_with_modifiers()).unwrap_or(6).max(1);
    let theirs = (endzone_distance(rc, m.home) + rcv_ma - 1) / rcv_ma;
    let left = m.turns_left;
    let active = g.field_model.player_state(rcv).map(|st| st.is_active()).unwrap_or(false);
    let rcv_d = endzone_distance(rc, m.home);
    if active && rcv_d <= rcv_ma + 2 {
        let rushes = (rcv_d - rcv_ma).max(0);
        return format!(
            " · can score this turn: unactivated, {rcv_d} from the line, MA {rcv_ma}{}",
            if rushes > 0 { format!(", needs {rushes} rush(es)") } else { String::new() }
        );
    }
    if own > left {
        if theirs <= left {
            format!(" · RESCUE: {own} turns to run it in, {left} left — he needs {theirs}")
        } else {
            format!(" · drive lost: {own} turns needed, {left} left, he needs {theirs}")
        }
    } else {
        format!(" · can still run it in: {own} turns, {left} left")
    }
}

fn pass_weight(
    f: &Features,
    g: &Game,
    thrower: &str,
    tc: FieldCoordinate,
    rcv: &str,
    m: &Mover,
) -> Option<f32> {
    let rc = g.field_model.player_coordinate(rcv)?;
    // The ENGINE's ruler: asymmetric per edition, and no Long/LongBomb at all in a Blizzard.
    // `None` means this is not a legal throw, so the option should not exist.
    let mech = crate::mechanic::pass_mechanic_for(g.rules);
    let dist = mech.find_passing_distance(g, Some(tc), Some(rc), false)?;
    let p = g.player(thrower)?;
    // Java (BallMoves.gradeFaces) bails ONLY on an illegal distance — it never consults the
    // minimum roll. Bailing here on `minimum_roll_simple == None` dropped every pass candidate
    // for a PA-less thrower (chaos bb2020 seed 4 k=15: the Minotaur, passing 0 — Java built
    // Home1/Pass:14, Rust built none and blitzed instead). The minimum is only trace fodder.
    let base_target = mech.minimum_roll_simple(p, dist, &[]);
    let tz_on_thrower = f.tz[side_idx(m.home)][ixc(tc)] as i32;

    // Roll all six faces through the engine's own grader, so FUMBLE is separated from merely
    // INACCURATE. Tackle zones on the thrower shift the effective roll rather than the target.
    let mut n_accurate = 0;
    let mut n_fumble = 0;
    for die in 1..=6i32 {
        match mech.evaluate_pass_simple(p, die - tz_on_thrower, dist, &[], false) {
            ffb_mechanics::pass_result::PassResult::ACCURATE => n_accurate += 1,
            ffb_mechanics::pass_result::PassResult::FUMBLE => n_fumble += 1,
            _ => {}
        }
    }
    let p_accurate = n_accurate as f32 / 6.0;
    let p_fumble = n_fumble as f32 / 6.0;
    let p_scatter = (1.0 - p_accurate - p_fumble).max(0.0);
    let p_throw = p_accurate;

    let r = receiver_of(f, g, rcv, tc, m)?;
    let p_complete = p_accurate * r.p_catch;
    // A scattered ball is not a turnover — it lands three squares away and either side may get it.
    // Only a fumble, or a dropped catch, hands the turn over outright.
    let p_lost = p_fumble + p_scatter * 0.45 + p_accurate * (1.0 - r.p_catch);

    // Can the CARRIER still run it in before the half ends?
    let ma_t = m.ma.max(1);
    let own_tts = (m.d_now + ma_t - 1) / ma_t;
    let hopeless = own_tts > m.turns_left;
    // A completion that leaves somebody who CAN still make it turns a dead drive into a live one.
    let rescues = hopeless && r.tts <= r.turns;

    let mut v = r.v;
    if r.scores_now {
        v = 1.0;
    } else if rescues {
        v = v.max(0.85);
    }
    // A drive that was going to score nothing has little left to lose, which the generic turnover
    // cost — priced off how many players are still unactivated — cannot see.
    let risk = c_turnover(m.unactivated, 0, false) * if hopeless { 0.30 } else { 1.0 };

    if std::env::var_os("FFB_PASS_TRACE2").is_some() {
        eprintln!(
            "PW dist={:?} tgt={:?} pAcc={:.2} pFum={:.2} pScat={:.2} pCatch={:.2} pC={:.2} \
             pLost={:.2} v={:.2} risk={:.2} scoresNow={} hopeless={} rescues={}",
            dist, base_target, p_accurate, p_fumble, p_scatter, r.p_catch, p_complete,
            p_lost, v, risk, r.scores_now, hopeless, rescues
        );
    }
    Some(p_complete * v - p_lost * risk)
}

/// A hand-off is a pass with no throw to fumble — only the catch can fail — so it is strictly the
/// safer way to move the ball one square. Priced the same way, which mostly means: a hand-off to an
/// unactivated Catcher who can then run it in is a touchdown, not "a slightly better square".
fn handoff_weight(f: &Features, g: &Game, tc: FieldCoordinate, rcv: &str, m: &Mover) -> Option<f32> {
    let r = receiver_of(f, g, rcv, tc, m)?;

    let ma_t = m.ma.max(1);
    let own_tts = (m.d_now + ma_t - 1) / ma_t;
    let hopeless = own_tts > m.turns_left;
    let rescues = hopeless && r.tts <= r.turns;

    let mut v = r.v;
    if r.scores_now {
        v = 1.0;
    } else if rescues {
        v = v.max(0.85);
    }
    let risk = c_turnover(m.unactivated, 0, false) * if hopeless { 0.30 } else { 1.0 };
    // No payoff multiplier: `scores_now` already carries P(touchdown).
    let payoff = 1.0;
    Some(r.p_catch * v * payoff - (1.0 - r.p_catch) * risk)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn p_roll_matches_the_table() {
        assert!((p_roll(2) - 0.8333).abs() < 1e-3);
        assert!((p_roll(4) - 0.5).abs() < 1e-3);
        assert!((p_roll(6) - 0.1667).abs() < 1e-3);
    }

    /// D1: the GFI target is 3+ in a Blizzard, in EVERY edition.
    #[test]
    fn gfi_target_is_three_in_a_blizzard() {
        assert_eq!(gfi_target(Weather::Blizzard), 3);
        assert_eq!(gfi_target(Weather::Nice), 2);
    }

    #[test]
    fn reroll_composition() {
        assert!((p_with_reroll(p_roll(4), 1.0) - 0.75).abs() < 1e-3);
    }

    /// P4: standing up costs 3 MA.
    #[test]
    fn stand_up_cost_is_three() {
        assert_eq!(STAND_UP_COST, 3);
    }

    /// A1: a risky move is worth less early in the turn than late.
    #[test]
    fn turnover_cost_falls_as_the_turn_empties() {
        assert!(c_turnover(10.0 / 11.0, 0, false) > c_turnover(1.0 / 11.0, 0, false));
    }

    /// A3: the same gain scores the same from deep and from close.
    #[test]
    fn advance_is_measured_against_what_the_activation_can_reach() {
        let ma = 6i32;
        let gain = 6i32;
        let far = (gain as f32) / (24i32.min(ma + 2).max(1) as f32);
        let near = (gain as f32) / (12i32.min(ma + 2).max(1) as f32);
        assert!((far - near).abs() < 1e-6);
    }

    /// The rush penalty must make a non-carrier's second rush clearly unattractive while leaving
    /// the carrier free to push for a touchdown.
    #[test]
    fn rush_penalty_is_much_harsher_for_a_non_carrier() {
        assert!(rush_penalty(2, false) > 3.0 * rush_penalty(2, true));
        assert_eq!(rush_penalty(0, false), 0.0);
    }

    /// §20.6 — the flat-index helpers must round-trip over the whole pitch.
    #[test]
    fn cell_index_round_trips() {
        for y in 0..H as i32 {
            for x in 0..W as i32 {
                assert_eq!(coord_of(ix(x, y)), FieldCoordinate::new(x, y));
            }
        }
    }

    /// P(2d6 >= n): 7+ is 21/36, 2+ certain, 13+ impossible.
    #[test]
    fn two_dice_table() {
        assert!((p_2d6_at_least(7) - 21.0 / 36.0).abs() < 1e-6);
        assert_eq!(p_2d6_at_least(2), 1.0);
        assert_eq!(p_2d6_at_least(13), 0.0);
    }

    /// Foul assists must raise the chance of breaking armour — the whole point of picking a victim.
    #[test]
    fn foul_assists_raise_the_break_chance() {
        let unassisted = p_2d6_at_least(8 + 1);
        let assisted = p_2d6_at_least(8 - 3 + 1);
        assert!(assisted > unassisted * 1.5);
    }

    /// §20.6 — the heap must be a MIN-heap on the quantised −log p key.
    #[test]
    fn heap_pops_the_cheapest_first() {
        let mut h = BinaryHeap::new();
        h.push(HeapItem { key: 500, cost: 2, idx: 7 });
        h.push(HeapItem { key: 100, cost: 1, idx: 3 });
        h.push(HeapItem { key: 900, cost: 3, idx: 9 });
        assert_eq!(h.pop().unwrap().key, 100);
        assert_eq!(h.pop().unwrap().key, 500);
        assert_eq!(h.pop().unwrap().key, 900);
    }

    /// The Dijkstra key increment must stay inside the range the clamp assumes, or the `as i64`
    /// conversion would start doing real work and the two languages would need to agree on
    /// out-of-range float-to-int semantics (they do not). `p_step` is a product of `p_roll`
    /// values, each in [1/6, 5/6], floored at 1e-6 by the caller.
    #[test]
    fn dijkstra_key_increment_stays_in_the_clamped_range() {
        let mut p = 1.0f32;
        // The worst case the search can reach: every step multiplies in the lowest `p_roll`.
        for _ in 0..16 {
            p *= p_roll(6);
            let raw = -ln_f32(p.max(1e-6)) * KEY_SCALE;
            assert!(raw >= 0.0, "increment must never be negative (p = {p})");
            assert!(raw <= 1.0e9, "increment must stay under the clamp (p = {p})");
            assert_eq!(
                raw.clamp(0.0, 1.0e9) as i64 as u32,
                raw as u32,
                "clamp must be a no-op over the reachable range"
            );
        }
    }

    /// Regenerates `testdata/sampler_golden.txt`, the cross-language pin for the SAMPLER --
    /// `unit()`, the eps escape, and the pick index. Same role as the det_math table: the Java
    /// twin asserts on this exact file, so a divergence fails a unit test instead of showing up
    /// as a mystery state-hash mismatch 200 steps into a game.
    ///
    /// The `Features` CORE raster tier, as a cross-language fixture.
    ///
    /// `occ`, `tz` and `row_prefix` are the foundation the whole value model stands on: `Reach`
    /// walks them, `build_threat` and `build_support` seed off them, and every arrival weight
    /// reads them. If the two languages disagree here, nothing downstream can agree — and a
    /// disagreement found in a 100-seed sweep costs a day, while one found here costs a minute.
    /// So the rasters get pinned BEFORE any of it is wired to a game, which is what the plan asks
    /// for.
    ///
    /// The boards are deliberately awkward: pitch corners, a full sideline column, prone players
    /// (no tackle zones, so they mark nothing but still occupy), and two players adjacent across
    /// the halfway line. `row_prefix` in particular is an exclusive prefix over `W + 1` columns
    /// per row, which is exactly the kind of off-by-one a from-scratch reimplementation gets
    /// wrong.
    ///
    /// `cargo test -p ffb-engine --lib agent::heuristic_agent::tests::emit_features_golden -- --ignored`
    #[test]
    #[ignore]
    fn emit_features_golden() {
        use std::fmt::Write as _;
        use ffb_model::enums::{PlayerState, PS_STANDING, PS_PRONE, Rules};
        use ffb_model::model::player::Player;

        // (is_home, nr, x, y, standing, active, ma, st)
        type Board = &'static [(bool, i32, i32, i32, bool, bool, i32, i32)];
        // (name, board, ball, ball_in_play, ball_moving, blitz_used_home, blitz_used_away)
        let boards: [(&str, Board, Option<(i32, i32)>, bool, bool, bool, bool); 6] = [
            ("empty", &[], None, false, false, false, false),
            ("single_centre", &[(true, 1, 13, 7, true, true, 6, 3)], None, false, false, false, false),
            (
                "corners",
                &[
                    (true, 1, 0, 0, true, true, 6, 3),
                    (true, 2, 25, 0, true, true, 6, 3),
                    (false, 1, 0, 14, true, true, 6, 3),
                    (false, 2, 25, 14, true, true, 6, 3),
                ],
                None, false, false, false, false,
            ),
            (
                "prone_marks_nothing",
                &[
                    (true, 1, 10, 7, true, true, 6, 3),
                    (true, 2, 10, 8, false, true, 6, 3),
                    (false, 1, 11, 7, true, true, 6, 3),
                    (false, 2, 11, 8, false, false, 6, 3),
                ],
                // A LOOSE ball: `build_support`'s screen term targets it when nobody carries.
                Some((10, 7)), true, true, false, false,
            ),
            (
                "line_of_scrimmage",
                &[
                    (true, 1, 12, 5, true, true, 6, 3), (true, 2, 12, 6, true, true, 6, 3),
                    (true, 3, 12, 7, true, true, 6, 3), (true, 4, 12, 8, true, false, 6, 3),
                    (true, 5, 12, 9, true, false, 6, 3), (true, 6, 11, 7, true, true, 6, 3),
                    (false, 1, 13, 5, true, true, 6, 3), (false, 2, 13, 6, true, true, 6, 3),
                    (false, 3, 13, 7, true, true, 6, 3), (false, 4, 13, 8, true, true, 6, 3),
                    (false, 5, 13, 9, false, true, 6, 3), (false, 6, 14, 7, true, true, 6, 3),
                ],
                // home_03 CARRIES it, so cage/mark/screen all have a target, and the away blitz is
                // already spent — which switches off `build_threat`'s block term for every
                // non-adjacent away player and is easy to drop in a reimplementation.
                Some((12, 7)), true, false, false, true,
            ),
            (
                // MIXED STRENGTH, on purpose. `build_threat` writes `threat_str` under a strict
                // `>` against `threat_reach`, so two opponents that reach a square equally TIE --
                // and that tie is what ITER1 found being resolved by HashMap order. An all-ST3
                // fixture cannot see it: every tie writes the same 3.
                "mixed_strength_ties",
                &[
                    (true, 1, 10, 7, true, true, 6, 3),
                    (false, 1, 9, 7, true, true, 6, 5),
                    (false, 2, 11, 7, true, true, 6, 3),
                    (false, 3, 10, 5, true, true, 8, 4),
                ],
                None, false, false, false, false,
            ),
        ];

        let mut out = String::new();
        writeln!(out, "# Features CORE raster golden -- heuristic_agent.rs and Features.java.").unwrap();
        writeln!(out, "# board <name> <n>").unwrap();
        writeln!(out, "# player <home|away> <nr> <x> <y> <standing|prone> <active|used>").unwrap();
        writeln!(out, "# occ <hex, one byte per cell, row-major over W=26 by H=15>").unwrap();
        writeln!(out, "# tz <side 0=home 1=away> <hex, one byte per cell>").unwrap();
        writeln!(out, "# rowprefix <side> <decimal, H*(W+1) entries, comma separated>").unwrap();
        writeln!(out, "# unact <side> <f32 bits>").unwrap();
        writeln!(out, "# ball <x> <y> <inplay> <moving>   (absent when there is no ball)").unwrap();
        writeln!(out, "# blitz <home_used> <away_used>").unwrap();
        writeln!(out, "# threatreach|threatmark|lane|support <side> <hex, 8 chars of f32 bits per cell>").unwrap();
        writeln!(out, "# threatstr <side> <hex, one signed byte per cell>").unwrap();

        for (name, board, ball, ball_in_play, ball_moving, blitz_home, blitz_away) in boards {
            let mut home = crate::step::framework::test_team("home", 0);
            let mut away = crate::step::framework::test_team("away", 0);
            for &(is_home, nr, _, _, _, _, ma, st) in board {
                let p = Player {
                    id: format!("{}_{:02}", if is_home { "home" } else { "away" }, nr),
                    nr,
                    movement: ma,
                    strength: st,
                    agility: 3,
                    armour: 8,
                    ..Default::default()
                };
                if is_home { home.players.push(p) } else { away.players.push(p) }
            }
            let mut g = Game::new(home, away, Rules::Bb2025);
            for &(is_home, nr, x, y, standing, active, _, _) in board {
                let id = format!("{}_{:02}", if is_home { "home" } else { "away" }, nr);
                g.field_model.set_player_coordinate(&id, FieldCoordinate::new(x, y));
                // ACTIVE is a bit of its own -- `PlayerState::new(PS_STANDING)` does NOT set it,
                // which is why `unactivated` was the one field the first fixture run disagreed on.
                g.field_model.set_player_state(
                    &id,
                    PlayerState::new(if standing { PS_STANDING } else { PS_PRONE })
                        .change_active(active),
                );
            }
            if let Some((bx, by)) = ball {
                g.field_model.ball_coordinate = Some(FieldCoordinate::new(bx, by));
            }
            g.field_model.ball_in_play = ball_in_play;
            g.field_model.ball_moving = ball_moving;
            g.turn_data_home.blitz_used = blitz_home;
            g.turn_data_away.blitz_used = blitz_away;
            // HEAVY: threat, lane and support as well as the core rasters.
            let f = Features::build(&g, positions_stamp(&g), true);

            writeln!(out, "board {name} {}", board.len()).unwrap();
            if let Some((bx, by)) = ball {
                writeln!(out, "ball {bx} {by} {ball_in_play} {ball_moving}").unwrap();
            }
            writeln!(out, "blitz {blitz_home} {blitz_away}").unwrap();
            for &(is_home, nr, x, y, standing, _, ma, st) in board {
                // `is_active()` is a SEPARATE bit from the standing/prone base, and
                // `PlayerState::new(PS_STANDING)` does not set it — so emit what the engine
                // actually computed rather than letting the two sides each assume. The Java
                // fixture reads this column; guessing it was the one thing that disagreed on the
                // first run, and it was the fixture guessing, not the arithmetic.
                let id = format!("{}_{:02}", if is_home { "home" } else { "away" }, nr);
                let active = g
                    .field_model
                    .player_state(&id)
                    .map(|st| st.is_active())
                    .unwrap_or(false);
                writeln!(out, "player {} {nr} {x} {y} {} {} {ma} {st}",
                    if is_home { "home" } else { "away" },
                    if standing { "standing" } else { "prone" },
                    if active { "active" } else { "used" }).unwrap();
            }
            let hex = |v: &[u8]| v.iter().map(|b| format!("{b:02x}")).collect::<String>();
            writeln!(out, "occ {}", hex(&f.occ)).unwrap();
            for side in 0..2 {
                writeln!(out, "tz {side} {}", hex(&f.tz[side])).unwrap();
            }
            for side in 0..2 {
                let rp: Vec<String> =
                    f.row_prefix[side].iter().map(|v| v.to_string()).collect();
                writeln!(out, "rowprefix {side} {}", rp.join(",")).unwrap();
            }
            for side in 0..2 {
                writeln!(out, "unact {side} {:08x}", f.unactivated[side].to_bits()).unwrap();
            }
            let fhex = |v: &[f32]| {
                v.iter().map(|x| format!("{:08x}", x.to_bits())).collect::<String>()
            };
            for side in 0..2 {
                writeln!(out, "threatreach {side} {}", fhex(&f.threat_reach[side])).unwrap();
            }
            for side in 0..2 {
                let v: String =
                    f.threat_str[side].iter().map(|x| format!("{:02x}", *x as u8)).collect();
                writeln!(out, "threatstr {side} {v}").unwrap();
            }
            for side in 0..2 {
                writeln!(out, "threatmark {side} {}", fhex(&f.threat_mark[side])).unwrap();
            }
            for side in 0..2 {
                writeln!(out, "lane {side} {}", fhex(&f.lane[side])).unwrap();
            }
            for side in 0..2 {
                writeln!(out, "support {side} {}", fhex(&f.support[side])).unwrap();
            }
        }

        // CARGO_MANIFEST_DIR, not `file!()`: the test's cwd is the package directory, so a
        // `file!()`-relative path writes a second nested `crates/ffb-engine` tree.
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/src/agent/testdata/features_golden.txt");
        std::fs::write(path, out).unwrap();
        eprintln!("wrote {path}");
    }

    /// `Reach` — the quantised-key Dijkstra — as a cross-language fixture.
    ///
    /// This is the piece where a from-scratch reimplementation is most likely to agree
    /// approximately and disagree exactly, and where "approximately" is worthless: `p_arrive` is
    /// `exp(-key / 4096)`, the value model compares arrival weights with `>`, and the whole search
    /// is a min-heap whose ties have to break the same way in both languages. Three separate
    /// hazards, none of them visible from a game log:
    ///
    /// 1. **the key increment.** `(-ln(p) * 4096) as u32` saturates in Rust and is undefined-ish in
    ///    Java for out-of-range floats, so the Rust side clamps explicitly and goes via `i64`.
    ///    Whether the Java twin reproduces the same integer for every step is exactly what this
    ///    pins.
    /// 2. **heap tie-breaking.** `HeapItem` is ordered on `(key, cost, idx)` — `idx` only to make
    ///    ties deterministic. A Java `PriorityQueue` comparing on `key` alone would pop equal-key
    ///    cells in insertion order and settle a different `prev`, giving a different PATH with an
    ///    identical arrival probability. The `prev` array below is what catches that; the keys
    ///    alone would not.
    /// 3. **`order`.** Sorted at the end precisely so the OUTPUT does not depend on the heap, but
    ///    its membership still has to match.
    ///
    /// The cases cover an open pitch (no dodges, pure GFI), a tackle-zone gauntlet (dodge targets
    /// that differ per destination square), a prone mover (the stand-up cost eats MA and the gate
    /// becomes `p_roll(4)`), a Blizzard (GFI target 3 rather than 2), and BB2016 (a different
    /// dodge-target formula entirely).
    ///
    /// `cargo test -p ffb-engine --lib agent::heuristic_agent::tests::emit_reach_golden -- --ignored`
    #[test]
    #[ignore]
    fn emit_reach_golden() {
        use std::fmt::Write as _;
        use ffb_model::enums::{PlayerState, PS_STANDING, PS_PRONE, Rules, Weather};
        use ffb_model::model::player::Player;

        // (is_home, nr, x, y, standing, ma, st, ag, dodge, sure_feet)
        type Mover = (bool, i32, i32, i32, bool, i32, i32, i32, bool, bool);
        let cases: [(&str, Rules, Weather, bool, &[Mover]); 5] = [
            // Open pitch: nothing to dodge out of, so every step past MA is a bare GFI.
            ("open", Rules::Bb2025, Weather::Nice, false,
             &[(true, 1, 5, 7, true, 6, 3, 3, false, false)]),
            // A gauntlet. The mover starts marked, so EVERY step out of his square is a dodge, and
            // the target depends on how many tackle zones the DESTINATION has.
            ("gauntlet", Rules::Bb2025, Weather::Nice, false,
             &[(true, 1, 10, 7, true, 6, 3, 3, false, false),
               (false, 1, 11, 7, true, 6, 3, 3, false, false),
               (false, 2, 11, 8, true, 6, 3, 3, false, false),
               (false, 3, 10, 6, true, 6, 3, 3, false, false),
               (false, 4, 9, 8, true, 6, 3, 3, false, false)]),
            // Prone with a team re-roll: MA drops by the stand-up cost and `gate` becomes p_roll(4),
            // and the re-roll applies to the FIRST roll of a path only.
            ("prone_with_reroll", Rules::Bb2025, Weather::Nice, true,
             &[(true, 1, 10, 7, false, 6, 3, 3, false, false),
               (false, 1, 11, 7, true, 6, 3, 3, false, false)]),
            // Blizzard: the GFI target is 3, not 2, in every edition. Sure Feet as well, so the GFI
            // re-roll branch is taken rather than the bare roll.
            ("blizzard_sure_feet", Rules::Bb2025, Weather::Blizzard, false,
             &[(true, 1, 5, 7, true, 4, 3, 3, false, true)]),
            // BB2016 computes the dodge target from `7 - AG` instead of the AG scale, and this
            // mover has Dodge, so the skill re-roll branch is taken on every dodge.
            ("bb2016_dodge", Rules::Bb2016, Weather::Nice, false,
             &[(true, 1, 10, 7, true, 6, 3, 3, true, false),
               (false, 1, 11, 7, true, 6, 3, 3, false, false),
               (false, 2, 9, 6, true, 6, 3, 3, false, false)]),
        ];

        let mut out = String::new();
        writeln!(out, "# Reach golden -- heuristic_agent.rs `reach_with` and Reach.java.").unwrap();
        writeln!(out, "# case <name> <rules> <weather> <team_rr>").unwrap();
        writeln!(out, "# mover <home|away> <nr> <x> <y> <standing|prone> <ma> <st> <ag> <dodge> <surefeet>").unwrap();
        writeln!(out, "# budget <ma> <spent> <cap> <gate f32 bits>").unwrap();
        writeln!(out, "# key <decimal per cell, comma separated; 4294967295 = unreached>").unwrap();
        writeln!(out, "# cost|gfi <decimal per cell>").unwrap();
        writeln!(out, "# prev <decimal per cell; 65535 = none>").unwrap();
        writeln!(out, "# order <decimal cell indices, ascending>").unwrap();
        writeln!(out, "# path <x> <y> <x,y;x,y;...>   the back-pointer walk to that square").unwrap();

        for (name, rules, weather, team_rr, movers) in cases {
            let mut home = crate::step::framework::test_team("home", 0);
            let mut away = crate::step::framework::test_team("away", 0);
            for &(is_home, nr, _, _, _, ma, st, ag, dodge, sure_feet) in movers {
                let mut p = Player {
                    id: format!("{}_{:02}", if is_home { "home" } else { "away" }, nr),
                    nr,
                    movement: ma,
                    strength: st,
                    agility: ag,
                    armour: 8,
                    ..Default::default()
                };
                if dodge {
                    p.starting_skills.push(ffb_model::model::skill_def::SkillWithValue {
                        skill_id: SkillId::Dodge,
                        value: None,
                    });
                }
                if sure_feet {
                    p.starting_skills.push(ffb_model::model::skill_def::SkillWithValue {
                        skill_id: SkillId::SureFeet,
                        value: None,
                    });
                }
                if is_home { home.players.push(p) } else { away.players.push(p) }
            }
            let mut g = Game::new(home, away, rules);
            g.field_model.weather = weather;
            for &(is_home, nr, x, y, standing, _, _, _, _, _) in movers {
                let id = format!("{}_{:02}", if is_home { "home" } else { "away" }, nr);
                g.field_model.set_player_coordinate(&id, FieldCoordinate::new(x, y));
                g.field_model.set_player_state(
                    &id,
                    PlayerState::new(if standing { PS_STANDING } else { PS_PRONE })
                        .change_active(true),
                );
            }
            let f = Features::build(&g, positions_stamp(&g), true);
            let mut sc = Scratch::default();
            let b = budget_of(&g, "home_01").expect("the mover is on the pitch");
            let r = reach_with(&f, &g, "home_01", &b, team_rr, &mut sc)
                .expect("a positive movement cap");

            writeln!(out, "case {name} {rules:?} {weather:?} {team_rr}").unwrap();
            for &(is_home, nr, x, y, standing, ma, st, ag, dodge, sure_feet) in movers {
                writeln!(out, "mover {} {nr} {x} {y} {} {ma} {st} {ag} {dodge} {sure_feet}",
                    if is_home { "home" } else { "away" },
                    if standing { "standing" } else { "prone" }).unwrap();
            }
            writeln!(out, "budget {} {} {} {:08x}", b.ma, b.spent, b.cap, b.gate.to_bits()).unwrap();
            let join = |v: Vec<String>| v.join(",");
            writeln!(out, "key {}", join(r.cell.iter().map(|c| c.key.to_string()).collect())).unwrap();
            writeln!(out, "cost {}", join(r.cell.iter().map(|c| c.cost.to_string()).collect())).unwrap();
            writeln!(out, "gfi {}", join(r.cell.iter().map(|c| c.gfi.to_string()).collect())).unwrap();
            writeln!(out, "prev {}", join(r.cell.iter().map(|c| c.prev.to_string()).collect())).unwrap();
            writeln!(out, "order {}", join(r.order.iter().map(|i| i.to_string()).collect())).unwrap();
            // Paths to a handful of reached squares, so `prev` is checked as a WALK and not only
            // cell by cell: a single wrong back-pointer is a wrong route.
            let mut path = Vec::new();
            let mut emitted = 0;
            for i in r.order.iter().map(|i| *i as usize) {
                if !r.reached(i) {
                    continue;
                }
                r.path_to(i, &mut path);
                if path.is_empty() {
                    continue;
                }
                let c = coord_of(i);
                let steps: Vec<String> =
                    path.iter().map(|p| format!("{},{}", p.x, p.y)).collect();
                writeln!(out, "path {} {} {}", c.x, c.y, steps.join(";")).unwrap();
                emitted += 1;
                if emitted >= 12 {
                    break;
                }
            }
        }

        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/src/agent/testdata/reach_golden.txt");
        std::fs::write(path, out).unwrap();
        eprintln!("wrote {path}");
    }

    /// `value_at` — the per-square value model — as a cross-language fixture.
    ///
    /// This is where the agent decides what the board is WORTH, and it is the last piece before
    /// `build_plans`. Everything it reads has already been pinned (`Features` in ITER24/25,
    /// `Reach` in ITER26); what is new here is the arithmetic on top, and it branches hard:
    ///
    /// - a CARRIER scores by advance, urgency and whether the endzone is reachable at all in the
    ///   turns the half has left (the `HOPELESS_DAMP` residual);
    /// - a mover standing on a LOOSE ball is a pickup, valued by the pickup roll;
    /// - everyone else is valued by the support raster, unless he is a plausible RECEIVER — ahead
    ///   of the ball, in throwing range, and able to run the rest in next turn.
    ///
    /// Each of those is a different formula, and the receiver branch in particular has three
    /// conditions that all have to agree or the intent fires on the wrong half of the pitch.
    ///
    /// Emitted for every square and every mover, so the branch selection is pinned by the `rule`
    /// column as well as the value: two implementations can agree on a number while disagreeing
    /// about WHY, and that disagreement shows up the moment the board changes.
    ///
    /// `cargo test -p ffb-engine --lib agent::heuristic_agent::tests::emit_value_golden -- --ignored`
    #[test]
    #[ignore]
    fn emit_value_golden() {
        use std::fmt::Write as _;
        use ffb_model::enums::{PlayerState, PS_STANDING, Rules};
        use ffb_model::model::player::Player;

        // (is_home, nr, x, y)
        type Board = &'static [(bool, i32, i32, i32)];
        // (home, is_carrier, ma, ag, str, sure_hands, side_step, has_catch, d_now, turns_left,
        //  unactivated)
        type M = (bool, bool, i32, i32, i32, bool, bool, bool, i32, i32, f32);

        let cases: [(&str, Board, Option<(i32, i32)>, bool, &[(&str, M)]); 4] = [
            (
                // A carrier with room, and the same board seen by a non-carrier team-mate who is
                // AHEAD of the ball -- which is what switches the receiver branch on.
                "carrier_and_receiver",
                &[
                    (true, 1, 8, 7, ), (true, 2, 14, 7), (true, 3, 8, 9),
                    (false, 1, 12, 6), (false, 2, 12, 8), (false, 3, 16, 7),
                ],
                Some((8, 7)), false,
                &[
                    ("carrier", (true, true, 6, 3, 3, false, false, false, 17, 8, 1.0)),
                    ("receiver_ahead", (true, false, 6, 3, 3, false, false, true, 17, 8, 1.0)),
                    ("receiver_no_catch", (true, false, 6, 3, 3, false, false, false, 17, 8, 1.0)),
                    ("away_defender", (false, false, 6, 3, 3, false, false, false, 8, 8, 1.0)),
                ],
            ),
            (
                // A LOOSE ball: the pickup branch, with and without Sure Hands, and under a tackle
                // zone so the pickup target is not the bare AG.
                "loose_ball",
                &[
                    (true, 1, 10, 7), (false, 1, 11, 7), (false, 2, 11, 8),
                ],
                Some((10, 8)), true,
                &[
                    ("plain", (true, false, 6, 3, 3, false, false, false, 15, 8, 1.0)),
                    ("sure_hands", (true, false, 6, 3, 3, true, false, false, 15, 8, 1.0)),
                ],
            ),
            (
                // A carrier who CANNOT reach the endzone in the turns left: `HOPELESS_DAMP`. Also
                // Side Step, which changes the sideline penalty from 0.25 to 1.0.
                "hopeless_and_sidestep",
                &[(true, 1, 2, 7), (false, 1, 20, 7)],
                Some((2, 7)), false,
                &[
                    ("hopeless", (true, true, 6, 3, 3, false, false, false, 23, 1, 1.0)),
                    ("hopeless_sidestep", (true, true, 6, 3, 3, false, true, false, 23, 1, 1.0)),
                    ("plenty_of_time", (true, true, 6, 3, 3, false, false, false, 23, 8, 1.0)),
                ],
            ),
            (
                // Mixed strength on the board, so `exposure`'s `strength_factor` is not the
                // identity: an ST 5 threat prices a square very differently for an ST 3 mover than
                // for an ST 5 one.
                "strength_factor",
                // away_03 is ST 3 and stands well clear of the other two, so the squares around
                // HIM carry threat_str 3 -- which is the only way an ST 7 mover can reach the
                // `2 * att < def` branch. With only ST 5 and ST 4 threats on the board that branch
                // is unreachable however strong the mover is.
                &[(true, 1, 10, 7), (false, 1, 12, 7), (false, 2, 12, 9), (false, 3, 4, 3)],
                None, false,
                &[
                    // `strength_factor(att, def)` takes the THREAT as `att` and the MOVER as
                    // `def`, so covering all five branches needs movers on both sides of the
                    // threats: ST 2 against an ST 5 threat gives 1.4, ST 7 against the default
                    // ST 3 gives 0.5, and the middle three fall out of 3 / 5 / equal. A fixture
                    // without the extremes silently skips two branches -- which is exactly what
                    // this one did until perturbing the 0.5 constant failed to break it.
                    // `strength_factor(att, def)` takes the THREAT as `att` and the MOVER as
                    // `def`, so all five branches need movers on BOTH sides of the threats: ST 2
                    // against an ST 5 threat gives 1.4, and ST 7 against the default ST 3 gives
                    // 0.5. Without the extremes the fixture silently skips two branches -- which
                    // is what it did, and perturbing the 0.5 constant proved it by NOT failing.
                    ("st3", (true, false, 6, 3, 3, false, false, false, 15, 8, 1.0)),
                    ("st5", (true, false, 6, 3, 5, false, false, false, 15, 8, 1.0)),
                    ("st1", (true, false, 6, 3, 1, false, false, false, 15, 8, 1.0)),
                    ("st2_vs_st5", (true, false, 6, 3, 2, false, false, false, 15, 8, 1.0)),
                    ("st7_outmuscles", (true, false, 6, 3, 7, false, false, false, 15, 8, 1.0)),
                    ("st2_vs_st5", (true, false, 6, 3, 2, false, false, false, 15, 8, 1.0)),
                    ("st7_outmuscles", (true, false, 6, 3, 7, false, false, false, 15, 8, 1.0)),
                ],
            ),
        ];

        let mut out = String::new();
        writeln!(out, "# value_at golden -- heuristic_agent.rs and ValueModel.java.").unwrap();
        writeln!(out, "# case <name>").unwrap();
        writeln!(out, "# player <home|away> <nr> <x> <y> <st>").unwrap();
        writeln!(out, "# ball <x> <y> <loose>   (absent when there is no ball)").unwrap();
        writeln!(out, "# mover <name> <home> <carrier> <ma> <ag> <str> <sure_hands> <side_step> <catch> <d_now> <turns_left> <unact f32 bits>").unwrap();
        writeln!(out, "# value <f32 bits per cell, 8 hex chars each>").unwrap();
        writeln!(out, "# rule <one char per cell: T=Touchdown A=Advance P=Pickup S=Support ?=other>").unwrap();

        for (name, board, ball, loose, movers) in cases {
            // Give the away side mixed strength on the `strength_factor` board only; elsewhere
            // everyone is ST 3 so the factor is the identity and the other terms are isolated.
            let st_of = |is_home: bool, nr: i32| -> i32 {
                if name == "strength_factor" && !is_home {
                    match nr { 1 => 5, 2 => 4, _ => 3 }
                } else {
                    3
                }
            };
            let mut home = crate::step::framework::test_team("home", 0);
            let mut away = crate::step::framework::test_team("away", 0);
            for &(is_home, nr, _, _) in board {
                let p = Player {
                    id: format!("{}_{:02}", if is_home { "home" } else { "away" }, nr),
                    nr,
                    movement: 6,
                    strength: st_of(is_home, nr),
                    agility: 3,
                    armour: 8,
                    ..Default::default()
                };
                if is_home { home.players.push(p) } else { away.players.push(p) }
            }
            let mut g = Game::new(home, away, Rules::Bb2025);
            for &(is_home, nr, x, y) in board {
                let id = format!("{}_{:02}", if is_home { "home" } else { "away" }, nr);
                g.field_model.set_player_coordinate(&id, FieldCoordinate::new(x, y));
                g.field_model
                    .set_player_state(&id, PlayerState::new(PS_STANDING).change_active(true));
            }
            if let Some((bx, by)) = ball {
                g.field_model.ball_coordinate = Some(FieldCoordinate::new(bx, by));
                g.field_model.ball_in_play = true;
                g.field_model.ball_moving = loose;
            }
            let f = Features::build(&g, positions_stamp(&g), true);

            writeln!(out, "case {name}").unwrap();
            for &(is_home, nr, x, y) in board {
                writeln!(out, "player {} {nr} {x} {y} {}",
                    if is_home { "home" } else { "away" }, st_of(is_home, nr)).unwrap();
            }
            if let Some((bx, by)) = ball {
                writeln!(out, "ball {bx} {by} {loose}").unwrap();
            }

            for &(mname, mv) in movers {
                let (home_, is_carrier, ma, ag, str_, sure_hands, side_step, has_catch, d_now,
                     turns_left, unact) = mv;
                let m = Mover {
                    home: home_,
                    is_carrier,
                    ma,
                    ag,
                    str_,
                    sure_hands,
                    side_step,
                    has_catch,
                    d_now,
                    turns_left,
                    unactivated: unact,
                };
                writeln!(out, "mover {mname} {home_} {is_carrier} {ma} {ag} {str_} {sure_hands} {side_step} {has_catch} {d_now} {turns_left} {:08x}",
                    unact.to_bits()).unwrap();
                let mut vals = String::new();
                let mut rules = String::new();
                for i in 0..CELLS {
                    let (v, rule) = value_at(&f, i, &m);
                    let _ = write!(vals, "{:08x}", v.to_bits());
                    rules.push(match rule {
                        Rule::ScoreTouchdown => 'T',
                        Rule::ScoreAdvance => 'A',
                        Rule::Pickup => 'P',
                        Rule::Support => 'S',
                        _ => '?',
                    });
                }
                writeln!(out, "value {vals}").unwrap();
                writeln!(out, "rule {rules}").unwrap();
            }
        }

        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/src/agent/testdata/value_golden.txt");
        std::fs::write(path, out).unwrap();
        eprintln!("wrote {path}");
    }

    /// `arrival_parts` — where the reach search and the value model are finally multiplied
    /// together — as a cross-language fixture.
    ///
    /// This is the composition step: `w = p·V − (1−p)·c_turnover − rush_penalty`, with a
    /// short-circuit for a carrier arriving IN the endzone (a touchdown ends the drive, so there is
    /// no "after" to lose and only the rush is priced). Both halves are already pinned separately;
    /// what is new is that they are combined the same way, that the GFI count carried out of the
    /// reach search is the one the penalties see, and that the touchdown branch fires on exactly
    /// the right squares.
    ///
    /// `w`, `p_arrive`, `v` and `gfi` are all emitted, not just `w` — three terms summing to the
    /// same total by different routes is precisely the failure a single number hides.
    ///
    /// `cargo test -p ffb-engine --lib agent::heuristic_agent::tests::emit_arrival_golden -- --ignored`
    #[test]
    #[ignore]
    fn emit_arrival_golden() {
        use std::fmt::Write as _;
        use ffb_model::enums::{PlayerState, PS_STANDING, Rules};
        use ffb_model::model::player::Player;

        // (is_home, nr, x, y)
        type Board = &'static [(bool, i32, i32, i32)];
        // (name, is_carrier, ma, ag, str, sure_hands, side_step, has_catch, d_now, turns_left,
        //  unactivated, team_rr)
        type M = (&'static str, bool, i32, i32, i32, bool, bool, bool, i32, i32, f32, bool);

        let cases: [(&str, Board, Option<(i32, i32)>, bool, &[M]); 3] = [
            (
                // The carrier stands 6 squares from the endzone with MA 6, so the touchdown
                // short-circuit is genuinely REACHABLE — the branch is dead in a fixture where the
                // endzone is out of range, which is the easy mistake here.
                "carrier_can_score",
                &[(true, 1, 19, 7), (true, 2, 18, 9), (false, 1, 21, 6), (false, 2, 21, 8)],
                Some((19, 7)), false,
                &[
                    ("carrier", true, 6, 3, 3, false, false, false, 6, 8, 1.0, false),
                    ("carrier_last_turn", true, 6, 3, 3, false, false, false, 6, 1, 1.0, false),
                    // Same board, same reach, but nobody left to activate: `c_turnover` shrinks and
                    // risky arrivals get relatively better.
                    ("carrier_alone", true, 6, 3, 3, false, false, false, 6, 8, 0.0, false),
                ],
            ),
            (
                // Marked on all sides, so every step is a dodge and the far squares are all GFI:
                // `rush_penalty` and the `gfi` factor in `c_turnover` both bite, and they bite four
                // times harder for a non-carrier.
                "gauntlet_rushes",
                &[
                    (true, 1, 10, 7),
                    (false, 1, 11, 7), (false, 2, 11, 8), (false, 3, 10, 6), (false, 4, 9, 8),
                ],
                None, false,
                &[
                    ("noncarrier", false, 6, 3, 3, false, false, false, 15, 8, 1.0, false),
                    ("noncarrier_with_rr", false, 6, 3, 3, false, false, false, 15, 8, 1.0, true),
                ],
            ),
            (
                // A loose ball inside the mover's reach: the pickup value and the arrival
                // probability multiply, which is the whole point of the composition.
                "loose_ball_in_reach",
                &[(true, 1, 10, 7), (false, 1, 14, 7)],
                Some((13, 7)), true,
                &[
                    ("chaser", false, 6, 3, 3, false, false, false, 15, 8, 1.0, false),
                    ("chaser_sure_hands", false, 6, 3, 3, true, false, false, 15, 8, 1.0, false),
                ],
            ),
        ];

        let mut out = String::new();
        writeln!(out, "# arrival_parts golden -- heuristic_agent.rs and Arrival.java.").unwrap();
        writeln!(out, "# case <name>").unwrap();
        writeln!(out, "# player <home|away> <nr> <x> <y>").unwrap();
        writeln!(out, "# ball <x> <y> <loose>   (absent when there is no ball)").unwrap();
        writeln!(out, "# mover <name> <carrier> <ma> <ag> <str> <sure_hands> <side_step> <catch> <d_now> <turns_left> <unact bits> <team_rr>").unwrap();
        writeln!(out, "# w|parrive|v <f32 bits per cell, 8 hex chars each>").unwrap();
        writeln!(out, "# gfi <decimal per cell, comma separated>").unwrap();

        for (name, board, ball, loose, movers) in cases {
            for &(mname, is_carrier, ma, ag, str_, sure_hands, side_step, has_catch, d_now,
                   turns_left, unact, team_rr) in movers
            {
                let mut home = crate::step::framework::test_team("home", 0);
                let mut away = crate::step::framework::test_team("away", 0);
                for &(is_home, nr, _, _) in board {
                    let p = Player {
                        id: format!("{}_{:02}", if is_home { "home" } else { "away" }, nr),
                        nr,
                        // The MOVER's own stats have to match the Mover struct, or the reach search
                        // and the value model would be describing different players.
                        movement: if is_home && nr == 1 { ma } else { 6 },
                        strength: if is_home && nr == 1 { str_ } else { 3 },
                        agility: if is_home && nr == 1 { ag } else { 3 },
                        armour: 8,
                        ..Default::default()
                    };
                    if is_home { home.players.push(p) } else { away.players.push(p) }
                }
                let mut g = Game::new(home, away, Rules::Bb2025);
                for &(is_home, nr, x, y) in board {
                    let id = format!("{}_{:02}", if is_home { "home" } else { "away" }, nr);
                    g.field_model.set_player_coordinate(&id, FieldCoordinate::new(x, y));
                    g.field_model
                        .set_player_state(&id, PlayerState::new(PS_STANDING).change_active(true));
                }
                if let Some((bx, by)) = ball {
                    g.field_model.ball_coordinate = Some(FieldCoordinate::new(bx, by));
                    g.field_model.ball_in_play = true;
                    g.field_model.ball_moving = loose;
                }
                let f = Features::build(&g, positions_stamp(&g), true);
                let mut sc = Scratch::default();
                let b = budget_of(&g, "home_01").expect("the mover is on the pitch");
                let r = reach_with(&f, &g, "home_01", &b, team_rr, &mut sc)
                    .expect("a positive movement cap");
                let m = Mover {
                    home: true,
                    is_carrier,
                    ma,
                    ag,
                    str_,
                    sure_hands,
                    side_step,
                    has_catch,
                    d_now,
                    turns_left,
                    unactivated: unact,
                };

                writeln!(out, "case {name}").unwrap();
                for &(is_home, nr, x, y) in board {
                    writeln!(out, "player {} {nr} {x} {y}",
                        if is_home { "home" } else { "away" }).unwrap();
                }
                if let Some((bx, by)) = ball {
                    writeln!(out, "ball {bx} {by} {loose}").unwrap();
                }
                writeln!(out, "mover {mname} {is_carrier} {ma} {ag} {str_} {sure_hands} {side_step} {has_catch} {d_now} {turns_left} {:08x} {team_rr}",
                    unact.to_bits()).unwrap();

                let (mut ws, mut ps, mut vs) = (String::new(), String::new(), String::new());
                let mut gs: Vec<String> = Vec::with_capacity(CELLS);
                for i in 0..CELLS {
                    let a = arrival_parts(&f, &r, i, &m);
                    let _ = write!(ws, "{:08x}", a.w.to_bits());
                    let _ = write!(ps, "{:08x}", a.p_arrive.to_bits());
                    let _ = write!(vs, "{:08x}", a.v.to_bits());
                    gs.push(a.gfi.to_string());
                }
                writeln!(out, "w {ws}").unwrap();
                writeln!(out, "parrive {ps}").unwrap();
                writeln!(out, "v {vs}").unwrap();
                writeln!(out, "gfi {}", gs.join(",")).unwrap();
            }
        }

        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/src/agent/testdata/arrival_golden.txt");
        std::fs::write(path, out).unwrap();
        eprintln!("wrote {path}");
    }

    /// The destination ORDERINGS `build_plans` enumerates from — `top_moves` and
    /// `run_up_squares` — as a cross-language fixture.
    ///
    /// The arrival weights themselves are already pinned (ITER28). What is not, and what nothing
    /// downstream can survive getting wrong, is the ORDER: the agent samples an INDEX into this
    /// list, so two implementations that agree on every weight and disagree on one comparison pick
    /// different squares. Three things have to match:
    ///
    /// 1. **descending by weight, ascending by cell index on ties.** Ties are not hypothetical —
    ///    every square a plain `Move` cannot improve on scores identically.
    /// 2. **which cells are in the list at all**, which is `Reach::order` and therefore the visit
    ///    set, not the whole pitch.
    /// 3. **`run_up_squares` puts the mover's CURRENT square first, unconditionally**, so "use
    ///    none of my move" is never lost, and then appends at most `THROW_SPOTS` others by a
    ///    different metric (arrival probability weighted by forward progress, NOT arrival weight).
    ///    Two different orderings in the same function is exactly the kind of thing a port
    ///    collapses into one.
    ///
    /// The golden stores the full ordered lists, so a single transposed pair fails with both
    /// indices named.
    ///
    /// `cargo test -p ffb-engine --lib agent::heuristic_agent::tests::emit_plans_golden -- --ignored`
    #[test]
    #[ignore]
    fn emit_plans_golden() {
        use std::fmt::Write as _;
        use ffb_model::enums::{PlayerState, PS_STANDING, Rules};
        use ffb_model::model::player::Player;

        type Board = &'static [(bool, i32, i32, i32)];
        // (name, is_carrier, ma, ag, str, d_now, turns_left, unactivated, team_rr)
        type M = (&'static str, bool, i32, i32, i32, i32, i32, f32, bool);

        let cases: [(&str, Board, Option<(i32, i32)>, bool, &[M]); 3] = [
            (
                // Open field: most reachable squares score identically, so the tie-break is doing
                // nearly all the ordering work. If it differs, this is where it shows.
                "open_field_ties",
                &[(true, 1, 8, 7), (false, 1, 20, 7)],
                None, false,
                &[
                    ("plain", false, 6, 3, 3, 17, 8, 1.0, false),
                    ("carrier", true, 6, 3, 3, 17, 8, 1.0, false),
                ],
            ),
            (
                // A carrier who can reach the endzone: the ordering has to put the scoring squares
                // on top, and `run_up_squares` weights forward progress rather than weight.
                "carrier_near_endzone",
                &[(true, 1, 19, 7), (true, 2, 18, 9), (false, 1, 21, 6), (false, 2, 21, 8)],
                Some((19, 7)), false,
                &[("carrier", true, 6, 3, 3, 6, 8, 1.0, false)],
            ),
            (
                // Marked on four sides: many squares are unreachable, so the LIST MEMBERSHIP is as
                // much of the contract as the order.
                "gauntlet",
                &[
                    (true, 1, 10, 7),
                    (false, 1, 11, 7), (false, 2, 11, 8), (false, 3, 10, 6), (false, 4, 9, 8),
                ],
                None, false,
                &[("plain", false, 6, 3, 3, 15, 8, 1.0, false)],
            ),
        ];

        let mut out = String::new();
        writeln!(out, "# top_moves / run_up_squares golden -- heuristic_agent.rs and Plans.java.").unwrap();
        writeln!(out, "# case <name>").unwrap();
        writeln!(out, "# player <home|away> <nr> <x> <y>").unwrap();
        writeln!(out, "# ball <x> <y> <loose>").unwrap();
        writeln!(out, "# mover <name> <carrier> <ma> <ag> <str> <d_now> <turns_left> <unact bits> <team_rr>").unwrap();
        writeln!(out, "# topmoves <cell index:weight bits, comma separated, IN ORDER>").unwrap();
        writeln!(out, "# runup <cell indices, comma separated, IN ORDER>").unwrap();
        writeln!(out, "# risked <w bits>:<p bits>:<result bits> for a few probe pairs").unwrap();
        writeln!(out, "# proxy <f32 bits>   the search-free tier-1 estimate").unwrap();

        for (name, board, ball, loose, movers) in cases {
            for &(mname, is_carrier, ma, ag, str_, d_now, turns_left, unact, team_rr) in movers {
                let mut home = crate::step::framework::test_team("home", 0);
                let mut away = crate::step::framework::test_team("away", 0);
                for &(is_home, nr, _, _) in board {
                    let p = Player {
                        id: format!("{}_{:02}", if is_home { "home" } else { "away" }, nr),
                        nr,
                        movement: if is_home && nr == 1 { ma } else { 6 },
                        strength: if is_home && nr == 1 { str_ } else { 3 },
                        agility: if is_home && nr == 1 { ag } else { 3 },
                        armour: 8,
                        ..Default::default()
                    };
                    if is_home { home.players.push(p) } else { away.players.push(p) }
                }
                let mut g = Game::new(home, away, Rules::Bb2025);
                for &(is_home, nr, x, y) in board {
                    let id = format!("{}_{:02}", if is_home { "home" } else { "away" }, nr);
                    g.field_model.set_player_coordinate(&id, FieldCoordinate::new(x, y));
                    g.field_model
                        .set_player_state(&id, PlayerState::new(PS_STANDING).change_active(true));
                }
                if let Some((bx, by)) = ball {
                    g.field_model.ball_coordinate = Some(FieldCoordinate::new(bx, by));
                    g.field_model.ball_in_play = true;
                    g.field_model.ball_moving = loose;
                }
                let f = Features::build(&g, positions_stamp(&g), true);
                let mut sc = Scratch::default();
                let b = budget_of(&g, "home_01").expect("the mover is on the pitch");
                let r = reach_with(&f, &g, "home_01", &b, team_rr, &mut sc)
                    .expect("a positive movement cap");
                let m = Mover {
                    home: true,
                    is_carrier,
                    ma,
                    ag,
                    str_,
                    sure_hands: false,
                    side_step: false,
                    has_catch: false,
                    d_now,
                    turns_left,
                    unactivated: unact,
                };
                let here = m_coord(&g, "home_01");

                writeln!(out, "case {name}").unwrap();
                for &(is_home, nr, x, y) in board {
                    writeln!(out, "player {} {nr} {x} {y}",
                        if is_home { "home" } else { "away" }).unwrap();
                }
                if let Some((bx, by)) = ball {
                    writeln!(out, "ball {bx} {by} {loose}").unwrap();
                }
                writeln!(out, "mover {mname} {is_carrier} {ma} {ag} {str_} {d_now} {turns_left} {:08x} {team_rr}",
                    unact.to_bits()).unwrap();

                let tops = top_moves(&f, &r, &m, usize::MAX);
                let t: Vec<String> =
                    tops.iter().map(|(w, i)| format!("{i}:{:08x}", w.to_bits())).collect();
                writeln!(out, "topmoves {}", t.join(",")).unwrap();

                let ru = run_up_squares(Some(&r), &m, here);
                let rs: Vec<String> = ru.iter().map(|i| i.to_string()).collect();
                writeln!(out, "runup {}", rs.join(",")).unwrap();

                // `risked` is a two-line function, but it is NOT `w * p` and the difference only
                // shows on a NEGATIVE weight -- so probe both signs explicitly.
                let probes: [(f32, f32); 4] =
                    [(0.8, 1.0), (0.8, 0.4), (-0.5, 0.9), (-0.5, 0.25)];
                let rr: Vec<String> = probes
                    .iter()
                    .map(|&(w, p)| {
                        format!("{:08x}:{:08x}:{:08x}",
                            w.to_bits(), p.to_bits(), risked(w, p, &m).to_bits())
                    })
                    .collect();
                writeln!(out, "risked {}", rr.join(",")).unwrap();

                // `proxy_value` is the §20.3 tier-1 stand-in: no Dijkstra at all, just the eight
                // adjacent squares exactly plus an admissible CEILING over everything inside MA+2,
                // read straight off the rasters and then discounted because it is optimistic by
                // construction. It is what every player the search did not run for is scored with,
                // so a disagreement here reorders the activation queue without touching a single
                // move.
                writeln!(out, "proxy {:08x}", proxy_value(&f, &g, "home_01", &m).to_bits())
                    .unwrap();
            }
        }

        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/src/agent/testdata/plans_golden.txt");
        std::fs::write(path, out).unwrap();
        eprintln!("wrote {path}");
    }

    /// `receiver_of`, `handoff_weight` and `foul_weight` — the ball-move and foul prices — as a
    /// cross-language fixture.
    ///
    /// `receiver_of` is the biggest single function in the plan layer and the one with the most
    /// ways to be subtly wrong. It answers "what happens if this player ends up with the ball", and
    /// the answer turns on four things that are easy to get one step out:
    ///
    /// - **`active`** — a receiver who still holds his activation can catch AND run this turn, and
    ///   one who does not has `reach_after = 0` and a turn fewer. That is the difference between a
    ///   touchdown and a token positional credit.
    /// - **`effective_d`** — where the BALL ends up, not where the receiver stands: `d_rcv` minus
    ///   what he can still cover. Using `d_rcv` alone silently prices every give as though the
    ///   receiver never moved.
    /// - **`scores_now`** — and its exclusion when the CARRIER could score by himself, which stops
    ///   the agent throwing away a run it was already going to make.
    /// - **`p_run_in`** — a five-way ladder on the receiver's own MA, with a GFI factor per rush.
    ///
    /// `foul_weight` is included because it reads the engine's real assist counts
    /// (`find_offensive_foul_assists` / `find_defensive_foul_assists`), which both sides must call
    /// rather than re-derive, and because its `victim` term is a three-way branch on where the ball
    /// is.
    ///
    /// `cargo test -p ffb-engine --lib agent::heuristic_agent::tests::emit_ballmoves_golden -- --ignored`
    #[test]
    #[ignore]
    fn emit_ballmoves_golden() {
        use std::fmt::Write as _;
        use ffb_model::enums::{PlayerState, PS_STANDING, PS_PRONE, Rules};
        use ffb_model::model::player::Player;

        // (is_home, nr, x, y, standing, active, ma, ag, st)
        type P2 = (bool, i32, i32, i32, bool, bool, i32, i32, i32);
        type Board = &'static [P2];

        // (name, board, ball, turn_nr, carrier d_now, turns_left, unactivated)
        let cases: [(&str, Board, Option<(i32, i32)>, i32, i32, i32, f32); 4] = [
            (
                // The give that scores: an ACTIVE receiver close enough to run it in, while the
                // carrier himself cannot. Both halves of `scores_now` matter here.
                "give_that_scores",
                &[
                    (true, 1, 14, 7, true, true, 6, 3, 3),
                    (true, 2, 20, 7, true, true, 6, 3, 3),
                    (false, 1, 15, 6, true, true, 6, 3, 3),
                ],
                Some((14, 7)), 3, 11, 5, 1.0,
            ),
            (
                // Same board, but the receiver has ALREADY acted: reach_after collapses to 0, he
                // gets a turn fewer, and the whole thing is worth a token credit instead.
                "receiver_already_acted",
                &[
                    (true, 1, 14, 7, true, true, 6, 3, 3),
                    (true, 2, 20, 7, true, false, 6, 3, 3),
                    (false, 1, 15, 6, true, true, 6, 3, 3),
                ],
                Some((14, 7)), 3, 11, 5, 1.0,
            ),
            (
                // The carrier can score by HIMSELF, so `scores_now` must be false for the receiver
                // however good he looks -- otherwise the agent gives away a run it had already won.
                "carrier_can_score_himself",
                &[
                    (true, 1, 20, 7, true, true, 6, 3, 3),
                    (true, 2, 22, 9, true, true, 6, 3, 3),
                    (false, 1, 21, 5, true, true, 6, 3, 3),
                ],
                Some((20, 7)), 3, 5, 5, 1.0,
            ),
            (
                // Fouls: a PRONE victim next to the ball, one carrying it, and one far away --
                // the three branches of the `victim` term -- with assists on both sides.
                "foul_targets",
                &[
                    (true, 1, 10, 7, true, true, 6, 3, 3),
                    (true, 2, 10, 8, true, true, 6, 3, 3),
                    (true, 3, 9, 6, true, true, 6, 3, 3),
                    (false, 1, 11, 7, false, true, 6, 3, 3),
                    (false, 2, 11, 8, false, true, 6, 3, 3),
                    (false, 3, 20, 2, false, true, 6, 3, 3),
                ],
                Some((11, 7)), 3, 15, 5, 1.0,
            ),
        ];

        let mut out = String::new();
        writeln!(out, "# receiver_of / handoff_weight / foul_weight golden.").unwrap();
        writeln!(out, "# case <name> <turn_nr> <d_now> <turns_left> <unact bits>").unwrap();
        writeln!(out, "# player <home|away> <nr> <x> <y> <standing|prone> <active|used> <ma> <ag> <st>").unwrap();
        writeln!(out, "# ball <x> <y>   (carried)").unwrap();
        writeln!(out, "# receiver <rcv nr> <from x> <from y> <pcatch>:<v>:<scoresnow>:<tts>:<turns>").unwrap();
        writeln!(out, "# handoff <rcv nr> <from x> <from y> <f32 bits, or - when None>").unwrap();
        writeln!(out, "# foul <def nr> <av> <off assists> <def assists> <f32 bits>").unwrap();

        for (name, board, ball, turn_nr, d_now, turns_left, unact) in cases {
            let mut home = crate::step::framework::test_team("home", 0);
            let mut away = crate::step::framework::test_team("away", 0);
            for &(is_home, nr, _, _, _, _, ma, ag, st) in board {
                let p = Player {
                    id: format!("{}_{:02}", if is_home { "home" } else { "away" }, nr),
                    nr,
                    movement: ma,
                    strength: st,
                    agility: ag,
                    armour: 8,
                    ..Default::default()
                };
                if is_home { home.players.push(p) } else { away.players.push(p) }
            }
            let mut g = Game::new(home, away, Rules::Bb2025);
            g.turn_data_home.turn_nr = turn_nr;
            g.turn_data_away.turn_nr = turn_nr;
            for &(is_home, nr, x, y, standing, active, _, _, _) in board {
                let id = format!("{}_{:02}", if is_home { "home" } else { "away" }, nr);
                g.field_model.set_player_coordinate(&id, FieldCoordinate::new(x, y));
                g.field_model.set_player_state(
                    &id,
                    PlayerState::new(if standing { PS_STANDING } else { PS_PRONE })
                        .change_active(active),
                );
            }
            if let Some((bx, by)) = ball {
                g.field_model.ball_coordinate = Some(FieldCoordinate::new(bx, by));
                g.field_model.ball_in_play = true;
                g.field_model.ball_moving = false;
            }
            let f = Features::build(&g, positions_stamp(&g), true);
            let m = Mover {
                home: true,
                is_carrier: true,
                ma: 6,
                ag: 3,
                str_: 3,
                sure_hands: false,
                side_step: false,
                has_catch: false,
                d_now,
                turns_left,
                unactivated: unact,
            };

            writeln!(out, "case {name} {turn_nr} {d_now} {turns_left} {:08x}", unact.to_bits())
                .unwrap();
            for &(is_home, nr, x, y, standing, active, ma, ag, st) in board {
                writeln!(out, "player {} {nr} {x} {y} {} {} {ma} {ag} {st}",
                    if is_home { "home" } else { "away" },
                    if standing { "standing" } else { "prone" },
                    if active { "active" } else { "used" }).unwrap();
            }
            if let Some((bx, by)) = ball {
                writeln!(out, "ball {bx} {by}").unwrap();
            }

            // Every home team-mate, from a couple of throwing squares, so `from` varies too.
            let here = m_coord(&g, "home_01");
            let froms = [here, FieldCoordinate::new(here.x + 1, here.y),
                         FieldCoordinate::new(here.x, here.y + 1)];
            for &(is_home, nr, _, _, _, _, _, _, _) in board {
                if !is_home || nr == 1 {
                    continue;
                }
                let rcv = format!("home_{nr:02}");
                for from in froms {
                    if !on_pitch(from.x, from.y) {
                        continue;
                    }
                    if let Some(r) = receiver_of(&f, &g, &rcv, from, &m) {
                        writeln!(out, "receiver {nr} {} {} {:08x}:{:08x}:{}:{}:{}",
                            from.x, from.y, r.p_catch.to_bits(), r.v.to_bits(),
                            r.scores_now, r.tts, r.turns).unwrap();
                    }
                    let hw = handoff_weight(&f, &g, from, &rcv, &m);
                    writeln!(out, "handoff {nr} {} {} {}", from.x, from.y,
                        match hw { Some(w) => format!("{:08x}", w.to_bits()),
                                   None => "-".to_string() }).unwrap();
                }
            }
            for &(is_home, nr, _, _, _, _, _, _, _) in board {
                if is_home {
                    continue;
                }
                let def = format!("away_{nr:02}");
                let w = foul_weight(&f, &g, "home_01", &def, &m);
                // The assist counts come from the ENGINE (`UtilPlayer`), and the Java mirror calls
                // the same method in production -- but the FIXTURE feeds them in, so what it pins
                // is the arithmetic on top rather than a second copy of the assist rules. Same
                // split as `Features::build` taking a snapshot instead of a Game.
                let off = ffb_model::util::util_player::UtilPlayer::find_offensive_foul_assists(
                    &g, "home_01", &def) as i32;
                let dfn = ffb_model::util::util_player::UtilPlayer::find_defensive_foul_assists(
                    &g, "home_01", &def) as i32;
                let av = g.player(&def).map(|q| q.armour_with_modifiers()).unwrap_or(8);
                writeln!(out, "foul {nr} {av} {off} {dfn} {:08x}", w.to_bits()).unwrap();
            }
        }

        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/src/agent/testdata/ballmoves_golden.txt");
        std::fs::write(path, out).unwrap();
        eprintln!("wrote {path}");
    }

    /// `pass_weight` — the last plan price — as a cross-language fixture.
    ///
    /// A fumble is a turnover on the spot, so a pass has to be an EXPECTATION rather than a
    /// preference, and the three outcomes are priced separately:
    ///
    /// ```text
    /// p_complete = p_accurate * p_catch
    /// p_lost     = p_fumble + p_scatter * 0.45 + p_accurate * (1 - p_catch)
    /// w          = p_complete * v - p_lost * risk
    /// ```
    ///
    /// The 0.45 on a scatter is the point: **a scattered ball is not a turnover.** It lands three
    /// squares away and either side may reach it, so pricing a scatter like a fumble makes the
    /// agent refuse every pass it should be making. That is the single most consequential constant
    /// in this function.
    ///
    /// The six-face loop is also load-bearing: it asks the ENGINE'S OWN grader which faces are
    /// ACCURATE and which FUMBLE, rather than deriving them from the target number, because the
    /// two are not the same question — a 1 fumbles regardless of target, and the accurate band
    /// depends on the edition.
    ///
    /// The golden carries `n_accurate` / `n_fumble` / the passing distance as INPUTS, the same
    /// split used for `Features` (a snapshot, not a Game) and for foul assists: those come from
    /// mechanics both engines already share and the parity matrix already covers, so what needs
    /// pinning here is the arithmetic on top of them.
    ///
    /// `cargo test -p ffb-engine --lib agent::heuristic_agent::tests::emit_pass_golden -- --ignored`
    #[test]
    #[ignore]
    fn emit_pass_golden() {
        use std::fmt::Write as _;
        use ffb_model::enums::{PlayerState, PS_STANDING, PS_PRONE, Rules, Weather};
        use ffb_model::model::player::Player;

        type P2 = (bool, i32, i32, i32, bool, i32, i32, i32);
        type Board = &'static [P2];

        // (name, rules, weather, board, ball, turn_nr, d_now, turns_left, unact)
        let cases: [(&str, Rules, Weather, Board, (i32, i32), i32, i32, i32, f32); 4] = [
            (
                // Short pass to an unmarked receiver who can run it in: the case a pass exists for.
                "short_to_scorer", Rules::Bb2025, Weather::Nice,
                &[
                    (true, 1, 12, 7, true, 6, 3, 3),
                    (true, 2, 18, 7, true, 6, 3, 3),
                    (false, 1, 13, 9, true, 6, 3, 3),
                ],
                (12, 7), 3, 13, 5, 1.0,
            ),
            (
                // The thrower is MARKED: `tz_on_thrower` shifts the effective roll, which changes
                // the accurate/fumble split rather than the target -- easy to apply to the wrong
                // side of the comparison.
                "thrower_marked", Rules::Bb2025, Weather::Nice,
                &[
                    (true, 1, 12, 7, true, 6, 3, 3),
                    (true, 2, 18, 7, true, 6, 3, 3),
                    (false, 1, 11, 7, true, 6, 3, 3),
                    (false, 2, 12, 6, true, 6, 3, 3),
                ],
                (12, 7), 3, 13, 5, 1.0,
            ),
            (
                // A BLIZZARD rules Long and LongBomb out entirely, so some throws stop being legal
                // at all -- `None`, and the option must not exist.
                "blizzard_long", Rules::Bb2025, Weather::Blizzard,
                &[
                    (true, 1, 4, 7, true, 6, 3, 3),
                    (true, 2, 20, 7, true, 6, 3, 3),
                    (true, 3, 9, 7, true, 6, 3, 3),
                ],
                (4, 7), 3, 21, 5, 1.0,
            ),
            (
                // BB2016 grades passes on a different table entirely.
                "bb2016", Rules::Bb2016, Weather::Nice,
                &[
                    (true, 1, 12, 7, true, 6, 3, 3),
                    (true, 2, 18, 7, true, 6, 3, 3),
                    (true, 3, 14, 9, true, 6, 3, 3),
                ],
                (12, 7), 3, 13, 5, 1.0,
            ),
        ];

        let mut out = String::new();
        writeln!(out, "# pass_weight golden -- heuristic_agent.rs and BallMoves.passWeight.").unwrap();
        writeln!(out, "# case <name> <rules> <weather> <turn_nr> <d_now> <turns_left> <unact bits>").unwrap();
        writeln!(out, "# player <home|away> <nr> <x> <y> <standing|prone> <ma> <ag> <st>").unwrap();
        writeln!(out, "# ball <x> <y>").unwrap();
        writeln!(out, "# pass <rcv nr> <from x> <from y> <dist|NONE> <n_accurate> <n_fumble> <tz_thrower> <w bits|->").unwrap();

        for (name, rules, weather, board, ball, turn_nr, d_now, turns_left, unact) in cases {
            let mut home = crate::step::framework::test_team("home", 0);
            let mut away = crate::step::framework::test_team("away", 0);
            for &(is_home, nr, _, _, _, ma, ag, st) in board {
                let p = Player {
                    id: format!("{}_{:02}", if is_home { "home" } else { "away" }, nr),
                    nr,
                    movement: ma,
                    strength: st,
                    agility: ag,
                    passing: 4,
                    armour: 8,
                    ..Default::default()
                };
                if is_home { home.players.push(p) } else { away.players.push(p) }
            }
            let mut g = Game::new(home, away, rules);
            g.field_model.weather = weather;
            g.turn_data_home.turn_nr = turn_nr;
            g.turn_data_away.turn_nr = turn_nr;
            for &(is_home, nr, x, y, standing, _, _, _) in board {
                let id = format!("{}_{:02}", if is_home { "home" } else { "away" }, nr);
                g.field_model.set_player_coordinate(&id, FieldCoordinate::new(x, y));
                g.field_model.set_player_state(
                    &id,
                    PlayerState::new(if standing { PS_STANDING } else { PS_PRONE })
                        .change_active(true),
                );
            }
            g.field_model.ball_coordinate = Some(FieldCoordinate::new(ball.0, ball.1));
            g.field_model.ball_in_play = true;
            g.field_model.ball_moving = false;
            let f = Features::build(&g, positions_stamp(&g), true);
            let m = Mover {
                home: true,
                is_carrier: true,
                ma: 6,
                ag: 3,
                str_: 3,
                sure_hands: false,
                side_step: false,
                has_catch: false,
                d_now,
                turns_left,
                unactivated: unact,
            };

            writeln!(out, "case {name} {rules:?} {weather:?} {turn_nr} {d_now} {turns_left} {:08x}",
                unact.to_bits()).unwrap();
            for &(is_home, nr, x, y, standing, ma, ag, st) in board {
                writeln!(out, "player {} {nr} {x} {y} {} {ma} {ag} {st}",
                    if is_home { "home" } else { "away" },
                    if standing { "standing" } else { "prone" }).unwrap();
            }
            writeln!(out, "ball {} {}", ball.0, ball.1).unwrap();

            let here = m_coord(&g, "home_01");
            let froms = [here, FieldCoordinate::new(here.x + 1, here.y)];
            let mech = crate::mechanic::pass_mechanic_for(g.rules);
            let thrower = g.player("home_01").expect("thrower");
            for &(is_home, nr, _, _, _, _, _, _) in board {
                if !is_home || nr == 1 {
                    continue;
                }
                let rcv = format!("home_{nr:02}");
                let rc = match g.field_model.player_coordinate(&rcv) {
                    Some(c) => c,
                    None => continue,
                };
                for from in froms {
                    if !on_pitch(from.x, from.y) {
                        continue;
                    }
                    let dist = mech.find_passing_distance(&g, Some(from), Some(rc), false);
                    let tz = f.tz[side_idx(m.home)][ixc(from)] as i32;
                    let (dname, na, nf) = match dist {
                        Some(d) => {
                            let mut na = 0;
                            let mut nf = 0;
                            for die in 1..=6i32 {
                                match mech.evaluate_pass_simple(thrower, die - tz, d, &[], false) {
                                    ffb_mechanics::pass_result::PassResult::ACCURATE => na += 1,
                                    ffb_mechanics::pass_result::PassResult::FUMBLE => nf += 1,
                                    _ => {}
                                }
                            }
                            (format!("{d:?}"), na, nf)
                        }
                        None => ("NONE".to_string(), 0, 0),
                    };
                    let w = pass_weight(&f, &g, "home_01", from, &rcv, &m);
                    writeln!(out, "pass {nr} {} {} {dname} {na} {nf} {tz} {}",
                        from.x, from.y,
                        match w { Some(w) => format!("{:08x}", w.to_bits()),
                                  None => "-".to_string() }).unwrap();
                }
            }
        }

        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/src/agent/testdata/pass_golden.txt");
        std::fs::write(path, out).unwrap();
        eprintln!("wrote {path}");
    }

    /// The tier-1 activation ranking — `w_player`, and the order it produces — as a fixture.
    ///
    /// Before any search runs, every eligible player gets a search-free score and the top `TIER2`
    /// of them are the ONLY ones that get a Dijkstra. So this ladder decides not just who is
    /// likeliest to be picked but **who is even considered properly**, and a disagreement here
    /// cannot be seen as a wrong move — it shows up as the right move by the wrong player, or as a
    /// player who was never scored at all.
    ///
    /// The ladder is a chain of `else if`s, so ORDER matters as much as the constants: a marked
    /// carrier is 0.95 and an unmarked one 0.88, but "can fetch a loose ball" sits BETWEEN them at
    /// 0.92 — testing that needs a board where the same player is both, which is what
    /// `carrier_and_fetcher` is for. The negatrait multiplier is applied AFTER the ladder and the
    /// `awaiting_run` override AFTER that, overwriting rather than scaling.
    ///
    /// The rank comparator is `w_player * max(proxy, 0.05)` descending, `canon_key` ascending on
    /// ties — the floor on `proxy` is what stops a player with proxy 0 from being unrankable.
    ///
    /// `cargo test -p ffb-engine --lib agent::heuristic_agent::tests::emit_activate_golden -- --ignored`
    #[test]
    #[ignore]
    fn emit_activate_golden() {
        use std::fmt::Write as _;
        use ffb_model::enums::{PlayerState, PS_STANDING, PS_PRONE, Rules};
        use ffb_model::model::player::Player;

        // (is_home, nr, x, y, standing, negatrait)
        type P2 = (bool, i32, i32, i32, bool, bool);
        type Board = &'static [P2];

        // (name, board, ball, ball_loose, turn_nr, awaiting_run nr or 0)
        let cases: [(&str, Board, Option<(i32, i32)>, bool, i32, i32); 6] = [
            (
                // Plain: nobody carries, nobody is marked. Every home player lands on the same two
                // bottom rungs, so the ORDER is decided entirely by proxy and the canonical tie.
                "flat_field",
                &[
                    (true, 1, 5, 5, true, false), (true, 2, 5, 7, true, false),
                    (true, 3, 5, 9, true, false), (false, 1, 20, 7, true, false),
                ],
                None, false, 3, 0,
            ),
            (
                // A carrier, MARKED: the top rung (0.95). The same board also has an unmarked
                // team-mate, so 0.95 and the lower rungs are compared directly.
                "marked_carrier",
                &[
                    (true, 1, 12, 7, true, false), (true, 2, 6, 3, true, false),
                    (false, 1, 13, 7, true, false),
                ],
                Some((12, 7)), false, 3, 0,
            ),
            (
                // A LOOSE ball within MA+2 of one player and not the other: the 0.92 rung, which
                // sits between the marked and unmarked carrier rungs and is therefore the one most
                // easily reordered by a port that flattens the chain.
                "carrier_and_fetcher",
                &[
                    (true, 1, 10, 7, true, false), (true, 2, 22, 12, true, false),
                    (false, 1, 20, 2, true, false),
                ],
                Some((12, 7)), true, 3, 0,
            ),
            (
                // PRONE and marked (0.70), plus a negatrait player whose whole score is multiplied
                // by 0.55 AFTER the ladder -- two effects that a port can easily merge.
                "prone_and_negatrait",
                &[
                    (true, 1, 10, 7, false, false), (true, 2, 14, 3, true, true),
                    (false, 1, 11, 7, true, false), (false, 2, 15, 3, true, false),
                ],
                None, false, 3, 0,
            ),
            (
                // A prone, MARKED player who ALSO has proxy > 0.25 -- the only way to observe the
                // order of those two rungs, since they are the only adjacent pair in the chain
                // that can both hold at once. He stands next to our own carrier, so the cage term
                // lifts the support raster around him and the discounted ceiling clears 0.25.
                //
                // Found because the bite-check did NOT fail when the rungs were swapped: a prone
                // marked player is normally hemmed in, so his proxy is low and the chain never has
                // to choose. Note the two rungs above are mutually EXCLUSIVE by construction --
                // `can_fetch` needs a loose ball and `is_carrier` needs a carried one -- so their
                // relative order carries no meaning and no board can pin it.
                "prone_marked_with_support",
                &[
                    (true, 1, 12, 7, true, false),
                    (true, 2, 12, 8, false, false),
                    (true, 3, 11, 7, true, false),
                    (false, 1, 13, 8, true, false),
                    (false, 2, 11, 9, true, false),
                ],
                Some((12, 7)), false, 3, 0,
            ),
            (
                // `awaiting_run`: the player who was just thrown the ball is forced to 1.0,
                // OVERWRITING the ladder and the negatrait multiplier both.
                "awaiting_run_overrides",
                &[
                    (true, 1, 10, 7, true, true), (true, 2, 16, 7, true, true),
                    (false, 1, 20, 7, true, false),
                ],
                None, false, 3, 2,
            ),
        ];

        let mut out = String::new();
        writeln!(out, "# tier-1 activation ranking golden -- heuristic_agent.rs handle_activate.").unwrap();
        writeln!(out, "# case <name> <turn_nr> <awaiting nr, 0 = none>").unwrap();
        writeln!(out, "# player <home|away> <nr> <x> <y> <standing|prone> <negatrait>").unwrap();
        writeln!(out, "# ball <x> <y> <loose>").unwrap();
        writeln!(out, "# cand <nr> <w_player bits> <proxy bits>").unwrap();
        writeln!(out, "# rank <nrs in ranked order, comma separated>").unwrap();

        for (name, board, ball, loose, turn_nr, awaiting) in cases {
            let mut home = crate::step::framework::test_team("home", 0);
            let mut away = crate::step::framework::test_team("away", 0);
            for &(is_home, nr, _, _, _, negatrait) in board {
                let mut p = Player {
                    id: format!("{}_{:02}", if is_home { "home" } else { "away" }, nr),
                    nr,
                    movement: 6,
                    strength: 3,
                    agility: 3,
                    armour: 8,
                    ..Default::default()
                };
                if negatrait {
                    p.starting_skills.push(ffb_model::model::skill_def::SkillWithValue {
                        skill_id: SkillId::BoneHead,
                        value: None,
                    });
                }
                if is_home { home.players.push(p) } else { away.players.push(p) }
            }
            let mut g = Game::new(home, away, Rules::Bb2025);
            g.home_playing = true;
            g.turn_data_home.turn_nr = turn_nr;
            g.turn_data_away.turn_nr = turn_nr;
            for &(is_home, nr, x, y, standing, _) in board {
                let id = format!("{}_{:02}", if is_home { "home" } else { "away" }, nr);
                g.field_model.set_player_coordinate(&id, FieldCoordinate::new(x, y));
                g.field_model.set_player_state(
                    &id,
                    PlayerState::new(if standing { PS_STANDING } else { PS_PRONE })
                        .change_active(true),
                );
            }
            if let Some((bx, by)) = ball {
                g.field_model.ball_coordinate = Some(FieldCoordinate::new(bx, by));
                g.field_model.ball_in_play = true;
                g.field_model.ball_moving = loose;
            }
            let f = Features::build(&g, positions_stamp(&g), true);
            let agent = HeuristicAgent::new(1, 1.0);

            writeln!(out, "case {name} {turn_nr} {awaiting}").unwrap();
            for &(is_home, nr, x, y, standing, negatrait) in board {
                writeln!(out, "player {} {nr} {x} {y} {} {negatrait}",
                    if is_home { "home" } else { "away" },
                    if standing { "standing" } else { "prone" }).unwrap();
            }
            if let Some((bx, by)) = ball {
                writeln!(out, "ball {bx} {by} {loose}").unwrap();
            }

            // Recompute the tier-1 score exactly as `handle_activate` does, for every home player.
            let mut scored: Vec<(i32, f32, f32)> = Vec::new();
            for &(is_home, nr, _, _, _, _) in board {
                if !is_home {
                    continue;
                }
                let pid = format!("home_{nr:02}");
                let st = g.field_model.player_state(&pid).expect("state");
                let m = agent.mover_of(&g, &f, &pid).expect("mover");
                let c = g.field_model.player_coordinate(&pid).expect("coord");
                let proxy = proxy_value(&f, &g, &pid, &m);
                let marked = f.tz[side_idx(true)][ixc(c)] > 0;
                let can_fetch = f.ball_loose
                    && f.ball.map(|b| c.distance_in_steps(b) <= m.ma + 2).unwrap_or(false);
                let mut w_player: f32 = if m.is_carrier && marked {
                    0.95
                } else if can_fetch {
                    0.92
                } else if m.is_carrier {
                    0.88
                } else if st.is_prone() && marked {
                    0.70
                } else if proxy > 0.25 {
                    0.45
                } else {
                    0.30
                };
                if g.player(&pid).map(has_negatrait).unwrap_or(false) {
                    w_player *= 0.55;
                }
                if awaiting == nr {
                    w_player = 1.0;
                }
                writeln!(out, "cand {nr} {:08x} {:08x}", w_player.to_bits(), proxy.to_bits())
                    .unwrap();
                scored.push((nr, w_player, proxy));
            }
            // Canonical order first, then rank -- so a tie can never depend on hash order.
            scored.sort_by_key(|(nr, _, _)| *nr);
            let mut rank: Vec<usize> = (0..scored.len()).collect();
            rank.sort_by(|&a, &b| {
                (scored[b].1 * scored[b].2.max(0.05))
                    .partial_cmp(&(scored[a].1 * scored[a].2.max(0.05)))
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then(scored[a].0.cmp(&scored[b].0))
            });
            let order: Vec<String> =
                rank.iter().map(|&i| scored[i].0.to_string()).collect();
            writeln!(out, "rank {}", order.join(",")).unwrap();
        }

        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/src/agent/testdata/activate_golden.txt");
        std::fs::write(path, out).unwrap();
        eprintln!("wrote {path}");
    }

    /// The two-level activation draw — declaration grouping plus the nested softmax — as a
    /// fixture.
    ///
    /// The agent does NOT sample flatly over candidates. It groups them by DECLARATION (the
    /// `(player, action)` pair the engine actually receives), scores each group by its best child,
    /// samples a group at `T = 0.18` and then a child within it at `T = 0.10`. The reason is in the
    /// shapes: a Move declaration can carry two thousand destinations and a Block nine, and a flat
    /// draw would let the Move branch drown the Block one purely by cardinality. Scoring a group by
    /// its MAX keeps argmax identical to a flat draw while fixing the sampled case.
    ///
    /// Three things have to agree, and only the first is obvious:
    ///
    /// 1. the grouping — CONTIGUOUS runs of `(player, pac)`, because `build_plans` emits one action
    ///    at a time. A keyed lookup would group non-adjacent runs together and change the tree.
    /// 2. `EndTurn` is its own group, appended AFTER all of them, and its weight is exactly 0.0 —
    ///    so it competes with negative-weight branches and loses to positive ones.
    /// 3. the DRAW COUNT: two `softmax_pick` calls, each spending one draw unless its list has a
    ///    single entry (or the temperature is 0, where nothing is drawn at all). A group of one
    ///    silently costs a draw fewer, and the stream desynchronises from there.
    ///
    /// `cargo test -p ffb-engine --lib agent::heuristic_agent::tests::emit_draw_golden -- --ignored`
    /// The two turn guards the random contract has always applied, mirrored from
    /// `ParityRunner`'s INIT_SELECTING arm: a team whose turn counter is still 0 does not act, and
    /// a non-REGULAR turn mode allows exactly ONE activation.
    ///
    /// A bb2016 Blitz! kickoff runs at `turn_nr == 0` in `TurnMode::Blitz`. Without these, the
    /// heuristic kept activating through a turn Java had already ended: seed 4 finished in 5 ms
    /// with eight players still in the box and diverged at its very first recorded step.
    #[test]
    fn heuristic_honours_the_turn_guards() {
        use ffb_model::enums::{PlayerState, TurnMode, PS_STANDING, Rules};
        use ffb_model::model::player::Player;

        let mut home = crate::step::framework::test_team("home", 0);
        let mut away = crate::step::framework::test_team("away", 0);
        for nr in 1..=2 {
            home.players.push(Player {
                id: format!("home_{:02}", nr),
                nr,
                movement: 6,
                strength: 3,
                agility: 3,
                armour: 8,
                ..Default::default()
            });
        }
        away.players.push(Player {
            id: "away_01".to_string(),
            nr: 1,
            movement: 6,
            strength: 3,
            agility: 3,
            armour: 8,
            ..Default::default()
        });
        let mut g = Game::new(home, away, Rules::Bb2016);
        g.home_playing = true;
        for (id, x, y) in [("home_01", 10, 6), ("home_02", 10, 8), ("away_01", 20, 7)] {
            g.field_model.set_player_coordinate(id, FieldCoordinate::new(x, y));
            g.field_model
                .set_player_state(id, PlayerState::new(PS_STANDING).change_active(true));
        }
        g.field_model.ball_coordinate = Some(FieldCoordinate::new(13, 7));
        g.field_model.ball_in_play = true;
        let f = Features::build(&g, positions_stamp(&g), true);
        let eligible: Vec<(String, Vec<PlayerAction>)> = vec![
            ("home_01".to_string(), vec![PlayerAction::Move]),
            ("home_02".to_string(), vec![PlayerAction::Move]),
        ];

        // Guard 1: turn counter still 0 -- nothing happens, whatever the mode.
        g.turn_mode = TurnMode::Blitz;
        g.turn_data_home.turn_nr = 0;
        let mut agent = HeuristicAgent::new(11, 0.0);
        assert!(
            matches!(agent.handle_activate(&g, &f, eligible.clone()), Action::EndTurn),
            "a team whose turn counter is 0 must not activate"
        );

        // Guard 2: a non-REGULAR mode gets exactly one activation, then ends.
        g.turn_data_home.turn_nr = 1;
        let mut agent = HeuristicAgent::new(11, 0.0);
        assert!(
            matches!(
                agent.handle_activate(&g, &f, eligible.clone()),
                Action::ActivatePlayer { .. }
            ),
            "the FIRST activation of a Blitz! turn is allowed"
        );
        assert!(
            matches!(agent.handle_activate(&g, &f, eligible.clone()), Action::EndTurn),
            "the SECOND must end the turn instead"
        );

        // A REGULAR turn is unaffected: a second activation still happens.
        g.turn_mode = TurnMode::Regular;
        let mut agent = HeuristicAgent::new(11, 0.0);
        assert!(matches!(
            agent.handle_activate(&g, &f, eligible.clone()),
            Action::ActivatePlayer { .. }
        ));
        assert!(
            matches!(
                agent.handle_activate(&g, &f, eligible.clone()),
                Action::ActivatePlayer { .. }
            ),
            "a REGULAR turn keeps activating"
        );

        // Guard 3, `justDeselected`: the turn AFTER a window closes is ended too, because in Java
        // that window's activation was the original team's last processed one.
        let mut agent = HeuristicAgent::new(11, 0.0);
        g.turn_mode = TurnMode::Blitz;
        g.turn_data_home.turn_nr = 1;
        assert!(matches!(
            agent.handle_activate(&g, &f, eligible.clone()),
            Action::ActivatePlayer { .. }
        ));
        assert!(matches!(
            agent.handle_activate(&g, &f, eligible.clone()),
            Action::EndTurn
        ));
        // A genuinely new REGULAR turn -- fresh key, nobody used -- and it STILL ends, once.
        g.turn_mode = TurnMode::Regular;
        g.turn_data_home.turn_nr = 2;
        assert!(
            matches!(agent.handle_activate(&g, &f, eligible.clone()), Action::EndTurn),
            "the turn after a window closes is ended too"
        );
        // ...and only once: the flag is consumed, so the turn after THAT plays normally.
        g.turn_data_home.turn_nr = 3;
        assert!(
            matches!(
                agent.handle_activate(&g, &f, eligible.clone()),
                Action::ActivatePlayer { .. }
            ),
            "justDeselected must be consumed, not sticky"
        );
    }

    /// A bribe granted MID-GAME must reach the foul weight.
    ///
    /// The "Get the Ref" kickoff hands +1 Bribe the Ref to BOTH teams, and the engine records it
    /// where Java does — in the turn data's inducement set. `Team::bribes` is a separate field
    /// that only the inducement-PURCHASE step writes, so it stays 0 for a granted bribe. Reading
    /// it priced every foul after that kickoff with the 0.45 ejection cost instead of 0.07
    /// (bb2025 seed 20 step 114: Java 0.064770 against Rust 0.003657, and Java fouled where Rust
    /// moved). The same seed failed identically in bb2020 — this was 8 of the 10 shared reds.
    #[test]
    fn foul_weight_sees_a_bribe_granted_by_the_kickoff() {
        use ffb_model::enums::{PlayerState, PS_PRONE, PS_STANDING, Rules};
        use ffb_model::inducement::inducement::Inducement;
        use ffb_model::inducement::usage::Usage;
        use ffb_model::model::player::Player;

        let mut home = crate::step::framework::test_team("home", 0);
        let away = crate::step::framework::test_team("away", 0);
        home.players.push(Player {
            id: "home_01".into(), nr: 1, movement: 6, strength: 3, agility: 3, armour: 8,
            ..Default::default()
        });
        let mut g = Game::new(home, away, Rules::Bb2025);
        g.team_away.players.push(Player {
            id: "away_01".into(), nr: 1, movement: 6, strength: 3, agility: 3, armour: 8,
            ..Default::default()
        });
        g.home_playing = true;
        g.field_model.set_player_coordinate("home_01", FieldCoordinate::new(12, 7));
        g.field_model
            .set_player_state("home_01", PlayerState::new(PS_STANDING).change_active(true));
        g.field_model.set_player_coordinate("away_01", FieldCoordinate::new(13, 7));
        g.field_model.set_player_state("away_01", PlayerState::new(PS_PRONE));
        g.field_model.ball_coordinate = Some(FieldCoordinate::new(2, 2));
        g.field_model.ball_in_play = true;

        let f = Features::build(&g, positions_stamp(&g), true);
        let m = HeuristicAgent::new(3, 0.0)
            .mover_of(&g, &f, "home_01")
            .expect("the fouler is a valid mover");

        // `Team::bribes` stays 0 throughout: the kickoff never writes it.
        assert_eq!(g.team_home.bribes, 0);
        let without = foul_weight(&f, &g, "home_01", "away_01", &m);

        // What "Get the Ref" actually does.
        g.turn_data_home.inducement_set.add_inducement(Inducement::new(
            "BRIBE".to_string(),
            1,
            vec![Usage::AVOID_BAN],
        ));
        let with = foul_weight(&f, &g, "home_01", "away_01", &m);

        assert!(
            with > without,
            "a bribe makes ejection cheap, so the foul must be worth MORE: {with} vs {without}"
        );
        assert_eq!(g.team_home.bribes, 0, "and it is still not on the team field");
    }

    /// SKIP_INACTIVE: the agent must never activate a player whose ACTIVE bit is clear.
    ///
    /// Java's `StepInitSelecting` accepts `CLIENT_ACTING_PLAYER` only when
    /// `playerState.isActive()`; for anyone else the command is IGNORED and the acting player
    /// stays null, so the activation is a silent no-op that moves nobody. Rust's engine has no
    /// such guard and happily executes it -- the two agree only if the agent does not ask.
    ///
    /// The random agent has honoured this since the contract was written. The heuristic replaced
    /// that pick loop wholesale and inherited none of it, so it moved players Java left standing
    /// still (bb2025 seed 2 step 12: Java left the inactive prone player at (11,7), Rust stood him
    /// up and moved three squares, with identical dice).
    #[test]
    fn heuristic_never_activates_an_inactive_player() {
        use ffb_model::enums::{PlayerState, PS_STANDING, Rules};
        use ffb_model::model::player::Player;

        // Two home players, identical in every way the scorer reads -- same square value, same
        // distance to the endzone -- except that home_01's ACTIVE bit is clear. Anything the
        // agent could prefer about him it must still refuse to act on.
        let mut home = crate::step::framework::test_team("home", 0);
        let mut away = crate::step::framework::test_team("away", 0);
        for nr in 1..=2 {
            home.players.push(Player {
                id: format!("home_{:02}", nr),
                nr,
                movement: 6,
                strength: 3,
                agility: 3,
                armour: 8,
                ..Default::default()
            });
        }
        away.players.push(Player {
            id: "away_01".to_string(),
            nr: 1,
            movement: 6,
            strength: 3,
            agility: 3,
            armour: 8,
            ..Default::default()
        });

        let mut g = Game::new(home, away, Rules::Bb2025);
        g.home_playing = true;
        g.turn_data_home.turn_nr = 3;
        g.turn_data_away.turn_nr = 3;
        for (id, x, y, active) in
            [("home_01", 10, 6, false), ("home_02", 10, 8, true), ("away_01", 20, 7, true)]
        {
            g.field_model.set_player_coordinate(id, FieldCoordinate::new(x, y));
            g.field_model
                .set_player_state(id, PlayerState::new(PS_STANDING).change_active(active));
        }
        g.field_model.ball_coordinate = Some(FieldCoordinate::new(13, 7));
        g.field_model.ball_in_play = true;

        let f = Features::build(&g, positions_stamp(&g), true);
        let eligible: Vec<(String, Vec<PlayerAction>)> = vec![
            ("home_01".to_string(), vec![PlayerAction::Move]),
            ("home_02".to_string(), vec![PlayerAction::Move]),
        ];

        // Argmax, so this is a fact about the option set and not about a lucky draw.
        let mut agent = HeuristicAgent::new(7, 0.0);
        match agent.handle_activate(&g, &f, eligible.clone()) {
            Action::ActivatePlayer { player_id, .. } => {
                assert_eq!(player_id, "home_02", "the inactive player must not be activated");
            }
            other => panic!("expected an activation of home_02, got {other:?}"),
        }

        // With the inactive player as the ONLY candidate there is nothing to activate at all --
        // it must end the turn rather than fall back to him.
        let mut agent = HeuristicAgent::new(7, 0.0);
        let only_inactive = vec![("home_01".to_string(), vec![PlayerAction::Move])];
        assert!(
            matches!(agent.handle_activate(&g, &f, only_inactive), Action::EndTurn),
            "an eligible list of only inactive players leaves nothing to do"
        );
    }

    #[test]
    #[ignore]
    fn emit_draw_golden() {
        use std::fmt::Write as _;

        // (player, pac-as-index, weight) — the shapes that matter, not real boards: one huge Move
        // run against a small Block run, singleton groups, negative weights, and an exact tie.
        type C = (&'static str, u8, f32);
        let cases: [(&str, &[C]); 5] = [
            (
                // The cardinality case the two-level draw exists for: 12 Move destinations for one
                // player against 2 Block targets for another, with the BEST option in the small
                // group.
                "many_moves_vs_few_blocks",
                &[
                    ("a", 0, 0.10), ("a", 0, 0.12), ("a", 0, 0.11), ("a", 0, 0.09),
                    ("a", 0, 0.13), ("a", 0, 0.08), ("a", 0, 0.12), ("a", 0, 0.10),
                    ("a", 0, 0.11), ("a", 0, 0.07), ("a", 0, 0.12), ("a", 0, 0.06),
                    ("b", 1, 0.55), ("b", 1, 0.40),
                ],
            ),
            (
                // The SAME player with two different actions, adjacent: two groups, not one.
                "same_player_two_actions",
                &[("a", 0, 0.30), ("a", 0, 0.20), ("a", 1, 0.50), ("b", 0, 0.25)],
            ),
            (
                // Non-adjacent runs of the same declaration. A keyed lookup merges these; the
                // contiguous rule keeps them apart, and the group weights differ as a result.
                "interleaved_runs",
                &[("a", 0, 0.30), ("b", 0, 0.60), ("a", 0, 0.90)],
            ),
            (
                // Every weight negative, so EndTurn's 0.0 is the best option on the board.
                "all_negative",
                &[("a", 0, -0.40), ("a", 0, -0.35), ("b", 1, -0.90)],
            ),
            (
                // An exact tie between two groups: whatever breaks it must break it the same way.
                "exact_tie",
                &[("a", 0, 0.50), ("b", 1, 0.50)],
            ),
        ];

        let mut out = String::new();
        writeln!(out, "# two-level activation draw golden -- heuristic_agent.rs handle_activate.").unwrap();
        writeln!(out, "# case <name>").unwrap();
        writeln!(out, "# cand <player> <pac> <weight bits>").unwrap();
        writeln!(out, "# groups <a,b,c|d,e|...>   candidate indices per group, EndTurn group last").unwrap();
        writeln!(out, "# groupw <f32 bits, comma separated>").unwrap();
        writeln!(out, "# draw <scale bits> <chosen candidate index> <draws spent>").unwrap();

        for (name, cs) in cases {
            writeln!(out, "case {name}").unwrap();
            for &(pl, pac, w) in cs {
                writeln!(out, "cand {pl} {pac} {:08x}", w.to_bits()).unwrap();
            }

            // Rebuild the Candidate list so the LIVE `group_declarations` is what runs.
            let cands: Vec<Candidate> = cs
                .iter()
                .map(|&(pl, pac, w)| Candidate {
                    weight: w,
                    player: pl.to_string(),
                    pac: if pac == 0 {
                        PlayerActionChoice::Move
                    } else {
                        PlayerActionChoice::Block
                    },
                    target: None,
                    kind: PlanKind::Move,
                    path: Vec::new(),
                    dest: None,
                    why: Rule::Flat,
                    note: String::new(),
                })
                .collect();
            let mut groups = group_declarations(&cands);
            // EndTurn is its own group and sits last, exactly as `handle_activate` appends it.
            let end_idx = cands.len();
            groups.push(vec![end_idx]);

            let mut all_w: Vec<f32> = cands.iter().map(|c| c.weight).collect();
            all_w.push(0.0); // EndTurn

            let gs: Vec<String> = groups
                .iter()
                .map(|g| g.iter().map(|i| i.to_string()).collect::<Vec<_>>().join(","))
                .collect();
            writeln!(out, "groups {}", gs.join("|")).unwrap();

            let gw: Vec<f32> = groups
                .iter()
                .map(|g| g.iter().map(|&j| all_w[j]).fold(f32::MIN, f32::max))
                .collect();
            let gwh: Vec<String> = gw.iter().map(|v| format!("{:08x}", v.to_bits())).collect();
            writeln!(out, "groupw {}", gwh.join(",")).unwrap();

            for scale in [0.0f32, 1.0, 1.0e6] {
                let mut a = HeuristicAgent::new(9, scale);
                let before = a.rng.clone();
                let (gi, _) = a.softmax_pick(&gw, 0.18);
                let cw: Vec<f32> = groups[gi].iter().map(|&j| all_w[j]).collect();
                let (ci, _) = a.softmax_pick(&cw, 0.10);
                let chosen = groups[gi][ci];
                // How many draws the pair actually spent, measured the same way the draw-count
                // tests do: advance a clone of the pre-state until it matches.
                let mut probe = before;
                let mut draws = 0usize;
                while probe.clone().next_u64() != a.rng.clone().next_u64() && draws < 8 {
                    probe.next_u64();
                    draws += 1;
                }
                writeln!(out, "draw {:08x} {chosen} {draws}", scale.to_bits()).unwrap();
            }
        }

        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/src/agent/testdata/draw_golden.txt");
        std::fs::write(path, out).unwrap();
        eprintln!("wrote {path}");
    }

    /// `build_plans` — the ENUMERATION — as a cross-language fixture.
    ///
    /// Every weight it reads is already pinned. What is not is the SHAPE of the list it produces:
    /// how many candidates each action contributes, in what order, with which `PlanKind` and which
    /// target. That shape is the input to the two-level draw, so a list that differs by one entry
    /// picks a different action even when every weight agrees.
    ///
    /// The things most easily lost in a port, all visible here:
    ///
    /// - **Move offers EVERY reachable square**, weight-ordered, not a top-K. Pruning to the best
    ///   arrival probabilities measured catastrophic once (1.76 touchdowns/game to 0.19) because
    ///   `p_arrive` is an admissible bound but not an admissible RANKING: a one-square shuffle
    ///   arrives with p = 1.0 and a six-square scoring run with p ≈ 0.3.
    /// - **A square holding a loose ball becomes `PlanKind::Pickup`**, not `Move` — the activation
    ///   may legitimately continue after picking it up.
    /// - **Blitz stops at adjacency.** A move-then-blitz does not dispatch in this engine build, so
    ///   offering it would waste the team's once-per-turn blitz; the branch deliberately `continue`s
    ///   past every non-adjacent victim rather than scoring it lower.
    /// - **HandOff enumerates squares NEXT TO each team-mate**, capped at `GIVE_SPOTS`, because the
    ///   carrier moves first and gives at the end — not just the mates he already touches.
    /// - **Pass enumerates run-up squares × receivers**, with `risked` folded in only when the
    ///   throw happens somewhere other than where he stands.
    ///
    /// The live-action list is supplied by the golden: which actions are legal is the harness's
    /// job in production (it is what `computeEligiblePlayers` answers), and re-deriving it here
    /// would pin a second copy of the eligibility rules instead of the enumeration.
    ///
    /// `cargo test -p ffb-engine --lib agent::heuristic_agent::tests::emit_planenum_golden -- --ignored`
    #[test]
    #[ignore]
    fn emit_planenum_golden() {
        use std::fmt::Write as _;
        use ffb_model::enums::{PlayerState, PS_STANDING, PS_PRONE, Rules};
        use ffb_model::model::player::Player;

        // (is_home, nr, x, y, standing)
        type P2 = (bool, i32, i32, i32, bool);
        type Board = &'static [P2];

        // (name, board, ball, loose, actions for home_01)
        let cases: [(&str, Board, Option<(i32, i32)>, bool, &[PlayerAction]); 5] = [
            (
                // Plain move in the open: the whole reachable set, weight-ordered.
                "move_open",
                &[(true, 1, 8, 7, true), (false, 1, 20, 7, true)],
                None, false, &[PlayerAction::Move],
            ),
            (
                // A loose ball inside the reachable set: exactly one candidate must be Pickup.
                "move_onto_loose_ball",
                &[(true, 1, 10, 7, true), (false, 1, 20, 2, true)],
                Some((12, 7)), true, &[PlayerAction::Move],
            ),
            (
                // Block: one candidate per adjacent standing opponent, in canonical order.
                "block_two_targets",
                &[
                    (true, 1, 10, 7, true),
                    (false, 1, 11, 7, true), (false, 2, 11, 8, true), (false, 3, 20, 2, true),
                ],
                None, false, &[PlayerAction::Block],
            ),
            (
                // Blitz: only the ADJACENT victim is offered; the distant one is skipped entirely
                // rather than scored low, so the candidate count is the assertion.
                "blitz_adjacent_only",
                &[
                    (true, 1, 10, 7, true),
                    (false, 1, 11, 7, true), (false, 2, 16, 7, true),
                ],
                None, false, &[PlayerAction::Blitz],
            ),
            (
                // HandOff and Pass from a carrier, with two team-mates: run-up squares against
                // receivers, and the GIVE_SPOTS cap per receiver.
                "give_and_pass",
                &[
                    (true, 1, 12, 7, true), (true, 2, 14, 7, true), (true, 3, 12, 9, true),
                    (false, 1, 18, 7, true),
                ],
                Some((12, 7)), false, &[PlayerAction::HandOver, PlayerAction::Pass],
            ),
        ];

        let mut out = String::new();
        writeln!(out, "# build_plans enumeration golden.").unwrap();
        writeln!(out, "# case <name>").unwrap();
        writeln!(out, "# player <home|away> <nr> <x> <y> <standing|prone>").unwrap();
        writeln!(out, "# ball <x> <y> <loose>").unwrap();
        writeln!(out, "# actions <comma separated PlayerAction names for home_01>").unwrap();
        writeln!(out, "# params <w_player bits> <proxy bits> <novelty bits>").unwrap();
        writeln!(out, "# n <candidate count>").unwrap();
        writeln!(out, "# c <index> <pac> <kind> <target|-> <dest|-> <weight bits> <path|->").unwrap();

        for (name, board, ball, loose, actions) in cases {
            let mut home = crate::step::framework::test_team("home", 0);
            let mut away = crate::step::framework::test_team("away", 0);
            for &(is_home, nr, _, _, _) in board {
                let p = Player {
                    id: format!("{}_{:02}", if is_home { "home" } else { "away" }, nr),
                    nr,
                    movement: 6,
                    strength: 3,
                    agility: 3,
                    passing: 4,
                    armour: 8,
                    ..Default::default()
                };
                if is_home { home.players.push(p) } else { away.players.push(p) }
            }
            let mut g = Game::new(home, away, Rules::Bb2025);
            g.home_playing = true;
            g.turn_data_home.turn_nr = 3;
            g.turn_data_away.turn_nr = 3;
            for &(is_home, nr, x, y, standing) in board {
                let id = format!("{}_{:02}", if is_home { "home" } else { "away" }, nr);
                g.field_model.set_player_coordinate(&id, FieldCoordinate::new(x, y));
                g.field_model.set_player_state(
                    &id,
                    PlayerState::new(if standing { PS_STANDING } else { PS_PRONE })
                        .change_active(true),
                );
            }
            if let Some((bx, by)) = ball {
                g.field_model.ball_coordinate = Some(FieldCoordinate::new(bx, by));
                g.field_model.ball_in_play = true;
                g.field_model.ball_moving = loose;
            }
            let f = Features::build(&g, positions_stamp(&g), true);
            let mut agent = HeuristicAgent::new(4, 0.0);
            let m = agent.mover_of(&g, &f, "home_01").expect("mover");
            let proxy = proxy_value(&f, &g, "home_01", &m);
            let mut sc = Scratch::default();
            let b = budget_of(&g, "home_01").expect("budget");
            let r = reach_with(&f, &g, "home_01", &b, false, &mut sc).expect("reach");

            let mut cands: Vec<Candidate> = Vec::new();
            agent.build_plans(
                &g,
                &f,
                TeamSide::Home,
                ("home_01", actions, &m, 1.0, proxy),
                Some(&r),
                0.0,
                false,
                &mut cands,
            );

            writeln!(out, "case {name}").unwrap();
            for &(is_home, nr, x, y, standing) in board {
                writeln!(out, "player {} {nr} {x} {y} {}",
                    if is_home { "home" } else { "away" },
                    if standing { "standing" } else { "prone" }).unwrap();
            }
            if let Some((bx, by)) = ball {
                writeln!(out, "ball {bx} {by} {loose}").unwrap();
            }
            let an: Vec<String> = actions.iter().map(|a| format!("{a:?}")).collect();
            writeln!(out, "actions {}", an.join(",")).unwrap();
            // `build_plans` takes these as PARAMETERS, so they cross the boundary as parameters.
            // Recomputing `w_player` on the far side pins the tier-1 ladder a second time and, if
            // the emitter ever passes something else, silently tests a different call.
            writeln!(out, "params {:08x} {:08x} {:08x}",
                1.0f32.to_bits(), proxy.to_bits(), 0.0f32.to_bits()).unwrap();
            writeln!(out, "n {}", cands.len()).unwrap();
            for (i, c) in cands.iter().enumerate() {
                let kind = match &c.kind {
                    PlanKind::Move => "Move".to_string(),
                    PlanKind::Pickup => "Pickup".to_string(),
                    PlanKind::Blitz { victim } => format!("Blitz:{victim}"),
                    PlanKind::Foul { victim } => format!("Foul:{victim}"),
                    PlanKind::Pass { receiver } => format!("Pass:{receiver}"),
                    PlanKind::HandOff { receiver } => format!("HandOff:{receiver}"),
                    PlanKind::Immediate => "Immediate".to_string(),
                };
                // HandOff and Pass carry a PATH rather than a `dest` -- the run-up is the plan --
                // so the path is what makes their enumerated square observable at all. Without it
                // every give candidate looks identical: same receiver, no dest, and a weight that
                // floors to 0 because the raw give price is negative.
                let path: Vec<String> =
                    c.path.iter().map(|p| format!("{},{}", p.x, p.y)).collect();
                writeln!(out, "c {i} {:?} {kind} {} {} {:08x} {}",
                    c.pac,
                    c.target.clone().unwrap_or_else(|| "-".to_string()),
                    c.dest.map(|d| d.to_string()).unwrap_or_else(|| "-".to_string()),
                    c.weight.to_bits(),
                    if path.is_empty() { "-".to_string() } else { path.join(";") }).unwrap();
            }
        }

        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/src/agent/testdata/planenum_golden.txt");
        std::fs::write(path, out).unwrap();
        eprintln!("wrote {path}");
    }

    /// `handle_activate` END TO END — the whole activation decision — as a fixture.
    ///
    /// Every part of this has its own golden already: the tier-1 ladder, the reach search, the
    /// value model, the arrival weights, the enumeration, the grouping and the two-level draw. What
    /// no other fixture covers is that they COMPOSE the same way — that the ranking feeds the right
    /// players into the search, that the search feeds the right candidates into the enumeration,
    /// and that the enumeration's order is the order the grouping sees.
    ///
    /// A composition bug is invisible to the part fixtures by construction: each of them can be
    /// perfectly right while the wiring between two of them is wrong. This is the one that catches
    /// that, and it is also the closest thing to the live gate that a fixture can be — same
    /// entry point, same return value, no engine.
    ///
    /// Emitted at all three temperature scales, because the draw is where the scales differ: at 0
    /// the whole thing is deterministic and consumes nothing, at 1.0 it spends two draws, and at
    /// 1e6 it spends the same two but on a nearly uniform distribution.
    ///
    /// `cargo test -p ffb-engine --lib agent::heuristic_agent::tests::emit_actend_golden -- --ignored`
    #[test]
    #[ignore]
    fn emit_actend_golden() {
        use std::fmt::Write as _;
        use ffb_model::enums::{PlayerState, PS_STANDING, PS_PRONE, Rules};
        use ffb_model::model::player::Player;

        // (is_home, nr, x, y, standing)
        type P2 = (bool, i32, i32, i32, bool);
        type Board = &'static [P2];
        // (player nr, actions)
        type Elig = &'static [(i32, &'static [PlayerAction])];

        let cases: [(&str, Board, Option<(i32, i32)>, bool, Elig); 5] = [
            (
                // Three movers, nothing else on offer: the choice is entirely the tier-1 ranking
                // feeding the reach search and the destination weights.
                "three_movers",
                &[
                    (true, 1, 6, 5, true), (true, 2, 6, 7, true), (true, 3, 6, 9, true),
                    (false, 1, 20, 7, true),
                ],
                None, false,
                &[(1, &[PlayerAction::Move]), (2, &[PlayerAction::Move]),
                  (3, &[PlayerAction::Move])],
            ),
            (
                // A carrier who can score, against two team-mates who cannot: the value model has
                // to win over the raw ranking.
                "carrier_can_score",
                &[
                    (true, 1, 20, 7, true), (true, 2, 6, 5, true), (true, 3, 6, 9, true),
                    (false, 1, 22, 3, true),
                ],
                Some((20, 7)), false,
                &[(1, &[PlayerAction::Move]), (2, &[PlayerAction::Move]),
                  (3, &[PlayerAction::Move])],
            ),
            (
                // Move against Block for the SAME player, plus a second player: two declarations
                // from one player is what the grouping exists to keep apart.
                "move_vs_block",
                &[
                    (true, 1, 10, 7, true), (true, 2, 6, 9, true),
                    (false, 1, 11, 7, true), (false, 2, 11, 8, true),
                ],
                None, false,
                &[(1, &[PlayerAction::Move, PlayerAction::Block]),
                  (2, &[PlayerAction::Move])],
            ),
            (
                // The RANKING is not the canonical order: home_02 carries the ball, so he outranks
                // home_01 and home_03. `build_plans` still walks the CANONICAL list -- the rank
                // only decides who gets a search -- so enumerating in ranked order instead
                // produces the same candidates in a different sequence, and the declaration
                // grouping is positional.
                //
                // Added because the bite-check did NOT fail on the first four boards: their
                // rankings happened to coincide with canonical order, so the distinction was
                // invisible.
                "ranking_differs_from_canonical",
                &[
                    (true, 1, 6, 5, true), (true, 2, 14, 7, true), (true, 3, 6, 9, true),
                    (false, 1, 20, 7, true),
                ],
                Some((14, 7)), false,
                &[(1, &[PlayerAction::Move]), (2, &[PlayerAction::Move]),
                  (3, &[PlayerAction::Move])],
            ),
            (
                // A loose ball one player can fetch and another cannot, with a prone player too.
                "loose_ball_scramble",
                &[
                    (true, 1, 10, 7, true), (true, 2, 22, 12, true), (true, 3, 9, 8, false),
                    (false, 1, 20, 2, true),
                ],
                Some((12, 7)), true,
                &[(1, &[PlayerAction::Move]), (2, &[PlayerAction::Move]),
                  (3, &[PlayerAction::Move])],
            ),
        ];

        let mut out = String::new();
        writeln!(out, "# handle_activate end-to-end golden.").unwrap();
        writeln!(out, "# case <name>").unwrap();
        writeln!(out, "# player <home|away> <nr> <x> <y> <standing|prone>").unwrap();
        writeln!(out, "# ball <x> <y> <loose>").unwrap();
        writeln!(out, "# eligible <nr> <comma separated actions>").unwrap();
        writeln!(out, "# chose <scale bits> <player|ENDTURN> <action|-> <target|->").unwrap();

        for (name, board, ball, loose, elig) in cases {
            writeln!(out, "case {name}").unwrap();
            for &(is_home, nr, x, y, standing) in board {
                writeln!(out, "player {} {nr} {x} {y} {}",
                    if is_home { "home" } else { "away" },
                    if standing { "standing" } else { "prone" }).unwrap();
            }
            if let Some((bx, by)) = ball {
                writeln!(out, "ball {bx} {by} {loose}").unwrap();
            }
            for &(nr, acts) in elig {
                let an: Vec<String> = acts.iter().map(|a| format!("{a:?}")).collect();
                writeln!(out, "eligible {nr} {}", an.join(",")).unwrap();
            }

            for scale in [0.0f32, 1.0, 1.0e6] {
                let mut home = crate::step::framework::test_team("home", 0);
                let mut away = crate::step::framework::test_team("away", 0);
                for &(is_home, nr, _, _, _) in board {
                    let p = Player {
                        id: format!("{}_{:02}", if is_home { "home" } else { "away" }, nr),
                        nr,
                        movement: 6,
                        strength: 3,
                        agility: 3,
                        passing: 4,
                        armour: 8,
                        ..Default::default()
                    };
                    if is_home { home.players.push(p) } else { away.players.push(p) }
                }
                let mut g = Game::new(home, away, Rules::Bb2025);
                g.home_playing = true;
                g.turn_data_home.turn_nr = 3;
                g.turn_data_away.turn_nr = 3;
                g.turn_data_home.rerolls = 0;
                for &(is_home, nr, x, y, standing) in board {
                    let id = format!("{}_{:02}", if is_home { "home" } else { "away" }, nr);
                    g.field_model.set_player_coordinate(&id, FieldCoordinate::new(x, y));
                    g.field_model.set_player_state(
                        &id,
                        PlayerState::new(if standing { PS_STANDING } else { PS_PRONE })
                            .change_active(true),
                    );
                }
                if let Some((bx, by)) = ball {
                    g.field_model.ball_coordinate = Some(FieldCoordinate::new(bx, by));
                    g.field_model.ball_in_play = true;
                    g.field_model.ball_moving = loose;
                }
                let f = Features::build(&g, positions_stamp(&g), true);
                let eligible: Vec<(String, Vec<PlayerAction>)> = elig
                    .iter()
                    .map(|&(nr, acts)| (format!("home_{nr:02}"), acts.to_vec()))
                    .collect();
                let mut agent = HeuristicAgent::new(21, scale);
                let act = agent.handle_activate(&g, &f, eligible);
                let (pl, pa, tgt) = match act {
                    Action::ActivatePlayer { player_id, player_action, block_defender_id } => (
                        player_id,
                        format!("{player_action:?}"),
                        block_defender_id.unwrap_or_else(|| "-".to_string()),
                    ),
                    Action::EndTurn => ("ENDTURN".to_string(), "-".to_string(), "-".to_string()),
                    other => (format!("{other:?}"), "-".to_string(), "-".to_string()),
                };
                writeln!(out, "chose {:08x} {pl} {pa} {tgt}", scale.to_bits()).unwrap();
            }
        }

        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/src/agent/testdata/actend_golden.txt");
        std::fs::write(path, out).unwrap();
        eprintln!("wrote {path}");
    }

    /// `replay_plan` — the plan-replay state machine — as an EXHAUSTIVE fixture.
    ///
    /// Every other golden in this campaign samples: a handful of boards chosen to reach the
    /// branches that matter, with a bite-check to confirm they do. Twice that sampling has been
    /// caught missing a branch (ITER27, ITER33) and once it was caught grading its own homework
    /// (ITER36).
    ///
    /// This one does not sample. The input space is ten booleans, seven plan kinds and seven
    /// relevant player actions, so all **50,176** combinations fit in one file — 49 lines of 1024
    /// verdicts, one character each. There is no branch it can miss, and no board to choose badly.
    ///
    /// Worth doing here specifically because the state machine is the piece with the most exits
    /// (seven) and the least structure: nothing about it suggests which combinations are
    /// interesting, so any sample would be a guess.
    ///
    /// `cargo test -p ffb-engine --lib agent::heuristic_agent::tests::emit_replay_golden -- --ignored`
    #[test]
    #[ignore]
    fn emit_replay_golden() {
        use std::fmt::Write as _;

        let kinds: [(&str, PlanKind); 7] = [
            ("Move", PlanKind::Move),
            ("Pickup", PlanKind::Pickup),
            ("Immediate", PlanKind::Immediate),
            ("Blitz", PlanKind::Blitz { victim: "v".into() }),
            ("Foul", PlanKind::Foul { victim: "v".into() }),
            ("Pass", PlanKind::Pass { receiver: "r".into() }),
            ("HandOff", PlanKind::HandOff { receiver: "r".into() }),
        ];
        // Every action the guards actually test, plus None and one that matches nothing.
        let actions: [(&str, Option<PlayerAction>); 7] = [
            ("None", None),
            ("Move", Some(PlayerAction::Move)),
            ("BlitzMove", Some(PlayerAction::BlitzMove)),
            ("KickEmBlitz", Some(PlayerAction::KickEmBlitz)),
            ("FoulMove", Some(PlayerAction::FoulMove)),
            ("PassMove", Some(PlayerAction::PassMove)),
            ("HandOverMove", Some(PlayerAction::HandOverMove)),
        ];

        let mut out = String::new();
        writeln!(out, "# replay_plan EXHAUSTIVE golden -- heuristic_agent.rs and MoveReplay.java.").unwrap();
        writeln!(out, "# row <kind> <pa_now> <1024 verdicts>").unwrap();
        writeln!(out, "# Bit order, most significant first, over the 10-bit index of each verdict:").unwrap();
        writeln!(out, "#   is_mine, path_empty, delivered, fired, has_blocked, has_fouled,").unwrap();
        writeln!(out, "#   target_adjacent, target_on_pitch, squares_include_next, squares_empty").unwrap();
        writeln!(out, "# Verdicts: D = DeliverPath, F = FireTerminal, E = EndPlayerAction, R = Replan").unwrap();

        for (kname, kind) in &kinds {
            for (aname, pa) in &actions {
                let mut row = String::with_capacity(1024);
                for bits in 0..1024u32 {
                    let b = |shift: u32| (bits >> shift) & 1 == 1;
                    let facts = ReplayFacts {
                        pa_now: *pa,
                        has_blocked: b(5),
                        has_fouled: b(4),
                        target_adjacent: b(3),
                        target_on_pitch: b(2),
                        squares_include_next: b(1),
                        squares_empty: b(0),
                    };
                    let v = replay_plan(kind, b(9), b(8), b(7), b(6), &facts);
                    row.push(match v {
                        Replay::DeliverPath => 'D',
                        Replay::FireTerminal => 'F',
                        Replay::EndPlayerAction => 'E',
                        Replay::Replan => 'R',
                    });
                }
                writeln!(out, "row {kname} {aname} {row}").unwrap();
            }
        }

        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/src/agent/testdata/replay_golden.txt");
        std::fs::write(path, out).unwrap();
        eprintln!("wrote {path}");
    }

    /// `cargo test -p ffb-engine --lib agent::heuristic_agent::tests::emit_sampler_golden -- --ignored`
    #[test]
    #[ignore]
    fn emit_sampler_golden() {
        use std::fmt::Write as _;
        let mut out = String::new();
        writeln!(out, "# sampler golden -- see agent/heuristic_agent.rs and Sampler.java.").unwrap();
        writeln!(out, "# unit <seed> <i> <bits>   : the i-th unit() draw, as f32 bits").unwrap();
        writeln!(out, "# pick <seed> <scale> <tbase> <n> <w0,w1,...> <idx> <draws>").unwrap();

        // The raw draw sequence. If this diverges, nothing downstream can agree.
        for seed in [1u64, 29, 33, 46, 12345] {
            let mut a = HeuristicAgent::new(seed, 1.0);
            for i in 0..24 {
                let u = a.unit();
                writeln!(out, "unit {seed} {i} {:08x}", u.to_bits()).unwrap();
            }
        }

        // Whole decisions, at every temperature band, including the two-option equal-weight case
        // that CoinChoice uses and where the eps escape is most visible.
        let cases: [(f32, f32, &[f32]); 10] = [
            (1.0, 1.00, &[0.5, 0.5]),
            (1.0, 0.30, &[0.65, 0.35]),
            (1.0, 0.15, &[0.9, 0.6, 0.4, 0.25, 0.1]),
            (1.0, 0.12, &[0.05, 0.7, 0.8, 0.95]),
            (0.05, 0.15, &[0.9, 0.6, 0.4]),
            (0.0, 0.15, &[0.9, 0.6, 0.4]),
            (1.0e6, 0.15, &[0.9, 0.6, 0.4]),
            (1.0, 0.18, &[0.3]),
            (1.0, 0.10, &[0.42, 0.42, 0.42, 0.42, 0.42, 0.42, 0.42, 0.42]),
            (1.0, 0.20, &[-0.4, 0.0, 0.55]),
        ];
        for seed in [1u64, 29, 7] {
            for (scale, tbase, ws) in cases.iter() {
                let mut a = HeuristicAgent::new(seed, *scale);
                // Repeat so the eps escape is actually exercised, not just skipped.
                for _rep in 0..40 {
                    a.buf.clear();
                    for w in ws.iter() {
                        a.buf.push(Action::EndTurn, *w, Rule::Flat, 0.0);
                    }
                    let before = a.rng.clone();
                    let idx = a.pick(*tbase);
                    let draws = {
                        let after_next = a.rng.clone().next_u64();
                        let mut k = 0;
                        loop {
                            let mut probe = before.clone();
                            for _ in 0..k {
                                probe.next_u64();
                            }
                            if probe.next_u64() == after_next || k > 6 {
                                break k;
                            }
                            k += 1;
                        }
                    };
                    let wstr: Vec<String> = ws.iter().map(|w| format!("{:08x}", w.to_bits())).collect();
                    writeln!(
                        out, "pick {seed} {:08x} {:08x} {} {} {idx} {draws}",
                        scale.to_bits(), tbase.to_bits(), ws.len(), wstr.join(",")
                    ).unwrap();
                }
            }
        }
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/src/agent/testdata/sampler_golden.txt");
        std::fs::write(path, out).expect("write golden");
        eprintln!("wrote {path}");
    }

    // ── the Java-port class ladder ─────────────────────────────────────────────

    /// Rung 0 of the ladder: with an empty mask the heuristic agent must be **indistinguishable**
    /// from `RandomAgent::new_parity` -- same actions, same RNG consumption, every prompt. That is
    /// what makes "the gate is 100/100 by construction before any Java exists" a fact rather than
    /// a hope, and it is what isolates a later rung's failure to the one class it switched on.
    #[test]
    fn rung_zero_is_indistinguishable_from_the_parity_agent() {
        use crate::step::new_game;

        let seed = 7u64;
        let mut a = new_game(seed);
        let mut b = new_game(seed);
        let mut heur = HeuristicAgent::with_classes(seed, 1.0, Mode::Wide, ClassMask::NONE);
        let mut rand = RandomAgent::new_parity(seed);

        let mut steps = 0;
        while !a.is_finished() && a.current_prompt().is_some() {
            let ha = heur.act(&a);
            let ra = rand.act(&b);
            assert_eq!(ha, ra, "rung 0 diverged at step {steps}");
            let side_a = a.active_side();
            let side_b = b.active_side();
            let _ = a.apply(side_a, ha);
            let _ = b.apply(side_b, ra);
            assert_eq!(
                a.state_hash_str(),
                b.state_hash_str(),
                "state diverged after step {steps}"
            );
            steps += 1;
            if steps > 4000 {
                break;
            }
        }
        assert!(steps > 20, "the harness game ended too early to prove anything: {steps} steps");
    }

    /// A partial mask must route by class, not by luck.
    #[test]
    fn class_mask_parses_and_routes() {
        assert!(ClassMask::ALL.has(PromptClass::Move));
        assert!(!ClassMask::NONE.has(PromptClass::Move));

        let m = ClassMask::parse("coin,receive").expect("parses");
        assert!(m.has(PromptClass::CoinChoice));
        assert!(m.has(PromptClass::ReceiveChoice));
        assert!(!m.has(PromptClass::Move));
        assert!(!m.has(PromptClass::Other));

        assert_eq!(ClassMask::parse("all").unwrap(), ClassMask::ALL);
        assert_eq!(ClassMask::parse("none").unwrap(), ClassMask::NONE);
        assert_eq!(ClassMask::parse("").unwrap(), ClassMask::NONE);
        assert!(ClassMask::parse("nonsense").is_err());

        // Every class must have a unique bit and a unique name, or a rung would silently switch
        // on more than it names.
        let mut seen = 0u32;
        for c in PromptClass::ALL {
            let bit = 1u32 << (c as u8);
            assert_eq!(seen & bit, 0, "duplicate bit for {}", c.name());
            seen |= bit;
        }
        let mut names: Vec<&str> = PromptClass::ALL.iter().map(|c| c.name()).collect();
        names.sort_unstable();
        let n = names.len();
        names.dedup();
        assert_eq!(names.len(), n, "duplicate class name");
    }

    /// The Pushback weight table, which `HeuristicDriver.pushbackChoice` mirrors on the Java side.
    /// Pinned here because it is a CROSS-LANGUAGE contract: the two engines must score the same
    /// squares identically or they push a player to different squares from the same board.
    ///
    /// Off the pitch is worth most (and more still when the pushed player carries the ball);
    /// a sideline square beats an interior one; and any square further from the DEFENDER's own
    /// endzone takes a 1.3 multiplier.
    #[test]
    fn pushback_weight_table() {
        // The bare weights, before the endzone multiplier.
        let off_with_ball = 1.0f32;
        let off_without = 0.95f32;
        let sideline = 0.55f32;
        let interior = 0.20f32;
        assert!(off_with_ball > off_without);
        assert!(off_without > sideline);
        assert!(sideline > interior);

        // The endzone multiplier is 1.3 and applies to whichever square is further from the
        // DEFENDER's own endzone -- i.e. the direction that hurts him.
        assert!((interior * 1.3 - 0.26).abs() < 1e-6);
        // …and it NEVER crosses a tier: even multiplied, an interior square stays below every
        // sideline square and a sideline square below every off-pitch one. So the ordering is
        // tier-first, multiplier-second — a useful invariant, because it means a disagreement about
        // the multiplier can only ever reorder squares WITHIN a tier.
        assert!(interior * 1.3 < sideline, "0.26 < 0.55");
        assert!(sideline * 1.3 < off_without, "0.715 < 0.95");

        // Geometry both sides share: home attacks toward x=25, away toward x=0.
        assert_eq!(endzone_x(true), XMAX);
        assert_eq!(endzone_x(false), 0);
        assert_eq!(endzone_distance(FieldCoordinate::new(20, 7), true), 5);
        assert_eq!(endzone_distance(FieldCoordinate::new(20, 7), false), 20);
        // Off-pitch detection, which decides the crowd-surf weight.
        assert!(!on_pitch(-1, 7));
        assert!(!on_pitch(26, 7));
        assert!(!on_pitch(10, -1));
        assert!(!on_pitch(10, 15));
        assert!(on_pitch(0, 0) && on_pitch(XMAX, YMAX));
    }

    /// The interception weights, and the constant the Java mirror cannot see.
    ///
    /// `HeuristicDriver.intercept()` hardcodes `pRoll(0)` because the Java dialog does not carry a
    /// target number. Both LIVE steps publish 0 — the legacy `engine.rs` prompt publishes 6, but
    /// that path is not the one the driver runs — so pin the constant here rather than leaving the
    /// two sides silently coupled through a literal.
    #[test]
    fn intercept_weights_and_the_target_number_the_live_steps_publish() {
        // p_roll clamps, so a "target number" of 0 is not a 100% chance: it is 5/6.
        assert!((p_roll(0) - 5.0 / 6.0).abs() < 1e-6);
        let attempt = p_roll(0) * 1.5;
        let decline = 0.20f32;
        // Attempting outweighs declining by more than 6x, so at argmax the agent always tries --
        // which is the whole point of switching the class on: the random contract picks a random
        // candidate, and 2 of 20 bb2025 seeds end on a different board because of it.
        assert!(attempt > 6.0 * decline);

        // Both live StepIntercept twins publish 0. If either changes, the Java side is wrong and
        // nothing in the sweep would say so directly.
        for src in [
            include_str!("../step/bb2025/pass/step_intercept.rs"),
            include_str!("../step/bb2020/pass/step_intercept.rs"),
        ] {
            let at = src.find("AgentPrompt::Interception").expect("prompt is raised here");
            let window = &src[at..at + 240];
            assert!(
                window.contains("target_number: 0"),
                "a live StepIntercept no longer publishes target_number 0"
            );
        }
    }

    /// An AWAY kick is emitted in the CLIENT frame, like every other away command.
    ///
    /// `StepKickoff` mirrors an away coach's coordinate back into the server frame, so a
    /// server-frame coordinate sent as-is lands in the KICKING half and touchbacks. `RandomAgent`
    /// pre-transforms for exactly this reason; this arm did not, and bb2025 seed 2 diverged at the
    /// first activation with the two engines' *picks* byte-identical — the scoring agreed and the
    /// frame did not.
    #[test]
    fn an_away_kick_is_emitted_in_the_client_frame() {
        use crate::step::new_game;

        let mut gs = new_game(11);
        let coord_for = |gs: &mut crate::step::driver::DriverGameState, home_playing: bool| {
            gs.game.home_playing = home_playing;
            gs.pending_prompt = Some(AgentPrompt::KickBall);
            let mut a = HeuristicAgent::with_classes(11, 0.0, Mode::Wide, ClassMask::ALL);
            match a.act(gs) {
                Action::KickBall { coord } => coord,
                other => panic!("expected KickBall, got {other:?}"),
            }
        };

        // Home kicks into the away half and needs no transform: server frame IS its client frame.
        let home = coord_for(&mut gs, true);
        assert!((13..=XMAX).contains(&home.x), "home kick landed at {home:?}");

        // Away scores the HOME half (x 0..12) and then mirrors, so what leaves the agent is the
        // high half — and mirroring it back reproduces the square that was scored.
        let away = coord_for(&mut gs, false);
        assert!((13..=XMAX).contains(&away.x), "away kick was not pre-transformed: {away:?}");
        let back = away.transform();
        assert!((0..=12).contains(&back.x), "away kick does not mirror back into the home half");
    }

    /// §6.10 — the follow-up table, and the one term that can flip the answer.
    #[test]
    fn follow_up_weight_table() {
        let w = |carries: bool, more_marked: bool, sideline: bool| {
            let mut v = 0.5f32;
            if carries { v -= 0.45 }
            if more_marked { v -= 0.35 }
            if sideline { v -= 0.30 }
            v.clamp(0.02, 0.98)
        };
        // An unencumbered follow-up into open ground is an even coin -- and a TIE at argmax, which
        // `argmax`'s first-strict-maximum rule resolves to index 0, i.e. FOLLOW. Both sides have to
        // agree on that, which is why the tie is worth pinning rather than avoiding.
        assert_eq!(w(false, false, false), 0.5);
        // Carrying the ball is the heaviest single term: chasing with the ball is how you lose it.
        assert!(w(true, false, false) < w(false, true, false));
        assert!(w(false, true, false) < w(false, false, true));
        // Any two terms together are enough to make following clearly wrong, and the clamp keeps
        // even the worst case from being impossible.
        assert!(w(true, true, true) >= 0.02);
        assert!(w(true, true, false) < 0.05);
    }

    /// The plan-replay state machine, exit by exit.
    ///
    /// Seven exits and four engine guards, and the ordering between them is the whole content —
    /// which is why it is a pure function rather than control flow tangled with the mutations it
    /// drives. Refactoring it out was verified behaviour-preserving by replaying ten full games
    /// with `--heur-classes all` and comparing Rust's own end-of-game hashes before and after.
    #[test]
    fn plan_replay_state_machine() {
        let facts = |pa: Option<PlayerAction>, blocked: bool, fouled: bool, adj: bool,
                     on_pitch: bool, next: bool, empty: bool| ReplayFacts {
            pa_now: pa,
            has_blocked: blocked,
            has_fouled: fouled,
            target_adjacent: adj,
            target_on_pitch: on_pitch,
            squares_include_next: next,
            squares_empty: empty,
        };
        let blitz = PlanKind::Blitz { victim: "v".into() };
        let give = PlanKind::HandOff { receiver: "r".into() };
        let plain = PlanKind::Move;

        // No movement left and nothing pending: end. This is the common exit.
        assert_eq!(
            replay_plan(&plain, true, true, false, false,
                &facts(None, false, false, false, false, false, true)),
            Replay::EndPlayerAction
        );
        // No movement left but a give IS pending: the give must still be sent. Bailing here threw
        // away every give whose run-up spent the carrier's whole move.
        assert_eq!(
            replay_plan(&give, true, true, false, false,
                &facts(Some(PlayerAction::HandOverMove), false, false, false, true, false, true)),
            Replay::FireTerminal
        );
        // A path is delivered only when the offered squares contain its next step...
        assert_eq!(
            replay_plan(&plain, true, false, false, false,
                &facts(None, false, false, false, false, true, false)),
            Replay::DeliverPath
        );
        // ...and when the board has moved under it, the plan re-decides rather than insisting.
        assert_eq!(
            replay_plan(&plain, true, false, false, false,
                &facts(None, false, false, false, false, false, false)),
            Replay::Replan
        );
        // A blitz fires only with the engine's own action set AND not having blocked AND adjacent.
        for (pa, blocked, adj, want) in [
            (Some(PlayerAction::BlitzMove), false, true, Replay::FireTerminal),
            (Some(PlayerAction::BlitzMove), true, true, Replay::Replan),
            (Some(PlayerAction::BlitzMove), false, false, Replay::Replan),
            (Some(PlayerAction::Move), false, true, Replay::Replan),
        ] {
            assert_eq!(
                replay_plan(&blitz, true, true, false, false,
                    &facts(pa, blocked, false, adj, true, false, false)),
                want,
                "blitz guard pa={pa:?} blocked={blocked} adj={adj}"
            );
        }
        // A delivered plain move ends: moving twice reaches the same square.
        assert_eq!(
            replay_plan(&plain, true, true, true, false,
                &facts(None, false, false, false, false, false, false)),
            Replay::EndPlayerAction
        );
        // Once fired, only a blitz has anything left -- its post-block movement.
        assert_eq!(
            replay_plan(&blitz, true, true, true, true,
                &facts(None, false, false, false, false, false, false)),
            Replay::Replan
        );
        assert_eq!(
            replay_plan(&give, true, true, true, true,
                &facts(None, false, false, false, false, false, false)),
            Replay::EndPlayerAction
        );
        // A pickup changed the value model, so it re-decides rather than ending.
        assert_eq!(
            replay_plan(&PlanKind::Pickup, true, true, true, true,
                &facts(None, false, false, false, false, false, false)),
            Replay::Replan
        );
        // A plan belonging to ANOTHER player is not replayed.
        assert_eq!(
            replay_plan(&plain, false, false, false, false,
                &facts(None, false, false, false, false, true, false)),
            Replay::Replan
        );
    }

    /// §6 — the touchback receiver table, which `HeuristicDriver.touchback` mirrors.
    #[test]
    fn touchback_weight_table() {
        let w = |ma: i32, sure_hands: bool, on_los: bool| {
            let mut v = 0.3f32 + 0.4 * (ma as f32 / 9.0).min(1.0);
            if sure_hands {
                v += 0.3;
            }
            if on_los {
                v -= 0.5;
            }
            v
        };
        // Faster is better, and the MA term SATURATES at 9 -- a MA 10 Gutter Runner scores the
        // same as a MA 9 one, so the tie is broken by the other two terms, not by speed.
        assert!(w(6, false, false) > w(5, false, false));
        assert_eq!(w(9, false, false), w(10, false, false));
        // Sure Hands is worth about two points of MA.
        assert!(w(6, true, false) > w(9, false, false));
        // …but the line-of-scrimmage penalty outweighs it: a Sure Hands player standing on the LOS
        // is a WORSE receiver than a plain lineman standing anywhere else, because he is about to
        // be blocked. That is the one ordering the two implementations must agree on, since it is
        // the only term that can reorder the list rather than just rescale it.
        assert!(w(6, true, true) < w(6, false, false));
        // The penalty is NOT unconditional, though: 0.5 is bigger than the Sure Hands bonus but
        // smaller than the MA span, so a fast Sure Hands player ON the line still outranks a slow
        // one off it. My first draft of this test asserted the opposite and failed — the LOS term
        // reorders the list, it does not partition it, and an implementation that treated it as a
        // veto would disagree exactly here.
        assert!(w(9, true, true) > w(3, false, false));
    }

    /// `PromptClass::Other` must be a NO-OP: the agent does not model those prompts, so switching
    /// the class on or off has to give the same answer.
    ///
    /// It did not. The unmodelled tail fell through to `UniformAgent`, whose `PlayerChoice` arm
    /// sorts candidates BY PLAYER ID — and the two engines generate different ids, so that arm
    /// could never agree with the Java harness. `RandomAgent` has a dozen reason-specific arms for
    /// this one prompt, every one coordinate-sorted for exactly that reason. bb2020 seed 26 has a
    /// single PlayerChoice in the whole game, raised by a prayer, and it diverged at step 0.
    #[test]
    fn the_unmodelled_tail_answers_with_the_parity_contract() {
        use crate::step::new_game;

        let prompt = || AgentPrompt::PlayerChoice {
            eligible_players: vec!["home_02".into(), "home_01".into()],
            reason: "WISDOM".into(),
            descriptions: Vec::new(),
        };

        let answer = |classes: ClassMask| {
            let mut gs = new_game(5);
            gs.pending_prompt = Some(prompt());
            let mut a = HeuristicAgent::with_classes(5, 1.0, Mode::Wide, classes);
            a.act(&gs)
        };
        // On and off must agree, and both must agree with the contract itself.
        let mut gs = new_game(5);
        gs.pending_prompt = Some(prompt());
        let contract = RandomAgent::new_parity(5).act(&gs);

        assert_eq!(answer(ClassMask::ALL), contract, "Other ON must use the contract");
        assert_eq!(answer(ClassMask::NONE), contract, "Other OFF already did");
    }

    /// An empty blitz-target list ends the TURN, not the action.
    ///
    /// The engine raises `BlitzTarget` whenever any in-bounds opponent can be blocked, but the
    /// candidate list holds only the ADJACENT ones -- so a blitzer with no neighbour is prompted
    /// with nothing to pick. `ParityRunner.sendBlitzTargetSelection` answers that with
    /// ClientCommandEndTurn, and `RandomAgent` mirrors it. This arm answered EndPlayerAction, and
    /// `StepSelectBlitzTarget` then waited forever for a target: bb2025 seed 8 aborted the game
    /// after 50 unchanged iterations, mid-drive, with the score still 0-0.
    #[test]
    fn an_empty_blitz_target_list_ends_the_turn() {
        use crate::step::new_game;

        let prompt = || AgentPrompt::BlitzTarget {
            attacker_id: "home_01".into(),
            eligible_players: Vec::new(),
        };
        let mut gs = new_game(3);
        gs.pending_prompt = Some(prompt());
        let mut heur = HeuristicAgent::with_classes(3, 0.0, Mode::Wide, ClassMask::ALL);
        assert_eq!(heur.act(&gs), Action::EndTurn);

        // …and it is the same answer the byte-matched random contract gives, which is what makes
        // the class switchable without moving the stream.
        let mut rand = RandomAgent::new_parity(3);
        assert_eq!(rand.act(&gs), Action::EndTurn);
    }

    /// §6.3 — the block-die table, and the push geometry the Java port re-implements.
    ///
    /// The weights themselves are inline in the `BlockChoice` arm, so this pins the ORDERING
    /// invariants rather than restating the arm; a disagreement with the Java mirror shows up as
    /// a different picked index at argmax, which is exactly what these assertions bound.
    #[test]
    fn block_choice_weight_table() {
        // Skull is always the worst face and POW always the best, whatever the skills are.
        let pow = 0.90f32;
        let skull = 0.05f32;
        let push = 0.40f32;
        let push_surf = 0.80f32;
        let pow_push_plain = 0.80f32;
        let pow_push_dodge = 0.30f32;
        assert!(pow > pow_push_plain && pow_push_plain > push && push > skull);
        // A crowd-surfing push beats a plain one, and a POW/push into the crowd beats both.
        assert!(push_surf > push);
        assert!(0.95f32 > push_surf);
        // Dodge (untackled) demotes POW/push BELOW a plain push -- the defender just stays up.
        assert!(pow_push_dodge < push);

        // Both Down: the four skill combinations, in the order the arm tests them.
        let bd_att_only_up = 0.70f32; // attacker has Block/Wrestle, defender does not
        let bd_both_down_ball = 0.50f32; // neither, but the defender carries the ball
        let bd_both_down = 0.30f32; // neither, no ball
        let bd_def_only_up = 0.10f32; // only the defender has Block
        let bd_both_up = 0.35f32; // both have Block
        assert!(bd_att_only_up > bd_both_down_ball);
        assert!(bd_both_down_ball > bd_both_up && bd_both_up > bd_both_down);
        assert!(bd_both_down > bd_def_only_up);
        // Knocking the CARRIER down is worth taking a fall for; knocking a plain lineman down
        // is not (0.30 < a plain push at 0.40) -- the one place the ball changes this table.
        assert!(bd_both_down_ball > push && bd_both_down < push);

        // The opponent's choice is the same table read backwards, so it needs no second table.
        assert!((1.0f32 - skull) > (1.0f32 - pow));

        // Push geometry. away_01 at (9,7) blocked by home_01 would be pushed toward +x; put the
        // defender on the sideline instead and the crowd is reachable.
        let mut g = tie_game(3, 3);
        g.field_model.set_player_coordinate("home_01", FieldCoordinate::new(9, 7));
        // Straight back plus the two flanks -- three squares, always.
        assert_eq!(push_squares(&g, "home_01", "away_02").len(), 3);
        assert_eq!(
            push_squares(&g, "home_01", "away_02")[0],
            FieldCoordinate::new(12, 7),
            "base square is the defender stepped one further along the block direction"
        );
        assert!(!can_surf(&g, "home_01", "away_02"), "mid-pitch: no crowd in reach");

        // Now against the sideline: y = 0, pushed straight up into the crowd.
        g.field_model.set_player_coordinate("home_01", FieldCoordinate::new(10, 1));
        g.field_model.set_player_coordinate("away_02", FieldCoordinate::new(10, 0));
        assert!(can_surf(&g, "home_01", "away_02"), "y=0 pushed to y=-1 is off the pitch");
    }

    // ── sampler draw-count contract (AGENT_CONTRACT_HEURISTIC.md section 2) ────

    /// How many `next_u64()` calls `f` consumed from the agent's stream.
    fn draws_used(agent: &mut HeuristicAgent, f: impl FnOnce(&mut HeuristicAgent)) -> usize {
        let before = agent.rng.clone();
        f(agent);
        let after_next = agent.rng.clone().next_u64();
        for k in 0..=6 {
            let mut probe = before.clone();
            for _ in 0..k {
                probe.next_u64();
            }
            if probe.next_u64() == after_next {
                return k;
            }
        }
        panic!("consumed more than 6 draws, or the stream diverged");
    }

    /// Java `ParityRunner`: a pass-block window mover is deselected immediately, because the
    /// engine flow never re-presents INIT_SELECTING phase 2 for him. `RandomAgent` has always
    /// mirrored this; the heuristic knew nothing about pass-block windows, which is precisely the
    /// roster that opens them — On the Ball is the Amazon Thrower's skill in bb2020 and bb2025.
    #[test]
    fn a_move_prompt_in_a_pass_block_window_deselects() {
        use ffb_model::enums::TurnMode;
        let mut g = tie_game(3, 3);
        let f = Features::build(&g, positions_stamp(&g), true);
        let pid = g.team_home.players[0].id.clone();
        let mut a = HeuristicAgent::new(1, 1.0);

        g.turn_mode = TurnMode::PassBlock;
        assert!(
            matches!(
                a.handle_move(&g, &f, pid.clone(), vec![FieldCoordinate::new(13, 8)]),
                Action::EndPlayerAction
            ),
            "a pass-block window mover must be deselected, not moved"
        );

        // ...and the rule is confined to that window: a REGULAR turn still decides normally, or
        // the fix would freeze every activation in the game.
        g.turn_mode = TurnMode::Regular;
        assert!(
            !matches!(
                a.handle_move(&g, &f, pid.clone(), vec![FieldCoordinate::new(13, 8)]),
                Action::EndPlayerAction
            ) || a.plan.is_none(),
            "the deselect must not leak into a regular turn"
        );
    }

    /// Java's board for the heuristic (`ActivationDriver.foes`) keeps an opponent only while
    /// `hasTacklezones()`, which is false for a CONFUSED player. A hexed player is STANDING and
    /// confused, so Java stops offering him as a block target; Rust's `legal_block_targets` answers
    /// the RANDOM contract's question (`can_be_blocked`) and does not check confusion, so the
    /// heuristic must filter. Making `legal_block_targets` itself Java-faithful is NOT an option:
    /// it took `--agent random` bb2025 amazon from 100 to 93/100.
    #[test]
    fn the_heuristic_does_not_block_a_confused_opponent() {
        use ffb_model::enums::{PS_STANDING, PlayerState};
        let mut g = tie_game(3, 3);
        let attacker = g.team_home.players[0].id.clone();
        let victim = g.team_away.players[0].id.clone();
        let here = g.field_model.player_coordinate(&attacker).unwrap();
        g.field_model
            .set_player_coordinate(&victim, FieldCoordinate::new(here.x + 1, here.y));
        g.field_model.set_player_state(&victim, PlayerState::new(PS_STANDING));

        let visible = |g: &Game| {
            legal_block_targets(g, &attacker, TeamSide::Home)
                .into_iter()
                .filter(|t| {
                    g.field_model.player_state(t).map(|s| s.has_tacklezones()).unwrap_or(false)
                })
                .collect::<Vec<_>>()
        };
        assert!(visible(&g).contains(&victim), "a standing neighbour is blockable");

        let st = g.field_model.player_state(&victim).unwrap();
        g.field_model.set_player_state(&victim, st.change_confused(true).change_active(false));
        assert!(
            !visible(&g).contains(&victim),
            "a hexed (confused) neighbour projects no tackle zone and is not a block target"
        );
        assert!(
            legal_block_targets(&g, &attacker, TeamSide::Home).contains(&victim),
            "the shared helper must keep him -- the random contract depends on it"
        );
    }

    fn agent_with_options(temp_scale: f32, n: usize) -> HeuristicAgent {
        let mut a = HeuristicAgent::new(1, temp_scale);
        for i in 0..n {
            a.buf.push(Action::EndTurn, i as f32 * 0.1, Rule::Flat, 0.0);
        }
        a
    }

    /// The draw count per decision is NOT constant, and every branch of it is load-bearing: the
    /// Java twin has to consume exactly as many, or the two streams desynchronise and every
    /// decision after that point is unrelated. Pins the table in
    /// `AGENT_CONTRACT_HEURISTIC.md` section 2.
    #[test]
    fn pick_draw_counts_match_the_contract() {
        // n <= 1: nothing to decide, so nothing is drawn -- at ANY temperature.
        for ts in [0.0, 0.05, 1.0, 1.0e6] {
            for n in [0usize, 1] {
                let mut a = agent_with_options(ts, n);
                assert_eq!(draws_used(&mut a, |a| { a.pick(0.15); }), 0, "ts={ts} n={n}");
            }
        }
        // argmax consumes nothing at all -- this is what makes `--heuristic 0` the cleanest
        // first rung of the parity ladder: no sampler to disagree about.
        let mut a = agent_with_options(0.0, 5);
        assert_eq!(draws_used(&mut a, |a| { a.pick(0.15); }), 0);

        // 0 < temp_scale < 0.1 disables the eps escape, and `eps > 0.0 &&` short-circuits so the
        // probe draw is never taken: exactly one draw.
        let mut a = agent_with_options(0.05, 5);
        assert_eq!(draws_used(&mut a, |a| { a.pick(0.15); }), 1);

        // temp_scale >= 0.1 always costs exactly two: the eps probe, then either the uniform
        // escape or the cumulative draw. Both branches must cost the same.
        for ts in [0.1, 1.0, 1.0e6] {
            let mut a = agent_with_options(ts, 5);
            assert_eq!(draws_used(&mut a, |a| { a.pick(0.15); }), 2, "ts={ts}");
        }
    }

    /// `softmax_pick` has no eps escape and always costs exactly one draw when it decides.
    #[test]
    fn softmax_pick_draw_counts_match_the_contract() {
        let w = [0.1f32, 0.5, 0.9];
        for ts in [0.1, 1.0, 1.0e6] {
            let mut a = agent_with_options(ts, 0);
            assert_eq!(draws_used(&mut a, |a| { a.softmax_pick(&w, 0.18); }), 1, "ts={ts}");
        }
        // Degenerate option sets and argmax draw nothing.
        let mut a = agent_with_options(1.0, 0);
        assert_eq!(draws_used(&mut a, |a| { a.softmax_pick(&[], 0.18); }), 0);
        let mut a = agent_with_options(1.0, 0);
        assert_eq!(draws_used(&mut a, |a| { a.softmax_pick(&[0.5], 0.18); }), 0);
        let mut a = agent_with_options(0.0, 0);
        assert_eq!(draws_used(&mut a, |a| { a.softmax_pick(&w, 0.18); }), 0);
    }

    // ── determinism / portability of the ordering key ──────────────────────────

    /// Two opponents of DIFFERENT strength, both adjacent to the same square, tie on `reach`
    /// (`d == 1` → `1.0`). `build_threat` writes `threat_str` under a strict `>`, so before
    /// `canon_players` the winner was whichever the randomly-seeded `HashMap` hasher yielded
    /// first — measured: every one of 20 human-vs-human seeds produced a DIFFERENT event stream
    /// between two runs of the same binary. Now the canonically-first (lowest `(side, nr)`)
    /// opponent wins, deterministically and identically in both engines.
    #[test]
    fn threat_str_tie_is_broken_canonically_not_by_hash_order() {
        let (weak_first, strong_first) = (build_tie_board(3, 5), build_tie_board(5, 3));
        // The square between the two opponents, whose threat the mover would read.
        let i = ix(10, 7);
        // away_01 is canonically first either way, so its ST is what lands in `threat_str`.
        assert_eq!(weak_first.threat_str[0][i], 3, "away_01 has ST3 here");
        assert_eq!(strong_first.threat_str[0][i], 5, "away_01 has ST5 here");
        // Both opponents are adjacent, so the reach itself saturates regardless.
        assert_eq!(weak_first.threat_reach[0][i], 1.0);
        assert_eq!(strong_first.threat_reach[0][i], 1.0);
    }

    /// `canon_players` must be sorted by `(side, nr)` — home before away, then jersey number.
    #[test]
    fn canon_players_is_sorted_by_side_then_nr() {
        let g = tie_game(3, 5);
        let keys: Vec<(u8, i32)> =
            canon_players(&g).iter().map(|(id, _)| canon_key(&g, id)).collect();
        let mut sorted = keys.clone();
        sorted.sort();
        assert_eq!(keys, sorted, "canon_players must already be in canonical order");
        assert!(keys.windows(2).all(|w| w[0] != w[1]), "no two on-pitch players may collide");
    }

    /// The claim that made the swap safe to land, stated precisely.
    ///
    /// All seven replaced sorts are SINGLE-SIDED — `c1`/`ps` hold only the acting team,
    /// `foes`/`ep` only opponents, `mates` only team-mates, and `Touchback` only the receiving
    /// team — and *within a side* `(side, nr)` reproduces the old lexicographic order of
    /// `home_NN`/`away_NN` ids exactly. So no decision changes on any roster whose ids follow
    /// that scheme, i.e. every roster not carrying a star.
    ///
    /// Across sides the two orders genuinely differ (canonical puts home first, lexicographic
    /// puts `away_*` first), which is harmless precisely because no site mixes sides — and is
    /// why this test asserts per-side rather than whole-roster.
    ///
    /// Verified end-to-end as well: the lineman AND human 100-seed event streams were
    /// byte-identical before and after the swap.
    #[test]
    fn canon_key_reproduces_id_order_within_a_side() {
        let g = tie_game(3, 3);
        for home in [true, false] {
            let mut by_id: Vec<String> = canon_players(&g)
                .into_iter()
                .map(|(id, _)| id)
                .filter(|id| g.team_home.has_player(id) == home)
                .collect();
            let by_canon = by_id.clone();
            by_id.sort();
            assert_eq!(by_canon, by_id, "side home={home}");
        }
    }

    fn tie_game(st_away_01: i32, st_away_02: i32) -> Game {
        use ffb_model::enums::Rules;
        use ffb_model::model::player::Player;
        use ffb_model::enums::{PlayerState, PS_STANDING};

        let mk = |id: &str, nr: i32, st: i32| Player {
            id: id.into(),
            nr,
            movement: 6,
            strength: st,
            agility: 3,
            armour: 8,
            ..Default::default()
        };
        let mut home = crate::step::framework::test_team("home", 0);
        let mut away = crate::step::framework::test_team("away", 0);
        home.players.push(mk("home_01", 1, 3));
        away.players.push(mk("away_01", 1, st_away_01));
        away.players.push(mk("away_02", 2, st_away_02));

        let mut g = Game::new(home, away, Rules::Bb2025);
        // Both opponents stand ADJACENT to (10, 7) so their `reach` there ties at 1.0.
        let place = |g: &mut Game, id: &str, x: i32, y: i32| {
            g.field_model.set_player_coordinate(id, FieldCoordinate::new(x, y));
            g.field_model.set_player_state(id, PlayerState::new(PS_STANDING));
        };
        place(&mut g, "home_01", 2, 7);
        place(&mut g, "away_01", 9, 7);
        place(&mut g, "away_02", 11, 7);
        g
    }

    fn build_tie_board(st_away_01: i32, st_away_02: i32) -> Features {
        let g = tie_game(st_away_01, st_away_02);
        Features::build(&g, positions_stamp(&g), true)
    }

    /// §20.1 — the plan kinds that end the activation for free are exactly the ones with no
    /// terminal action left to send.
    #[test]
    fn only_terminal_free_plans_end_eagerly() {
        let ends = |k: &PlanKind| matches!(k, PlanKind::Move | PlanKind::Immediate);
        assert!(ends(&PlanKind::Move));
        assert!(ends(&PlanKind::Immediate));
        assert!(!ends(&PlanKind::Pickup));
        assert!(!ends(&PlanKind::Blitz { victim: "x".into() }));
        assert!(!ends(&PlanKind::Pass { receiver: "x".into() }));
        assert!(!ends(&PlanKind::HandOff { receiver: "x".into() }));
        assert!(!ends(&PlanKind::Foul { victim: "x".into() }));
    }
}
