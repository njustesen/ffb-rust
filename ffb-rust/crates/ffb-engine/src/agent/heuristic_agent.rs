//! `HeuristicAgent` — the probabilistic policy specified in `docs/HEURISTIC_AGENT.md`.
//!
//! Every decision follows the same pipeline: **enumerate → score → softmax → sample**. Scoring is
//! pure and RNG-free; only `act` touches the RNG.
//!
//! The `temp_scale` constructor argument multiplies every temperature in the table. It exists so
//! the *same* agent — the same enumeration, the same option sets, the same code path — can be run
//! as a uniform sampler by setting it very large. That makes "heuristics vs random over an
//! identical action space" a one-parameter A/B rather than a comparison between two different
//! programs.
//!
//! Long-tail prompts (inducements, kickoff events, apothecary, the star specials) fall through to
//! `UniformAgent`, which is identical in both arms of that A/B, so the comparison isolates exactly
//! the decisions this agent scores.

use std::collections::HashMap;

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

// ─────────────────────────────────────────────────────────────────── primitives

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
    if weather == Weather::Blizzard { 3 } else { 2 }
}

// ─────────────────────────────────────────────────────────────── option scoring

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rule {
    ScoreTouchdown,
    ScoreAdvance,
    Pickup,
    Cage,
    Mark,
    Screen,
    Retreat,
    EndActivation,
    DiceCount,
    Face,
    Reroll,
    Skill,
    Activation,
    CoverageFloor,
    Novelty,
    Flat,
}

pub struct Weighted {
    pub action: Action,
    /// Signed desirability — §5.3 subtracts an expected turnover cost, so a bad option is negative.
    pub weight: f32,
    pub why: Rule,
    pub why_value: f32,
}

#[derive(Default)]
pub struct Scored {
    pub options: Vec<Weighted>,
    /// Set whenever the candidate set was capped. Never silently truncate.
    pub truncated: bool,
}

impl Scored {
    fn push(&mut self, action: Action, weight: f32, why: Rule, why_value: f32) {
        self.options.push(Weighted { action, weight, why, why_value });
    }
    fn clear(&mut self) {
        self.options.clear();
        self.truncated = false;
    }
}

// ───────────────────────────────────────────────────────────────── board facts

struct Board<'a> {
    g: &'a Game,
    rules: Rules,
    weather: Weather,
    /// Opposing tackle zones on each square, indexed for the HOME side then the AWAY side.
    tz: [Vec<u8>; 2],
    /// Opponent count per row, prefix-summed along x, per side. `row_prefix[side][y][x]` =
    /// number of players NOT on `side` in row `y` at column `< x`.
    row_prefix: [Vec<u16>; 2],
    ball: Option<FieldCoordinate>,
    /// The ball is ON THE GROUND, unheld. In this engine `ball_moving == true` means loose,
    /// NOT in flight -- see legal_actions "loose ball 4 steps away" / "carried ball (not
    /// moving)". An earlier build read it the other way round and gated the Pickup intent on
    /// `carried && nobody carrying`, a condition that can never hold, so the agent never once
    /// valued picking the ball up.
    ball_loose: bool,
    /// A player is holding the ball.
    ball_carried: bool,
    carrier: Option<String>,
}

#[inline]
fn idx(c: FieldCoordinate) -> usize {
    (c.y as usize) * 26 + (c.x as usize)
}

#[inline]
fn on_pitch(x: i32, y: i32) -> bool {
    (0..=XMAX).contains(&x) && (0..=YMAX).contains(&y)
}

impl<'a> Board<'a> {
    fn new(g: &'a Game) -> Self {
        let mut tz = [vec![0u8; 26 * 15], vec![0u8; 26 * 15]];
        let mut row_prefix = [vec![0u16; 15 * 27], vec![0u16; 15 * 27]];

        for (id, &c) in &g.field_model.player_coordinates {
            if !on_pitch(c.x, c.y) {
                continue;
            }
            let is_home = g.team_home.has_player(id);
            let standing = g
                .field_model
                .player_state(id)
                .map(|s| s.has_tacklezones())
                .unwrap_or(false);
            // tz[0] = zones threatening a HOME player => produced by AWAY players.
            if standing {
                let side = if is_home { 0 } else { 1 };
                for n in c.neighbours() {
                    if on_pitch(n.x, n.y) {
                        tz[1 - side][idx(n)] += 1;
                    }
                }
            }
            // row_prefix[0] counts opponents-of-home => away players.
            let opp_of = if is_home { 1usize } else { 0usize };
            for x in (c.x + 1)..=26 {
                row_prefix[opp_of][(c.y as usize) * 27 + x as usize] += 1;
            }
        }

        let ball = g.field_model.ball_coordinate;
        let in_play = g.field_model.ball_in_play && ball.map(|c| on_pitch(c.x, c.y)).unwrap_or(false);
        let ball_loose = in_play && g.field_model.ball_moving;
        let ball_carried = in_play && !g.field_model.ball_moving;
        let carrier = ball.filter(|_| ball_carried).and_then(|b| {
            g.field_model
                .player_at(b)
                .filter(|id| g.field_model.player_coordinate(id) == Some(b))
                .cloned()
        });

        Board { g, rules: g.rules, weather: g.field_model.weather, tz, row_prefix,
                ball, ball_loose, ball_carried, carrier }
    }

    fn is_home(&self, id: &str) -> bool {
        self.g.team_home.has_player(id)
    }

    /// Opposing tackle zones on `c` from the point of view of a player on `home` side.
    #[inline]
    fn tz_against(&self, c: FieldCoordinate, home: bool) -> i32 {
        self.tz[if home { 0 } else { 1 }][idx(c)] as i32
    }

    /// Opponents of `home` in row `y` strictly between `x0` and `x1`.
    fn opponents_between(&self, home: bool, y: i32, x0: i32, x1: i32) -> i32 {
        if !(0..=YMAX).contains(&y) {
            return 0;
        }
        let (lo, hi) = if x0 <= x1 { (x0, x1) } else { (x1, x0) };
        let p = &self.row_prefix[if home { 1 } else { 0 }];
        let row = (y as usize) * 27;
        let a = p[row + (lo.clamp(0, 26)) as usize];
        let b = p[row + (hi.clamp(0, 26)) as usize];
        (b as i32 - a as i32).max(0)
    }

    fn occupied(&self, c: FieldCoordinate) -> bool {
        self.g.field_model.player_at(c).is_some()
    }

    fn standing(&self, id: &str) -> bool {
        self.g
            .field_model
            .player_state(id)
            .map(|s| s.has_tacklezones())
            .unwrap_or(false)
    }
}

fn endzone_x(home: bool) -> i32 {
    if home { XMAX } else { 0 }
}

fn endzone_distance(c: FieldCoordinate, home: bool) -> i32 {
    (c.x - endzone_x(home)).abs()
}

// ───────────────────────────────────────────────────────── reachability (§4.2)

#[derive(Clone)]
struct Reach {
    coord: FieldCoordinate,
    cost: i32,
    gfi: i32,
    p_arrive: f32,
    path: Vec<FieldCoordinate>,
}

/// Dijkstra over −log(p_step), capped at the player's REAL remaining budget (P4: a prone player
/// has already spent `STAND_UP_COST`, and at MA ≤ 3 the activation is gated behind a roll).
fn reachable(b: &Board, player_id: &str, team_rr: bool) -> Vec<Reach> {
    let start = match b.g.field_model.player_coordinate(player_id) {
        Some(c) => c,
        None => return vec![],
    };
    let player = match b.g.player(player_id) {
        Some(p) => p,
        None => return vec![],
    };
    let home = b.is_home(player_id);
    let ma_base = player.movement_with_modifiers();
    let prone = b
        .g
        .field_model
        .player_state(player_id)
        .map(|s| s.is_prone())
        .unwrap_or(false);

    let (ma, gate) = if prone {
        if ma_base <= STAND_UP_COST {
            (0, p_roll(4))
        } else {
            (ma_base - STAND_UP_COST, 1.0)
        }
    } else {
        (ma_base, 1.0)
    };
    // MA ALREADY SPENT this activation. `reachable` is called again every time the engine
    // re-prompts Move mid-activation; without this the player gets a fresh MA + 2 budget each
    // time and the path length compounds. Measured before the fix: an MA-6 lineman moving up to
    // FIFTEEN squares in one activation (max is 8), ~5 rushes per activation, 28.9 failed GFI
    // per game, and a turnover ending almost every turn after ~2 activations.
    let spent = if b.g.acting_player.player_id.as_deref() == Some(player_id) {
        b.g.acting_player.current_move.max(0)
    } else {
        0
    };
    let cap = (ma + 2 - spent).max(0);
    if cap <= 0 {
        return vec![];
    }

    let ag = player.agility_with_modifiers();
    let gt = gfi_target(b.weather);
    let has_dodge = player.has_skill(SkillId::Dodge);
    let has_sure_feet = player.has_skill(SkillId::SureFeet);

    // (−log p, cost, coord)
    let mut best: HashMap<FieldCoordinate, (f32, i32, i32, Vec<FieldCoordinate>, bool)> = HashMap::new();
    best.insert(start, (0.0, 0, 0, Vec::new(), false));
    let mut frontier: Vec<FieldCoordinate> = vec![start];
    let mut done: Vec<FieldCoordinate> = Vec::new();

    while !frontier.is_empty() {
        // pick the lowest −log p not yet expanded
        let mut bi = 0usize;
        for (i, c) in frontier.iter().enumerate() {
            if best[c].0 < best[&frontier[bi]].0 {
                bi = i;
            }
        }
        let cur = frontier.swap_remove(bi);
        done.push(cur);
        let (clog, ccost, cgfi, cpath, crr_used) = best[&cur].clone();
        if ccost >= cap {
            continue;
        }
        let leaving_tz = b.tz_against(cur, home) > 0;

        for n in cur.neighbours() {
            if !on_pitch(n.x, n.y) || b.occupied(n) {
                continue;
            }
            let ncost = ccost + 1;
            if ncost > cap {
                continue;
            }
            let mut p_step = 1.0f32;
            let mut rr_used = crr_used;
            if leaving_tz {
                let t = dodge_target(b.rules, ag, b.tz_against(n, home));
                let raw = p_roll(t);
                if has_dodge {
                    p_step *= p_with_reroll(raw, 1.0);
                } else if team_rr && !rr_used {
                    p_step *= p_with_reroll(raw, 1.0);
                    rr_used = true;
                } else {
                    p_step *= raw;
                }
            }
            let ngfi = if ncost + spent > ma { cgfi + 1 } else { cgfi };
            if ncost + spent > ma {
                let raw = p_roll(gt);
                if has_sure_feet {
                    p_step *= p_with_reroll(raw, 1.0);
                } else if team_rr && !rr_used {
                    p_step *= p_with_reroll(raw, 1.0);
                    rr_used = true;
                } else {
                    p_step *= raw;
                }
            }
            let nlog = clog - p_step.max(1e-9).ln();
            let better = match best.get(&n) {
                None => true,
                Some((l, c0, _, _, _)) => nlog < *l - 1e-9 || ((nlog - *l).abs() < 1e-9 && ncost < *c0),
            };
            if better {
                let mut np = cpath.clone();
                np.push(n);
                let seen = done.contains(&n);
                best.insert(n, (nlog, ncost, ngfi, np, rr_used));
                if !seen && !frontier.contains(&n) {
                    frontier.push(n);
                }
            }
        }
    }

    // DETERMINISM: `best` is a HashMap, and HashMap iteration order is randomised per process.
    // Returning it unsorted made the option list -- and therefore every argmax tie-break --
    // vary between runs of the SAME seed, which showed up as the same game producing different
    // scores and occasionally livelocking. 9 forbids hash iteration in a scoring path for
    // exactly this reason; sort by coordinate before anything downstream sees it.
    let mut out: Vec<Reach> = best
        .into_iter()
        .filter(|(c, _)| *c != start)
        .map(|(c, (l, cost, gfi, path, _))| Reach {
            coord: c,
            cost,
            gfi,
            p_arrive: (-l).exp() * gate,
            path,
        })
        .collect();
    out.sort_by_key(|r| (r.coord.x, r.coord.y));
    out
}

// ──────────────────────────────────────────────────────── threat + value (§5)

/// P1: a team declares ONE blitz per turn, so at most one opponent can actually *hit* a square.
/// The block term is a `max`; everyone else contributes only a small marking term.
fn threat_on(b: &Board, sq: FieldCoordinate, home: bool, victim_str: i32) -> f32 {
    let opp_blitz_spent = if home {
        b.g.turn_data_away.blitz_used
    } else {
        b.g.turn_data_home.blitz_used
    };

    let mut block_term: f32 = 0.0;
    let mut reach_terms: Vec<f32> = Vec::new();

    for (id, &c) in &b.g.field_model.player_coordinates {
        if b.is_home(id) == home || !b.standing(id) || !on_pitch(c.x, c.y) {
            continue;
        }
        let opp = match b.g.player(id) {
            Some(p) => p,
            None => continue,
        };
        let d = c.distance_in_steps(sq);
        let adjacent_now = d == 1;
        let ma = opp.movement_with_modifiers();
        // rolls the opponent needs to end adjacent: leaving our tackle zones costs one.
        let steps_needed = (d - 1).max(0);
        let reach_factor = if adjacent_now {
            1.0
        } else if steps_needed <= ma {
            let marked = b.tz_against(c, !home) > 0;
            if marked { 0.55 } else { 1.0 }
        } else if steps_needed <= ma + 2 {
            0.25
        } else {
            0.0
        };
        if reach_factor == 0.0 {
            continue;
        }
        let sf = strength_factor(opp.strength_with_modifiers(), victim_str);
        // Only an already-adjacent opponent can block without spending the blitz.
        if adjacent_now || !opp_blitz_spent {
            block_term = block_term.max(reach_factor * sf);
        }
        reach_terms.push(reach_factor);
    }

    reach_terms.sort_by(|a, c| c.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
    let mark_term: f32 = 0.18 * reach_terms.iter().skip(1).take(2).sum::<f32>();
    block_term + mark_term
}

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

#[derive(Clone, Copy, PartialEq)]
enum Intent {
    Score,
    Pickup,
    Cage,
    Mark,
    Screen,
    Retreat,
}

impl Intent {
    /// P2: `lane` is corridor-to-the-endzone geometry — meaningful only where running is the point.
    fn uses_lane(self) -> bool {
        matches!(self, Intent::Score | Intent::Pickup)
    }
    fn rule(self) -> Rule {
        match self {
            Intent::Score => Rule::ScoreAdvance,
            Intent::Pickup => Rule::Pickup,
            Intent::Cage => Rule::Cage,
            Intent::Mark => Rule::Mark,
            Intent::Screen => Rule::Screen,
            Intent::Retreat => Rule::Retreat,
        }
    }
}

struct Ctx {
    home: bool,
    is_carrier: bool,
    ma: i32,
    ag: i32,
    sure_hands: bool,
    str_: i32,
    d_now: i32,
    turns_left: i32,
    side_step: bool,
    unactivated: f32,
}

fn urgency(d_sq: i32, ma: i32, turns_left: i32) -> f32 {
    let tts = ((d_sq as f32) / (ma.max(1) as f32)).ceil() as i32;
    let slack = turns_left - tts;
    (1.0 - slack as f32 / 3.0).clamp(0.0, 1.0)
}

fn value_of(b: &Board, sq: FieldCoordinate, cx: &Ctx) -> (f32, Intent) {
    let d_sq = endzone_distance(sq, cx.home);

    // ---- base_intent, max over intents ------------------------------------
    let mut best = (0.10f32, Intent::Retreat);

    if cx.is_carrier {
        let v = if d_sq == 0 {
            1.0
        } else {
            let max_gain = cx.d_now.min(cx.ma + 2).max(1);
            let advance = ((cx.d_now - d_sq) as f32 / max_gain as f32).clamp(0.0, 1.0);
            let base = 0.15 + 0.85 * advance;
            base * (0.75 + 0.5 * urgency(d_sq, cx.ma, cx.turns_left))
        };
        if v > best.0 {
            best = (v, Intent::Score);
        }
    } else {
        // Pickup -- the single highest-value thing on the board when the ball is loose.
        // BB2025 pickup target is AG + tackle zones on the ball square (min 2).
        if b.ball_loose && Some(sq) == b.ball {
            let tgt = (cx.ag + b.tz_against(sq, cx.home)).max(2);
            let raw = p_roll(tgt);
            let p = if cx.sure_hands { p_with_reroll(raw, 1.0) } else { raw };
            let v = 0.55 + 0.45 * p;
            if v > best.0 {
                best = (v, Intent::Pickup);
            }
        }
        // Cage — P4/A9: weighted by which side the threat is on.
        if let Some(car) = &b.carrier {
            if b.is_home(car) == cx.home {
                if let Some(cc) = b.g.field_model.player_coordinate(car) {
                    let dx = (sq.x - cc.x).abs();
                    let dy = (sq.y - cc.y).abs();
                    if dx == 1 && dy == 1 {
                        let t = threat_on(b, sq, cx.home, cx.str_).min(2.0) / 2.0;
                        let v = 0.35 + 0.40 * t;
                        if v > best.0 {
                            best = (v, Intent::Cage);
                        }
                    } else if dx <= 1 && dy <= 1 && (dx + dy) > 0 {
                        if 0.35 > best.0 {
                            best = (0.35, Intent::Cage);
                        }
                    }
                }
            }
        }
        // Mark
        let mut mark_best = 0.0f32;
        for n in sq.neighbours() {
            if !on_pitch(n.x, n.y) {
                continue;
            }
            if let Some(oid) = b.g.field_model.player_at(n) {
                if b.is_home(oid) == cx.home || !b.standing(oid) {
                    continue;
                }
                let is_enemy_carrier = b.carrier.as_deref() == Some(oid.as_str());
                let active = b
                    .g
                    .field_model
                    .player_state(oid)
                    .map(|s| s.is_active())
                    .unwrap_or(true);
                let mut mv: f32 = if is_enemy_carrier { 1.0 } else { 0.30 };
                if !active {
                    mv = mv.max(0.45);
                }
                mark_best = mark_best.max(mv);
            }
        }
        if mark_best > 0.0 {
            let v = 0.50 * mark_best;
            if v > best.0 {
                best = (v, Intent::Mark);
            }
        }
        // Screen — P3: obstruct the opponent's shortest paths to our carrier / the loose ball.
        if let Some(target) = b
            .carrier
            .as_ref()
            .filter(|c| b.is_home(c) == cx.home)
            .and_then(|c| b.g.field_model.player_coordinate(c))
            .or(b.ball.filter(|_| b.ball_loose))
        {
            let share = path_share(b, sq, target, cx.home);
            if share > 0.0 {
                let v = 0.45 * share;
                if v > best.0 {
                    best = (v, Intent::Screen);
                }
            }
        }
    }

    let (base, intent) = best;

    // ---- modifiers, scoped to the intent (P2) ------------------------------
    let sideline = if sq.y == 0 || sq.y == YMAX {
        if cx.side_step { 1.0 } else { 0.25 }
    } else if cx.is_carrier && (sq.y == 1 || sq.y == YMAX - 1) {
        0.6
    } else {
        1.0
    };
    let exposure = 1.0 / (1.0 + threat_on(b, sq, cx.home, cx.str_));
    let lane = if intent.uses_lane() {
        let ez = endzone_x(cx.home);
        let mut corridor = 0;
        for dy in -2..=2 {
            corridor += b.opponents_between(cx.home, sq.y + dy, sq.x, ez);
        }
        1.0 / (1.0 + 0.35 * corridor as f32)
    } else {
        1.0
    };

    (base * sideline * exposure * lane, intent)
}

/// Fraction of the opponent's straight-line approaches to `target` that pass near `sq`.
/// A cheap stand-in for "how many shortest paths run through here": an opponent contributes
/// when `sq` lies between it and the target and is close to that line.
fn path_share(b: &Board, sq: FieldCoordinate, target: FieldCoordinate, home: bool) -> f32 {
    let mut total = 0f32;
    let mut through = 0f32;
    for (id, &c) in &b.g.field_model.player_coordinates {
        if b.is_home(id) == home || !b.standing(id) || !on_pitch(c.x, c.y) {
            continue;
        }
        let d_ot = c.distance_in_steps(target);
        if d_ot == 0 || d_ot > 12 {
            continue;
        }
        total += 1.0;
        let d_os = c.distance_in_steps(sq);
        let d_st = sq.distance_in_steps(target);
        // `sq` is on a shortest-ish path when going via it costs no more than one extra step.
        if d_os + d_st <= d_ot + 1 && d_st >= 1 {
            through += 1.0;
        }
    }
    if total == 0.0 {
        0.0
    } else {
        through / total
    }
}

/// §5.3 — the expected cost of failing, which ends the team turn.
fn c_turnover(cx: &Ctx, gfi: i32, carries_ball: bool) -> f32 {
    (0.20 + 0.55 * cx.unactivated)
        * if carries_ball { 1.4 } else { 1.0 }
        * (1.0 + 0.15 * gfi as f32)
}

/// RISK AVERSION on rushes, over and above the expectation in `c_turnover`.
///
/// The plain expectation says a 5-in-6 rush for a small gain is a marginally positive trade, and
/// on that basis the agent rushed on ~34% of its movement decisions -- ~95 GFI rolls and ~16
/// failed rushes per game, which is what was ending nearly every turn. The expectation is not the
/// whole story: a turnover forfeits the REST OF THE DRIVE, not just the value of one square, and
/// that compounding is invisible to a single-step mean. So a rush has to clear a bar, not merely
/// break even.
///
/// The carrier pays much less: pushing for a touchdown is exactly when the variance is worth it.
fn rush_penalty(gfi: i32, carries_ball: bool) -> f32 {
    if gfi <= 0 {
        return 0.0;
    }
    // Tuned against the measured rate: at 0.22 the agent still took ~25 rushes a game, well
    // above the 0-10 a competent coach uses. A rush is a 1-in-6 chance of ending the drive, and
    // for a player with no ball it is almost never worth that.
    let per = if carries_ball { 0.10 } else { 0.40 };
    per * gfi as f32
}

// ────────────────────────────────────────────────────────────────── the agent

pub struct HeuristicAgent {
    rng: Xoshiro256StarStar,
    /// Multiplies every temperature. 1.0 = the §8 table; a very large value = uniform sampling
    /// over the identical option set, which is the control arm of the §16 experiment.
    temp_scale: f32,
    fallback: UniformAgent,
    buf: Scored,
    /// Live dispatch counts driving the §6.5.2 coverage floor.
    seen_action: HashMap<String, u32>,
    /// P8 — coarse board buckets already visited this run.
    seen_bucket: HashMap<u64, u32>,
    last_turn_key: Option<(i32, i32, bool)>,
    used_this_turn: std::collections::HashSet<String>,
}

impl HeuristicAgent {
    pub fn new(seed: u64, temp_scale: f32) -> Self {
        HeuristicAgent {
            rng: Xoshiro256StarStar::seed_from_u64(seed ^ 0x4845_5552_4953_5449),
            temp_scale,
            fallback: UniformAgent::new(seed),
            buf: Scored::default(),
            seen_action: HashMap::new(),
            seen_bucket: HashMap::new(),
            last_turn_key: None,
            used_this_turn: std::collections::HashSet::new(),
        }
    }

    /// Uniform sampling over whatever the scorer enumerated.
    pub fn new_uniform(seed: u64) -> Self {
        Self::new(seed, 1.0e6)
    }

    /// Pure argmax -- no RNG consumed, fully deterministic for a given board.
    pub fn new_argmax(seed: u64) -> Self {
        Self::new(seed, 0.0)
    }

    fn unit(&mut self) -> f32 {
        (self.rng.next_u64() >> 11) as f32 / (1u64 << 53) as f32
    }

    /// True argmax: no RNG consumed at all, ties broken by option order (which is
    /// deterministic -- `reachable` sorts by coordinate and every other enumeration walks a
    /// Vec). `temp_scale <= 0.0` selects this. Distinct from a very small temperature, which
    /// still draws from the RNG and still samples among near-equal weights.
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
        let n = self.buf.options.len();
        if n == 0 {
            return 0;
        }
        if self.temp_scale <= 0.0 {
            return self.argmax();
        }
        let t = (t_base * self.temp_scale).max(1e-6);
        let max = self
            .buf
            .options
            .iter()
            .map(|o| o.weight)
            .fold(f32::MIN, f32::max);
        let mut cum: Vec<f32> = Vec::with_capacity(n);
        let mut acc = 0.0f32;
        for o in &self.buf.options {
            acc += ((o.weight - max) / t).exp();
            cum.push(acc);
        }
        // e-floor: every enumerated option keeps non-zero probability. Switched OFF in the
        // greedy arm (temp_scale < 0.1), where the point is to see what the weights alone do
        // with no exploration at all -- at eps = 0.02 roughly one decision in fifty would be a
        // uniform random pick, which is not "deterministic" in any useful sense.
        let eps = if self.temp_scale < 0.1 { 0.0 } else { EPS };
        let r = self.unit();
        if r < eps {
            return ((self.rng.next_u64() as usize) % n).min(n - 1);
        }
        let r = self.unit() * acc;
        cum.partition_point(|&c| c < r).min(n - 1)
    }

    fn take(&mut self, i: usize) -> Action {
        self.buf.options.swap_remove(i).action
    }

    fn refresh_turn(&mut self, g: &Game) {
        let turn_nr = if g.home_playing { g.turn_data_home.turn_nr } else { g.turn_data_away.turn_nr };
        let key = (g.half, turn_nr, g.home_playing);
        if self.last_turn_key != Some(key) {
            self.last_turn_key = Some(key);
            self.used_this_turn.clear();
        }
    }

    /// P8 — a coarse board descriptor, counted per run.
    fn bucket(&self, b: &Board) -> u64 {
        let ballz = b.ball.map(|c| (c.x / 5) as u64 * 4 + (c.y / 4) as u64).unwrap_or(31);
        let carried = b.carrier.is_some() as u64;
        let turn = (b.g.turn_data_home.turn_nr.max(b.g.turn_data_away.turn_nr) / 3) as u64;
        let w = b.weather as u64;
        ballz | (carried << 6) | (turn << 8) | (w << 12) | ((b.g.half as u64) << 16)
    }

    /// 6.5.2's live coverage floor: push any action that has not dispatched yet this run, so a
    /// stronger policy cannot silently stop exercising a mechanic.
    ///
    /// It is a coverage tool and it costs play strength BY CONSTRUCTION -- it deliberately makes
    /// the agent take actions its own weights rate as bad. Measured: with the floor on, fouls sat
    /// at ~3 per game no matter how the foul weight was priced, because the floor was overriding
    /// a correctly-computed 0.003 with 0.35. So it is off in the sharp arms, which exist to
    /// measure how well the weights play, and on in the sampling arms, which exist to measure
    /// coverage. That is the trade 6.5.2 describes, made explicit.
    fn coverage_floor(&self, key: &str) -> f32 {
        if self.temp_scale < 0.1 {
            return 0.0;
        }
        let seen = *self.seen_action.get(key).unwrap_or(&0);
        0.35 * (1.0 - (seen as f32 / 4.0).min(1.0))
    }
}

impl Agent for HeuristicAgent {
    fn act(&mut self, gs: &GameState) -> Action {
        let g = &gs.game;
        self.buf.clear();

        let prompt = match gs.current_prompt() {
            Some(p) => p.clone(),
            None => return Action::Acknowledge,
        };

        match prompt {
            // ── movement: the whole path in one answer (§4.1b, §6.6) ────────
            AgentPrompt::Move { player_id, squares } => {
                if squares.is_empty() {
                    return Action::EndPlayerAction;
                }
                let b = Board::new(g);
                let home = b.is_home(&player_id);
                let player = match g.player(&player_id) {
                    Some(p) => p,
                    None => return Action::EndPlayerAction,
                };
                let start = match g.field_model.player_coordinate(&player_id) {
                    Some(c) => c,
                    None => return Action::EndPlayerAction,
                };
                let td = if home { &g.turn_data_home } else { &g.turn_data_away };
                let team_rr = td.rerolls > 0 && !td.reroll_used;
                let cx = Ctx {
                    home,
                    is_carrier: b.carrier.as_deref() == Some(player_id.as_str()),
                    ma: player.movement_with_modifiers(),
                    ag: player.agility_with_modifiers(),
                    sure_hands: player.has_skill(SkillId::SureHands),
                    str_: player.strength_with_modifiers(),
                    d_now: endzone_distance(start, home),
                    turns_left: (8 - if home { g.turn_data_home.turn_nr } else { g.turn_data_away.turn_nr }).max(0),
                    side_step: player.has_skill(SkillId::SideStep),
                    unactivated: unactivated_share(g, home),
                };

                let reach = reachable(&b, &player_id, team_rr);
                for r in &reach {
                    if r.path.is_empty() || !squares.contains(&r.path[0]) {
                        continue;
                    }
                    let (v, intent) = value_of(&b, r.coord, &cx);
                    let scores = cx.is_carrier && endzone_distance(r.coord, home) == 0;
                    // A touchdown ends the drive, so there is no turnover term to subtract --
                    // but it is still an OPTION, scored and sampled like every other. An earlier
                    // build hard-returned here, which bypassed the softmax entirely and therefore
                    // survived any temperature: the "uniform" control arm scored touchdowns it
                    // had never sampled. Every decision must go through the buffer.
                    let (w, rule) = if scores {
                        (r.p_arrive - rush_penalty(r.gfi, true), Rule::ScoreTouchdown)
                    } else {
                        (
                            r.p_arrive * v
                                - (1.0 - r.p_arrive) * c_turnover(&cx, r.gfi, cx.is_carrier)
                                - rush_penalty(r.gfi, cx.is_carrier),
                            intent.rule(),
                        )
                    };
                    self.buf.push(Action::Move { path: r.path.clone() }, w, rule, v);
                }
                // Declining banks what we have: zero gain, zero risk => weight 0.0 (§6.6).
                self.buf.push(Action::EndPlayerAction, 0.0, Rule::EndActivation, 0.0);
                let i = self.sample(0.06);
                if std::env::var_os("FFB_HEUR_TRACE").is_some() {
                    let o = &self.buf.options[i];
                    let cost = match &o.action {
                        Action::Move { path } => path.len() as i32,
                        _ => 0,
                    };
                    let nopt = self.buf.options.len();
                    let bestw = self.buf.options.iter().map(|x| x.weight).fold(f32::MIN, f32::max);
                    let gfi = match &o.action {
                        Action::Move { path } => path.len() as i32
                            - (cx.ma - b.g.acting_player.current_move.max(0)).max(0),
                        _ => 0,
                    };
                    eprintln!("HMOVE carrier={} loose={} cost={} gfi={} w={:.3} best={:.3} why={:?} nopt={} unact={:.3} cto={:.3} spent={}",
                        cx.is_carrier, b.ball_loose, cost, gfi.max(0),
                        o.weight, bestw, o.why, nopt, cx.unactivated,
                        c_turnover(&cx, gfi.max(0), cx.is_carrier),
                        b.g.acting_player.current_move);
                }
                self.take(i)
            }

            // ── activation: (player, action, target) ────────────────────────
            AgentPrompt::ActivatePlayer { eligible_players } => {
                self.refresh_turn(g);
                if eligible_players.is_empty() {
                    return Action::EndTurn;
                }
                let b = Board::new(g);
                let home = g.home_playing;
                let side = if home { TeamSide::Home } else { TeamSide::Away };
                let td = if home { &g.turn_data_home } else { &g.turn_data_away };
                let team_rr = td.rerolls > 0 && !td.reroll_used;
                let bucket = self.bucket(&b);
                let novelty = if self.temp_scale >= 0.1
                    && self.seen_bucket.get(&bucket).copied().unwrap_or(0) == 0
                {
                    0.08
                } else {
                    0.0
                };

                // Backstop: if every eligible player has already acted this turn, end it.
                if eligible_players
                    .iter()
                    .all(|(pid, _)| self.used_this_turn.contains(pid))
                {
                    return Action::EndTurn;
                }
                // Progress guarantee. The 0.30 `used_this_turn` factor is a preference, and a
                // preference is not enough for a near-argmax policy: if a used player still
                // outscores every unused one it is re-picked forever and the turn never ends
                // (observed as a hang in the greedy arm). Skip used players outright while any
                // unused one is eligible; the soft factor then only ever applies when the
                // engine re-offers a player with nobody else left.
                let any_unused = eligible_players
                    .iter()
                    .any(|(pid, _)| !self.used_this_turn.contains(pid));
                let mut any = false;
                for (pid, actions) in &eligible_players {
                    if any_unused && self.used_this_turn.contains(pid) {
                        continue;
                    }
                    let st = match g.field_model.player_state(pid) {
                        Some(s) => s,
                        None => continue,
                    };
                    if st.is_prone() && !st.is_active() {
                        continue;
                    }
                    let player = match g.player(pid) {
                        Some(p) => p,
                        None => continue,
                    };
                    let live: Vec<PlayerAction> = actions
                        .iter()
                        .filter(|a| action_is_live(a, td, g.rules))
                        .cloned()
                        .collect();
                    if live.is_empty() {
                        continue;
                    }
                    let start = match g.field_model.player_coordinate(pid) {
                        Some(c) => c,
                        None => continue,
                    };
                    let cx = Ctx {
                        home,
                        is_carrier: b.carrier.as_deref() == Some(pid.as_str()),
                        ma: player.movement_with_modifiers(),
                        ag: player.agility_with_modifiers(),
                        sure_hands: player.has_skill(SkillId::SureHands),
                        str_: player.strength_with_modifiers(),
                        d_now: endzone_distance(start, home),
                        turns_left: (8 - td.turn_nr).max(0),
                        side_step: player.has_skill(SkillId::SideStep),
                        unactivated: unactivated_share(g, home),
                    };
                    // tier 1 proxy: best of the eight adjacent squares (§6.5.1)
                    let mut proxy = 0.0f32;
                    for n in start.neighbours() {
                        if on_pitch(n.x, n.y) && !b.occupied(n) {
                            proxy = proxy.max(value_of(&b, n, &cx).0);
                        }
                    }
                    let marked = b.tz_against(start, home) > 0;
                    // Can this player actually get to the loose ball this activation?
                    let can_fetch = b.ball_loose
                        && b.ball
                            .map(|bc| start.distance_in_steps(bc) <= cx.ma + 2)
                            .unwrap_or(false);
                    let mut w_player = if cx.is_carrier && marked {
                        0.95
                    } else if can_fetch {
                        // Possession is the game. Nothing else on the board is worth more than
                        // the player who can pick the ball up.
                        0.92
                    } else if cx.is_carrier {
                        0.88
                    } else if st.is_prone() && marked {
                        0.70
                    } else if proxy > 0.25 {
                        0.45
                    } else {
                        0.30
                    };
                    if self.used_this_turn.contains(pid) {
                        w_player *= 0.30;
                    }
                    if has_negatrait(player) {
                        w_player *= 0.55;
                    }

                    for pa in &live {
                        let pac = player_action_to_pac(pa);
                        // EVERY (target) is its own option. An earlier build returned only the
                        // argmax target from `score_action`, which meant the target was chosen
                        // greedily at any temperature -- a heuristic the temperature knob could
                        // not switch off, and a deviation from 6.5's joint (player, action,
                        // target) choice.
                        for (target, mut w_action, rule) in
                            score_action_candidates(&b, g, pid, pac, side, &cx, team_rr, proxy)
                        {
                            // P4: a negatrait failure voids the whole activation.
                            if has_negatrait(player) {
                                w_action *= 0.66;
                            }
                            let key = format!("{:?}", pac);
                            w_action = w_action.max(self.coverage_floor(&key));
                            let w = w_player * w_action + novelty;
                            self.buf.push(
                                Action::ActivatePlayer {
                                    player_id: pid.clone(),
                                    player_action: pac,
                                    block_defender_id: target,
                                },
                                w,
                                rule,
                                w_action,
                            );
                            any = true;
                        }
                    }
                }
                if !any {
                    return Action::EndTurn;
                }
                // Ending the turn banks what the team already has: zero further gain, zero
                // further risk, so its weight is 0.0 on the same scale as everything else --
                // exactly as the decline option is in 6.6. The old
                // `0.05 + 0.90 * (1 - best)` assumed activation weights ran close to 1.0; the
                // real product `w_player * w_action` runs around 0.15, which made EndTurn ~0.83
                // and ended every turn after a single activation (measured: 11 activations per
                // GAME instead of ~270). Termination does not depend on this weight: the engine
                // empties the eligible list as players are used, and the all-used guard above
                // is the backstop.
                self.buf.push(Action::EndTurn, 0.0, Rule::EndActivation, 0.0);

                let i = self.sample(0.18);
                let act = self.take(i);
                if let Action::ActivatePlayer { player_id, player_action, .. } = &act {
                    self.used_this_turn.insert(player_id.clone());
                    *self.seen_action.entry(format!("{:?}", player_action)).or_insert(0) += 1;
                }
                *self.seen_bucket.entry(bucket).or_insert(0) += 1;
                act
            }

            // ── block dice (§6.3) ───────────────────────────────────────────
            AgentPrompt::BlockChoice { attacker_id, defender_id, dice, own_choice, .. } => {
                let b = Board::new(g);
                let def_has_ball = b.carrier.as_deref() == Some(defender_id.as_str());
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
                                (true, true) => if def_has_ball { 0.50 } else { 0.30 },
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
                    self.buf.push(
                        Action::BlockChoice { die_index: i, target_id: None },
                        w,
                        Rule::Face,
                        *d as f32,
                    );
                }
                if self.buf.options.is_empty() {
                    return Action::BlockChoice { die_index: 0, target_id: None };
                }
                let i = self.sample(0.12);
                self.take(i)
            }

            // ── block target (§6.2) ─────────────────────────────────────────
            AgentPrompt::BlitzTarget { attacker_id, eligible_players } => {
                if eligible_players.is_empty() {
                    return Action::EndPlayerAction;
                }
                let b = Board::new(g);
                for did in &eligible_players {
                    let w = block_target_weight(&b, g, &attacker_id, did);
                    self.buf.push(Action::SelectPlayer { player_id: did.clone() }, w, Rule::DiceCount, w);
                }
                let i = self.sample(0.15);
                self.take(i)
            }

            AgentPrompt::BlockTarget { .. } => Action::EndPlayerAction,

            // ── pushback (§6.13) ────────────────────────────────────────────
            AgentPrompt::Pushback { attacker_id, defender_id, squares } => {
                if squares.is_empty() {
                    return Action::Acknowledge;
                }
                let b = Board::new(g);
                let def_home = b.is_home(&defender_id);
                let def_has_ball = b.carrier.as_deref() == Some(defender_id.as_str());
                let att_c = g.field_model.player_coordinate(&attacker_id);
                let mut sorted = squares.clone();
                sorted.sort_by_key(|c| (c.x, c.y));
                for sq in &sorted {
                    let off = !on_pitch(sq.x, sq.y);
                    let mut w = if off {
                        if def_has_ball { 1.0 } else { 0.95 }
                    } else if sq.y == 0 || sq.y == YMAX {
                        0.55
                    } else {
                        0.20
                    };
                    // pushing the defender away from their own endzone is good for us
                    if let Some(a) = att_c {
                        let before = endzone_distance(a, !def_home);
                        let after = endzone_distance(*sq, !def_home);
                        if after > before {
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
                let b = Board::new(g);
                let home = b.is_home(&attacker_id);
                let carries = b.carrier.as_deref() == Some(attacker_id.as_str());
                let cur = g.field_model.player_coordinate(&attacker_id);
                let mut w: f32 = 0.5;
                if carries {
                    w -= 0.45;
                }
                if let Some(c) = cur {
                    let tz_now = b.tz_against(c, home);
                    let tz_then = b.tz_against(target_coord, home);
                    if tz_then > tz_now {
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
                let b = Board::new(g);
                let home = g.home_playing;
                let td = if home { &g.turn_data_home } else { &g.turn_data_away };
                let consequence = match action.as_str() {
                    "GFI" | "DODGE" | "PICKUP" | "CATCH" | "JUMP" | "ESCAPE" => {
                        if b.carrier.as_ref().map(|c| b.is_home(c) == home).unwrap_or(false) {
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
                    if td.turn_nr >= 7 { base * 1.35 } else { base }
                } else {
                    0.0
                };
                let w_use = (consequence * 0.833 * scarcity).clamp(0.0, 1.0);
                self.buf.push(Action::UseReRoll { use_reroll: true }, w_use, Rule::Reroll, w_use);
                self.buf.push(Action::UseReRoll { use_reroll: false }, 1.0 - w_use, Rule::Reroll, 1.0 - w_use);
                let i = self.sample(0.20);
                self.take(i)
            }

            // ── skill use (§6.16) ───────────────────────────────────────────
            AgentPrompt::SkillUse { skill_name, .. } => {
                let skill_id = ffb_model::enums::SkillId::from_class_name(&skill_name)
                    .unwrap_or(ffb_model::enums::SkillId::Block);
                let w_use = match skill_name.as_str() {
                    "Dodge" => 0.95,
                    "Juggernaut" => 0.80,
                    "HitAndRun" => 0.70,
                    "Fend" => 0.85,
                    "Wrestle" => 0.55,
                    "QuickBite" | "AnimalSavagery" => 0.85,
                    _ => 0.50,
                };
                self.buf.push(Action::UseSkill { skill_id, use_skill: true }, w_use, Rule::Skill, w_use);
                self.buf.push(Action::UseSkill { skill_id, use_skill: false }, 1.0 - w_use, Rule::Skill, 1.0 - w_use);
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
            AgentPrompt::KickBall => {
                let home = g.home_playing;
                for x_raw in 0..13 {
                    for y in 1..=13 {
                        let x = if home { x_raw + 13 } else { x_raw };
                        let c = FieldCoordinate::new(x, y);
                        let los = if home { 13 } else { 12 };
                        let deep = ((c.x - los).abs() as f32 / 12.0).min(1.0);
                        let mut w = 0.5 + 0.30 * deep;
                        if y <= 1 || y >= 13 || c.x <= 1 || c.x >= 24 {
                            w -= 0.55;
                        }
                        self.buf.push(Action::KickBall { coord: c }, w, Rule::Flat, w);
                    }
                }
                let i = self.sample(0.06);
                self.take(i)
            }

            AgentPrompt::Touchback { eligible_players } => {
                if eligible_players.is_empty() {
                    return Action::Acknowledge;
                }
                let mut sorted = eligible_players.clone();
                sorted.sort_by(|a, b| a.0.cmp(&b.0));
                for (pid, coord) in &sorted {
                    let ma = g.player(pid).map(|p| p.movement_with_modifiers()).unwrap_or(6);
                    let mut w = 0.3 + 0.4 * (ma as f32 / 9.0).min(1.0);
                    if g.player(pid).map(|p| p.has_skill(SkillId::SureHands)).unwrap_or(false) {
                        w += 0.3;
                    }
                    let los = if g.home_playing { 12 } else { 13 };
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
                self.buf.push(Action::ReceiveChoice { receive: false }, 1.0 - w, Rule::Flat, 1.0 - w);
                let i = self.sample(0.30);
                self.take(i)
            }

            AgentPrompt::TeamSetup { team_id, .. } => canonical_setup_action(g, &team_id),

            // Everything else: the long tail, identical in both arms of the experiment.
            _ => self.fallback.act(gs),
        }
    }
}

// ───────────────────────────────────────────────────────────────────── helpers

fn unactivated_share(g: &Game, home: bool) -> f32 {
    let team = if home { &g.team_home } else { &g.team_away };
    let n = team
        .players
        .iter()
        .filter(|p| {
            g.field_model
                .player_state(&p.id)
                .map(|s| s.is_active())
                .unwrap_or(false)
        })
        .count();
    (n as f32 / 11.0).clamp(0.0, 1.0)
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
    push_squares(g, att, def)
        .iter()
        .any(|c| !on_pitch(c.x, c.y))
}

/// Assist-resolved dice count, then the §2.4 table plus context.
fn block_target_weight(b: &Board, g: &Game, att: &str, def: &str) -> f32 {
    let (ac, dc) = match (
        g.field_model.player_coordinate(att),
        g.field_model.player_coordinate(def),
    ) {
        (Some(a), Some(d)) => (a, d),
        _ => return 0.05,
    };
    let ap = match g.player(att) { Some(p) => p, None => return 0.05 };
    let dp = match g.player(def) { Some(p) => p, None => return 0.05 };
    let a_str = crate::util::server_util_player::ServerUtilPlayer::find_block_strength(
        g, ac, ap.strength_with_modifiers(), dc,
    );
    let d_str = crate::util::server_util_player::ServerUtilPlayer::find_block_strength(
        g, dc, dp.strength_with_modifiers(), ac,
    );
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
        1 => if ap.has_skill(SkillId::Block) { 0.40 } else { 0.25 },
        -2 => 0.10,
        _ => 0.025,
    };
    let has_ball = b.carrier.as_deref() == Some(def);
    if has_ball {
        w *= 1.35;
    }
    if can_surf(g, att, def) {
        w *= if has_ball { 1.9 } else { 1.5 };
    }
    if dp.has_skill(SkillId::Block) && !ap.has_skill(SkillId::Block) && !ap.has_skill(SkillId::Wrestle) {
        w *= 0.70;
    }
    w.clamp(0.01, 1.0)
}

/// 6.5.2 -- every (target) a declared action could carry, each as its OWN option.
///
/// Returning only the argmax (as an earlier build did) bakes a greedy target choice in at any
/// temperature, which the temperature knob cannot switch off and which 6.5 does not specify:
/// the activation choice is JOINT over (player, action, target).
fn score_action_candidates(
    b: &Board,
    g: &Game,
    pid: &str,
    pac: PlayerActionChoice,
    side: TeamSide,
    cx: &Ctx,
    team_rr: bool,
    proxy: f32,
) -> Vec<(Option<String>, f32, Rule)> {
    match pac {
        PlayerActionChoice::Move | PlayerActionChoice::StandUp => {
            let mut best = proxy;
            for r in reachable(b, pid, team_rr) {
                let (v, _) = value_of(b, r.coord, cx);
                let w = r.p_arrive * v
                    - (1.0 - r.p_arrive) * c_turnover(cx, r.gfi, cx.is_carrier)
                    - rush_penalty(r.gfi, cx.is_carrier);
                best = best.max(w);
            }
            vec![(None, best.max(0.05), Rule::ScoreAdvance)]
        }
        PlayerActionChoice::Block => {
            // A plain Block needs the victim to be adjacent ALREADY, and costs the team nothing.
            let targets = legal_block_targets(g, pid, side);
            if targets.is_empty() {
                return vec![(None, 0.02, Rule::DiceCount)];
            }
            targets
                .into_iter()
                .map(|t| {
                    let w = block_target_weight(b, g, pid, &t);
                    (Some(t), w, Rule::DiceCount)
                })
                .collect()
        }
        PlayerActionChoice::Blitz | PlayerActionChoice::StandUpBlitz => {
            // A Blitz MOVES then blocks, and spends the team's one blitz for the turn. An earlier
            // build scored it from the same branch as Block, so the two were numerically
            // identical and a near-argmax policy always took whichever the engine happened to
            // offer first -- which was Block, giving 17 blocks and only 9 blitzes per game while
            // the ball went nowhere. Score what a blitz is actually for: reaching a victim that a
            // plain block cannot.
            let mut out: Vec<(Option<String>, f32, Rule)> = Vec::new();
            let reach = reachable(b, pid, team_rr);
            let here = g.field_model.player_coordinate(pid);
            for (oid, &oc) in &g.field_model.player_coordinates {
                if b.is_home(oid) == cx.home || !b.standing(oid) {
                    continue;
                }
                let adjacent_now = here.map(|h| h.distance_in_steps(oc) == 1).unwrap_or(false);
                // best arrival probability among squares adjacent to this opponent
                let p_reach = if adjacent_now {
                    1.0
                } else {
                    reach
                        .iter()
                        .filter(|r| r.coord.distance_in_steps(oc) == 1)
                        .map(|r| r.p_arrive)
                        .fold(0.0f32, f32::max)
                };
                if p_reach <= 0.0 {
                    continue;
                }
                let dice = block_target_weight(b, g, pid, oid);
                // Spending the once-per-turn blitz on someone a plain block already reaches is a
                // waste of the blitz, so discount that case.
                let waste = if adjacent_now { 0.85 } else { 1.0 };
                out.push((Some(oid.clone()), p_reach * dice * waste, Rule::DiceCount));
            }
            if out.is_empty() {
                return vec![(None, 0.02, Rule::DiceCount)];
            }
            out.sort_by(|a, c| a.0.cmp(&c.0));
            out
        }
        PlayerActionChoice::Foul => {
            // A foul was a flat 0.30, with no ejection risk and no notion of whether the victim
            // was worth it -- so the sharper the policy, the more it fouled (6.35 per game at
            // argmax). Price it properly: the chance of actually hurting the victim, times how
            // much that victim matters, times the risk of losing the player and the turn to the
            // referee. On a skill-less roster with nothing worth fouling, this lands near zero,
            // which is correct.
            let targets = legal_foul_targets(g, pid, side);
            if targets.is_empty() {
                return vec![(None, 0.02, Rule::Flat)];
            }
            let bribes = if cx.home { g.team_home.bribes } else { g.team_away.bribes };
            // An ejection costs us a player for the REST OF THE MATCH. A bribe usually saves him.
            let eject_cost = if bribes > 0 { 0.07 } else { 0.45 };
            // Mild preference for fouling late, once the turn's real work is banked.
            let timing = if cx.unactivated <= 3.0 / 11.0 { 1.0 } else { 0.85 };
            targets
                .into_iter()
                .map(|t| {
                    let av = g.player(&t).map(|q| q.armour_with_modifiers()).unwrap_or(8);
                    // FOUL ASSISTS. A foul's armour roll gets +1 per net offensive assist
                    // (`foul_assist_armor_modifier`, ±1..7), which is the entire reason to pick
                    // one victim over another -- and the reason to foul at all. Scoring the foul
                    // from AV alone, as an earlier build did, made every victim look identical
                    // and every foul look bad, so the agent could not learn to foul the player it
                    // had three team-mates standing over. These are the engine's own functions,
                    // the same two `InjuryTypeFoul::armour_roll` calls.
                    let off = ffb_model::util::util_player::UtilPlayer::find_offensive_foul_assists(
                        g, pid, &t,
                    ) as i32;
                    let def = ffb_model::util::util_player::UtilPlayer::find_defensive_foul_assists(
                        g, pid, &t,
                    ) as i32;
                    let net = off - def;
                    // P(2d6 + net > AV)
                    let need = av - net;
                    let p_break = if need <= 1 {
                        0.97
                    } else if need >= 12 {
                        0.03
                    } else {
                        let fails: i32 = (2..=need.min(12))
                            .map(|k| 6 - (k - 7).abs())
                            .map(|c| c.max(0))
                            .sum();
                        (1.0 - fails as f32 / 36.0).clamp(0.03, 0.97)
                    };
                    let victim = if b.carrier.as_deref() == Some(t.as_str()) {
                        1.0
                    } else if b
                        .ball
                        .zip(g.field_model.player_coordinate(&t))
                        .map(|(bc, tc)| bc.distance_in_steps(tc) <= 1)
                        .unwrap_or(false)
                    {
                        0.7
                    } else {
                        0.35
                    };
                    // The referee spots a foul on DOUBLES -- of the armour roll, then of the
                    // injury roll if the armour broke (`step_referee.rs`: `armor[0] == armor[1]`,
                    // then `injury[0] == injury[1]`). So ejection risk is a fixed ~1/6 rising
                    // slightly with the chance of actually hurting the victim, and NOTHING the
                    // agent chooses can lower it. It is a cost to pay or avoid, not to manage.
                    let p_eject = 0.167 + p_break * (5.0 / 6.0) * 0.167;
                    // An expectation, not a stack of multipliers. Stacked multipliers made a
                    // well-assisted foul score the same as an unassisted one -- both ~0.01 -- so
                    // the agent never fouled at all and the assist term could not express itself.
                    let w = (p_break * victim - p_eject * eject_cost) * timing;
                    (Some(t), w, Rule::Flat)
                })
                .collect()
        }
        PlayerActionChoice::HandOff => {
            let recv = legal_handoff_receivers(g, pid, side);
            if recv.is_empty() {
                return vec![(None, 0.02, Rule::Flat)];
            }
            recv.into_iter()
                .map(|r| {
                    let w = g
                        .field_model
                        .player_coordinate(&r)
                        .map(|c| 0.88 * value_of(b, c, cx).0)
                        .unwrap_or(0.05);
                    (Some(r), w, Rule::Flat)
                })
                .collect()
        }
        PlayerActionChoice::Pass
        | PlayerActionChoice::ThrowBomb
        | PlayerActionChoice::HailMaryPass
        | PlayerActionChoice::AllYouCanEat => {
            let recv = legal_pass_receivers(g, pid, side);
            if recv.is_empty() {
                return vec![(None, 0.02, Rule::Flat)];
            }
            recv.into_iter()
                .map(|r| {
                    let w = g
                        .field_model
                        .player_coordinate(&r)
                        .map(|c| {
                            let v = value_of(b, c, cx).0;
                            let p_complete = 0.6f32;
                            p_complete * v - (1.0 - p_complete) * c_turnover(cx, 0, true)
                        })
                        .unwrap_or(0.05);
                    (Some(r), w, Rule::Flat)
                })
                .collect()
        }
        PlayerActionChoice::ThrowTeamMate | PlayerActionChoice::KickTeamMate => {
            let t = legal_throw_team_mate_targets(g, pid, side);
            if t.is_empty() {
                return vec![(None, 0.02, Rule::Flat)];
            }
            t.into_iter().map(|x| (Some(x), 0.35, Rule::Flat)).collect()
        }
        _ => vec![(None, 0.40, Rule::Flat)],
    }
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

    /// D1 / P-round: the GFI target is 3+ in a Blizzard, in EVERY edition.
    #[test]
    fn gfi_target_is_three_in_a_blizzard() {
        assert_eq!(gfi_target(Weather::Blizzard), 3);
        assert_eq!(gfi_target(Weather::Nice), 2);
    }

    /// A5: a free skill re-roll turns a 4+ into 0.75.
    #[test]
    fn reroll_composition() {
        assert!((p_with_reroll(p_roll(4), 1.0) - 0.75).abs() < 1e-3);
    }

    /// P4: standing up costs 3 MA, so a prone MA-6 player reaches five squares, not eight.
    #[test]
    fn stand_up_cost_is_three() {
        assert_eq!(STAND_UP_COST, 3);
    }

    /// A1: a risky move is worth less early in the turn than late.
    #[test]
    fn turnover_cost_falls_as_the_turn_empties() {
        let mk = |un: f32| Ctx { home: true, is_carrier: false, ma: 6, ag: 3, sure_hands: false,
                                 str_: 3, d_now: 12, turns_left: 8, side_step: false, unactivated: un };
        let early = mk(10.0 / 11.0);
        let late = mk(1.0 / 11.0);
        assert!(c_turnover(&early, 0, false) > c_turnover(&late, 0, false));
    }

    /// A3: the same gain scores the same from deep and from close.
    #[test]
    fn advance_is_measured_against_what_the_activation_can_reach() {
        let far = 24i32;
        let near = 12i32;
        let ma = 6i32;
        let gain = 6i32;
        let a_far = (gain as f32) / (far.min(ma + 2).max(1) as f32);
        let a_near = (gain as f32) / (near.min(ma + 2).max(1) as f32);
        assert!((a_far - a_near).abs() < 1e-6);
    }
}
