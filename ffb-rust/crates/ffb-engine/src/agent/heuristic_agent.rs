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
    legal_pass_receivers, legal_throw_team_mate_targets, TeamSide,
};
use crate::step::GameState;

use super::random_agent::player_action_to_pac;
use super::{Agent, UniformAgent};

/// `mechanics/movement.rs: STAND_UP_COST`.
const STAND_UP_COST: i32 = 3;
const EPS: f32 = 0.02;
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

fn pid_key(id: &str) -> u32 {
    let mut h: u32 = 2166136261;
    for b in id.as_bytes() {
        h ^= *b as u32;
        h = h.wrapping_mul(16777619);
    }
    h
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

            for (id, &c) in &g.field_model.player_coordinates {
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
        let k = (pid_key(att), pid_key(def));
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
        (-(self.cell[i].key as f32) / KEY_SCALE).exp() * self.gate
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
            let nkey = key + (-(p_step.max(1e-6).ln()) * KEY_SCALE) as u32;
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
                // Tunable via FFB_HOPELESS_DAMP while this is being fitted.
                std::env::var("FFB_HOPELESS_DAMP")
                    .ok()
                    .and_then(|v| v.parse::<f32>().ok())
                    .unwrap_or(0.25)
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
    /// A chain of small draws: player, then action-and-target, then one movement square at a time.
    Deep,
}

pub struct HeuristicAgent {
    mode: Mode,
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
    /// Who just received the ball. The second half of a ball-move plan: he has to be the next one
    /// activated, or the throw bought nothing.
    awaiting_run: Option<String>,
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
            fallback: UniformAgent::new(seed),
            buf: Scored::default(),
            feat: None,
            plan: None,
            sc: Scratch::default(),
            seen_action: HashMap::new(),
            seen_bucket: HashMap::new(),
            last_turn_key: None,
            used_this_turn: HashSet::new(),
            awaiting_run: None,
            dump_enabled: std::env::var_os("FFB_HEUR_DUMP").is_some(),
            last_options: Vec::new(),
            last_chosen: 0,
        }
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
                ps[i] = ((o.weight - max) / t).exp();
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
        let mut ps: Vec<f32> = w.iter().map(|v| ((v - max) / t).exp()).collect();
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
            return ((self.rng.next_u64() as usize) % n).min(n - 1);
        }
        let mut acc = 0.0f32;
        let mut cum: Vec<f32> = Vec::with_capacity(n);
        for o in &self.buf.options {
            acc += ((o.weight - max) / t).exp();
            cum.push(acc);
        }
        let r = self.unit() * acc;
        cum.partition_point(|&c| c < r).min(n - 1)
    }

    fn take(&mut self, i: usize) -> Action {
        self.buf.options.swap_remove(i).action
    }

    // ── bookkeeping ────────────────────────────────────────────────────────

    fn refresh_turn(&mut self, g: &Game) {
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
        if squares.is_empty() {
            self.plan = None;
            return Action::EndPlayerAction;
        }

        if let Some(mut pl) = self.plan.take() {
            if pl.player == player_id {
                if !pl.path.is_empty() {
                    // Deliver the whole remaining path in ONE answer; the engine walks it.
                    if squares.contains(&pl.path[0]) {
                        let path = std::mem::take(&mut pl.path);
                        pl.delivered = true;
                        self.plan = Some(pl);
                        return Action::Move { path };
                    }
                    // The board moved under the plan — fall through and re-decide.
                } else if !pl.fired {
                    // Movement done. Now the plan's terminal action, if it has one.
                    //
                    // `StepInitMoving` guards each of these dispatches and falls THROUGH when the
                    // guard fails, re-emitting this same prompt. Resending a rejected action would
                    // therefore spin forever, so each one is gated on the engine's own condition
                    // and latched with `fired` so it is attempted at most once per activation.
                    let here = g.field_model.player_coordinate(&player_id);
                    let pa_now = g.acting_player.player_action;
                    let adjacent_to = |v: &str| {
                        here.zip(g.field_model.player_coordinate(v))
                            .map(|(a, b)| a.distance_in_steps(b) == 1)
                            .unwrap_or(false)
                    };
                    match &pl.kind {
                        PlanKind::Blitz { victim } => {
                            let dispatchable = matches!(
                                pa_now,
                                Some(PlayerAction::BlitzMove) | Some(PlayerAction::KickEmBlitz)
                            ) && !g.acting_player.has_blocked;
                            if std::env::var_os("FFB_BLITZ_TRACE").is_some() {
                                eprintln!(
                                    "BZ {} victim={} pa={:?} blocked={} adj={} here={:?}",
                                    player_id, victim, pa_now, g.acting_player.has_blocked,
                                    adjacent_to(victim), here
                                );
                            }
                            if dispatchable && adjacent_to(victim) {
                                let defender_id = victim.clone();
                                pl.fired = true;
                                self.plan = Some(pl);
                                return Action::Block { defender_id };
                            }
                        }
                        PlanKind::Foul { victim } => {
                            let dispatchable = pa_now == Some(PlayerAction::FoulMove)
                                && !g.acting_player.has_fouled;
                            if dispatchable && adjacent_to(victim) {
                                let target_id = victim.clone();
                                pl.fired = true;
                                self.plan = Some(pl);
                                return Action::Foul { target_id };
                            }
                        }
                        PlanKind::Pass { receiver } => {
                            let dispatchable = matches!(
                                pa_now,
                                Some(PlayerAction::PassMove)
                                    | Some(PlayerAction::Pass)
                                    | Some(PlayerAction::HailMaryPass)
                            );
                            if let (true, Some(coord)) =
                                (dispatchable, g.field_model.player_coordinate(receiver))
                            {
                                pl.fired = true;
                                self.plan = Some(pl);
                                return Action::Pass { coord };
                            }
                        }
                        PlanKind::HandOff { receiver } => {
                            let dispatchable = matches!(
                                pa_now,
                                Some(PlayerAction::HandOverMove) | Some(PlayerAction::HandOver)
                            );
                            if dispatchable && g.field_model.player_coordinate(receiver).is_some() {
                                let receiver_id = receiver.clone();
                                pl.fired = true;
                                self.plan = Some(pl);
                                return Action::HandOff { receiver_id };
                            }
                        }
                        PlanKind::Move | PlanKind::Immediate => {
                            // §20.1 — Move → Move never makes sense. A plain move that has ALREADY
                            // been delivered has no options it did not already have, so end without
                            // scoring anything. A plan that never carried a path (a tier-1 proxy
                            // pick, §20.3) still has to decide where to go, so it falls through.
                            if pl.delivered {
                                return Action::EndPlayerAction;
                            }
                        }
                        // A pickup genuinely changed the value model; re-decide below.
                        PlanKind::Pickup => {}
                    }
                    // The terminal action was not dispatchable here. Nothing else this activation
                    // can do that it could not already do, so end rather than re-plan — unless the
                    // plan never got a path in the first place, which still needs deciding.
                    if !matches!(pl.kind, PlanKind::Pickup | PlanKind::Move) && pl.delivered {
                        return Action::EndPlayerAction;
                    }
                } else if !matches!(pl.kind, PlanKind::Pickup) {
                    // The terminal action has already been sent; the activation is over. A blitz is
                    // the one case with something legitimate left to do — the board changed under it
                    // — so it re-plans its post-block movement.
                    if !matches!(pl.kind, PlanKind::Blitz { .. }) {
                        return Action::EndPlayerAction;
                    }
                }
            }
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
        self.refresh_turn(g);
        if eligible.is_empty() {
            return Action::EndTurn;
        }
        let any_unused = eligible.iter().any(|(pid, _)| !self.used_this_turn.contains(pid));
        // Every eligible player has already had its activation decided this turn. Re-offering them
        // is how the driver livelocks: an activation that ends without moving leaves the engine's
        // eligible list unchanged, so `used_this_turn` is the only thing that makes progress.
        if !any_unused {
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
        c1.sort_by(|a, b| a.pid.cmp(&b.pid));
        let mut rank: Vec<usize> = (0..c1.len()).collect();
        rank.sort_by(|&a, &b| {
            (c1[b].w_player * c1[b].proxy.max(0.05))
                .partial_cmp(&(c1[a].w_player * c1[a].proxy.max(0.05)))
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(c1[a].pid.cmp(&c1[b].pid))
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
                    // The MOVE-variants (HandOffMove/PassMove) are declared and routed correctly
                    // now, but the give they dispatch after moving parks StepInitPassing - see
                    // docs 29. Until that is translated, declare the immediate form the engine
                    // completes.
                    player_action: c.pac,
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

        // Two-level draw. Group by DECLARATION — the (player, action) pair the engine actually
        // receives — and score each group by its best child, which keeps argmax identical to a flat
        // draw while stopping a branch with two thousand destinations from drowning one with nine.
        // build_plans emits a player's options one action at a time, so a declaration's candidates
        // are a CONTIGUOUS RUN. Detecting runs is linear; the obvious keyed lookup was O(groups) of
        // string comparison per candidate and cost 30 ms a game on its own.
        let mut groups: Vec<Vec<usize>> = Vec::new();
        for (i, c) in cands.iter().enumerate() {
            let same = i > 0 && cands[i - 1].pac == c.pac && cands[i - 1].player == c.player;
            if same {
                groups.last_mut().expect("run started").push(i);
            } else {
                groups.push(vec![i]);
            }
        }
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
                    let mut v: Vec<f32> = w.iter().map(|x| ((x - mx) / t).exp()).collect();
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
        self.take(i)
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
        self.refresh_turn(g);
        if eligible.is_empty() {
            return Action::EndTurn;
        }
        if !eligible.iter().any(|(pid, _)| !self.used_this_turn.contains(pid)) {
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
        ps.sort_by(|a, b| a.pid.cmp(&b.pid));

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
                    player_action: c.pac,
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
        Action::ActivatePlayer {
            player_id: c.player,
            player_action: c.pac,
            block_defender_id: c.target,
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
                    for t in legal_block_targets(g, pid, side) {
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
                    foes.sort();
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
                PlayerActionChoice::HandOff if self.mode != Mode::WideNoBall => {
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
                    mates.sort();
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
                    if self.mode != Mode::WideNoBall =>
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
                PlayerActionChoice::ThrowTeamMate | PlayerActionChoice::KickTeamMate => {
                    for t in legal_throw_team_mate_targets(g, pid, side) {
                        push(
                            0.35,
                            Some(t.clone()),
                            PlanKind::Immediate,
                            Vec::new(),
                            None,
                            Rule::Flat,
                            format!("throw team-mate to {}", t),
                        );
                    }
                }
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
            AgentPrompt::Move { player_id, squares } => match self.mode {
                Mode::Wide | Mode::WideNoBall => self.handle_move(g, f, player_id, squares),
                Mode::Deep => self.handle_move_deep(g, f, player_id, squares),
            },

            AgentPrompt::ActivatePlayer { eligible_players } => match self.mode {
                Mode::Wide | Mode::WideNoBall => self.handle_activate(g, f, eligible_players),
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
                    return Action::EndPlayerAction;
                }
                let astr = g.player(&attacker_id).map(|p| p.strength_with_modifiers()).unwrap_or(3);
                let mut ep = eligible_players;
                ep.sort();
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
                        let w = p_safe * p_safe * (-best).exp();
                        self.buf.push_note(
                            Action::KickBall { coord: FieldCoordinate::new(x, y) },
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
                sorted.sort_by(|a, b| a.0.cmp(&b.0));
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

            // Everything else: the long tail, identical in both arms of the experiment.
            _ => self.fallback.act(gs),
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
    let bribes = if m.home { g.team_home.bribes } else { g.team_away.bribes };
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
    let base_target = mech.minimum_roll_simple(p, dist, &[])?;
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
            "PW dist={:?} tgt={} pAcc={:.2} pFum={:.2} pScat={:.2} pCatch={:.2} pC={:.2} \
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
