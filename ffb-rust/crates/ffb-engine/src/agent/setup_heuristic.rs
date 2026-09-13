//! Kick-off setup for the heuristic agent — `docs/HEURISTIC_AGENT.md` §6.21.
//!
//! One `TeamSetup` prompt is ONE placement: the agent samples a `(player, square)` pair from the
//! joint option set, the engine re-prompts, and the loop ends with `ConfirmSetup` once eleven
//! players stand on the pitch or the reserves are empty. There is no formation book. The
//! formation emerges from per-placement scoring that reads what is already on the board (the
//! line of scrimmage count, the wings, the depth, who is next to whom), so every square of the
//! own half is reachable by every player, and the mass concentrates on the placements a coach
//! would make.
//!
//! **Legality is by construction, not by validation.** The rules `SetupMechanic::check_setup`
//! enforces are the three the enumeration never violates:
//!
//! - the first `min_los` placements are restricted to the line of scrimmage (x = 12, y ∈ 4..=10),
//!   so a legal setup always holds at least `min(min_los, available)` there;
//! - a wide zone (y ∈ 0..=3 / 11..=14) that already holds `max_wide` players is not offered;
//! - placement stops at `max_field`, and a player with `needsToBeSetUp` is forced in before the
//!   slots run out.
//!
//! Sideline rows and the own endzone column are only PENALISED, never excluded: the uniform arm
//! (`temp_scale = 1e6`) must be able to put any player anywhere, and at the table temperature a
//! −3.0 term makes the sideline a once-a-season event rather than a rule.
//!
//! **Parity.** `SetupPlacement.java` is a line-for-line mirror. Every weight below is an `f32`
//! sum of small constants times integer-derived terms — no transcendental functions — so the two
//! languages agree bit for bit. Options are enumerated players-by-jersey then squares
//! column-major in the HOME frame (`x 0..=12` outer, `y 0..=14` inner); player ids never enter
//! an ordering (`AGENT_CONTRACT_HEURISTIC.md` §5). The `Action::PlacePlayer` coordinate is the
//! RAW home-frame square, exactly as `canonical_setup_action` sends it — `setup_player`
//! transforms it for the away side.

use ffb_model::enums::{Rules, SkillId};
use ffb_model::model::game::Game;
use ffb_model::model::player::Player;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::option::{game_option_id, util_game_option};
use ffb_model::types::FieldCoordinate;

/// Temperature for one placement (`AGENT_CONTRACT_HEURISTIC.md` §3).
pub const SETUP_T: f32 = 0.30;
/// Columns of the own half in the home frame (x = 0 is the endzone, x = 12 the line of scrimmage).
pub const HW: usize = 13;
/// Rows of the pitch.
pub const HH: usize = 15;

/// Option-count defaults when the game carries no setup options (unit tests build bare games;
/// the parity harness always sets all three, and Java's option objects carry the same defaults).
const DEFAULT_MAX_FIELD: i32 = 11;
const DEFAULT_MAX_WIDE: i32 = 2;
const DEFAULT_MIN_LOS: i32 = 3;

/// What the scorer knows about one player. Extracted once per prompt from the `Player`; the
/// scoring functions read only this, which is what makes them a fixture the Java test can feed
/// from the golden file without a `Game`.
#[derive(Clone, Debug, PartialEq)]
pub struct SetupPlayer {
    pub nr: i32,
    pub st: i32,
    pub av: i32,
    pub ma: i32,
    /// Agility on the BB2016 scale, higher is better: the raw AG in BB2016, `7 - target` in
    /// BB2020/BB2025 (a 3+ becomes 4, a 2+ becomes 5).
    pub agg: i32,
    /// Position cost in thousands of gold.
    pub cost_k: i32,
    pub block: bool,
    pub guard: bool,
    pub stand_firm: bool,
    pub side_step: bool,
    pub fend: bool,
    pub thick_skull: bool,
    pub mighty: bool,
    pub frenzy: bool,
    pub stunty: bool,
    pub sure_hands: bool,
    pub passer: bool,
    pub catcher: bool,
    pub nerves: bool,
    pub dodge: bool,
    pub sprint: bool,
    pub leap: bool,
    pub big_hand: bool,
    pub kick: bool,
    pub kor: bool,
    pub negatrait: bool,
    pub no_hands: bool,
    pub loner: bool,
    pub ball_and_chain: bool,
    pub must_field: bool,
}

impl SetupPlayer {
    pub fn from_player(p: &Player, rules: Rules) -> SetupPlayer {
        let ag = p.agility_with_modifiers();
        let agg = if rules == Rules::Bb2016 { ag } else { 7 - ag };
        SetupPlayer {
            nr: p.nr,
            st: p.strength_with_modifiers(),
            av: p.armour_with_modifiers(),
            ma: p.movement_with_modifiers(),
            agg,
            cost_k: p.position_cost / 1000,
            block: p.has_skill(SkillId::Block) || p.has_skill(SkillId::Wrestle),
            guard: p.has_skill(SkillId::Guard),
            stand_firm: p.has_skill(SkillId::StandFirm),
            side_step: p.has_skill(SkillId::SideStep) || p.has_skill(SkillId::Sidestep),
            fend: p.has_skill(SkillId::Fend),
            thick_skull: p.has_skill(SkillId::ThickSkull),
            mighty: p.has_skill(SkillId::MightyBlow) || p.has_skill(SkillId::Claw),
            frenzy: p.has_skill(SkillId::Frenzy),
            stunty: p.has_skill(SkillId::Stunty) || p.has_skill(SkillId::Titchy),
            sure_hands: p.has_skill(SkillId::SureHands),
            passer: p.has_skill(SkillId::Pass) || p.has_skill(SkillId::Accurate),
            catcher: p.has_skill(SkillId::Catch) || p.has_skill(SkillId::DivingCatch),
            nerves: p.has_skill(SkillId::NervesOfSteel),
            dodge: p.has_skill(SkillId::Dodge),
            sprint: p.has_skill(SkillId::Sprint) || p.has_skill(SkillId::SureFeet),
            leap: p.has_skill(SkillId::Leap),
            big_hand: p.has_skill(SkillId::BigHand) || p.has_skill(SkillId::ExtraArms),
            kick: p.has_skill(SkillId::Kick),
            kor: p.has_skill(SkillId::KickOffReturn),
            negatrait: p.has_skill(SkillId::BoneHead)
                || p.has_skill(SkillId::ReallyStupid)
                || p.has_skill(SkillId::WildAnimal)
                || p.has_skill(SkillId::TakeRoot)
                || p.has_skill(SkillId::BloodLust)
                || p.has_skill(SkillId::AnimalSavagery)
                || p.has_skill(SkillId::UnchannelledFury),
            no_hands: p.has_skill(SkillId::NoHands),
            loner: p.has_skill(SkillId::Loner),
            ball_and_chain: p.has_skill(SkillId::BallAndChain),
            must_field: p.has_skill_property(NamedProperties::NEEDS_TO_BE_SET_UP),
        }
    }

    #[inline]
    fn slow(&self) -> bool {
        self.ma <= 5
    }
}

#[inline]
fn clamp(v: f32, lo: f32, hi: f32) -> f32 {
    v.max(lo).min(hi)
}

/// The "cheap unit" bonus: linemen at 50k are the ones a coach spends on the line. Stunties get
/// nothing from it — they are cheap BECAUSE they die on the line.
pub fn cheapness(p: &SetupPlayer) -> f32 {
    if p.stunty {
        0.0
    } else {
        0.35 * clamp((70 - p.cost_k) as f32 / 30.0, -1.5, 1.0)
    }
}

/// How good a line-of-scrimmage body this player is: strength and armour first, block skills,
/// cheapness; ball handlers and fast catchers are pulled off the line.
pub fn line_score(p: &SetupPlayer) -> f32 {
    let mut w = 0.55 * (p.st - 3) as f32 + 0.30 * (p.av - 8) as f32;
    if p.av <= 7 {
        w -= 0.6;
    }
    if p.block {
        w += 0.5;
    }
    if p.guard {
        w += 0.4;
    }
    if p.stand_firm {
        w += 0.3;
    }
    if p.fend || p.side_step {
        w += 0.2;
    }
    if p.thick_skull {
        w += 0.2;
    }
    if p.mighty {
        w += 0.15;
    }
    if p.stunty {
        w -= 1.0;
    } else {
        w += cheapness(p);
    }
    if p.negatrait {
        w -= 0.15;
    }
    if p.sure_hands || p.passer {
        w -= 0.35;
    }
    if p.catcher && p.ma >= 7 {
        w -= 0.25;
    }
    w
}

/// Who should stand deep and pick the ball up: Sure Hands, agility, a passing skill, speed.
pub fn handler_score(p: &SetupPlayer) -> f32 {
    let mut w = 0.35 * (p.agg - 3) as f32 + 0.12 * (p.ma - 6) as f32;
    if p.sure_hands {
        w += 0.6;
    }
    if p.passer {
        w += 0.35;
    }
    if p.big_hand {
        w += 0.2;
    }
    if p.kor {
        w += 0.25;
    }
    if p.dodge {
        w += 0.1;
    }
    if p.sprint {
        w += 0.1;
    }
    if p.leap {
        w += 0.1;
    }
    if p.no_hands || p.ball_and_chain {
        w -= 3.0;
    }
    if p.negatrait {
        w -= 0.8;
    }
    if p.loner {
        w -= 0.3;
    }
    if p.stunty {
        w -= 0.3;
    }
    if p.st >= 5 {
        w -= 0.4;
    }
    w
}

/// Who should be out on a wing waiting for the ball: speed, agility, Catch, Dodge.
pub fn receiver_score(p: &SetupPlayer) -> f32 {
    let mut w = 0.35 * (p.ma - 6) as f32 + 0.25 * (p.agg - 3) as f32;
    if p.catcher {
        w += 0.5;
    }
    if p.nerves {
        w += 0.2;
    }
    if p.dodge {
        w += 0.25;
    }
    if p.sprint {
        w += 0.15;
    }
    if p.leap {
        w += 0.1;
    }
    if p.no_hands || p.ball_and_chain {
        w -= 3.0;
    }
    if p.negatrait {
        w -= 0.6;
    }
    if p.st >= 5 {
        w -= 0.4;
    }
    w
}

/// How much this player needs protecting: expensive AND thin-skinned, or a stunty. 0..=1.
pub fn fragile(p: &SetupPlayer) -> f32 {
    let mut f = clamp((p.cost_k - 70) as f32 / 40.0, 0.0, 1.0) * clamp((9 - p.av) as f32 / 2.0, 0.0, 1.0);
    if p.stunty {
        f += 0.5;
    }
    f.min(1.0)
}

/// One player's contribution to team strength, for the weaker/stronger comparison.
pub fn power(p: &SetupPlayer) -> f32 {
    let mut w = p.st as f32 + 0.5 * (p.av - 8) as f32;
    if p.block {
        w += 0.5;
    }
    if p.guard {
        w += 0.4;
    }
    if p.mighty {
        w += 0.3;
    }
    if p.stand_firm || p.side_step {
        w += 0.3;
    }
    if p.frenzy {
        w += 0.2;
    }
    if p.stunty {
        w -= 0.4;
    }
    if p.negatrait {
        w -= 0.3;
    }
    w
}

/// Team strength: the eleven strongest available players, by `power`. Sorted by (power desc,
/// nr asc) so a tie is broken by jersey and never by list order.
pub fn team_power(players: &[SetupPlayer]) -> f32 {
    let mut ps: Vec<(f32, i32)> = players.iter().map(|p| (power(p), p.nr)).collect();
    ps.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal).then(a.1.cmp(&b.1)));
    let mut acc = 0.0f32;
    for (i, (w, _)) in ps.iter().enumerate() {
        if i >= 11 {
            break;
        }
        acc += *w;
    }
    acc
}

/// Mean movement of the available players (0 for an empty list).
pub fn mean_ma(players: &[SetupPlayer]) -> f32 {
    if players.is_empty() {
        return 0.0;
    }
    let mut sum = 0i32;
    for p in players {
        sum += p.ma;
    }
    sum as f32 / players.len() as f32
}

/// Everything about the board a placement is scored against, in the HOME frame of the team that
/// is setting up (`x = 12` is that team's line of scrimmage whichever side it plays).
#[derive(Clone, Debug, PartialEq)]
pub struct SetupBoard {
    pub home: bool,
    /// True when this team RECEIVES (`Game::setup_offense`: the kicking team sets up first with
    /// the flag clear, the receiving team second with it set).
    pub offence: bool,
    pub weaker: bool,
    pub stronger: bool,
    pub fast: bool,
    pub opp_frenzy: bool,
    pub max_field: i32,
    pub max_wide: i32,
    pub min_los: i32,
    /// Any player standing on this raw square (own placed, or an opponent left over from the
    /// previous drive). Never offered.
    pub occupied: [[bool; HH]; HW],
    /// Own players already placed, raw frame.
    pub mine: [[bool; HH]; HW],
    /// Opponents adjacent to this raw square, counted in the stored frame.
    pub opp_adj: [[u8; HH]; HW],
    /// Opponents on THEIR line of scrimmage.
    pub los_opp: i32,
}

/// Counters derived from `mine`, recomputed per prompt.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Counts {
    pub placed: i32,
    pub los_mine: i32,
    pub wing_up: i32,
    pub wing_low: i32,
    pub deep_have: i32,
    pub deepest: i32,
    pub up: i32,
    pub low: i32,
}

impl SetupBoard {
    pub fn empty(home: bool, offence: bool) -> SetupBoard {
        SetupBoard {
            home,
            offence,
            weaker: false,
            stronger: false,
            fast: false,
            opp_frenzy: false,
            max_field: DEFAULT_MAX_FIELD,
            max_wide: DEFAULT_MAX_WIDE,
            min_los: DEFAULT_MIN_LOS,
            occupied: [[false; HH]; HW],
            mine: [[false; HH]; HW],
            opp_adj: [[0u8; HH]; HW],
            los_opp: 0,
        }
    }

    /// Raw (home-frame) square → the coordinate the field model stores for this team.
    #[inline]
    pub fn stored(&self, x: i32, y: i32) -> FieldCoordinate {
        let raw = FieldCoordinate::new(x, y);
        if self.home { raw } else { raw.transform() }
    }

    /// Mark one of our own players at a raw square.
    pub fn place_mine(&mut self, x: i32, y: i32) {
        if (0..HW as i32).contains(&x) && (0..HH as i32).contains(&y) {
            self.mine[x as usize][y as usize] = true;
            self.occupied[x as usize][y as usize] = true;
        }
    }

    /// Register an opponent standing at a STORED coordinate: it occupies its square if that lies
    /// in our half, it counts on their line of scrimmage, and it marks the raw squares next to it.
    pub fn add_opponent(&mut self, stored: FieldCoordinate) {
        let raw = if self.home { stored } else { stored.transform() };
        if (0..HW as i32).contains(&raw.x) && (0..HH as i32).contains(&raw.y) {
            self.occupied[raw.x as usize][raw.y as usize] = true;
        }
        let their_los_x = if self.home { 13 } else { 12 };
        if stored.x == their_los_x && stored.y >= 4 && stored.y <= 10 {
            self.los_opp += 1;
        }
        for x in 0..HW as i32 {
            for y in 0..HH as i32 {
                let s = self.stored(x, y);
                let dx = (s.x - stored.x).abs();
                let dy = (s.y - stored.y).abs();
                if dx <= 1 && dy <= 1 && (dx | dy) != 0 {
                    self.opp_adj[x as usize][y as usize] += 1;
                }
            }
        }
    }

    pub fn counts(&self) -> Counts {
        let mut c = Counts::default();
        for x in 0..HW as i32 {
            for y in 0..HH as i32 {
                if !self.mine[x as usize][y as usize] {
                    continue;
                }
                c.placed += 1;
                let d = 12 - x;
                if x == 12 && y >= 4 && y <= 10 {
                    c.los_mine += 1;
                }
                if y <= 3 {
                    c.wing_up += 1;
                }
                if y >= 11 {
                    c.wing_low += 1;
                }
                if d >= 5 {
                    c.deep_have += 1;
                }
                if d > c.deepest {
                    c.deepest = d;
                }
                if y < 7 {
                    c.up += 1;
                }
                if y > 7 {
                    c.low += 1;
                }
            }
        }
        c
    }

    fn friends_adj(&self, x: i32, y: i32) -> i32 {
        let mut n = 0;
        for dx in -1..=1 {
            for dy in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let (nx, ny) = (x + dx, y + dy);
                if (0..HW as i32).contains(&nx) && (0..HH as i32).contains(&ny) && self.mine[nx as usize][ny as usize] {
                    n += 1;
                }
            }
        }
        n
    }

    /// Offence only: how the square sits against the defence that is already on the pitch. One
    /// opponent in base contact is what a Block player wants; two is a double-team; a fragile
    /// player wants none at all.
    fn contact_term(&self, p: &SetupPlayer, x: i32, y: i32) -> f32 {
        if !self.offence {
            return 0.0;
        }
        let c = self.opp_adj[x as usize][y as usize] as i32;
        let mut w = 0.0f32;
        if c == 1 {
            w += if p.block { 0.5 } else { 0.1 };
        } else if c >= 2 {
            w -= 0.45 * (c - 1) as f32;
            if !p.block {
                w -= 0.2;
            }
        }
        w -= 0.7 * fragile(p) * c as f32;
        w
    }

    /// Defence only: a mirrored, balanced formation. The mirror square across the centre row
    /// already held by one of ours is worth taking; the emptier flank is worth filling.
    fn symmetry_term(&self, c: &Counts, x: i32, y: i32) -> f32 {
        if self.offence {
            return 0.0;
        }
        let mut w = 0.0f32;
        if y != 7 && self.mine[x as usize][(14 - y) as usize] {
            w += 0.6;
        }
        let diff = c.low - c.up;
        if y < 7 {
            w += 0.25 * diff.clamp(-2, 2) as f32;
        } else if y > 7 {
            w += 0.25 * (-diff).clamp(-2, 2) as f32;
        }
        w
    }

    /// Phase 1 — the mandatory line-of-scrimmage placements (x = 12, y ∈ 4..=10).
    pub fn score_los(&self, p: &SetupPlayer, c: &Counts, y: i32) -> f32 {
        let mut w = line_score(p);
        w += if y == 7 {
            0.15
        } else if y == 6 || y == 8 {
            0.10
        } else {
            0.0
        };
        w += self.contact_term(p, 12, y);
        w += self.symmetry_term(c, 12, y);
        w
    }

    /// Phase 2 — any legal square of the own half.
    pub fn score_square(&self, p: &SetupPlayer, c: &Counts, x: i32, y: i32) -> f32 {
        let d = 12 - x;
        let mid = (y - 7).abs();
        let wide = y <= 3 || y >= 11;
        let sideline = y == 0 || y == 14;
        let edge = y == 1 || y == 13;
        let on_los = x == 12 && y >= 4 && y <= 10;
        let wing_count = if y <= 3 { c.wing_up } else { c.wing_low };

        let mut w = 0.0f32;
        // Never next to the sideline: a crowd-surf waiting to happen.
        if sideline {
            w -= 3.0;
        } else if edge {
            w -= 0.6;
        }
        if x == 0 {
            w -= 1.0;
        } else if x == 1 {
            w -= 0.4;
        }
        // A frenzied opponent chains pushes into the crowd — unless we cannot be pushed.
        if self.opp_frenzy && !(p.side_step || p.stand_firm) {
            if edge {
                w -= 0.6;
            } else if y == 2 || y == 12 {
                w -= 0.3;
            }
        }
        // Slow players near the scrimmage and the centre.
        if p.slow() {
            w += if d <= 2 { 0.25 } else { -0.15 };
            w += 0.05 * (3 - mid.min(3)) as f32;
        }
        let fr = fragile(p);

        if self.offence {
            w += match d {
                0 => 0.0,
                1 | 2 => 0.30,
                3 => 0.10,
                4 => -0.20,
                _ => -0.50,
            };
            // One or two deep, to pick the ball up: the best handler, and not a third.
            if d >= 5 && d <= 9 && mid <= 3 {
                w += 0.9 * handler_score(p);
                w += if c.deep_have == 0 {
                    0.9
                } else if c.deep_have == 1 {
                    0.35
                } else {
                    -1.0
                };
                if d >= 6 && d <= 8 {
                    w += 0.15;
                }
            }
            // More on the scrimmage than the defence, with real line bodies.
            if on_los {
                w += if c.los_mine <= self.los_opp { 0.5 } else { -0.3 };
                w += 0.5 * line_score(p);
            }
            w += self.contact_term(p, x, y);
            // Fast catchers on the wings, close to the halfway line.
            if (y == 2 || y == 3 || y == 11 || y == 12) && d >= 1 && d <= 3 {
                w += 0.7 * receiver_score(p);
                if wing_count == 0 {
                    w += 0.25;
                }
            }
            // Fragile players behind friends, not in the front row.
            let f = self.friends_adj(x, y).min(2);
            w += fr * (0.45 * f as f32 - if d <= 1 { 0.3 } else { 0.0 });
        } else {
            let target_los = if self.weaker {
                3
            } else if self.stronger {
                6
            } else {
                4
            };
            if on_los {
                w += if c.los_mine < target_los { 0.45 + 0.5 * line_score(p) } else { -0.8 };
            }
            w += match d {
                0 => 0.0,
                1 => 0.10,
                2 => 0.35,
                3 => 0.30,
                4 => 0.10,
                5 => -0.10,
                _ => -0.40,
            };
            // Weak and fast: give ground, stay off the touchlines, make them come to us.
            if self.weaker && self.fast {
                if d >= 3 && d <= 6 {
                    w += 0.5;
                }
                if d <= 1 {
                    w -= 0.6;
                }
                if y <= 2 || y >= 12 {
                    w -= 0.4;
                }
            }
            // One safety deep in the middle.
            if c.deepest < 5 && d >= 5 && d <= 7 && mid <= 2 {
                w += 0.5 + 0.4 * receiver_score(p);
            }
            w += self.symmetry_term(c, x, y);
            // Cover each wing.
            if wide && d >= 1 && d <= 3 {
                w += if wing_count == 0 { 0.4 } else { 0.15 };
            }
            // Kick may only be used off the scrimmage and out of the wide zones.
            if p.kick && d >= 2 && y >= 4 && y <= 10 {
                w += 0.6;
            }
            w += fr * if d <= 1 { -0.5 } else { 0.2 };
        }
        w
    }
}

/// One scored placement: index into the candidate list, raw square, weight.
#[derive(Clone, Debug, PartialEq)]
pub struct SetupOption {
    pub player: usize,
    pub x: i32,
    pub y: i32,
    pub w: f32,
}

/// Enumerate every legal `(player, square)` for the next placement, in contract order. Empty
/// when the setup is complete (eleven placed, or nothing left to place).
///
/// `candidates` are the reserves in jersey order. The LOS phase applies while fewer than
/// `min(min_los, held + remaining)` of ours hold the line; the must-field gate restricts the
/// player list to `needsToBeSetUp` players once the free slots are down to their number.
pub fn enumerate(board: &SetupBoard, candidates: &[SetupPlayer]) -> Vec<SetupOption> {
    let c = board.counts();
    let mut out = Vec::new();
    if candidates.is_empty() || c.placed >= board.max_field {
        return out;
    }
    let remaining = (c.placed + candidates.len() as i32).min(board.max_field) - c.placed;
    if remaining <= 0 {
        return out;
    }
    let must: Vec<usize> = candidates.iter().enumerate().filter(|(_, p)| p.must_field).map(|(i, _)| i).collect();
    let idxs: Vec<usize> = if !must.is_empty() && must.len() as i32 >= remaining {
        must
    } else {
        (0..candidates.len()).collect()
    };
    let need_los = c.los_mine < board.min_los.min(c.los_mine + remaining);
    for &i in &idxs {
        let p = &candidates[i];
        if need_los {
            for y in 4..=10 {
                if board.occupied[12][y as usize] {
                    continue;
                }
                out.push(SetupOption { player: i, x: 12, y, w: board.score_los(p, &c, y) });
            }
        } else {
            for x in 0..HW as i32 {
                for y in 0..HH as i32 {
                    if board.occupied[x as usize][y as usize] {
                        continue;
                    }
                    if y <= 3 && c.wing_up >= board.max_wide {
                        continue;
                    }
                    if y >= 11 && c.wing_low >= board.max_wide {
                        continue;
                    }
                    out.push(SetupOption { player: i, x, y, w: board.score_square(p, &c, x, y) });
                }
            }
        }
    }
    out
}

/// The game-facing entry: the reserves of `team_id` in jersey order plus the board they are
/// placed on. Returns `None` when there is nothing left to place (the caller confirms).
pub fn setup_options(game: &Game, team_id: &str) -> Option<(Vec<Player>, Vec<SetupOption>)> {
    let home = team_id == game.team_home.id;
    let (team, opp) = if home {
        (&game.team_home, &game.team_away)
    } else {
        (&game.team_away, &game.team_home)
    };
    let rules = game.rules;

    let mut board = SetupBoard::empty(home, game.setup_offense);
    let opt = |id: &str, dflt: i32| {
        let v = util_game_option::get_int_option(game, id);
        if v <= 0 { dflt } else { v }
    };
    board.max_field = opt(game_option_id::MAX_PLAYERS_ON_FIELD, DEFAULT_MAX_FIELD);
    board.max_wide = opt(game_option_id::MAX_PLAYERS_IN_WIDE_ZONE, DEFAULT_MAX_WIDE);
    board.min_los = opt(game_option_id::MIN_PLAYERS_ON_LOS, DEFAULT_MIN_LOS);

    // Our players: placed ones mark the board, reserves are the candidates. Jersey order.
    let mut mine: Vec<&Player> = team.players.iter().collect();
    mine.sort_by_key(|p| p.nr);
    let mut candidates: Vec<Player> = Vec::new();
    let mut avail_mine: Vec<SetupPlayer> = Vec::new();
    for p in &mine {
        let state = game.field_model.player_state(&p.id);
        let settable = state.map(|s| s.can_be_set_up_next_drive()).unwrap_or(true);
        if settable {
            avail_mine.push(SetupPlayer::from_player(p, rules));
        }
        if let Some(c) = game.field_model.player_coordinate(&p.id) {
            if c.is_on_pitch() {
                let raw = if home { c } else { c.transform() };
                board.place_mine(raw.x, raw.y);
                continue;
            }
        }
        // At the very first setup no player_state entries exist yet: those are reserves in all
        // but name (same rule as `canonical_setup_action`).
        let reserve = state.map(|s| s.base() == ffb_model::enums::PS_RESERVE).unwrap_or(true);
        if reserve {
            candidates.push((*p).clone());
        }
    }

    // The opposition: what stands on the pitch, and what the roster can field.
    let mut avail_opp: Vec<SetupPlayer> = Vec::new();
    let mut opp_sorted: Vec<&Player> = opp.players.iter().collect();
    opp_sorted.sort_by_key(|p| p.nr);
    for p in &opp_sorted {
        let state = game.field_model.player_state(&p.id);
        let settable = state.map(|s| s.can_be_set_up_next_drive()).unwrap_or(true);
        if settable {
            avail_opp.push(SetupPlayer::from_player(p, rules));
        }
        if let Some(c) = game.field_model.player_coordinate(&p.id) {
            if c.is_on_pitch() {
                board.add_opponent(c);
            }
        }
    }
    let my_power = team_power(&avail_mine);
    let opp_power = team_power(&avail_opp);
    board.weaker = my_power < opp_power - 1.0;
    board.stronger = my_power > opp_power + 1.0;
    board.fast = mean_ma(&avail_mine) >= 6.5;
    board.opp_frenzy = avail_opp.iter().any(|p| p.frenzy);

    let sp: Vec<SetupPlayer> = candidates.iter().map(|p| SetupPlayer::from_player(p, rules)).collect();
    let options = enumerate(&board, &sp);
    if options.is_empty() {
        return None;
    }
    Some((candidates, options))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{PlayerState, PS_RESERVE, PS_STANDING};
    use ffb_model::model::SkillWithValue;

    fn mk(nr: i32, st: i32, av: i32, ma: i32, ag: i32, cost: i32, skills: &[SkillId]) -> Player {
        Player {
            id: format!("home_{nr:02}"),
            name: format!("p{nr}"),
            nr,
            movement: ma,
            strength: st,
            agility: ag,
            passing: 4,
            armour: av,
            position_cost: cost,
            starting_skills: skills.iter().map(|s| SkillWithValue::new(*s)).collect(),
            ..Default::default()
        }
    }

    /// A human-shaped squad: linemen, a thrower, catchers, blitzers, an ogre, a halfling.
    fn human_squad() -> Vec<Player> {
        vec![
            mk(1, 3, 8, 6, 3, 50_000, &[]),
            mk(2, 3, 8, 6, 3, 50_000, &[]),
            mk(3, 3, 8, 6, 3, 50_000, &[]),
            mk(4, 3, 8, 6, 3, 50_000, &[]),
            mk(5, 3, 8, 6, 3, 80_000, &[SkillId::SureHands, SkillId::Pass]),
            mk(6, 2, 7, 8, 2, 65_000, &[SkillId::Catch, SkillId::Dodge]),
            mk(7, 2, 7, 8, 2, 65_000, &[SkillId::Catch, SkillId::Dodge]),
            mk(8, 3, 9, 7, 3, 85_000, &[SkillId::Block]),
            mk(9, 3, 9, 7, 3, 85_000, &[SkillId::Block]),
            mk(10, 5, 9, 5, 4, 140_000, &[SkillId::BoneHead, SkillId::MightyBlow, SkillId::ThickSkull, SkillId::Loner]),
            mk(11, 2, 6, 5, 3, 30_000, &[SkillId::Stunty, SkillId::Dodge, SkillId::RightStuff]),
            mk(12, 3, 8, 6, 3, 50_000, &[]),
        ]
    }

    fn sp(p: &Player) -> SetupPlayer {
        SetupPlayer::from_player(p, Rules::Bb2025)
    }

    #[test]
    fn role_scores_rank_the_obvious_players() {
        let sq = human_squad();
        let s: Vec<SetupPlayer> = sq.iter().map(sp).collect();
        // The ogre is the best line body, the halfling the worst.
        let best = s.iter().max_by(|a, b| line_score(a).partial_cmp(&line_score(b)).unwrap()).unwrap();
        let worst = s.iter().min_by(|a, b| line_score(a).partial_cmp(&line_score(b)).unwrap()).unwrap();
        assert_eq!(best.nr, 10);
        assert_eq!(worst.nr, 11);
        // The thrower is the handler, a catcher the receiver.
        let h = s.iter().max_by(|a, b| handler_score(a).partial_cmp(&handler_score(b)).unwrap()).unwrap();
        assert_eq!(h.nr, 5);
        let r = s.iter().max_by(|a, b| receiver_score(a).partial_cmp(&receiver_score(b)).unwrap()).unwrap();
        assert!(r.nr == 6 || r.nr == 7);
        // A lineman is cheaper than a blitzer; the halfling gets no cheapness at all.
        assert!(cheapness(&s[0]) > cheapness(&s[7]));
        assert_eq!(cheapness(&s[10]), 0.0);
        assert!(fragile(&s[10]) >= 0.5, "stunties are fragile");
        assert_eq!(fragile(&s[0]), 0.0, "a lineman is not");
    }

    #[test]
    fn agility_is_normalised_to_the_bb2016_scale() {
        let p = mk(1, 3, 8, 6, 3, 50_000, &[]);
        assert_eq!(SetupPlayer::from_player(&p, Rules::Bb2016).agg, 3);
        // A BB2025 "3+" is a BB2016 AG4.
        assert_eq!(SetupPlayer::from_player(&p, Rules::Bb2025).agg, 4);
        assert_eq!(SetupPlayer::from_player(&p, Rules::Bb2020).agg, 4);
    }

    #[test]
    fn first_placements_are_line_of_scrimmage_only() {
        let sq = human_squad();
        let s: Vec<SetupPlayer> = sq.iter().map(sp).collect();
        let mut b = SetupBoard::empty(true, false);
        for placed in 0..3 {
            let opts = enumerate(&b, &s[placed..]);
            assert!(!opts.is_empty());
            assert!(opts.iter().all(|o| o.x == 12 && (4..=10).contains(&o.y)), "placement {placed} must be on the LOS");
            assert_eq!(opts.len(), (7 - placed) * (s.len() - placed), "every reserve × every free LOS square");
            b.place_mine(12, 5 + placed as i32);
        }
        // Three held: the whole half opens up, LOS included, sideline included.
        let opts = enumerate(&b, &s[3..]);
        assert!(opts.iter().any(|o| o.x != 12));
        assert!(opts.iter().any(|o| o.x == 12 && o.y == 4), "extra LOS squares stay on offer");
        assert!(opts.iter().any(|o| o.y == 0), "the sideline is penalised, not excluded");
    }

    #[test]
    fn wide_zone_cap_and_field_cap_are_hard() {
        let sq = human_squad();
        let s: Vec<SetupPlayer> = sq.iter().map(sp).collect();
        let mut b = SetupBoard::empty(true, true);
        for y in 5..=7 {
            b.place_mine(12, y);
        }
        b.place_mine(10, 2);
        b.place_mine(10, 3);
        let opts = enumerate(&b, &s[5..]);
        assert!(opts.iter().all(|o| o.y > 3), "upper wide zone is full");
        assert!(opts.iter().any(|o| o.y >= 11), "lower wide zone still open");
        // Eleven on the pitch: nothing more.
        let mut full = SetupBoard::empty(true, true);
        for y in 4..=10 {
            full.place_mine(12, y);
        }
        for y in 5..=8 {
            full.place_mine(9, y);
        }
        assert_eq!(full.counts().placed, 11);
        assert!(enumerate(&full, &s[..1]).is_empty());
    }

    #[test]
    fn must_field_players_are_forced_in_before_the_slots_run_out() {
        let sq = human_squad();
        let mut s: Vec<SetupPlayer> = sq.iter().map(sp).collect();
        s[11].must_field = true;
        let mut b = SetupBoard::empty(true, false);
        for y in 4..=10 {
            b.place_mine(12, y);
        }
        for y in 5..=7 {
            b.place_mine(9, y);
        }
        // Ten placed, one slot, two reserves of which one must be fielded.
        let opts = enumerate(&b, &s[10..]);
        assert!(!opts.is_empty());
        assert!(opts.iter().all(|o| o.player == 1), "only the must-field player is offered");
    }

    #[test]
    fn offence_reads_the_defence_and_defence_mirrors_itself() {
        let sq = human_squad();
        let s: Vec<SetupPlayer> = sq.iter().map(sp).collect();
        let blitzer = &s[7];
        let lineman = &s[0];
        // Offence, home: a lone defender at stored (13,7) is in contact with raw (12,6..=8).
        let mut b = SetupBoard::empty(true, true);
        b.add_opponent(FieldCoordinate::new(13, 7));
        assert_eq!(b.opp_adj[12][7], 1);
        assert_eq!(b.opp_adj[12][8], 1);
        assert_eq!(b.opp_adj[12][9], 0);
        assert_eq!(b.los_opp, 1);
        let c = b.counts();
        // A Block player wants the one-on-one more than a lineman does.
        assert!(b.score_los(blitzer, &c, 7) - b.score_los(blitzer, &c, 4) > b.score_los(lineman, &c, 7) - b.score_los(lineman, &c, 4));
        // Away side: the same defender is stored at (12,7) and adjacency maps through transform.
        let mut a = SetupBoard::empty(false, true);
        a.add_opponent(FieldCoordinate::new(12, 7));
        assert_eq!(a.opp_adj[12][7], 1);
        assert_eq!(a.los_opp, 1);
        // Defence: a player at (10,4) makes (10,10) worth 0.6 more than (10,9) via the mirror.
        let mut d = SetupBoard::empty(true, false);
        for y in 5..=7 {
            d.place_mine(12, y);
        }
        d.place_mine(10, 4);
        let c = d.counts();
        let mirrored = d.score_square(lineman, &c, 10, 10);
        let beside = d.score_square(lineman, &c, 10, 9);
        assert!(mirrored > beside, "mirror {mirrored} vs beside {beside}");
    }

    #[test]
    fn sideline_is_the_worst_row_and_deep_is_for_handlers_on_offence() {
        let sq = human_squad();
        let s: Vec<SetupPlayer> = sq.iter().map(sp).collect();
        let thrower = &s[4];
        let ogre = &s[9];
        let mut b = SetupBoard::empty(true, true);
        for y in 5..=7 {
            b.place_mine(12, y);
        }
        let c = b.counts();
        for x in 1..=11 {
            assert!(b.score_square(thrower, &c, x, 0) < b.score_square(thrower, &c, x, 1));
            assert!(b.score_square(thrower, &c, x, 14) < b.score_square(thrower, &c, x, 13));
        }
        let deep_t = b.score_square(thrower, &c, 5, 7);
        let deep_o = b.score_square(ogre, &c, 5, 7);
        assert!(deep_t > deep_o + 1.0, "thrower deep {deep_t}, ogre deep {deep_o}");
        assert!(deep_t > b.score_square(thrower, &c, 11, 7), "the thrower prefers depth to the front row");
    }

    /// Drives the game-facing entry through a real `Game`: the flow of one placement at a time
    /// ends legal by `SetupMechanic::check_setup` for both sides and both roles, and every
    /// player takes every square of the half at least once across many uniform setups.
    #[test]
    fn full_setups_are_legal_and_cover_the_half() {
        use crate::mechanic::bb2025::setup_mechanic::SetupMechanic;
        use crate::mechanic::setup_mechanic::SetupMechanic as _;
        use crate::util::util_server_setup::UtilServerSetup;
        use rand_core::{RngCore, SeedableRng};

        let mut seen = [[false; HH]; HW];
        let mut per_player_los = std::collections::HashSet::new();
        let mut per_player_deep = std::collections::HashSet::new();
        let mut rng = rand_xoshiro::Xoshiro256StarStar::seed_from_u64(7);
        for round in 0..240u32 {
            let home_side = round % 2 == 0;
            let offence = (round / 2) % 2 == 0;
            let mut home = crate::step::framework::test_team("home", 0);
            let mut away = crate::step::framework::test_team("away", 0);
            home.players = human_squad();
            away.players = human_squad()
                .into_iter()
                .map(|mut p| {
                    p.id = p.id.replace("home", "away");
                    p
                })
                .collect();
            let mut g = Game::new(home, away, Rules::Bb2025);
            g.options.set(game_option_id::MAX_PLAYERS_ON_FIELD, "11");
            g.options.set(game_option_id::MIN_PLAYERS_ON_LOS, "3");
            g.options.set(game_option_id::MAX_PLAYERS_IN_WIDE_ZONE, "2");
            g.home_playing = home_side;
            g.setup_offense = offence;
            let team_id = if home_side { "home".to_string() } else { "away".to_string() };
            let ids: Vec<String> = g.team_home.players.iter().chain(g.team_away.players.iter()).map(|p| p.id.clone()).collect();
            for id in &ids {
                g.field_model.set_player_state(id, PlayerState::new(PS_RESERVE));
            }
            if offence {
                // The kicking side already stands on its LOS and a wing.
                let opp = if home_side { "away" } else { "home" };
                let ox = if home_side { 13 } else { 12 };
                for (i, y) in [5, 6, 7, 8].iter().enumerate() {
                    let id = format!("{opp}_{:02}", i + 1);
                    g.field_model.set_player_coordinate(&id, FieldCoordinate::new(ox, *y));
                    g.field_model.set_player_state(&id, PlayerState::new(PS_STANDING));
                }
            }
            let mut steps = 0;
            loop {
                steps += 1;
                assert!(steps < 40, "setup did not terminate");
                let Some((cands, opts)) = setup_options(&g, &team_id) else { break };
                // Uniform over the option set: every legal placement must be possible.
                let pick = &opts[(rng.next_u64() % opts.len() as u64) as usize];
                let pid = cands[pick.player].id.clone();
                let raw = FieldCoordinate::new(pick.x, pick.y);
                let before = g.field_model.player_coordinate(&pid);
                UtilServerSetup::setup_player(&mut g, &pid, raw);
                assert_ne!(before, g.field_model.player_coordinate(&pid), "placement {pid}@{raw:?} was rejected");
                seen[pick.x as usize][pick.y as usize] = true;
                if pick.x == 12 && (4..=10).contains(&pick.y) {
                    per_player_los.insert(cands[pick.player].nr);
                }
                if pick.x <= 6 {
                    per_player_deep.insert(cands[pick.player].nr);
                }
            }
            assert!(SetupMechanic::new().check_setup(&mut g, home_side), "round {round}: illegal setup");
            let on_pitch = g
                .field_model
                .player_coordinates
                .iter()
                .filter(|(id, c)| id.starts_with(&team_id) && c.is_on_pitch())
                .count();
            assert_eq!(on_pitch, 11);
        }
        for x in 0..HW {
            for y in 0..HH {
                assert!(seen[x][y], "square ({x},{y}) never used");
            }
        }
        assert_eq!(per_player_los.len(), 12, "every player reached the LOS");
        assert_eq!(per_player_deep.len(), 12, "every player reached the backfield");
    }

    #[test]
    fn setup_options_is_none_when_nothing_is_left() {
        let mut home = crate::step::framework::test_team("home", 0);
        let away = crate::step::framework::test_team("away", 0);
        home.players = human_squad();
        let mut g = Game::new(home, away, Rules::Bb2025);
        for p in g.team_home.players.clone() {
            g.field_model.set_player_state(&p.id, PlayerState::new(PS_STANDING));
            g.field_model.set_player_coordinate(&p.id, FieldCoordinate::new(-1, p.nr));
        }
        // Everyone is boxed but STANDING, i.e. not in reserve: no candidates.
        assert!(setup_options(&g, "home").is_none());
    }

    /// Regenerates `testdata/setup_golden.txt`, the cross-language pin for `SetupPlacement.java`.
    ///
    /// `cargo test -p ffb-engine --lib agent::setup_heuristic::tests::emit_setup_golden -- --ignored`
    #[test]
    #[ignore]
    fn emit_setup_golden() {
        use std::fmt::Write as _;
        let sq = human_squad();
        let players: Vec<SetupPlayer> = sq.iter().map(sp).collect();
        // (name, home, offence, weaker, stronger, fast, opp_frenzy, mine raw, opponents stored, first candidate)
        type Fx = (&'static str, bool, bool, bool, bool, bool, bool, &'static [(i32, i32)], &'static [(i32, i32)], usize);
        let fixtures: [Fx; 6] = [
            ("def_home_empty", true, false, false, false, false, false, &[], &[], 0),
            ("def_away_los_held", false, false, true, false, true, true, &[(12, 5), (12, 6), (12, 7), (10, 4)], &[], 4),
            ("off_home_vs_line", true, true, false, true, false, false, &[(12, 6), (12, 7), (12, 8)], &[(13, 5), (13, 6), (13, 7), (13, 8), (13, 9), (15, 2), (15, 12), (19, 7)], 3),
            ("off_away_two_deep", false, true, false, false, true, true, &[(12, 5), (12, 6), (12, 7), (5, 7), (6, 6), (10, 2)], &[(12, 6), (12, 7), (12, 8), (11, 3), (11, 11), (9, 7)], 6),
            ("def_home_wing_full", true, false, false, true, false, false, &[(12, 4), (12, 5), (12, 6), (11, 2), (11, 3), (9, 7)], &[], 6),
            ("off_home_ten_placed", true, true, true, false, false, false, &[(12, 4), (12, 5), (12, 6), (12, 7), (11, 9), (10, 10), (9, 7), (8, 6), (4, 7), (11, 12)], &[(13, 4), (13, 5), (13, 6)], 10),
        ];
        let mut out = String::new();
        writeln!(out, "# Setup placement golden -- setup_heuristic.rs and SetupPlacement.java.").unwrap();
        writeln!(out, "# player <nr> <st> <av> <ma> <agg> <costK> <flags: 24 chars of 0/1 in struct order>").unwrap();
        writeln!(out, "# fixture <name> <home> <offence> <weaker> <stronger> <fast> <oppFrenzy> <firstCandidate>").unwrap();
        writeln!(out, "# mine <x> <y>      (raw frame)   |  opp <x> <y>  (stored frame)").unwrap();
        writeln!(out, "# roles <nr> <line> <handler> <receiver> <fragile> <power>   (hex f32)").unwrap();
        writeln!(out, "# options <n> then one 'opt <player> <x> <y> <hex f32>' per option, in enumeration order").unwrap();
        writeln!(out, "# teampower <hex f32> meanma <hex f32>").unwrap();
        for p in &players {
            let flags = [
                p.block, p.guard, p.stand_firm, p.side_step, p.fend, p.thick_skull, p.mighty, p.frenzy, p.stunty,
                p.sure_hands, p.passer, p.catcher, p.nerves, p.dodge, p.sprint, p.leap, p.big_hand, p.kick, p.kor,
                p.negatrait, p.no_hands, p.loner, p.ball_and_chain, p.must_field,
            ];
            let fs: String = flags.iter().map(|b| if *b { '1' } else { '0' }).collect();
            writeln!(out, "player {} {} {} {} {} {} {}", p.nr, p.st, p.av, p.ma, p.agg, p.cost_k, fs).unwrap();
        }
        for p in &players {
            writeln!(
                out,
                "roles {} {:08x} {:08x} {:08x} {:08x} {:08x}",
                p.nr,
                line_score(p).to_bits(),
                handler_score(p).to_bits(),
                receiver_score(p).to_bits(),
                fragile(p).to_bits(),
                power(p).to_bits()
            )
            .unwrap();
        }
        writeln!(out, "teampower {:08x} meanma {:08x}", team_power(&players).to_bits(), mean_ma(&players).to_bits()).unwrap();
        for (name, home, offence, weaker, stronger, fast, frenzy, mine, opps, first) in fixtures {
            let mut b = SetupBoard::empty(home, offence);
            b.weaker = weaker;
            b.stronger = stronger;
            b.fast = fast;
            b.opp_frenzy = frenzy;
            for &(x, y) in mine {
                b.place_mine(x, y);
            }
            for &(x, y) in opps {
                b.add_opponent(FieldCoordinate::new(x, y));
            }
            writeln!(out, "fixture {name} {home} {offence} {weaker} {stronger} {fast} {frenzy} {first}").unwrap();
            for &(x, y) in mine {
                writeln!(out, "mine {x} {y}").unwrap();
            }
            for &(x, y) in opps {
                writeln!(out, "opp {x} {y}").unwrap();
            }
            let opts = enumerate(&b, &players[first..]);
            writeln!(out, "options {}", opts.len()).unwrap();
            for o in &opts {
                writeln!(out, "opt {} {} {} {:08x}", o.player, o.x, o.y, o.w.to_bits()).unwrap();
            }
        }
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/src/agent/testdata/setup_golden.txt");
        std::fs::write(path, out).unwrap();
    }

    /// The live scorer must agree with the golden the Java test reads — otherwise the golden
    /// pins nothing.
    #[test]
    fn setup_golden_matches_live_scorer() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/src/agent/testdata/setup_golden.txt");
        let text = std::fs::read_to_string(path).expect("run emit_setup_golden first");
        let sq = human_squad();
        let players: Vec<SetupPlayer> = sq.iter().map(sp).collect();
        let mut board: Option<SetupBoard> = None;
        let mut first = 0usize;
        let mut expected: Vec<(usize, i32, i32, u32)> = Vec::new();
        let mut checked = 0;
        let flush = |board: &Option<SetupBoard>, first: usize, expected: &mut Vec<(usize, i32, i32, u32)>, checked: &mut i32| {
            if let Some(b) = board {
                let opts = enumerate(b, &players[first..]);
                assert_eq!(opts.len(), expected.len());
                for (o, e) in opts.iter().zip(expected.iter()) {
                    assert_eq!((o.player, o.x, o.y, o.w.to_bits()), *e);
                }
                *checked += 1;
            }
            expected.clear();
        };
        for line in text.lines() {
            let f: Vec<&str> = line.split_whitespace().collect();
            match f.first().copied() {
                Some("fixture") => {
                    flush(&board, first, &mut expected, &mut checked);
                    let mut b = SetupBoard::empty(f[2] == "true", f[3] == "true");
                    b.weaker = f[4] == "true";
                    b.stronger = f[5] == "true";
                    b.fast = f[6] == "true";
                    b.opp_frenzy = f[7] == "true";
                    first = f[8].parse().unwrap();
                    board = Some(b);
                }
                Some("mine") => board.as_mut().unwrap().place_mine(f[1].parse().unwrap(), f[2].parse().unwrap()),
                Some("opp") => board
                    .as_mut()
                    .unwrap()
                    .add_opponent(FieldCoordinate::new(f[1].parse().unwrap(), f[2].parse().unwrap())),
                Some("opt") => expected.push((
                    f[1].parse().unwrap(),
                    f[2].parse().unwrap(),
                    f[3].parse().unwrap(),
                    u32::from_str_radix(f[4], 16).unwrap(),
                )),
                Some("roles") => {
                    let p = players.iter().find(|p| p.nr == f[1].parse::<i32>().unwrap()).unwrap();
                    assert_eq!(line_score(p).to_bits(), u32::from_str_radix(f[2], 16).unwrap());
                    assert_eq!(handler_score(p).to_bits(), u32::from_str_radix(f[3], 16).unwrap());
                    assert_eq!(receiver_score(p).to_bits(), u32::from_str_radix(f[4], 16).unwrap());
                    assert_eq!(fragile(p).to_bits(), u32::from_str_radix(f[5], 16).unwrap());
                    assert_eq!(power(p).to_bits(), u32::from_str_radix(f[6], 16).unwrap());
                }
                _ => {}
            }
        }
        flush(&board, first, &mut expected, &mut checked);
        assert_eq!(checked, 6);
    }
}
