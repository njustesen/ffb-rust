# The Heuristic Agent — a probabilistic policy over every decision in the game

**Status:** design specification. Nothing in this document is implemented yet.
**Target:** a third agent, `HeuristicAgent`, alongside `RandomAgent` (parity) and `UniformAgent`
(coverage), living in `crates/ffb-engine/src/agent/`.

**Scope:** a standalone probabilistic policy. **Nothing here builds a search.** A clean
score/sample split happens to be the shape a tree search would later want, and that is a fine
reason to prefer it over a monolithic `act()` — but no tree, prior, value head, node or playout is
in scope, and no engine change is to be made on their behalf. Where earlier drafts of this document
specified those, they have been removed rather than deferred.

---

## 0. Why a third agent

The two existing agents are both *structurally* incapable of playing Blood Bowl:

| | `RandomAgent` | `UniformAgent` |
|---|---|---|
| Purpose | byte-match Java `ParityRunner` | maximise mechanic-surface coverage |
| Move policy | **one square per activation** (except the carrier) | same, plus a carrier-advance bias |
| Target choice | uniform over the legal list | uniform over the legal list |
| Block dice | `pick(dice.len())` — uniform | uniform |
| Re-rolls | fixed / uniform coin flip | uniform coin flip |
| Constraint | RNG-consumption order is load-bearing — **must not change** | none |

The consequence is recorded in `MEMORY.md` (dead-step frontier): *both agents move one square per
activation, which blocks GFI + touchdowns + the whole scoring/Punt family.* 33 steps were never
dispatched in 5.7M dispatch lines. Random play never scores, so every step downstream of a
touchdown is unreachable, and the reds we do find are concentrated in the first 20% of a drive.

`HeuristicAgent` exists to fix that. Three requirements, in priority order:

1. **Probabilistic** — every legal option gets non-zero probability. No option is ever pruned to
   zero: coverage lives in the tail, and a policy that can only ever produce its favourite move
   stops being a sampler.
2. **Fast** — a decision must cost O(board), with no game-tree search and no cloning of `Game`.
   Budget: **< 100 ms of agent time per game** (see §9 for the per-class breakdown).
3. **Good** — the *mode* of the distribution should be a move a competent coach would make. Good
   play is what reaches the endzone, and reaching the endzone is what unlocks the dead steps.

---

## 0.5. Evidence — measured from the parity runs

Everything in this section is counted from the checked-in Rust event logs,
`parity/<matchup>/seed_<N>_rust_events.jsonl`, 100 seeds per tier, using the aggregation
procedure in `docs/COVERAGE_REPORT.md`. Two tiers: `lineman_vs_lineman` (skill-less, bb2025
tier-3) and `human_vs_human` (the richest green tier — full roster, real skills).

### 0.5.1 The one-square ceiling is not an approximation — it is exact

Walking each event stream and counting `playerMoved` events between consecutive `playerAction`
events:

| | `lineman_vs_lineman` | `human_vs_human` |
|---|---|---|
| Seeds | 100 | 100 |
| Activations | 27,713 | 29,545 |
| Squares moved, total | 26,297 | 27,138 |
| **Mean squares per activation** | **0.95** | **0.92** |
| **Maximum squares in ANY activation** | **1** | **1** |
| Activations that moved 0 squares | 1,416 (5.1%) | 2,407 (8.1%) |
| Activations that moved 1 square | 26,297 (94.9%) | 27,138 (91.9%) |
| Activations that moved 2+ squares | **0** | **0** |

Not "usually one square" — *never more than one*, across 57,258 activations. And it is worse than
the mean suggests, because the mean is carried entirely by `Move`:

| Declared action | n (lineman) | mean squares | max | n (human) | mean squares |
|---|---|---|---|---|---|
| `Move` | 26,297 | 1.00 | 1 | 27,402 | 0.99 |
| `Blitz` | 607 | **0.00** | 0 | 705 | **0.00** |
| `Block` | 448 | 0.00 | 0 | 536 | 0.00 |
| `Foul` | 232 | **0.00** | 0 | 247 | **0.00** |
| `Pass` | 90 | **0.00** | 0 | 84 | **0.00** |
| `HandOver` | 39 | 0.00 | 0 | 62 | 0.00 |
| `ThrowTeamMate` | — | — | — | 509 | 0.00 |

A Blitz never moves. A Foul never repositions. A Pass is thrown from wherever the player already
stood. This is what §4's move engine and §6.5's joint activation scorer exist to fix.

### 0.5.2 What never happens at all

Grepping the full 100-seed event catalogs (65,910 events for lineman, ~69k for human):

| Event | lineman | human |
|---|---|---|
| Touchdowns | **0** | **0** |
| Go-For-It / GFI rolls | **0** | **0** |
| Re-rolls consumed (`"rerolled":true`) | **0** | 25 |
| `skillUse` events | 0 | 16 |
| `throwTeamMateRoll` | — | **1** (from 509 declarations) |
| `rightStuffRoll` | — | **1** |

The Throw-Team-Mate line is the cleanest illustration of why §6.5.2's **coverage floor** rule
exists: the action was *declared* 509 times in 100 games and actually *rolled* once. A policy that
merely gets stronger, with no floor, would stop declaring it entirely and the number would go to
zero — a coverage regression dressed up as an improvement.

`docs/DEAD_STEP_INVENTORY.md` (re-measured 2026-08-25) puts the consequence in its own terms:
**167 of 200 `StepId` variants reached, 33 dead** — and category C is five steps
(`AssignTouchdowns`, `InitPunt`, `EndPunt`, `PuntDirection`, `PuntDistance`) marked
*"Out of scope until the agent is allowed to score."*

### 0.5.3 Dodges the agent should never have attempted

Because movement is an unweighted single-square walk, the agent dodges out of tackle zones with no
regard for the target number:

| Dodge target | lineman | human |
|---|---|---|
| 3+ (67%) | 438 | 357 |
| 4+ (50%) | 263 | 365 |
| 5+ (33%) | 72 | 118 |
| 6+ (17%) | 5 | 38 |
| 7+ (auto-fail territory) | 0 | **2** |
| **Total / failed** | 778 / **356 (45.8%)** | 880 / — |

356 failed dodges over 100 lineman games is **3.6 turnovers per game from dodging alone**. The
§4.2 reachability scorer prices a 6+ dodge at `p = 0.167` and a 5+ at `0.333`, and multiplies it
into the destination's weight — those squares stop being chosen unless the value on the far side
genuinely justifies the risk.

Supporting roll rates for the same tier: `dodgeRoll` 422 pass / 356 fail, `pickupRoll` 63 / 31,
`catchRoll` 53 / 56.

### 0.5.4 Block-die choice: replaying §6.3 over the real dice

The `blockRoll` event records `dice`, `selected_index`, `own_choice` and `nr_of_dice` — everything
§6.3's base table needs. So the heuristic can be replayed against the dice the parity runs actually
rolled. This is a scoring-only replay: no engine, no state mutation, just the recorded dice pushed
through the §6.3 weights, softmax at `T = 0.12`, ε = 0.02.

It uses the **context-free** base weights, because the event does not carry out-of-bounds geometry,
ball possession, or the two players' skills. Every context multiplier in §6.3 is therefore missing,
which makes these numbers a **lower bound** on the full heuristic.

| Tier | Side | n | Optimal pick — actual | Optimal pick — heuristic | Attacker equity — actual | → heuristic |
|---|---|---|---|---|---|---|
| lineman | attacker picks (+2D) | 170 | 62.4% | **92.7%** | 0.475 | **0.614** (+29%) |
| lineman | defender picks (−2D) | 82 | 59.8% | **94.6%** | 0.391 | **0.233** (−40%) |
| human | attacker picks (+2D/+3D) | 294 | 62.6% | **92.2%** | 0.489 | **0.627** (+28%) |
| human | defender picks (−2D/−3D) | 334 | 61.7% | **91.5%** | 0.456 | **0.319** (−30%) |

Both directions move the right way: as the attacker the heuristic takes the best die 92% of the
time instead of 62%, and as the defender it drives the *attacker's* equity down by 30–40% instead
of handing back a random face. The residual ~8% is deliberate — that is the `T = 0.12` tail plus
the 2% ε-floor doing their job.

The single most concrete line:

| | lineman | human |
|---|---|---|
| Attacker took a **Skull** with a better die on the table | **23** | **31** |
| Expected under the §6.3 heuristic | **2.1** | **3.5** |

54 self-inflicted turnovers across 200 games, reduced to ~6 — from one table of five constants,
with none of the context multipliers switched on.

### 0.5.5 What the block dice actually look like

Useful for calibration, since it says which rows of the §2.4 table carry the weight:

| Dice count | lineman | human |
|---|---|---|
| 1D | 790 | 528 |
| 2D | 170 | 294 |
| −2D | 82 | 333 |
| −3D | 0 | 1 |
| 3D | 0 | 0 |
| **Total blocks** | **1,042** | **1,156** |

Skill-less ST3 linemen can never reach 3D, and the human tier reaches it zero times in 100 games —
so the 3D row of the dice table is presently untested by any parity tier. The `−2D` count more than
quadruples from lineman to human (82 → 333), which is Guard and assists showing up: the defender-
picks branch of §6.3 is the one that matters most on real rosters, and it is the one both current
agents answer with a coin flip.

---

## 1. The contract

Every decision is reduced to the same three-step pipeline:

```
enumerate  →  score  →  softmax  →  sample
```

```rust
/// One legal option with its heuristic desirability.
#[derive(Clone, Debug)]
pub struct Weighted {
    pub action: Action,
    /// SIGNED desirability, roughly [-1, 1]. §5.3 subtracts an expected turnover cost, so a
    /// genuinely bad option scores below zero. Softmax is shift-invariant, so negatives are
    /// fine; this is a logit input, never a probability.
    pub weight: f32,
    /// Which rule fired, as an id. A `&'static str` cannot carry the numbers the §6 rules
    /// produce ("best destination w=0.49") and §9 forbids `String` in the hot path, so the
    /// rule and its number are stored separately and formatted only under `agent-trace`.
    pub why: Rule,
    pub why_value: f32,
}

/// Rule ids, one per §6 clause. `#[repr(u8)]`, `Copy`, and `Display`-able behind the trace
/// feature — the pair `(why, why_value)` reconstructs every explanation in this document.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rule { ScoreTouchdown, ScoreAdvance, Pickup, Cage, Mark, Screen, Retreat,
                DiceCount, Surf, CarrierTarget, RerollConsequence, CoverageFloor, /* … */ }

/// What `score` fills in. A struct rather than a bare `Vec` so a capped candidate set is
/// never mistaken for a genuinely small one, and so the buffer can be reused (§9).
#[derive(Default)]
pub struct Scored {
    pub options: Vec<Weighted>,
    /// Set whenever the option set was capped — §6.5's top-K activation candidates, §6.30's
    /// inducement subsets, §6.39's multi-block pairs. Paired with a `log()` line naming what
    /// was dropped: silent truncation reads as "considered everything" when it did not.
    pub truncated: bool,
}

/// The scoring half — pure, deterministic, no RNG. Separated from sampling so it can be
/// unit-tested, traced, and benchmarked on its own. Writes into a caller-owned buffer:
/// a `Move` prompt has ~200 options, so returning a fresh `Vec` would allocate on every
/// single move.
pub trait Policy {
    fn score(&self, gs: &GameState, out: &mut Scored);
}

pub struct HeuristicAgent {
    policy: Heuristics,
    rng: Xoshiro256StarStar,
    /// Per-prompt-class temperature (§8), read together with the option count.
    temps: TempTable,
    /// Recomputed once per prompt when the relevant stamp changes (§3).
    features: RefCell<Option<BoardFeatures>>,
    /// Reused scoring buffer — never reallocated between prompts.
    buf: Scored,
}

impl Agent for HeuristicAgent {
    fn act(&mut self, gs: &GameState) -> Action {
        self.buf.options.clear();
        self.buf.truncated = false;
        self.policy.score(gs, &mut self.buf);
        if self.buf.options.is_empty() { return fallback(gs); }
        let t = self.temps.for_prompt(gs.current_prompt(), self.buf.options.len());
        let idx = sample_softmax(&mut self.rng, &self.buf.options, t);
        self.buf.options.swap_remove(idx).action
    }
}
```

### 1.1 Softmax and temperature

```rust
fn sample_softmax(rng: &mut impl RngCore, opts: &[Weighted], t: f32) -> usize {
    let max = opts.iter().map(|o| o.weight).fold(f32::MIN, f32::max);
    let mut acc = 0.0f32;
    let mut cum: Vec<f32> = Vec::with_capacity(opts.len());
    for o in opts {
        acc += ((o.weight - max) / t).exp();   // shift for numerical stability
        cum.push(acc);
    }
    let r = (rng.next_u64() as f64 / u64::MAX as f64) as f32 * acc;
    cum.partition_point(|&c| c < r).min(opts.len() - 1)
}
```

**Design note on the scale.** The weights in the brief (0.9 / 0.8 / 0.05 …) live in [0, 1]. A plain
softmax at `T = 1` over that range is almost uniform: `exp(0.9)/exp(0.05) = 2.3`, so "Attacker
Down" would be picked 20% of the time against "Defender Down". That is not what the numbers mean —
they are meant to read as *strong* preferences. So temperature is the knob that turns the stated
weights into the intended behaviour, and it is **per prompt class**, not global (§7).

Reference points for the block-dice example (weights 0.9 / 0.8 / 0.7 / 0.4 / 0.05):

| T | P(best) | P(worst) | Character |
|---|---|---|---|
| 1.00 | 0.27 | 0.12 | ~uniform — useless |
| 0.30 | 0.42 | 0.05 | exploratory |
| **0.15** | **0.55** | **0.007** | **default: clearly good, still samples the tail** |
| 0.05 | 0.86 | ~0 | near-greedy |
| 0.02 | 0.99 | ~0 | greedy — effectively argmax |

Two floors are non-negotiable:

- **`T > 0` always.** Never argmax. Coverage dies without the tail.
- **ε-floor.** After softmax, mix with the uniform distribution:
  `p_i ← (1 − ε) · p_i + ε / n`, with `ε = 0.02`. This guarantees every legal option is reachable
  even when the weight spread is extreme, which is the "consider ALL actions" requirement made
  literal, and it guarantees a long coverage run eventually reaches every branch.

### 1.2 Why score and sample are separate

`Policy::score` is pure and RNG-free; `HeuristicAgent::act` is the only thing that touches the RNG.
That split costs nothing and buys three concrete things:

- **Testability** — every §6 heuristic can be asserted on directly, with no RNG to stub. §10.1's
  ordering tests all call `score`.
- **Traceability** — dumping the scored option set with its `why` strings explains any decision the
  agent made, which is how a bad weight gets found.
- **Benchmarking** — §9's budget is measured on `score`, the part that actually costs anything.

`HeuristicAgent` must be **deterministic given (state, rng state)**. No `HashMap` iteration order in
any scoring path — enumerate from sorted vectors only. This is the same discipline the parity agents
already follow.

> *Motivation, not scope:* this is also the interface a Monte-Carlo tree search would want if one
> is ever built — scores as a prior, near-zero temperature for playouts. That is a reason the split
> is worth keeping, **not** a thing to build here. See the scope note at the top.

---

## 2. Numeric primitives

All of these are `#[inline]`, allocation-free, and shared by every heuristic.

### 2.1 Single-die success

```rust
/// P(d6 >= target), with the natural-1-always-fails / natural-6-always-succeeds rule the engine
/// already applies in DiceInterpreter::is_skill_roll_successful.
#[inline]
pub fn p_roll(target: i32) -> f32 {
    let p = (7 - target) as f32 / 6.0;
    p.clamp(1.0 / 6.0, 5.0 / 6.0)
}
```

| target | 2+ | 3+ | 4+ | 5+ | 6+ |
|---|---|---|---|---|---|
| p | .833 | .667 | .500 | .333 | .167 |

### 2.2 Re-roll composition

```rust
/// A roll that may be re-rolled once. `p_rr` is the probability the RE-ROLL is available AND
/// taken — for a skill re-roll (Dodge, Catch, Sure Feet, Sure Hands, Pro) it is 1.0; for a team
/// re-roll under Loner(n) it is p_roll(n).
#[inline]
pub fn p_with_reroll(p: f32, p_rr_available: f32) -> f32 {
    p + (1.0 - p) * p_rr_available * p
}
```

`p_roll(4) = 0.5` → with a free skill re-roll, `0.75`. With a Loner(4) team re-roll, `0.625`.

### 2.3 Chained rolls

A move path is a chain of independent rolls (dodges, GFIs, and a pickup at the end):

```rust
#[inline]
pub fn p_chain(steps: &[f32]) -> f32 { steps.iter().product() }
```

Cost of failure is *not* symmetric — failing the last dodge of a 6-square run is a turnover just
like failing the first, but the value already banked differs. The movement scorer handles this by
multiplying value by `p_chain`, which is the correct expectation for a turnover-terminates-drive
model (§5.3).

### 2.4 Block-dice outcome probabilities

Given `n` dice picked by the attacker (`n > 0`) or by the defender (`n < 0`), the probability that
a given face is *available to choose*:

```rust
/// P(at least one of |n| dice shows a face in `set`), attacker picking.
/// For n < 0 the defender picks, so the attacker's realised outcome is the WORST available.
#[inline]
pub fn p_face_available(n: i32, faces: u8) -> f32 {
    let q = faces as f32 / 6.0;            // P(one die is in the set)
    1.0 - (1.0 - q).powi(n.abs())
}
```

Face counts on the block die: Skull 1, Both Down 1, Pushback 2, Pow!/Pushback 1, Pow! 1.

**Block equity is computed, not hand-tuned.** An earlier draft carried six hand-set constants and
bolted skills on afterwards as multipliers (`× 0.70` if the defender has Block). That can never cover
the combinations: Block, Wrestle, Dodge, Tackle and Fend interact, and a 1-die block with Block
against a defender without it is a completely different proposition from the bare 1D the constant
describes. The quantity is a **closed form** — over `|n|` dice there are only `6^|n|` equally likely
outcomes — so compute the whole table once at startup:

```rust
/// Expected desirability of a block at `n` dice, given the skills that change what a face means.
/// 6^3 = 216 outcomes at the worst, × a few hundred skill combinations = a table built in
/// microseconds at startup and read with two array indices forever after.
pub fn block_equity(n: i32, flags: BlockSkills) -> f32 {
    // for every combination of |n| dice: take the best (or, for n < 0, the worst) face by the
    // §6.3 face value under `flags`, and average.
}

bitflags! { pub struct BlockSkills: u8 {
    const ATT_BLOCK = 1; const ATT_WRESTLE = 2; const ATT_TACKLE = 4;
    const DEF_BLOCK = 8; const DEF_DODGE = 16; const DEF_FEND = 32;
} }
```

The skill-less row of that table reproduces the brief's calibration closely enough to keep it as the
sanity check, which is why the numbers are pinned in a test (§10.1):

| dice | 3D | 2D | 1D +Block | 1D | −2D | −3D |
|---|---|---|---|---|---|---|
| brief weight | 0.90 | 0.60 | 0.40 | 0.25 | 0.10 | 0.025 |

The same computation supplies §6.4's `E[best]` — one function, two callers, no second table to keep
in sync. Context that is *not* a skill (out-of-bounds pushes, ball possession, tempo) stays a
multiplier on top, in §6.2.

**Edition note.** The block die and the `find_nr_of_block_dice` ladder are **edition-invariant**;
only the multi-block modifiers differ, and `find_nr_of_block_dice` already takes `rules`. Nothing in
this subsection needs an edition gate — unlike the armour, casualty and TTM/KTM budget rules, which
do (§14, D9).

---

## 3. Board features — computed once, read many

Almost every heuristic wants the same handful of derived facts. Computing them per option is what
would make this slow; computing them once per prompt makes it free.

```rust
pub struct BoardFeatures {
    /// Identity of the POSITION — coordinates, states, ball. Everything below except
    /// `acting` is keyed on this.
    positions_stamp: u64,
    /// Identity of the ACTING PLAYER — who is up and how much MA they have spent.
    acting_stamp: u64,
    /// tz[coord] = number of opposing tackle zones on that square, per side.
    tz_home: [u8; 26 * 15],
    tz_away: [u8; 26 * 15],
    /// occupancy: None / Some(side, standing)
    occ: [Occ; 26 * 15],
    /// Opponent counts per row, prefix-summed along x — the `lane` term (§5.1) reads a
    /// corridor between a square and the endzone in O(1) instead of scanning players.
    row_prefix: [[u8; 27]; 15],
    ball: BallInfo,          // loose / carried-by / who / where
    carrier: Option<PlayerId>,
    carrier_side: Option<TeamSide>,
    /// Who has already acted this turn. Read by `mark_value` (§5.5) and by the
    /// `used_this_turn` factor (§6.5.1); cleared on the turn key both existing agents
    /// already track. A bitset, not a `HashSet` — §9 forbids `HashMap`/`HashSet` iteration
    /// in any scoring path, and 22 players fit in a `u32`.
    activated: BitSet22,
    /// Per opposing player: the set of squares they can reach THIS coming turn, bucketed by
    /// how many rolls it costs them (0 rolls / 1 roll / 2+ rolls). Used for the threat map.
    /// Built LAZILY and only over the squares a caller asks about (§5.2).
    threat: ThreatMap,
    /// Team-level state: re-rolls left, turn number, score, half, turns left in the half.
    tempo: Tempo,
}
```

**Invalidation — two stamps, not one.** `positions_stamp` is
`(turn_nr, half, home_playing, field_model_version)`; `acting_stamp` is
`(acting_player_id, current_move)`. They are separate because the acting player spending a square
changes nothing about the tackle-zone map, the row prefixes or the threat map — and with whole-path
answers (§4.1b) an activation now stays alive across several prompts, so a single combined stamp
would rebuild all three on every square for no reason.

`field_model_version` is a counter the engine already bumps on every coordinate/state write, or, if
that does not exist, a 64-bit FNV over `player_coordinates ++ player_states ++ ball_coordinate` —
30 players, ~2 µs. Anything cheaper than recomputing the threat map is a win.

**Tackle-zone map.** One pass over `field_model.player_coordinates`: for each standing player with
tackle zones, `+1` to all eight neighbours in the opposing team's array. ~240 increments, ~1 µs.
This replaces the naive per-square "scan all 30 players" loop that would otherwise make the movement
scorer O(squares × players).

**Row prefixes.** One pass over the occupancy grid: `row_prefix[y][x]` = number of opposing players
in row `y` at column `< x`. 405 additions, well under a microsecond, and it turns the `lane` term's
"how many opponents lie between this square and the endzone" into two array reads.

**Threat map** (§5.2) is the expensive one — a bounded BFS per opposing standing player, MA + 2
deep, ~O(30 × 60) = 1800 node visits worst case. Two things keep it affordable: it is built only for
prompts that read it (`Move`, `ActivatePlayer`, `Pushback`, `FollowUp`, `HitAndRun`), and it is
**bounded to the squares the caller actually asks about** — the `Move` scorer only ever reads it on
the 60–200 squares in `reach`, so the BFS is restricted to that set's one-square dilation rather
than the whole pitch.

---

## 4. The move engine

This is the single most valuable component, and the one that unblocks the dead-step frontier. It is
also the only place where the design deliberately breaks with the existing agents' behaviour.

### 4.1 The problem with the current `Move` prompt

`AgentPrompt::Move` offers only the **eight adjacent squares** (`legal_move_targets`), and
`Action::Move { path }` is answered with a single square. The engine re-prompts after each step. So
a "movement decision" is really a sequence of up to MA + 2 single-square decisions, and the existing
agents end the activation after the first one.

### 4.1a The engine already walks a whole path — this is not a new capability

**Verified in the code, not assumed.** `Action::Move { path }` is handled by each edition's
`StepInitMoving::handle_command`, which stores the whole path in `self.move_stack`
(`step/bb2025/move_/step_init_moving.rs:70`). `execute_step` then pops **one square per pass**
(`:245`), republishing the remainder as `StepParameter::MoveStack` so the sequence's other steps
carry it, and `AgentPrompt::Move` is emitted **only when the stack is empty** (`:376`).

This is a faithful port of Java's `fMoveStack` / `UtilServerPlayerMove.fetchMoveStack`. So:

> Submitting `Action::Move { path: [a, b, c, d, e, f] }` walks all six squares — dodges, GFIs and
> all — with **zero further `Move` prompts**.

Both current agents send `vec![single_square]`. The step-by-step protocol is an **agent
convention, not an engine constraint**, and it is the whole reason the measured maximum in §0.5.1
is exactly 1.

### 4.1b So: submit the whole path

Given 4.1a, the earlier plan-and-walk-with-a-cache design is solving a problem that does not exist.
Answer the first `Move` prompt of an activation with the complete path and let the engine walk it.

```rust
let reach = reachable(&gs.game, &features, player_id);      // one Dijkstra, ~20 µs
let dest  = sample_softmax(&mut self.rng, &score_all(&reach), T_MOVE);
Action::Move { path: reach[dest].path.clone() }             // the whole thing, in order
```

What this removes: the `MovePlan` cache, the stamp-lineage bookkeeping, the "is the next planned
square still in the offered set" check, and N−1 agent re-entries per activation.

**Be honest about the size of the win.** The prompt loop is in-process function calls, not IPC, so
the round-trips were never expensive. Against §9's budget a 6-square move costs 20 µs + 6 × 1 µs
either way; the real saving is one Dijkstra instead of a cache-validity decision per square, and a
great deal less agent state. Take it for the simplicity, not the microseconds.

**What still interrupts.** Submitting the path does not mean the agent goes quiet. The engine will
still prompt mid-path for dodge re-roll offers, skill uses, Diving Tackle, Shadowing and Fend —
those are different prompt variants, answered by §6.14 and §6.16. Only the *movement* decision is
pre-committed.

### 4.1c The blocker: `is_valid_move` is ported but never called

`UtilServerPlayerMove::is_valid_move` exists and is correct
(`util/util_server_player_move.rs:61` — "checks that the acting player is at coordinateFrom, drops
the command if not"). **No edition's `StepInitMoving` calls it.** All three carry an explicit
no-op comment:

```
step/bb2025/move_/step_init_moving.rs:25
  no-op: UtilServerPlayerMove.isValidMove path validation not ported;
         agent-submitted paths are trusted.
```

Java calls it in `handleCommand` before accepting the move. Today the gap is invisible because both
agents submit one square at a time and it is always valid. Submit a six-square path, let a Diving
Tackle or a Shadowing move relocate the player at square three, and the engine keeps popping the
**original** coordinates — the player teleports, where Java would have dropped the command as
out-of-sync.

> **Turning multi-square movement on is exactly what surfaces this latent divergence.** Wire
> `is_valid_move` into each edition's `StepInitMoving::handle_command` *first* — it is a call to an
> already-ported function plus a re-prompt on failure — and treat it as a prerequisite of phase 4,
> not a follow-up. Add it to `docs/BACKLOG.md`.

The re-prompt on failure is also what gives the agent its re-planning trigger back, for free and in
the right place: the engine says "your path is stale", the agent runs `reachable` again from where
the player actually is.

### 4.1d Free input the engine already computes

`UtilServerPlayerMove::update_move_squares` is **fully ported** and writes
`field_model.move_squares`, where each `MoveSquare` carries `minimum_roll_dodge` and
`minimum_roll_gfi` (`types/move_square.rs:14`) — the engine's own target numbers for every square
adjacent to the acting player.

Use them for the first step of any path instead of recomputing, and keep §4.2's own computation for
steps 2..n (the move-square set is only refreshed as the player actually moves). Same principle as
everywhere else in this document: if the engine already knows the number, do not derive it again.

### 4.2 Probability-weighted reachability

The existing `PathFinderWithPassBlockSupport` is a *distance* A\*: it blocks tackle zones outright
(`block_tacklezones`) rather than costing them. That is the wrong objective — the agent needs
"cheapest **in expectation**", where a dodge is a probabilistic cost, not a wall.

Add a new module, `crates/ffb-engine/src/agent/reach.rs`, leaving the existing pathfinder untouched
(it is used by client logic modules and by `on_the_ball_mechanic`):

```rust
pub struct ReachEntry {
    pub coord: FieldCoordinate,
    /// Squares of MA spent to arrive (including GFI squares).
    pub cost: u8,
    /// Product of every roll on the best path here (dodges, GFIs, but NOT the terminal action).
    pub p_arrive: f32,
    /// Number of GFI squares used — a turnover risk multiplier the value function reads.
    pub gfi: u8,
    /// The best path to this square, in order, EXCLUDING the start and INCLUDING `coord`.
    /// This is what goes straight into `Action::Move { path }` — see §4.1b.
    pub path: SmallVec<[FieldCoordinate; 10]>,
    /// Per-step survival probability, parallel to `path`. Lets a caller find the risky
    /// square, truncate a path at an acceptable risk level, or explain the choice in `why`.
    pub p_step: SmallVec<[f32; 10]>,
}

/// Dijkstra over (−log p_arrive) with a hard cap at the player's REAL remaining budget:
///   standing  → MA + 2
///   prone     → MA − STAND_UP_COST + 2   (STAND_UP_COST = 3, mechanics/movement.rs)
///   prone and MA ≤ 3 → the whole activation is gated behind a stand-up roll, so every
///                      p_arrive is additionally multiplied by p_roll(4)
/// A prone MA-6 lineman reaches FIVE squares, not eight; scoring it against eight is
/// choosing among options that were never available.
/// Returns EVERY reachable
/// square with its best path and that path's success probability — one call per activation
/// answers the whole movement question.
/// ~60 reachable squares typical, ~120 worst case. Target: < 20 µs.
pub fn reachable(g: &Game, f: &BoardFeatures, player_id: &str) -> Vec<ReachEntry>;
```

Edge cost from `a` to `b` for player `p`:

```
p_step = 1.0
if tz_opposing(a) > 0:                        # leaving a tackle zone → dodge
    target = DodgeModifierFactory::minimum_roll_edition(
                 st, ag, &dodge_factory.find_applicable(&DodgeContext::new(g, ap, a, b)), rules)
    p_step *= p_with_reroll(p_roll(target), rr_dodge(p))      # Dodge skill = 1.0, else team RR
if cost_so_far >= MA:                         # GFI / Rush square
    target = GoForItModifierFactory::minimum_roll_going_for_it(
                 &gfi_factory.find_applicable(&GoForItContext::new_with_moles(
                     g, p, teams_with_moles_under_pitch)))
    p_step *= p_with_reroll(p_roll(target), rr_gfi(p))        # Sure Feet = 1.0, else team RR
```

**The GFI target is not always 2+.** Both `modifiers/bb2025/go_for_it_modifier_collection.rs` and
`modifiers/mixed/…` register a **Blizzard +1** — `minimum_roll_going_for_it` returns **3** in a
blizzard, in every edition — plus two *Moles under the Pitch* entries from the kickoff table. The
mixed collection's own doc comment records this exact mistake already producing a divergence (undead
bb2016 seed 21 step 5: a prone Mummy's stand-up rush with a 2, which Java failed and Rust let
through). An earlier draft of this document hardcoded `target = 2`; it was the same bug class as the
weather-gated pass range that fixed human seed 16 (`e1f28183`).

`minimum_roll_edition` and `find_applicable` are exactly the functions the engine's own dodge step
uses (`crates/ffb-mechanics/src/modifiers/dodge_modifier_factory.rs:294` / `:58`), so the agent's
probabilities are *the engine's own numbers* — Stunty, Break Tackle, Tackle, Prehensile Tail, Titchy
and every edition difference are handled for free. This is the single most important reuse decision
in the whole design: **never re-derive a target number the mechanics crate can compute.**

Cost of the Dijkstra: 60–120 nodes, one `find_applicable` per TZ-exit edge. `find_applicable` is the
likely hot spot, but **the obvious memo key is unsound**: the applicable set depends on *which*
opponents are adjacent, not how many. Prehensile Tail and Tackle are per-opponent skills, and
`count_prehensile_tails` is a separate term in the factory, so two destinations with one tackle zone
each can carry different targets. Key the memo on **`(from_sq, to_sq)`** for the duration of a single
`reachable` call — sound, still hits on revisits, and dies with the call so it can never go stale.
Measure before assuming it is needed at all.

The first edge of every path is free: §4.1d's `field_model.move_squares` already carries
`minimum_roll_dodge` and `minimum_roll_gfi` for each square adjacent to the acting player, computed
by the engine itself. Read those and skip the factory entirely on the most frequently evaluated edge
in the whole search.

**Team re-roll accounting.** `rr_gfi` / `rr_dodge` must not assume the team re-roll is available for
*every* roll on a path — it can be spent once per turn. Model it as: the re-roll is worth its full
value on the **first** roll of the path and zero afterwards, unless the player has the matching
skill. This is a deliberate under-estimate for long paths, which is the safe direction.

**And it must know whether the re-roll still exists.** `tempo.team_rr_available` gates the whole
thing: once the re-roll has been consumed — on a dodge two activations ago, on a block — every
subsequent path must price its first roll at the bare `p_roll(target)`. Without it a 4+ dodge scores
0.75 for the entire turn when after the first use it is really 0.50. It also has to be part of the
`reachable` **cache key**, or two identical boards with different re-roll state share an entry and
one of them gets the wrong answer.

---

## 5. The value model

`reachable` says *can I get there and how likely*. The value model says *do I want to be there*.

### 5.1 Square value, per intent

```
V(sq, p) = base_intent(sq, p) · sideline(sq, p) · exposure(sq, p) · lane_if_relevant(sq, p)
```

**Modifiers belong to the intent, not to the formula.** `lane` measures opponents in a corridor
between the square and *the endzone*, which is the right question for a runner and the wrong one for
a marker — a marker WANTS to be near opponents, and a global `lane` multiply penalises it for doing
its job. So the intents carry their own modifier sets:

| intent | `sideline` | `exposure` | `lane` |
|---|---|---|---|
| `Score`, `Pickup` | ✅ | ✅ | ✅ |
| `Cage`, `Mark`, `Screen` | ✅ | ✅ | — |
| `Retreat` | ✅ | ✅ | — |

**`base_intent`** — the term the brief calls "distance to endzone". What makes a square of ground
valuable is that it saves *activations*, so progress is measured against what this activation could
possibly achieve, not against the distance remaining:

```rust
let d_now    = endzone_distance(cur, side);       // |x - 25| for home, |x - 0| for away
let d_sq     = endzone_distance(sq, side);
let max_gain = d_now.min(ma + 2);                 // the most this activation could cover
let advance  = ((d_now - d_sq) as f32 / max_gain.max(1) as f32).clamp(0.0, 1.0);
```

An earlier draft divided by `d_now`, which made the same six squares worth 0.50 from `d = 12` and
0.25 from `d = 24` — progress is not a fraction of what is left, and the old form quietly told a
deep carrier that running was only half as useful.

**Urgency — the clock.** A drive is a multi-turn plan, and whether it can still finish changes every
weight on the board:

```rust
let turns_left     = (TURNS_PER_HALF - turn_nr).max(0);          // 8 per half
let turns_to_score = ((d_sq as f32) / (ma as f32)).ceil() as i32;
let slack          = turns_left - turns_to_score;
let urgency        = (1.0 - slack as f32 / 3.0).clamp(0.0, 1.0); // 1.0 once it is tight
```

`urgency` multiplies the carrier's `base_intent` by `0.75 + 0.5 · urgency` (so a patient turn 1 is
0.75 and a turn-7 sprint is 1.25), and scales `w(EndTurn)` by `1 − 0.4 · urgency` — with the half
running out, stopping early gets less attractive at exactly the rate scoring gets more so.

| intent | `base_intent` |
|---|---|
| **Score** (carrier, `d_sq == 0`) | `1.0` — a touchdown ends the scoring blackout; this is the whole point |
| **Score** (carrier, otherwise) | `0.15 + 0.85 · advance` |
| **Pickup** (loose ball on `sq`) | `p_pickup(sq, p)` — see §5.4 |
| **Cage** (non-carrier, own carrier exists) | `0.35 + 0.40 · threat_share(corner)` on the four diagonal cage squares, `0.35` if merely adjacent, else `0.0`. A real cage is built on **the side the opponent is on**: `threat_share` is that corner's share of the total threat around the carrier, so the exposed corners get filled first and a flat 0.55 no longer treats all four alike |
| **Mark opponent** (non-carrier) | `0.50 · mark_value(opp)` for the best opponent adjacent to `sq` (§5.5) |
| **Screen** (non-carrier) | `0.45 · path_share(sq)` — the fraction of the opponent's shortest paths to our carrier (or to the loose ball) that pass through `sq`, straight off §5.2's threat BFS |
| **Retreat / stand-by** | `0.10` — the floor that keeps "shuffle harmlessly" in the support |

The intents are not mutually exclusive: **take the max**, and record which one won in `why`. A
square that both advances the carrier and is a cage corner scores as the better of the two.

**A screen is a line, not a huddle.** An earlier draft scored `Screen` as
`0.30 · (1 − distance_to_ball)`, which is maximised by standing **on** the ball — the opposite of
screening. A screen sits *between* the ball and the threat, spaced so nobody slips through. Scoring a
square by how many of the opponent's shortest paths to our carrier run through it gets both
properties for free: the screen forms on the threatened side, and it spaces itself, because once a
square is occupied the paths through it stop being shortest. The BFS that produces those paths is
already built for §5.2.

**`sideline`** — straight from the brief:

```rust
let sideline = if !is_next_to_sideline(sq) { 1.0 }
               else if p.has(SkillId::SideStep) { 1.0 }
               else { 0.25 };
```

Sideline squares invite a push-out crowd-surf, which for a carrier is a turnover *and* an injury
roll. The pitch is `x ∈ 0..=25`, `y ∈ 0..=14`, with the sidelines at `y == 0` and `y == 14`
(`FieldCoordinateBounds::SIDELINE_UPPER` / `SIDELINE_LOWER`), so `is_next_to_sideline(sq)` is
`sq.y == 0 || sq.y == 14`. Also half-penalise (`0.6`) `y == 1` and `y == 13` for a carrier, because
a single push from there reaches the crowd.

**`exposure`** — the brief's "enemies that can reach us without re-rolls":

```rust
let threat = f.threat.pressure_on(sq, opposing_side);   // Σ over opponents, see §5.2
let exposure = 1.0 / (1.0 + threat);
```

**`lane`** — is there anything actually *between* this square and the endzone? `base_intent` is a
pure function of `|x − endzone|`, so without this term every square in a column scores identically:
the §7.2 example ties five squares at the best weight and §7.3 ties ten, visible as bands in the
heatmaps. `lane` is the missing lateral signal, and it is a real one rather than a tie-breaker —
running into an empty corridor is worth more than running into the pack:

```rust
// opponents within ±2 rows of sq.y, between sq.x and the endzone — two prefix reads per row
let corridor = (sq.y - 2 ..= sq.y + 2)
    .filter(|y| (0..=14).contains(y))
    .map(|y| f.opponents_between(y, sq.x, endzone_x, side))
    .sum::<u32>();
let lane = 1.0 / (1.0 + 0.35 * corridor as f32);
```

Five rows wide because a carrier can drift; `row_prefix` (§3) makes each row O(1). This is what
makes a wide-zone run and a run into a three-man screen score differently, which no other term
does.

### 5.2 The threat map

For each *opposing* standing player `o`, a bounded BFS (MA + 2, tackle zones cost nothing since the
opponent will be *entering* our zones on their turn, and a blitz is one of their moves):

```
reach_factor(o, sq) =
      1.00   if o can end adjacent to sq with ZERO rolls
      0.55   if it costs exactly one roll (one dodge or one GFI)
      0.25   if it costs two rolls
      0.00   otherwise
```

**A team gets ONE blitz per turn, so only one opponent can actually hit the square.** Everyone else
can move adjacent and mark, which is a much smaller thing. Summing a block term over every reachable
opponent — as an earlier draft did — triples the threat of three approaching players and prices
squares near the pack as though the whole pack could block:

```
threat(sq) =  block_term  +  mark_term

block_term =  max over o of  reach_factor(o, sq) · strength_factor(o, victim)     # the ONE blitz
              — but any opponent ALREADY adjacent needs no blitz, so those are
                maxed in separately and are not gated on `blitz_used`
mark_term  =  0.18 · Σ over the next 2 best o of reach_factor(o, sq)              # they can only mark
```

`max` for the block, a small `sum` for the marking. If the opposing team has already spent its blitz
(`turn_data.blitz_used`, the flag both existing agents filter on), the block term drops to only those
opponents already standing adjacent.

where `strength_factor` scales by how bad the resulting block would be for us:

```rust
let n = ServerUtilBlock::find_nr_of_block_dice(
            find_block_strength(g, o_coord, o_str, sq),   // assists counted at the DESTINATION
            find_block_strength(g, sq, victim_str, o_coord),
            false, false, rules, o_has_horns_and_blitzing);
let strength_factor = match n { 3 => 1.4, 2 => 1.2, 1 => 1.0, -2 => 0.7, -3 => 0.5, _ => 1.0 };
```

`find_block_strength` (`util/server_util_player.rs:23`) already counts assists, Guard, and the
Guard-cancel rule — reuse it verbatim.

Only the top-3 contributions are summed; beyond three markers the marginal danger flattens and the
sum would otherwise dominate every other term.

**Cost control.** The threat map is the expensive feature. Build it once per prompt, and only for
prompts that read it. If profiling says it is still too hot, degrade to the cheap approximation
`threat ≈ tz_opposing(sq) · 0.6 + adjacent_standing_opponents(sq) · 0.4`, which needs no BFS at all
and is already dramatically better than what exists today.

### 5.3 Combining reach and value — and the cost of failing

```rust
weight(sq) = p_arrive(sq) · V(sq, p)  −  (1 − p_arrive(sq)) · c_turnover(sq, p)
```

**This is an expectation, not a discount.** An earlier draft used
`p_arrive · V · gfi_penalty`, which prices a failed dodge as *"you gain nothing"*. In Blood Bowl a
failed dodge, a failed rush or a fumbled pass ends the **team turn**: every player who has not yet
acted is forfeited, and if the mover carried the ball, possession usually goes with it. Multiplying
by `p_arrive` cannot express that, because it has no term for what is lost.

```rust
let unactivated = f.tempo.eligible_remaining as f32 / 11.0;
let c_turnover  = (0.20 + 0.55 * unactivated)      // early in the turn a turnover costs far more
                * if carries_ball { 1.4 } else { 1.0 }
                * (1.0 + 0.15 * gfi as f32);       // a failed rush also drops the player prone
```

Three correct behaviours fall out of this and are not written anywhere else in the document:

- **Risk migrates to the end of the turn.** With ten players still to act, `c_turnover ≈ 0.70`; with
  one, `≈ 0.25`. The same dodge is priced differently depending on how much of the turn it would
  destroy, which is exactly how a coach sequences a turn.
- **A 6+ dodge becomes negative, not merely small.** At `p = 0.167` and `c_turnover = 0.70` a square
  must be worth more than 3.5 to break even; nothing on the board is. It stays in the support via
  the ε-floor and stops being chosen.
- **The carrier stops running through three tackle zones on turn 1** without a special case for it.

The `gfi` term replaces the old `gfi_penalty` table, and does so for a stated reason rather than as
a fudge: a failed rush is a turnover *plus* an armour roll on the player who fell.

**Recalibrate the temperatures after this lands.** Weights are now signed and their spread is wider,
which makes any fixed temperature effectively sharper. §8's values were measured on the
multiplicative form (§7.7a); re-run that sweep and check the top-20 mass — §10.4 is the mechanism.

For the **carrier**, add a hard override: if any reachable square is in the opponent endzone,
`weight = p_arrive` for that square — no turnover subtraction, because scoring ends the drive
anyway — and the prompt is scored with `T = 0.05` (near-greedy). Not scoring when a touchdown is
available is never the right move for either play strength or coverage.

For a **non-carrier when the ball is loose**, the pickup square is scored as
`p_arrive(sq) · p_pickup(sq)` and given intent `Pickup`; every other square is scored normally, so
"walk toward the ball but do not risk the pickup" stays in the distribution.

### 5.4 Action success probabilities (shared helpers)

Every one of these delegates to the mechanics crate; none re-implements a modifier table.

| Roll | Target from | Skill re-roll |
|---|---|---|
| Dodge | `DodgeModifierFactory::minimum_roll_edition` | `Dodge` (once/turn) |
| GFI / Rush | `GoForItModifierFactory::minimum_roll_going_for_it` — **2+ only in fair weather**; Blizzard is +1 and Moles under the Pitch applies too, in every edition | `SureFeet` |
| Pickup | `AgilityMechanic::minimum_roll_pickup` + `PickupModifierFactory` | `SureHands` |
| Catch | `AgilityMechanic::minimum_roll_catch` + `CatchModifierFactory` | `Catch` |
| Pass | `PassMechanic` + `PassModifierFactory` (range ruler, weather) | `Pass` |
| Interception | `AgilityMechanic::minimum_roll_interception` | — |
| Jump / Leap | `AgilityMechanic::minimum_roll_jump` | `Leap`-adjacent |
| Right Stuff (TTM landing) | `AgilityMechanic::minimum_roll_right_stuff` | — |
| Hypnotic Gaze | `AgilityMechanic::minimum_roll_hypnotic_gaze` | — |
| Armour break | `ArmorModifierFactory` + AV | — |
| Negatrait (Bone Head, Really Stupid, Wild Animal, Blood Lust, Take Root, Animal Savagery) | the skill's own registered roll | `Loner`-gated team RR |

### 5.5 `mark_value(opp)` — who is worth standing next to

```
mark_value(opp) =
      1.00  if opp is the enemy ball carrier
      0.70  if opp is adjacent to the loose ball
      0.55  if opp is the fastest unmarked enemy in our half (a deep threat)
      0.45  if opp has already been activated this turn  (free marking — they cannot punish it)
      0.30  otherwise
    · (0.5 if opp is prone or stunned else 1.0)
    · (0.6 if opp already has 2+ of our markers else 1.0)
```

The "already activated" case is the brief's *"0.5 for marking own ball carrier that has already been
activated"* generalised: marking something that cannot immediately move away is cheap value. It
reads `BoardFeatures::activated` (§3) — a 22-bit set, cleared on the turn key both existing agents
already track. Not a `HashSet`: §9 forbids hash iteration in any scoring path, because iteration
order is a determinism hazard.

---

## 6. Per-decision heuristics

Every `AgentPrompt` variant in `crates/ffb-model/src/prompts/agent_prompt.rs`, in enum order.
"Options" is the enumerated legal set; "Weight" is the `Weighted::weight` formula.

Notation: `p` = the player the prompt is about, `me` = the side being asked, `carrier` = current ball
carrier, `T` = softmax temperature for that prompt class.

---

### 6.1 `BlockTarget { attacker_id }` — mid-sequence "no defender" ask

**Context.** Emitted when a block sequence is re-pushed with no defender (bb2016 Blood Lust). Both
existing agents answer `EndPlayerAction`.

**Options.** `legal_block_targets(game, attacker_id, side)` ∪ `{EndPlayerAction}`.

**Weight.** Same table as `BlitzTarget` (§6.2) for each target; `EndPlayerAction` gets
`0.20 + 0.60 · (1 − best_target_weight)` — declining is attractive exactly when every block is bad.

**T = 0.15.**

---

### 6.2 `BlitzTarget { attacker_id, eligible_players }`

**Options.** The supplied candidates (already coordinate-sorted).

**Weight.**

```
w(def) = dice_weight(n)                                  # §2.4 table
       · (1 + 0.35 · is_ball_carrier(def))
       · (1 + 0.30 · can_be_pushed_out_of_bounds(att, def))
       · (1 + 0.20 · def_is_only_marker_of_our_carrier(def))
       · (1 − 0.30 · def_has(Block) · att_lacks(Block, Wrestle))   # both-down risk
       · (1 + 0.15 · att_has(MightyBlow | Claws | PilingOn))
       · (1 + 0.25 · att_has(StripBall) · has_ball(def) · def_lacks(SureHands))
       · (1 − 0.40 · def_has(FoulAppearance) · att_lacks(Nurgle's Rot immunity))
       · surf_bonus(def)
```

where `n = find_nr_of_block_dice(find_block_strength(att…), find_block_strength(def…), …)` and

```
can_be_pushed_out_of_bounds(att, def) = any pushback square from att→def is off-pitch
surf_bonus(def) = 1.9 if can_be_pushed_out_of_bounds && has_ball(def)
                  1.5 if can_be_pushed_out_of_bounds
                  1.0 otherwise
```

Clamp the product to [0.01, 1.0]. **T = 0.15.**

---

### 6.3 `BlockChoice { dice, own_choice, nr_of_dice }` — *the brief's worked example*

**Options.** One per die index. When `own_choice == false` the defender is choosing, and the agent is
answering *as the defender*: invert every weight (`w ← 1 − w`) so the defender picks the result worst
for the attacker.

**Weight** — the brief's table, made state-dependent:

```rust
match block_result_for_roll(dice[i]) {
    Pow => 0.90,

    PowPushback => {                                     // "Defender stumbles"
        if def_has(Dodge) && !att_has(Tackle) { 0.30 }   // becomes a plain push
        else if pushes_out_of_bounds { 0.95 }
        else { 0.80 }
    }

    BothDown => {
        let att_down = !att_has(Block) && !att_has(Wrestle);
        let def_down = !def_has(Block);
        match (att_down, def_down) {
            (false, true)  => 0.70,
            (true,  true)  => if has_ball(def) { 0.50 } else { 0.30 },
            (true,  false) => 0.10,
            (false, false) => 0.35,                      // nothing happens; tempo only
        }
    }

    Pushback => {
        if pushes_out_of_bounds { 0.80 }
        else if has_ball(def) && !def_has(SureHands) && att_has(StripBall) { 0.60 }
        else if pushes_def_off_the_cage_corner || pushes_def_away_from_our_carrier { 0.50 }
        else { 0.40 }
    }

    Skull => 0.05,
}
```

Two refinements over the brief, both cheap:

- `pushes_out_of_bounds` is computed from the *actual* pushback squares
  (`attacker → defender` direction, three candidate squares, any failing `is_on_pitch()`), not
  guessed from `y ∈ {0, 14}` — a defender on the sideline cannot always be surfed; it depends on
  the push direction.
- `(false, false)` Both Down (both have Block) is a real, distinct case: it is a *wasted* block, so
  0.35 — better than Skull, worse than a push.

**T = 0.12** — block-dice choice is where sloppy sampling costs the most, so it runs hotter toward
greedy than the default.

---

### 6.4 `BlockChoiceProperties { can_reroll, reroll_sources }`

This is the "re-roll the block dice?" window. **Options:** `{use, decline}` × each re-roll source.

```
w(use, src) = clamp(best_after − best_now, 0.0, 1.0) · rr_scarcity(src)
w(decline)  = 1.0 − w(use, best_src)
```

`best_now` = the max `BlockChoice` weight over the dice currently showing (§6.3).
`best_after` = `block_equity(n, flags)` from §2.4 — the *same* computed table, so the re-roll
decision and the declaration decision can never disagree about what a fresh set of dice is worth.
An earlier draft carried a five-row constant here that ignored skills entirely; it is gone.

For reference, the skill-less column of that table:

| n | E[best weight], no skills |
|---|---|
| 3 | 0.83 |
| 2 | 0.74 |
| 1 | 0.55 |
| −2 | 0.36 |
| −3 | 0.24 |

`rr_scarcity(src)`: a free per-player source (Brawler, Hatred, Pro, Consummate) = `1.0`; the team
re-roll = `0.55 + 0.45 · (rerolls_left / rerolls_at_half_start)` scaled by turn urgency
(`1.3` on turns 7–8 and 15–16 — hoarding a re-roll into halftime is a pure loss).

**T = 0.20.** Prefer the cheapest sufficient source: when several are available, weight them by
`rr_scarcity` so Brawler/Pro get spent before the team re-roll.

---

### 6.5 `ActivatePlayer { eligible_players }` — the biggest decision

This is a **joint** choice of (player, action, target). Enumerating the full cross product is both
slow and pointless — most of it is bad. Factorise:

```
w(player, action, target) = w_player(player) · w_action(player, action) · w_target(…)
```

and generate candidates as: for each eligible player, for each live action, take the **top 2
targets** by `w_target` plus one uniformly-sampled target from the remainder (so the tail stays
reachable). Set `truncated = true` so the trace and the benchmark can tell a capped option set
apart from a genuinely small one — silent truncation reads as "considered everything" when it did
not. Typical candidate count: 11 players × ~2.5 actions ×
3 targets ≈ 80 options — well inside budget.

#### 6.5.1 `w_player` — activation ordering

Blood Bowl activation order matters enormously, and it is one of the cheapest wins available.

**Two tiers, or this prompt blows the budget.** Every rule below that says "a useful destination"
wants `best_move_weight(p)`, which is a `reachable` call — and running one per eligible player is
**eleven Dijkstras per prompt**, ~220 µs against §9's 50 µs. So:

1. **Tier 1, all eligible players** — a proxy that needs no search: is it the carrier, is it marked,
   is it prone, what is the best block dice count from where it already stands, how far is it from
   the ball, and the best value among its eight *adjacent* squares. That is enough to resolve every
   row below except "free mover".
2. **Tier 2, the top three by proxy** — the full `reachable` call, which refines their weights and
   supplies the destination the declaration will carry.

Tier 2 results are cached by `(positions_stamp, player)` (§3), so the `Move` prompt that follows the
activation reuses the Dijkstra the activation already paid for.

```
w_player(p) =
    0.95   if p can score a touchdown this activation           (reach ∩ endzone, p_arrive > 0.5)
    0.90   if p is the carrier and the carrier is marked        (get out before the blitz)
    0.85   if p can pick up a loose ball with p > 0.5
    0.80   if p is a blitz candidate with 3D or 2D on the enemy carrier
    0.70   if p is prone/needs standing up and is marked
    0.60   if p is a blocker with 3D/2D available                (spend blocks BEFORE moves —
                                                                  assists are worth more early)
    0.45   if p is a free mover with a useful destination
    0.30   if p is a lineman with nothing to do
  · 0.55   if p has an unresolved negatrait (Bone Head, Really Stupid, Take Root, Wild Animal,
           Blood Lust) — a failed roll ends the activation, so activate these when the turn's
           critical work is already banked. NOTE this is the ORDERING factor only; the VALUE of
           everything such a player is offered is separately multiplied by the negatrait's
           success probability in `w_action`, because with probability (1 − p) none of it happens
  · 0.30   if already in `used_this_turn`
```

The `used_this_turn` factor is `0.30` rather than `0.0` on purpose: the engine's eligible list can
legitimately re-offer a player, and hard exclusion is what makes the existing agents' turns end
early. Combined with the ε-floor it stays sampleable but rare. Keep the `is_prone && !is_active`
filter both existing agents apply — that one *is* a correctness filter, not a preference.

`EndTurn` is always in the option set, with

```
w(EndTurn) = 0.05 + 0.90 · (1 − max_player_weight)
```

so the turn ends when nothing useful remains, and essentially never before.

#### 6.5.2 `w_action` — which action to declare

Apply the same staleness filters `UniformAgent` already implements (`blitz_used`, `pass_used`,
`hand_over_used`, `foul_used`, `ttm_used`/`ktm_used`, and the edition-specific TTM/KTM budget
sharing) — a stale declaration wastes the whole activation.

| Action | Weight |
|---|---|
| `Move` | `best_move_weight(p)` from §5.3 |
| `Block` | `max_target dice_weight(n) · context` (§6.2) |
| `Blitz` | `max over (destination, target)` of `p_arrive(dest) · block_equity(n at dest) · context`, `× 1.25` if the target is the enemy carrier, `× 0.5` if the team's blitz is better spent later (turn ≤ 2 and the carrier is not yet threatened). **Only the top 3 destinations by `p_arrive` get a full assist recount** — `find_block_strength` per (destination, target) pair is the most expensive thing in the scorer, and adjacent candidate squares differ by one player's position, so the assist count is updated incrementally along the frontier rather than rescanned |
| `StandUpBlitz` | as `Blitz` × `0.85` |
| `Pass` | §6.5.3; `× 0.3` if the receiver is marked, `× 0.15` if a hand-off would do the same job |
| `HandOff` | `p_catch(receiver) · V(receiver_sq) · 1.1` — cheaper than a pass and cannot be intercepted |
| `HailMaryPass` | `0.10 + 0.35 · (turn is 8 or 16)` — a desperation action; low but never zero |
| `Foul` | §6.5.4 |
| `ThrowTeamMate` / `KickTeamMate` | `p_right_stuff(landing) · V(landing_sq)`, `× 2.0` if the thrown player carries the ball and lands in scoring range |
| `ThrowBomb` | `0.35 · (number of opponents within 1 of the target square) / 3` |
| `Punt` | `0.25`, `× 2.0` if the carrier is trapped with no scoring path |
| `MultipleBlock` | `dice_weight(n₁) · dice_weight(n₂) · 0.85` (the −2 attacker modifier is already in `find_nr_of_block_dice`) |
| `Stab`, `BreatheFire`, `ProjectileVomit`, `HypnoticGaze` | `0.45 · target_value(def)`; Gaze `× 1.6` if the target is the carrier (removing tackle zones opens a path) |
| Star specials (`Treacherous`, `BlackInk`, `RaidingParty`, `LookIntoMyEyes`, `BalefulHex`, `CatchOfTheDay`, `ThenIStartedBlastin`, `AllYouCanEat`, `FuriousOutburst`, `ThrowKeg`) | `0.40` base, raised to the coverage floor while the live dispatch counter for that step is still zero. They are rare and coverage-critical, and hand-tuning ten of them is not worth it until traces say otherwise. **Never below the `Move` floor**, or they go dead again the way Kick Team-Mate did |
| `StandUp` | `0.50 + 0.40 · (p is marked)` |

**Coverage floor — live, not a static table.** Any action whose `StepId` has not dispatched **yet in
this run** gets `max(w, 0.35)`; the floor decays to nothing once it has fired a handful of times:

```rust
let seen = self.dispatch_counts[step_id];        // incremented from the engine's own dispatch
let floor = 0.35 * (1.0 - (seen as f32 / 4.0).min(1.0));
w = w.max(floor);
```

An earlier draft read a static table "regenerated from `docs/DEAD_STEP_INVENTORY.md`". That is stale
the moment a mechanic starts firing, cannot react within a run, and quietly rots as the inventory
ages — the inventory itself warns that `CatchOfTheDay` left the dead list purely by going from 2
seeds to 10. Counting live makes the floor a coverage *search*: whatever has not happened gets
pushed, and stops being pushed as soon as it does.

This is the mechanism that keeps a good agent from *reducing* coverage relative to the uniform
one — the exact failure mode a competent policy would otherwise introduce, and the reason 509
Throw-Team-Mate declarations in §0.5.2 are a feature to preserve rather than noise to optimise
away.

**A second axis: rare situations, not just rare actions.** The floor above boosts *declarations*. A
great deal of engine code is reachable only from a board **configuration** — a throw-in from a
particular sideline, a touchback, a full cage, a player with three markers, a ball bouncing into the
crowd. Nothing in a per-action floor ever pushes toward those.

```rust
// coarse board descriptor -> a few thousand buckets, counted per run
let bucket = hash(ball_zone, carrier_marked_bucket, players_in_crowd, turn_bucket, weather);
let novelty = if self.seen_buckets[bucket] == 0 { 0.08 } else { 0.0 };
w += novelty;
```

Deliberately **additive and small** — capped well below the action floor, so it can nudge between
otherwise comparable options and can never override a real decision. A novelty bonus large enough to
change genuine play has stopped being a coverage tool and become a bug; log it, and keep it that
way.

#### 6.5.3 Pass / hand-off receiver weight

```
p_complete = p_pass(thrower, recv_sq)              # PassMechanic + range ruler + weather
           · p_catch(recv)                         # CatchModifierFactory
           · (1 − p_intercepted(line))             # any opponent on the line with AG check

w(recv) = p_complete · V(recv_sq, recv) · (1 + 0.5 · recv_can_score_after_catch)
        − (1 − p_complete) · c_turnover            # §5.3 — a fumble ends the turn on the spot
```

The turnover term is not optional here. A **fumbled** pass is the worst outcome in the game short of
a goal-line turnover: the ball drops at the thrower's feet, the team turn ends, and every unactivated
player is forfeited. Scoring a pass on completion probability alone — as an earlier draft did — makes
a Long Bomb in a Blizzard look like a merely low-value option instead of a catastrophic one.

**Throwing it away is a legal option and belongs in the set.** At the end of a turn, with the carrier
about to be surrounded, throwing at an empty square deep in the opponent's half is a real play: it
cannot be intercepted into a touchdown and it moves the ball away from the pack. Enumerate the
best few empty target squares alongside the receivers, at

```
w(empty_sq) = 0.25 · (1 − p_opponent_recovers_first(empty_sq)) · urgency_inverse
```

where `urgency_inverse` is `1 − urgency` (§5.1) — throwing the ball away is for when the drive is
already lost, not when it is live.

Weather matters and the engine already knows it — a Blizzard truncates the range bands, and this is
the exact bug that fixed human seed 16 (`e1f28183`). `PassModifierFactory` handles it.

#### 6.5.4 Foul target weight

```
w(foul_target) =
      p_armour_break(target, net_assists)          # ArmorModifierFactory with foul assists
    · injury_value(target)                         # 1.0 carrier, 0.8 key positional, 0.4 lineman
    · (0.35 + 0.65 · already_prone_and_valuable)
    · ref_risk_factor                              # 1.0 with a bribe available, 0.55 without
    · (0.4 if it is not the last action of the turn else 1.0)   # foul last — ejection ends the turn
```

Note `legal_foul_targets` already restricts to prone/stunned opponents.

**T = 0.25** for `ActivatePlayer` — deliberately the *flattest* of the play temperatures. Activation
is where option diversity buys the most coverage, and where the heuristic is least certain.

---

### 6.6 `Move { player_id, squares }`

**Submit the whole path** (§4.1b). On a `Move` prompt:

1. `reach = reachable(game, features, player_id)` — one Dijkstra, every square with its best path
2. score every entry with §5.3
3. add `EndPlayerAction` with **`w = 0.0`** — see below
4. softmax-sample a destination
5. answer with `Action::Move { path: reach[dest].path.clone() }` — the complete path

The engine's `move_stack` walks it (§4.1a) and does not re-prompt for `Move` until the stack is
empty. A second `Move` prompt for the same player therefore means something interrupted — a failed
dodge that did not end the activation, a Shadowing/Diving Tackle relocation, or (once §4.1c is
wired) a rejected stale path. Treat it as a fresh decision: re-run `reachable` from wherever the
player actually is now. No cache, no stamp lineage, no plan invalidation logic.

**The decline option is worth exactly zero, and that is not a tuning choice.** Ending the activation
banks what the player already has: no gain, no risk. On §5.3's expectation scale — where every other
option is `p·V − (1−p)·c_turnover` — that is `0.0` by construction. Options with positive expectation
beat it; options with negative expectation (a 6+ dodge into a screen) lose to it, automatically.

An earlier draft used `w = 0.05 + 0.55 · (1 − best_dest_weight)`, which silently assumed destination
weights ran close to 1.0. On real boards they run around 0.37, so that formula produced **0.399 —
higher than every single destination**. Both §7 examples had "stand still" as their top-weighted
option and nobody noticed until the numbers were computed on an actual pitch. With `w = 0.0` the
decline lands at p ≈ 0.0003, which is what it should be for a carrier with the whole turn ahead of
it.

Important: the offered `squares` are the eight neighbours, so `path[0]` must be one of them —
assert it and fall back to the offered set if not. Never answer with a first square the prompt did
not offer.

`squares.is_empty()` → `EndPlayerAction` (both existing agents do this; an empty Move loops forever).

**T = 0.20**, except the touchdown override (`T = 0.05`, §5.3).

---

### 6.7 `ThrowTeamMateTarget { thrower_id, thrown_player_id }`

**Options.** Every square within the TTM range ruler (short/long), each scored:

```
w(sq) = p_right_stuff_landing(thrown, sq)      # AgilityMechanic::minimum_roll_right_stuff
      · (1 − p_scatter_off_pitch(sq))          # 3 scatter squares from sq
      · V(sq, thrown)
      · (2.0 if thrown carries the ball && endzone_distance(sq) == 0)
      · (0.4 if any opponent is adjacent to sq)
```

Note the existing agents throw a fixed 3 squares forward. Keep that square in the option set (it is
the parity-mirrored answer) but let the scorer choose.

**T = 0.20.**

---

### 6.8 `PuntTarget { player_id, squares }`

**Options.** The offered squares.

```
w(sq) = deep_factor(sq) · (1 − crowd_risk(sq)) · (1 − 0.6 · nearest_opponent_can_recover(sq))
deep_factor(sq) = endzone_distance_for_OPPONENT(sq) / 25   # punt it as far from us as possible
```

Currently the parity harness always aborts a punt with `EndTurn`; that abort stays as an option with
`w = 0.15` so the parity path is still sampleable.

**T = 0.25.**

---

### 6.9 `SwoopTarget { player_id, squares }`

The step has **no decline path** — something must answer or the game stalls.

```
w(sq) = V(sq, player) · (1 − 0.5 · opponents_adjacent(sq) / 3)
```

Remember the coordinate transform: away-team answers are sent transformed
(`if !is_home { target.transform() }`), exactly as `UniformAgent` does.

**T = 0.25.**

---

### 6.10 `FollowUp { attacker_id, target_coord }`

**Options.** `{follow, stay}`.

```
w(follow) = 0.5
          + 0.30 · (attacker has Frenzy)                        # a second block needs adjacency
          + 0.25 · (target_coord is closer to the enemy carrier)
          + 0.20 · (attacker is escorting our carrier forward)
          − 0.35 · (target_coord has more opposing tackle zones than the current square)
          − 0.45 · (attacker carries the ball)                  # never walk the ball into the pack
          − 0.30 · (target_coord is next to the sideline && no SideStep)
w(stay)   = 1.0 − w(follow)
```

Clamp both into [0.02, 0.98]. **T = 0.30** — genuinely close to a coin flip in many positions, and
flat sampling here is cheap coverage.

---

### 6.11 `HitAndRun { player_id, squares }`

**Options.** Each square + decline.

```
w(sq)      = V(sq, p) · exposure(sq)      # getting away from the pack is the point
w(decline) = 0.25
```

**T = 0.25.**

---

### 6.12 `FuriousOutburstSquare`, `RaidingParty`, `TricksterMove`

All three are "pick a square, no real decline". Same shape:

```
w(sq) = V(sq, moved_player) · exposure(sq)
```

with a per-mechanic twist:

- **`FuriousOutburst`** — the first square wants adjacency to the stab target
  (`+0.4` if adjacent to the intended victim); the second wants to be *away* from opponents.
- **`RaidingParty`** — the moved team-mate should land where it adds an assist or a marker:
  `+0.35` if the square is adjacent to an opposing player we intend to block.
- **`TricksterMove`** — this is a *defensive* dodge out of a block; weight purely by
  `exposure(sq) · (1 − adjacent_opponents(sq)/3)`.

Empty list → the mechanic-specific abort the existing agents use (`EndPlayerAction` for Furious
Outburst — its Java step has no `CLIENT_END_TURN` handler — `EndTurn` for Raiding Party,
`Acknowledge` for Trickster). **Do not "unify" these; they are three different Java contracts.**

**T = 0.25.**

---

### 6.13 `Pushback { attacker_id, defender_id, squares }`

Today: deterministic min-`(x, y)`, zero RNG (AGENT_CONTRACT §7). The heuristic version:

```
w(sq) = 0.95   if sq is off-pitch (crowd surf) — 1.0 if the defender carries the ball
      | 0.75   if sq moves the defender away from our carrier / off a cage corner
      | 0.70   if sq sets up a chain-push into the crowd
      | 0.55   if sq is toward the sideline (sets up next turn's surf)
      | 0.40   if sq puts the defender in one of OUR tackle zones
      | 0.20   otherwise
    · (1.3 if the defender is the enemy carrier and sq is away from their endzone)
```

**T = 0.15.** Watch out: this prompt has a hard parity constraint — see §11.

---

### 6.14 `ReRollOffer { source, action, team_id }` — *the brief's worked example*

**Options.** `{use, decline}`.

The general form, keyed on the `action` string (`GFI`, `DODGE`, `PICKUP`, `CATCH`, `STAND_UP`,
`RIGHT_STUFF`, `JUMP`, `ESCAPE`, `TENTACLES`, `FOUL_APPEARANCE`, `HYPNOTIC_GAZE`, `CHAINSAW`,
`PROJECTILE_VOMIT`, `ALWAYS_HUNGRY`, `BLOCK`, `CATCH`, `Interception`, `Bloodlust`, `Animosity`,
`Regeneration`, `Winnings`…):

```
w(use)     = consequence(action) · p_success_on_reroll · rr_scarcity(source)
w(decline) = 1.0 − w(use)
```

`consequence(action)` — how bad is failing?

| Situation | consequence |
|---|---|
| Carrier, and success scores a touchdown | **0.95** |
| Carrier, failure is a turnover with the ball loose in traffic | **0.85** |
| Carrier, failure is a turnover | **0.80** |
| Failure is a turnover but the ball is safe elsewhere | 0.55 |
| Failure ends only this activation (Bone Head, Take Root, Tentacles) | 0.35 |
| Failure costs nothing beyond the action (Foul Appearance, Interception, Argue) | 0.20 |
| Regeneration / Winnings / post-game rolls | 0.25 |

`p_success_on_reroll` is the *same* target number rolled again — the brief's
`0.5 × (probability of succeeding re-roll)` generalised. So the brief's three GFI cases come out of
the general formula directly:

| Case | consequence | p | rr_scarcity | w(use) |
|---|---|---|---|---|
| Carrier will score | 0.95 | 0.833 | 1.0 | **0.79** → with T = 0.15, ≈ 0.98 use |
| Carrier | 0.85 | 0.833 | 1.0 | **0.71** |
| Otherwise (`0.5 · p`) | 0.55 | 0.833 | 1.0 | **0.46** |

`rr_scarcity(source)`: a free skill source (`Dodge`, `SureFeet`, `SureHands`, `Catch`, `Pass`,
`Loner`-free star re-rolls) = `1.0`. The team re-roll:

```
rr_scarcity(TEAM) = (0.45 + 0.55 · rerolls_left / rerolls_at_half_start)
                  · (1.35 if turn ∈ {7, 8, 15, 16})     # use it or lose it
                  · (0.75 if turn ≤ 2 && half == 1)     # early hoarding is correct
                  · p_roll(loner_value)                 # Loner: the RR itself can fail
```

Two hard overrides:

- **Never** re-roll a roll that is already impossible to fail or already succeeded (defensive: the
  engine should not offer it, but do not trust it).
- If `p_success_on_reroll < 1/6` (a 6+ that will stay a 6+), cap `w(use)` at `0.25` — burning the
  team re-roll on a 6+ is a classic beginner loss.

**T = 0.20.**

---

### 6.15 `ReRollForTargets { … }` — multi-block re-roll target picking

Currently unimplemented in both agents (`UniformAgent` flags it `last_unhandled_prompt`).

**Options.** `(target, source)` for each still-failing target × each available source, plus decline.

```
w(target, src) = (1 − p_roll(minimum_rolls[target]))     # how much is currently failing
               · target_value(target)                    # §5.5 mark_value
               · rr_scarcity(src)
w(decline)     = 1.0 − max w(target, src)
```

Source preference order falls out of `rr_scarcity`: `re_roll_skill` (free) > `pro_re_roll` >
`consummate` > `single_use_re_roll_source` > `team_re_roll`.

**T = 0.25.**

---

### 6.16 `SkillUse { player_id, skill_id, skill_name }`

A per-skill table, keyed on `skill_name` (which is the `Debug`/`class_name` string and resolves via
`SkillId::from_class_name`). **The answer must echo the offered skill's id** — the `UniformAgent`
bug where a hardcoded `SkillId::Block` left Hit-and-Run unanswered and the step re-prompted forever
is a permanent lesson here.

| Skill offered | `w(use)` | Rationale |
|---|---|---|
| **Wrestle** (as attacker) | `0.90` if the defender has the ball; `0.10` if the attacker has the ball; else `0.50` | *the brief's example* — wrestle drops both, which is good when it separates the carrier from the ball and terrible when it drops our own |
| **Wrestle** (as defender) | `0.85` if it prevents a Pow on our carrier; else `0.55` | |
| **Dodge** (block-dodge, avoid a knockdown) | `0.95` | almost free; decline only to save the once-per-turn use when the knockdown is harmless (`0.60` if the player is on the sideline with nothing at stake) |
| **Juggernaut** | `0.80`, `0.95` if it cancels the defender's Wrestle/Fend and we carry no ball | |
| **DumpOff** (as defender) | `p_pass · p_catch · V(receiver)`, `× 1.5` if the blocked player carries the ball | dumping the ball out of a doomed carrier is exactly the point |
| **HitAndRun** | `0.70` — usually correct to disengage; `0.30` if we are the escort for our own carrier | |
| **PilingOn** (see also §6.17) | `0.75` if the target is the carrier or a key positional; `0.35` otherwise; `0.05` if we are the last standing player near our own carrier | going prone has a real cost |
| **Frenzy** second block | `0.85` if the follow-up push surfs the defender; `0.55` normally; `0.20` if the second block hands the opponent a −2D swing (Frenzy into a Guard cluster) | |
| **Fend** (as defender) | `0.85` | nearly free |
| **Grab** / **SideStep** push redirection | route to the `Pushback` scorer (§6.13) and use `max(w) − w(default_square)` | |
| **Tackle / Prehensile Tail / Diving Tackle** (as defender) | `0.80`, `0.95` against the carrier | |
| **Shadowing** | `0.70` against the carrier, `0.30` otherwise | |
| **Taunt** / follow-up forcing skills | `w(FollowUp)` from §6.10 | it *is* a follow-up decision |
| **Trickster** | `0.60` | |
| **Swoop** (BB2025, optional) | `0.55` | |
| **Pass** (skill re-roll offer) | route to `ReRollOffer` (§6.14) | |
| **ProjectileVomit / BreatheFire / Chainsaw** | `0.65 · target_value` | |
| **QuickBite / AnimalSavagery** (vampire) | `0.85` — declining generally means losing the activation | |
| **PutridRegurgitation** | `0.60` | |
| **anything unrecognised** | `0.50` | never a hardcoded decline — an unknown skill is a coverage opportunity |

`w(decline) = 1.0 − w(use)`. **T = 0.20.**

---

### 6.17 `PilingOn { player_id, target_id }`

Distinct prompt from the generic `SkillUse` route. Same weights as the Piling On row above; the extra
context available here is the target id, so `target_value(target_id)` is exact.

```
w(use) = 0.35 + 0.55 · is_carrier(target) + 0.20 · target_is_key_positional
       − 0.30 · (we are the only standing player adjacent to our own carrier)
       − 0.25 · (bb2016: Piling On costs the team re-roll and none remain)
```

**T = 0.25.**

---

### 6.18 `DefenderAction { player_id, actions }`

**Options.** The supplied action strings (Dodge / Block / Stab / … depending on edition).

```
w(a) = match a { "Block" => dice_weight(n), "Dodge"|"Wrestle" => 0.6, "Stab" => 0.5, _ => 0.4 }
```

Both existing agents `Acknowledge` this. Enumerate for real — this prompt is a live decision.

**T = 0.25.**

---

### 6.19 `Interception { player_id, target_number, candidates }`

**Options.** `attempt(candidate)` for each candidate, plus decline.

```
w(attempt, c) = p_roll(target_number)
              · (0.8 + 0.7 · possession_value)     # a caught interception hands us the ball
              · (1.0 + 0.6 · pass_would_lead_to_a_touchdown)
w(decline)    = 0.20
```

**A successful interception is usually the largest single swing available** — it does not merely deny
the pass, it gives us possession, in their half, with our turn still to come. An earlier draft scaled
only by "the pass would lead to a touchdown" and left the possession gain out entirely.
`possession_value` is the §5.1 `base_intent` the interceptor's square would score as a carrier, so an
interception deep in the opponent's half is worth far more than one on our own goal line.

A failed attempt costs **nothing** in any edition — no turnover, no fall — so the only reason to
decline is that the attempt itself is unlikely. Java's `RandomStrategy` always declines; that is a
policy choice, not a rule, and it is what keeps the whole interception family under-exercised.
`w(decline) = 0.20` keeps the declining path sampled without making it the mode.

**T = 0.20.**

---

### 6.20 `ApothecaryChoice { player_id, can_heal }` and `UseApothecary`

```
w(use) = 0.30
       + 0.45 · (the player is the ball carrier or a star/key positional)
       + 0.25 · (the injury is a Dead or a permanent stat loss)
       + 0.15 · (it is the first half — the apo is worth more early)
       − 0.30 · (the injury is a mere KO and the half is nearly over)
w(decline) = 1.0 − w(use)
```

`UseApothecary` (the follow-up "which result do you keep") picks the better of the two results:
`w(keep_new) = severity(old) − severity(new)` normalised into [0, 1], with severity
`Badly Hurt 0.2 < Miss Next Game 0.5 < Niggling 0.65 < Stat loss 0.8 < Dead 1.0`.

**T = 0.25.**

---

### 6.21 `TeamSetup { team_id, players }`

**Options.** Formations, not individual placements. Enumerating placements is combinatorial and
useless; enumerating *formations* is exactly right and is what a human coach does.

Ship a small formation book (each a fixed list of (slot, coordinate) for offence and defence,
validated against the 3-on-LOS / max-2-per-wide-zone rules):

| Formation | Weight when receiving | Weight when kicking |
|---|---|---|
| `canonical` (the existing `canonical_setup_action` — keep it, it is the parity answer) | 0.45 | 0.45 |
| `deep_receiver` (one fast player deep) | 0.85 | 0.10 |
| `spread_defence` | 0.10 | 0.75 |
| `cage_ready` (tight, wide-zone light) | 0.70 | 0.20 |
| `zone_defence` (wide-zone heavy) | 0.10 | 0.65 |

Assign players to slots by role: highest MA + Catch to the deep/receiver slots, highest ST/AV to the
LOS slots, Guard players to the interior. Sort by `(role_score, player_id)` so it stays deterministic.

**The legality rule, stated rather than implied.** A formation is dropped from the option set unless
it satisfies all of:

- **exactly three players on the line of scrimmage** (`FieldCoordinateBounds::LOS_HOME` / `LOS_AWAY`,
  x = 12 / 13, y ∈ 4..=10) — three is the minimum and the engine rejects fewer;
- **at most two players in each wide zone** (`UPPER_WIDE_ZONE_*` y ∈ 0..=3, `LOWER_WIDE_ZONE_*`
  y ∈ 11..=14);
- **at most eleven players placed**, and never more than the roster can field;
- every placement inside the team's own half.

An earlier draft said only that an illegal formation "is dropped", which on a short roster (a
Journeyman-thin team, or after casualties) silently collapses the whole book to
`canonical_setup_action` with nothing in the trace to say why. Log the drop.
`canonical_setup_action` remains the guaranteed-legal fallback that must always be present.

**T = 0.30.**

---

### 6.22 `SetupError { team_id, error }`

Single legal response: `Acknowledge`, then re-run setup with the canonical formation (never retry the
formation that just failed). Not a real decision — no scoring.

---

### 6.23 `CoinChoice { is_home }`

Truly 50/50. `w(heads) = w(tails) = 0.5`, **T = 1.0**. Documenting it explicitly so nobody "improves"
it later.

---

### 6.24 `ReceiveChoice { team_id }`

```
w(receive) = 0.65   in the first half   # score, then get the ball back after halftime
w(receive) = 0.85   if the score is level or we are behind in the second half
w(receive) = 0.25   if we lead in the second half and can run the clock
w(kick)    = 1.0 − w(receive)
```

**T = 0.30.**

---

### 6.25 `Touchback { eligible_players }`

```
w(p) = 0.3 + 0.4 · normalized(MA(p)) + 0.3 · has(p, SureHands | Catch | Dodge)
     − 0.5 · (p is on the line of scrimmage)
```

Give the ball to a fast, safe, deep player. **T = 0.20.**

---

### 6.26 `KickoffReturn { eligible_players }`

Currently `Acknowledge` in both agents. When it becomes a real pick, weight identically to
`Touchback`. **T = 0.20.**

---

### 6.27 `KickBall`

**Options.** Every legal target square in the opponent's half
(`legal_kickoff_targets`, and note the `x` offset the existing agents apply for home/away).

```
w(sq) = 0.5
      + 0.30 · deep(sq)                       # (distance from the LOS) / 12 — deep kicks buy time
      − 0.55 · touchback_risk(sq)             # within 1 of a sideline or an endzone: scatter risk
      + 0.20 · corner_bias(sq)                # away from the receiving team's cluster
```

Kicking deep and to a corner is standard, and it also produces more *interesting* drives, which is
what coverage wants.

**T = 0.30.**

---

### 6.28 `KickoffEventPlacement { team_id, mode }`

Quick Snap / Solid Defence / High Kick. Both agents decline with `EndTurn` — which is *exactly* how
these mechanics stay under-tested.

**Options.** For each eligible player, each legal placement square, plus decline.

| mode | weight |
|---|---|
| `QuickSnap` | `w(move p → sq) = V(sq, p)` (free move, no rolls) — decline `0.10` |
| `SolidDefence` | reposition toward the ball's likely landing square — decline `0.15` |
| `HighKick` | `w(p → landing_sq) = 0.9 · p_catch(p)` for the best catcher — decline `0.05` |

Cap at the top 5 placements + decline. **T = 0.25.**

---

### 6.29 `BombRethrow { player_id }`

The step *parks* until the bomb is thrown; a decline cannot advance it. So: always throw, and pick
the target by the `ThrowBomb` rule (§6.5.2):

```
w(sq) = 0.2 + 0.8 · (opponents within 1 of sq) / 3   over every legal pass target
```

Include the "throw it back at the thrower's own cluster" squares with low weight rather than
excluding them — the bomb chain is a coverage-rich mechanic.

**T = 0.25.**

---

### 6.30 `BuyInducements` / `BuyPrayersAndInducements` / `PettyCash`

`legal_inducement_purchases` already enumerates every affordable subset.

```
w(subset) = Σ_item value(item) / budget_used, normalised into [0, 1]
value(item) = 0.9  Bloodweiser Keg / Wizard / Chef / Star Player
            | 0.7  extra team re-roll, Babes
            | 0.5  Bribe, Igor, apothecary
            | 0.3  anything unrecognised
w(buy nothing) = 0.15
```

**Cap the enumeration.** `legal_inducement_purchases` returns *every affordable subset*, which is
combinatorial — a dozen affordable items is thousands of options. That is both a performance hazard
and a distribution so flat that softmax cannot express a preference at any temperature. Take the
**top 24 subsets by value density** (`Σ value / cost`) plus **8 uniformly sampled** from the
remainder, set `Scored::truncated`, and `log()` how many were dropped. §6.5.2's no-silent-caps rule
applies here too: a capped set that reports itself as complete is how "we considered everything"
becomes false without anyone noticing.

`PettyCash` — accept whatever is offered (`Acknowledge` today); when a real amount choice exists,
weight proportionally to `amount / max_amount`.

**T = 0.35** — inducements are pre-game and their weights are the least reliable in this document, so
sample them broadly. That also maximises the roster/inducement coverage surface.

---

### 6.31 `UseInducement { team_id, inducement_id }`

```
w(use) = 0.6, and for a Bribe specifically → route to §6.33 (`BriberyAndCorruption`)
w(decline) = 0.4
```

**T = 0.30.**

---

### 6.32 `WizardSpell { team_id, target_coord }`

The prompt carries no spell list yet. When it does:

```
w(Zap, target)       = 0.7 · target_value(target)
w(Fireball, coord)   = 0.4 + 0.6 · (opponents in the 3×3 at coord − own players in it) / 4
w(decline)           = 0.25   — but 0.05 on turn 8 / 16 (use it or lose it)
```

Until the prompt is extended, `Acknowledge` with a `last_unhandled_prompt` flag, as `UniformAgent`
does. **T = 0.30.**

---

### 6.33 `BriberyAndCorruption` / `ArgueTheCall` — the referee family

```
ArgueTheCall:  w(argue) = 0.75 · player_value(p)      # a 1 ejects the coach; still usually right
               w(decline) = 1.0 − w(argue)
Bribe:         w(use) = 0.55 + 0.35 · is_key_player   # the bribe itself is a 2+ (or 4+ Chainsaw)
               w(decline) = 1.0 − w(use)
```

**T = 0.25.**

---

### 6.34 `ConcedeGame { team_id }`

```
w(concede) = 0.02   — flat, essentially never, but never structurally impossible
w(play on) = 0.98
```

**T = 0.30.** Guard rail: never concede in the first half. This is one of the only places a hard
`0.0` is acceptable, because conceding truncates the game and destroys coverage.

---

### 6.35 `ConfirmEndAction { team_id }`

`w(confirm) = 0.9`, `w(cancel) = 0.1` — the agent already decided; cancelling re-opens the same
decision and risks a loop. Cap consecutive cancels at 1.

**T = 0.20.**

---

### 6.36 `InformationOkay`, `StartGame`, `GameStatistics`

Single legal response. `Acknowledge`. Not decisions.

---

### 6.37 `BloodlustAction { player_id }`

Vampire failed Blood Lust: change the declared action, or don't.

```
w(change) = 0.75     # keeping a Move/Block after a failed Blood Lust usually feeds a thrall
w(keep)   = 0.25, but 0.70 if the vampire carries the ball and can still score
```

**T = 0.25.**

---

### 6.38 `SwarmingPlayers { team_id, eligible_players }`

When it becomes a real pick (currently `Acknowledge`): choose the `n` players to bring on.

```
w(p) = 0.4 + 0.4 · normalized(MA(p)) + 0.2 · normalized(ST(p))
```

**T = 0.30.**

---

### 6.39 `MultiBlockTargets { player_id, eligible_players }`

**Options.** All unordered pairs (cap at the top 8 pairs by weight, plus one random pair).

```
w(d1, d2) = dice_weight(n₁) · dice_weight(n₂)
          · (1 + 0.4 · (d1 or d2 is the carrier))
          · (1 + 0.3 · (either can be surfed))
```

`n₁`/`n₂` from `find_nr_of_block_dice(…, using_multi_block = true, …)` so the edition modifiers are
right. **T = 0.20.**

---

### 6.40 `PlayerChoice { eligible_players, reason, descriptions }`

A catch-all used by many mechanics. Key the weight on `reason`:

| `reason` contains | weight |
|---|---|
| a "who gets hurt / removed" choice | `1.0 − target_value(p)` — pick the least valuable |
| a "who benefits / gets the skill / gets healed" choice | `target_value(p)` |
| a "who moves / who acts" choice | `best_move_weight(p)` |
| anything unrecognised | `0.5` (uniform), and flag `last_unhandled_reason` for the trace |

The unrecognised-reason flag is important — it is how new `PlayerChoice` call sites get discovered
and tuned rather than silently falling into uniform play.

**T = 0.25.**

---

### 6.41 `SelectPosition { available_positions }`

No `Action` variant carries a position choice yet (both agents flag it unhandled). When one exists:

```
w(pos) = 0.4 + 0.6 · positional_scarcity(pos)   # a roster's rare positionals are worth more
```

**T = 0.30.**

---

### 6.42 `SelectSkill { player_id, skill_ids, reason }`

Intensive Training / Wisdom of the White Dwarf. Resolve ids through `SkillFactory` exactly as
`UniformAgent` does, then:

```
w(skill) = base_skill_value(skill) · role_fit(skill, player)
```

`base_skill_value`: Block 1.0, Dodge 0.85, Sure Hands 0.8, Guard 0.8, Mighty Blow 0.75, Tackle 0.7,
Side Step 0.65, Catch 0.6, everything else 0.45.
`role_fit`: `1.2` if the player is a likely ball carrier and the skill is ball-related; `1.2` if the
player is a blocker and the skill is a block skill; `0.8` on a mismatch.

**T = 0.30.**

---

### 6.43 `SelectWeather { options }`

```
w(w) = 0.7  Nice
     | 0.5  Very Sunny / Pouring Rain
     | 0.4  Sweltering Heat / Blizzard
```

...but bias by *our* roster: a passing team hates Pouring Rain and Blizzard; a bash team is happy in
either. `w ·= roster_weather_fit(w, my_team)`.

**T = 0.35** — weather choice is rare and its coverage value (different weather = different code
paths) argues for flat sampling.

---

## 7. Worked examples on real game states

Four decision points taken from **actual parity runs**, with the §6 weights and the resulting
probability distribution computed on each. The rendered version of this section — pitch heatmaps and
distribution bars — is `docs/heuristic_agent.html`.

### 7.1 Provenance

```bash
FFB_TRACE=1 ./target/release/ffb-parity.exe \
    --home lineman --away lineman --edition bb2025 --tier 3 --seeds 15-20
```

`FFB_TRACE` makes the runner emit one `RUST_STEP` line per decision carrying
`state_string(&game)` — which, per `ffb-model/src/util/state_hash.rs`, contains **every player's
coordinate, state and effective stats, the ball, the weather, the re-rolls and the per-turn
flags**. That is a complete board, so the examples below are reconstructed positions from a real
game, not invented ones. Both engines agreed on every state used here.

**What is computed exactly**, straight from the engine's own rules:

| Quantity | Source |
|---|---|
| Dodge target | `max(2, AG + tackle_zones_on_destination)` — BB2025 `DodgeModifierFactory::minimum_roll_edition` with the `TACKLEZONE` modifier whose value equals the count |
| GFI target | 2+ |
| Reachability | Dijkstra over `−log p_step`, capped at MA + 2, tie-broken on square count (§4.2) |
| Block dice | assist-resolved strengths through the `find_nr_of_block_dice` comparison ladder |
| Push squares / surf | the real three-square push fan for the attacker→defender direction |

**Weather is live in these examples.** Seed 16 is played in a **Blizzard**, so its rushes are at
**3+**, not 2+ — 79 of that board's 200 options use a GFI square and they carry 29% of the
probability mass. The first version of this section computed them at 2+ and was wrong by exactly the
margin D1 describes; the numbers below are the corrected ones. Seed 18 is Nice Weather and seed 11 is
Very Sunny, so neither is affected.

**What is approximated:** the threat map uses the **cheap variant §5.2 already documents**
(`0.6 · tz_opposing(sq) + 0.4 · adjacent_standing_opponents(sq)`) rather than the per-opponent BFS.
The lineman tier is skill-less, so no skill multiplier is being silently skipped — there are no
Block, Dodge, Guard or Side Step players on the pitch.

### 7.2 A marked ball carrier — seed 16, step i=1

Away `a02` carries the ball at (13, 8), **two opposing tackle zones on him**, 13 squares from his
endzone, MA 6 / ST 3 / AG 3 / AV 8. Start of the team turn, so all eleven players are live.

| | Current agent | §6.6 heuristic |
|---|---|---|
| Options in the support | **1** | **201** |
| Expected squares advanced | **1.00** | **2.48** |
| Probability mass on advancing squares | 100% | 84.2% |
| Expected arrival probability | 0.750 | 0.834 |

Read the last two rows together: the heuristic gets **2.5× the ground** *and* arrives more often
(0.834 vs 0.750). It is not trading safety for distance — `a02` is in two tackle zones, so the one
square the current agent can take is itself a 4+ dodge, while the heuristic has 200 destinations to
choose among and can find routes that leave the zones cheaply.

The current agent has exactly **one** legal option here: it may move a single square, and the
carrier-advance filter leaves one empty advancing neighbour. There is no decision to make. The
heuristic considers 201 destinations, and **five squares tie** at the best weight
`w = 0.366` (`p_arrive = 0.889 × V = 0.411`) — the whole x=9 column, see §7.7(b) — taking **41.7%**
of the mass between them. Ending the activation is weighted `0.0` (§6.6) and takes 0.03%.

Note what the 77.6% means: a fifth of the mass is deliberately *not* advancing. Those are the
squares that trade ground for getting out of the two tackle zones, and they stay in the
distribution because sometimes escaping is worth more than three yards.

### 7.3 The same job with nobody on him — seed 18, step i=143

Home `h10` carries the ball at (1, 7), unmarked, **24 squares** from the endzone.

| | Current agent | §6.6 heuristic |
|---|---|---|
| Options in the support | **3** | **142** |
| Expected squares advanced | **1.00** | **5.48** |
| Probability mass on advancing squares | 100% | 97.7% |
| Expected arrival probability | 1.000 | 0.982 |

**5.5× the ground per activation** for a 0.018 drop in arrival probability. An unmarked run needs no
dodges, so the only risk on the board is the rush squares at the very end — and with `c_turnover`
now pricing a failed rush properly (§5.3), the distribution mostly declines to take them. This single state is the §0.5.1 finding in miniature: the
engine would have walked all six squares if anyone had asked it to.

### 7.4 Which opponent to block — seed 11, step i=160

Away `a00` at (13, 7) is adjacent to two standing opponents. Assists resolve to different dice:

| Target | Square | Strength | Dice | w | **p** |
|---|---|---|---|---|---|
| `h00` | (12, 7) | 4 v 3 | **+2D** | 0.600 | **90.3%** |
| `h01` | (12, 6) | 3 v 3 | +1D | 0.250 | 9.7% |

Both current agents pick uniformly here — 50/50 between a 2-dice block and a 1-dice block. The
heuristic puts 90% on the 2-dice block and keeps the 1-dice option at 9.7%, because occasionally
the 1D target is the one worth hitting and the tail has to stay reachable.

### 7.5 Which block die to keep

Three real dice shapes from the block log, scored by §6.3 at `T = 0.12`:

| Dice shown | Option | w | **p** | Current agent |
|---|---|---|---|---|
| `[6, 2]` +2D | **Pow!** | 0.90 | **98.3%** | 50% |
| | Both Down (skill-less: both fall) | 0.30 | 1.7% | 50% |
| `[1, 4]` +2D | **Push** | 0.40 | **94.0%** | 50% |
| | Skull — turnover | 0.05 | 6.0% | 50% |
| `[6, 3]` −2D, **defender** picks | **Push** (worst for the attacker) | 0.60 | **97.5%** | 50% |
| | Pow! | 0.10 | 2.5% | 50% |

The `[1, 4]` row is the shape behind §0.5.4's 23 self-inflicted turnovers: a Skull sitting next to a
Push, chosen by coin flip. And the third row is the defender-picks branch that §0.5.5 shows
quadruples in frequency on real rosters.

### 7.6 Who to activate first — seed 16, step i=1

The same board as §7.2, all eleven players live, at `T = 0.18`:

| Player | w | **p** | Why |
|---|---|---|---|
| `a02` | 0.90 | **53.6%** | carrier is marked — move before the blitz |
| `a00`…`a10` (ten others) | 0.45 | 4.5% each | free mover |
| `EndTurn` | 0.14 | 0.9% | nothing is finished yet |

The marked carrier takes over half the mass and every other player keeps a real 4.5% share. Both
current agents sample uniformly over the eligible list: **9.1% each**, with the carrier no more
likely to move than a lineman standing in his own half.

### 7.7 Two spec bugs this exercise exposed

Running the numbers on real boards found two problems that reading the spec did not.

**(a) The temperature table was calibrated for small option sets.** §8's temperatures were chosen
against a 2–5 option decision like block dice. A `Move` prompt has ~200 options, and softmax spreads
mass over all of them. Measured on these two boards, as **mass held by the twenty best
destinations**:

| T | `carrier_marked` (n=201) | `carrier_open` (n=142) |
|---|---|---|
| **0.20 (as specified)** | **22.0%** | **25.5%** |
| 0.12 | 34.4% | 33.9% |
| 0.08 | 53.0% | 43.8% |
| **0.06 (corrected)** | **70.3%** | **52.3%** |
| 0.04 | 90.2% | 65.2% |
| 0.02 | 98.0% | 83.7% |

At the specified `T = 0.20` the twenty best destinations out of ~200 hold barely a fifth of the
probability — the policy is a slightly-biased random walk, which is the exact failure it exists to
replace. **§8's `Move` temperature is corrected to 0.06**, and the table now carries the rule that
made the error possible: *a temperature is only meaningful against an option count*. Any prompt
whose option set is O(100) — `Move`, `KickBall`, `PuntTarget`, TTM targets — needs a temperature an
order of magnitude below a five-option decision.

**Judge by top-20, not top-5.** An earlier version of this table used the five best options, which
is unusable here: §7.3's board has **ten squares tied** at the best weight (§7.7b), so top-5 can
never exceed half the tied mass no matter how sharp the temperature, and the metric reports a
failure that is really a tie. Top-20 — or the mass on the whole tied-best tier, which the
calibration script also reports — is tie-robust.

**(b) The value function had no lateral preference.** In §7.2 the five best destinations are
(9,8), (9,9), (9,10), (9,11), (9,12) — an entire column, at identical weight; §7.3 ties ten. At the
time this was written, `base_intent` for a carrier was a pure function of `endzone_distance`
(`|x − ez|`), so every square in a column scored the same and only `sideline` and `exposure` broke
the tie.

**§5.1 now carries a `lane` term** for exactly this: opponents in a five-row corridor between the
square and the endzone, read in O(1) off §3's `row_prefix`. Running into an empty corridor is worth
more than running into a three-man screen, which no other term expressed. Ties will not vanish —
two genuinely equivalent squares should tie — but they stop being the *default* outcome of a
column.

---

## 8. Temperature table

One place, one table, tuned as a unit.

| Prompt class | Typical options | T | Rationale |
|---|---|---|---|
| `BlockChoice` | 2–3 | 0.12 | biggest swing per decision |
| `BlitzTarget`, `BlockTarget`, `Pushback`, `MultiBlockTargets` | 2–8 | 0.15 | mistakes are expensive |
| `ReRollOffer`, `SkillUse`, `Interception`, `BlockChoiceProperties` | 2–4 | 0.20 | consequential but recoverable |
| `FollowUp`, `HitAndRun`, `Trickster`/`RaidingParty`/`FuriousOutburst`, apothecary, foul | 2–10 | 0.25 | genuinely close calls |
| `ActivatePlayer` | ~12 | 0.18 | flattest of the small-set play decisions — max coverage value |
| `ReceiveChoice`, weather, `SelectSkill`, `TeamSetup` (formations) | 2–6 | 0.30–0.35 | low information, high coverage value |
| `CoinChoice` | 2 | 1.0 | genuinely uniform |
| **`Move`** (normal) | **~200** | **0.06** | §7.7(a): 0.20 left the top 20 squares only 22% of the mass |
| `Move` (touchdown available) | ~200 | 0.05 | near-greedy override |
| **`KickBall`** | **~195** (13 × 15) | **0.06** | same bug as `Move`; at 0.32 "kick deep to a corner" never happened |
| **`PuntTarget`** | every offered square | **0.08** | |
| **`ThrowTeamMateTarget`** | ~30–50 (range ruler) | **0.10** | landing preference was diluted to nothing |
| **`BuyInducements`** | 32 after the §6.30 cap | **0.12** | uncapped it was thousands; see D8 |
| `KickoffEventPlacement` | 6 after the top-5 cap | 0.25 | |

**A temperature is only meaningful against an option count** (§7.7a), which is why the table now
carries the count in its own column — the four bold rows are prompts that had inherited a small-set
temperature and were, in effect, sampling uniformly. Re-tune whenever an option set changes size, and
judge the result by the **top-20 probability mass** (or the mass on the tied-best tier), never by
the top-1 and never by the top-5: §7.3's board has ten squares tied at the best weight, so a top-5
metric reports a failure that is really a tie.

The lookup therefore takes both: `TempTable::for_prompt(prompt, n_options)`. Where a prompt's option
count varies by orders of magnitude between boards — `PuntTarget` and `ThrowTeamMateTarget` both
do — interpolate on `log n` between the small-set and large-set anchors rather than pinning one
constant.

**These will move once §5.3's turnover cost lands.** Weights become signed and their spread widens,
which makes any fixed temperature effectively sharper. §10.4's paired-seed loop is where they get
re-measured; do not treat this table as settled until it has.

Expose the whole table as a `TempTable` struct with `Default`, plus an env override
(`FFB_HEURISTIC_TEMP_SCALE`) that multiplies every entry — one knob to sweep the
exploration/strength trade-off in a coverage run without a rebuild.

---

## 9. Performance budget

Target: **< 100 ms of agent time per game**, so a 100-seed matrix run is not agent-bound.

An earlier draft set a single "< 50 µs per prompt" cap, and the design did not fit inside it: §6.5.1's
rules each wanted `best_move_weight`, which is one Dijkstra per eligible player — about **220 µs** for
a single `ActivatePlayer` prompt, seven times the line item budgeted for it. A flat per-prompt cap was
the wrong shape anyway, because the prompts are not equally frequent. The measured mix (§0.5) is
**277 activations and 263 moves per game**, so the budget is stated per class and rolled up:

| Component | Frequency | Budget | Per game |
|---|---|---|---|
| `positions_stamp` / `acting_stamp` check | per prompt | 1 µs | 1.5 ms |
| TZ + occupancy + row prefixes | per positions change | 4 µs | 1.5 ms |
| Threat map, bounded to the reach set | per **opponent** positions change, lazily | 12 µs | 0.4 ms |
| `reachable` Dijkstra | cached on `(opp_stamp, own_occ, player, rr)` | 20 µs | — |
| `find_applicable` | memoised per `(from_sq, to_sq)`; first edge free from `move_squares` | 0.4 µs/edge | — |
| `ActivatePlayer` tier 1 — proxy over ~11 players | per activation | 6 µs | 1.7 ms |
| `ActivatePlayer` tier 2 — up to 3 `reachable`, usually 1–2 cache misses | per activation | 45 µs | 12 ms |
| Scoring a `Move` prompt from the cached reach (~200 options × `V`) | per prompt | 8 µs | 2 ms |
| Every other prompt class | ~1000 per game | 3 µs | 3 ms |
| Softmax + sample | per prompt | 1 µs | 1.5 ms |
| | | **total** | **≈ 24 ms** |

The headroom is deliberate: `V` is the term most likely to grow (§5.1 gained `lane` and `urgency`
this pass), and a 3× overrun still lands inside the 100 ms target.

Rules that keep it there:

- **No `Game` clone, ever.** Every heuristic reads `&Game`.
- **No allocation in the hot path.** `Scored::options` is a `Vec` owned by the agent and cleared per
  prompt — *not* a `SmallVec<[Weighted; 32]>`, which a ~200-option `Move` prompt would spill to the
  heap on every single move, defeating the rule it was written to satisfy.
- **Two stamps, not one** (§3) — the acting player spending MA must not rebuild the maps.
- **Cache `reachable` on `(opp_positions_stamp, own_occupancy_hash, player, rr_state)`** — *not* on a
  whole-board stamp. A whole-board stamp changes whenever anyone moves, including the player who just
  moved, so across eleven activations the cache would be cold nearly every time and §9 would be
  budgeting a saving it never gets. Keying on what a player's reach actually depends on makes an
  unrelated move elsewhere on the pitch a cache hit. If the benchmark still shows it hot, repair the
  affected region incrementally rather than recomputing — a single square of movement changes
  reachability only locally.
- **Key the threat map on `opp_positions_stamp` alone.** The opponent does not move during our turn:
  their positions change only when we knock one down, push one, or remove one. An earlier draft keyed
  it on the full position stamp, so the most expensive feature in the design rebuilt on every one of
  our own moves to produce an almost identical answer. Our occupancy does block paths, so apply it as
  a cheap overlay at read time rather than folding it into the key.
- **Bound the threat map to the squares that will be asked about**, not the whole pitch.
- **No `HashMap`/`HashSet` iteration** in any scoring path (also a determinism requirement) — hence
  `BitSet22` for the activated set.
- **Integer milli-weights, from the start.** §11 requires `i32` milli-weights (`0.90 → 900`) for any
  parity port, because cross-language `f32` tie-breaking is not guaranteed. Writing the heuristics in
  `f32` now means rewriting every one of them later, and `f32` also makes the *Rust* agent's
  reduction order load-bearing for determinism. Do it once: score in `i32`, convert only inside
  `sample_softmax`.
- `why: Rule` + `why_value: f32`, never `String`.
- Feature-gate the trace: `#[cfg(feature = "agent-trace")]` around anything that formats.

---

## 10. Testing and calibration

### 10.1 Unit tests (per heuristic)

Every §6 subsection gets at least one test that pins the *ordering*, not the exact number:

```rust
#[test]
fn block_choice_prefers_pow_over_skull() { … }
#[test]
fn defender_stumbles_is_downgraded_when_defender_has_dodge_and_attacker_lacks_tackle() { … }
#[test]
fn wrestle_is_preferred_when_the_defender_has_the_ball_and_avoided_when_we_do() { … }
#[test]
fn gfi_reroll_weight_is_highest_when_the_carrier_would_score() { … }
#[test]
fn carrier_move_prefers_the_endzone_square_when_reachable() { … }
#[test]
fn sideline_squares_are_penalised_without_sidestep_but_not_with_it() { … }

// Added with the §14 backlog pass — each pins a rule that was wrong or missing.
#[test]
fn gfi_target_is_three_in_a_blizzard_in_every_edition() { … }              // D1
#[test]
fn two_squares_with_one_tacklezone_each_can_have_different_dodge_targets() { … } // D2
#[test]
fn a_risky_move_is_worth_less_early_in_the_turn_than_late() { … }          // A1
#[test]
fn six_plus_dodge_scores_below_zero_with_ten_players_unactivated() { … }   // A1
#[test]
fn urgency_rises_as_the_half_runs_out() { … }                             // A2
#[test]
fn equal_gain_scores_equally_from_deep_and_from_close() { … }             // A3
#[test]
fn lane_breaks_the_tie_between_two_squares_in_the_same_column() { … }     // A4
#[test]
fn block_equity_skill_less_row_matches_the_briefs_calibration() { … }     // A5
#[test]
fn fumbled_pass_is_priced_as_a_turnover_not_as_zero_gain() { … }          // A8
#[test]
fn coverage_floor_decays_once_a_step_has_dispatched() { … }               // C1
#[test]
fn formation_with_two_on_the_line_of_scrimmage_is_rejected() { … }        // C7

/// The step between `score` and `act` is otherwise untested: sampling N times from a fixed
/// option set must reproduce the softmax the scorer computed. A chi-square over ~1e5 draws.
/// This is the test that would have caught D10 — the decline option quietly holding the
/// highest weight on every board — the moment it was written.
#[test]
fn sampler_reproduces_the_scorers_distribution() { … }                    // P10
```

Ordering tests survive re-calibration; exact-value tests do not. Pin values only where the brief
gives one explicitly (the block-dice table), so a regression in the calibrated numbers is visible.

### 10.2 Invariant tests (whole agent)

```rust
#[test] fn every_legal_option_has_nonzero_probability() { … }   // the ε-floor
#[test] fn same_seed_same_state_same_action() { … }             // determinism
#[test] fn no_panic_on_any_prompt_variant() { … }               // exhaustive over AgentPrompt
#[test] fn answers_only_offered_squares_and_ids() { … }         // never invent an option
#[test] fn a_turn_always_terminates() { … }                     // EndTurn weight rises to 0.95
```

The last one is the most important structural guard: the `used_this_turn` soft factor (§6.5.1)
replaces a hard filter, so the turn-termination proof now rests on `w(EndTurn)` growing. Test it
with a board where every player is boxed in.

### 10.3 Behavioural benchmarks

Run 100 seeds of `heuristic vs heuristic` and `heuristic vs uniform` and track:

| Metric | Measured today (§0.5) | Target |
|---|---|---|
| Touchdowns per game | **0** (0 in 200 games) | > 1.5 |
| Mean squares per activation | **0.95** / **0.92**, max 1 | > 3 |
| Squares moved on a Blitz | **0.00** | > 1 |
| GFI attempts per game | **0** | > 5 |
| Failed dodges per game | **3.6** | < 2 |
| Optimal block-die pick (attacker) | **62.4%** / **62.6%** | > 90% |
| Skulls taken with a better die available | **23** / **31** per 100 games | < 5 |
| Distinct steps dispatched | 167/200 | > 190/200 |
| Completions / interceptions per game | ~0 | > 1 / > 0.1 |
| Agent time per game | — | < 100 ms |

Baselines are the `lineman_vs_lineman` / `human_vs_human` figures from §0.5, so the benchmark
script reports against numbers that already exist rather than against an estimate.

Wire this as `scripts/agent_bench.py`, reusing the existing coverage-sweep plumbing. The
touchdowns-per-game number is the headline: it is the single metric that tells you whether the whole
scoring/Punt/kickoff-return family is reachable.

### 10.4 Calibration loop

The weights in §6 are hand-set priors, not fitted values. To improve them without a full RL setup:

1. Run an N-seed round-robin at several `FFB_HEURISTIC_TEMP_SCALE` values.
2. For a single weight `w`, run `w ± 0.15` variants head-to-head (500 games each, paired seeds).
3. Keep the change only if the win rate moves by > 2σ **and** coverage does not regress.

Coverage-does-not-regress is a hard gate. A stronger agent that stops declaring Kick Team-Mate is a
net loss for this repo's actual purpose.

**Fit the temperatures directly, not through win rate.** Steps 1–3 measure play strength, which is a
very noisy and indirect way to discover that a temperature is leaving 78% of the probability mass
outside the top twenty options. `scripts/fit_temperatures.py` does it directly:

```bash
FFB_TRACE=1 ffb-parity --seeds 1-N ...          # RUST_STEP lines carry the full board (§7.1)
python scripts/fit_temperatures.py trace.txt    # replay §6's scorer over every recorded prompt,
                                                # solve for the T per class hitting a target top-20 mass
```

That is exactly the procedure §7.7(a) performed by hand on two boards, automated over thousands —
turning §8 from a hand-tuned table into a measured one, and giving every future weight change a cheap
way to check it did not quietly flatten a distribution. Run it whenever a weight moves; run the
paired-seed loop only for the questions it is actually good at.

**Run every step of this per edition.** Everything calibrated so far — §7's worked examples, §0.5's
evidence, every hand-set weight in §6 — comes from *one skill-less roster in one edition*,
`lineman_vs_lineman` on bb2025. The editions are not interchangeable:

| | bb2016 | bb2020 | bb2025 |
|---|---|---|---|
| Dodge minimum | `max(2, (7 − min(AG,6)) − 1 + mods)` | AG-target | AG-target |
| Casualty roll | d6 + d8 via the edition `RollMechanic` | — | — |
| Kick Team-Mate budget | spends the team **blitz** | spends `ktm_used` | spends `ktm_used` |
| Throw Team-Mate budget | spends the **pass** as well | spends the pass as well | `ttm_used` only |
| Interception minimum | `max(2, 7 − min(AG,6) + 2 + mods)` | AG-based | AG-based |

`§4.2` delegates the dodge scale correctly, and `find_nr_of_block_dice` already takes `rules`, so the
*mechanics* are edition-safe. The **weights** are not: a foul model or a re-roll scarcity curve tuned
on bb2025 has never been measured against a bb2016 casualty table. A weight that only helps bb2025 is
not a win — the gate is `bb2016 30/30, bb2020 30/30, bb2025 30/30`, and the benchmark table (§10.3)
should be reported three times.

**Mark every table edition-invariant or edition-gated.** §2.4's block-dice ladder genuinely is
invariant and now says so. The armour, casualty, TTM/KTM-budget and interception rules are not.
Anything unmarked is an untested assumption that bb2025 generalises.

---

## 11. Parity implications — read before touching anything shared

This is the constraint that most easily gets violated.

- **`RandomAgent` must not change.** Its decision/action RNG consumption order is byte-matched
  against `ParityRunner.java`. `HeuristicAgent` is a *new* type; it shares helpers only through
  pure, RNG-free functions.
- **The parity matrix keeps running on `RandomAgent`.** `HeuristicAgent` improving coverage does not
  make it a parity driver — a parity run compares Rust against Java, and Java's `ParityRunner` makes
  the *other* half of every decision.
- **To use `HeuristicAgent` for parity**, every heuristic in §6 must be ported to
  `ffb-ai/ParityRunner.java` and produce identical picks from identical RNG draws. That is a large
  but tractable project, and it is the only way the parity matrix ever exercises a touchdown. If it
  is attempted:
  - the scoring functions must be integer/fixed-point, not `f32` — cross-language float ordering is
    not guaranteed to be identical, and a tie broken differently is a desync;
  - use **i32 milli-weights** (`0.90 → 900`) throughout, with an integer softmax over a shared
    lookup table for `exp`;
  - the RNG draw count per prompt must be fixed and documented per prompt class, exactly as
    `AGENT_CONTRACT.md` does today (one draw per sample, always, even when `n == 1`).
- The recommended path is **staged**: land `HeuristicAgent` for coverage sweeps first (no parity
  risk at all), measure the coverage delta, and only then decide whether the Java port is worth
  it.

---

## 12. Implementation plan

Ten phases, each independently useful and independently gateable. No phase builds a search.
Phases 8 and 9 are **engine and harness work**, not agent work — they are the items the §14 review
found that cannot be fixed by writing a better heuristic.

| Phase | Contents | Done when |
|---|---|---|
| **1. Skeleton** | `agent/heuristic/mod.rs`, `Weighted` + `Rule`, `Scored`, `Policy`, `sample_softmax`, `TempTable::for_prompt(prompt, n)`, exhaustive prompt match that falls back to `UniformAgent`'s answer with a `last_unscored_prompt` flag | all `UniformAgent` tests pass with `HeuristicAgent` substituted; zero prompts flagged unscored in a 10-seed sweep |
| **2. Primitives** | §2 probability helpers, the computed `block_equity` table, `BoardFeatures` (§3) with TZ / occupancy / row prefixes / `activated`, **two-stamp** invalidation | unit tests on `p_roll` / `p_with_reroll`; `block_equity` skill-less row matches the brief; TZ map matches a naive per-square recount on 100 random boards; the acting player spending MA does not rebuild the maps |
| **3. Block family** | §6.2, §6.3, §6.4, §6.13, §6.39 — the whole block chain, all reading `block_equity` | the brief's worked example reproduces exactly; block-choice ordering tests green |
| **4. Move engine** | **Prerequisite: wire `is_valid_move` into all three `StepInitMoving`s (§4.1c).** Then §4 `reach.rs` (path + `p_step`, GFI via `GoForItModifierFactory`, `(from_sq, to_sq)` memo), §5 value model incl. `c_turnover` / `urgency` / `lane`, §6.6 whole-path answers | touchdowns-per-game > 0; squares-per-activation > 3; a deliberately stale path is rejected, not teleported; GFI is 3+ in a blizzard |
| **5. Re-rolls and skills** | §6.14, §6.15, §6.16, §6.17 | GFI-reroll worked example reproduces; every `SkillUse` skill_name hits a real table row or the documented default |
| **6. Activation** | §6.5 — the **two-tier** joint scorer with candidate capping and the live coverage floor | activation-order tests; the per-game budget in §9 holds; the floor decays once a step dispatches |
| **7. Long tail** | every remaining §6 subsection — kickoff events, inducements (capped), setup formations with the real legality validator | zero prompts scored by the "unrecognised → 0.5" fallback in a 100-seed sweep |
| **8. Prompt / Action extensions** *(engine work)* | the six prompts whose heuristics cannot run for data-model reasons: `ReRollForTargets` (needs a legal-set enumerator), `SelectPosition` (needs an `Action` variant), `WizardSpell` (prompt carries no spell list), `Journeymen`, `SwarmingPlayers`, `PettyCash`; plus a `legal_actions` helper for `KickoffEventPlacement`'s Quick Snap / Solid Defence / High Kick placements | all 43 §6 decisions are **live**, not merely specified; `last_unhandled_prompt` never fires in a 100-seed sweep |
| **9. Inducement campaign** *(engine + harness)* | teach BOTH harnesses to buy in lockstep, the same shape as the TTM / KTM / interception / bomb campaigns | the six category-D dead steps (`MasterChef`, `WeatherMage`, `Wizard`, `PlayCard`, `FanFactor`, `PrayerRoll`) dispatch, at 30/30/30 |
| **10. Calibration** | `scripts/agent_bench.py`, §10.3 metrics, the §10.4 loop — **run per edition** | the benchmark table hits its targets on bb2016, bb2020 and bb2025 |

**Do phase 4 before phase 6.** The movement engine is what unblocks scoring, scoring is what unblocks
the five category-C dead steps (`AssignTouchdowns`, `InitPunt`, `EndPunt`, `PuntDirection`,
`PuntDistance`) — those need **no** further work beyond phase 4 — and the activation scorer is much
easier to tune once `best_move_weight` is real.

**Category E is not a work item.** The six star specials the inventory lists as dead
(`Treacherous`, `LookIntoMyEyes`, `InitLookIntoMyEyes`, `Swoop`, `ThrowARock`, `QuickBite`) are a
**sampling artifact** — all were recorded live in the §§9–11 star campaigns. Verify each on its own
star's roster at 30+ seeds before treating it as anything else. §6.5.2's live coverage floor is the
only agent-side lever they need.

---

## 13. Open questions

1. ~~**Multi-square `Move` answers.**~~ **Resolved** — the engine already walks a whole path via
   `move_stack` and only re-prompts when it is empty (§4.1a). The open item that replaced it is
   §4.1c: `is_valid_move` is ported but never called, so a stale path teleports the player instead
   of being rejected. Prerequisite for phase 4, and a `docs/BACKLOG.md` entry in its own right.
2. **Team re-roll accounting across a path** (§4.2) is deliberately pessimistic. If it visibly
   under-values long dodge chains in practice, upgrade to tracking "one re-roll, best-placed" via a
   two-state Dijkstra (with/without the re-roll spent) — doubles the node count, still cheap.
3. **`PlayerChoice` reasons** — the `reason` strings need an inventory before §6.40's table can be
   anything but a default. Grep the call sites and enumerate them into a static table.
4. **Whether to port to `ParityRunner.java`** (§11) — a goal decision, not an implementation one.

---

## 14. Review — improvement backlog (applied)

A pass over §§1–13 looking for anything that would make the agent **better**, **faster**, or reach
**more mechanics**. Items marked *verified* were checked against the engine source; the rest were
design judgement.

**All 35 are resolved, and applying them turned up three more (D10–D12) which are resolved too.**
Thirty-four were spec defects or design gaps and are now folded into the sections above — this table is the ledger of where each one landed, so a reader who saw the earlier
draft can find what moved. The remaining four are engine and harness work, not heuristics, and are
scheduled as §12 phases 4, 8 and 9.

### 14.1 Defects

| # | Item | Status |
|---|---|---|
| **D1** | GFI hardcoded to 2+, ignoring Blizzard (+1) and Moles under the Pitch — *verified* against both `go_for_it_modifier_collection`s | ✅ §4.2 calls `GoForItModifierFactory::minimum_roll_going_for_it`; §5.4's row corrected |
| **D2** | The dodge-modifier memo key `(tz_from, tz_to, player)` is unsound — Prehensile Tail and Tackle are per-*opponent* | ✅ §4.2 keys on `(from_sq, to_sq)` for one `reachable` call; dies with the call so it cannot go stale |
| **D3** | `truncated` had no home after `PolicyOutput` was deleted | ✅ §1's `Scored { options, truncated }`; `score` writes into a caller-owned buffer |
| **D4** | `why: &'static str` cannot carry the numbers §6's rules produce | ✅ §1's `why: Rule` + `why_value: f32`, formatted only under `agent-trace` |
| **D5** | The `ActivatePlayer` budget was ~7× optimistic — 11 Dijkstras against a 30 µs line | ✅ §6.5.1 is two-tier; §9 re-budgeted per prompt *class* and rolled up per game |
| **D6** | The stamp over-invalidated: `current_move` rebuilt maps that do not depend on it | ✅ §3 splits `positions_stamp` / `acting_stamp` |
| **D7** | Only `Move` got the §7.7(a) temperature fix | ✅ §8 carries an option-count column; `KickBall` 0.32 → **0.06**, `PuntTarget` → 0.08, `ThrowTeamMateTarget` → 0.10, `BuyInducements` → 0.12 |
| **D8** | `legal_inducement_purchases` is combinatorial | ✅ §6.30 caps at top-24 by value density + 8 sampled, sets `truncated`, logs the drop |
| **D9** | Edition-invariance assumed but never stated | ✅ §2.4 marked invariant with its reason; §10.4 carries the per-edition difference table |
| **D10** | *Found while applying the rest:* the decline weight `0.05 + 0.55·(1 − best_dest)` assumed destination weights near 1.0. Real ones are ~0.37, so **"stand still" outscored every destination** on both §7 boards — 0.399 against a best of 0.366 | ✅ §6.6 sets it to **`0.0`**, which is what "no gain, no risk" means on §5.3's expectation scale. Decline now lands at p ≈ 0.0003 |
| **D11** | *Found while applying the rest:* §7's own worked example is played in a **Blizzard**, and was computed at GFI 2+ — the D1 bug, in the document's own evidence. 79 of that board's 200 options rush, carrying 29% of the mass | ✅ §7 recomputed with the weather-aware target; §7.1 says so explicitly |
| **D12** | *Found while applying the rest:* §7.7(a) judged temperature by **top-5 mass**, unusable when ten squares tie at the best weight | ✅ §7.7(a) and §8 judge by **top-20** (or the tied-best tier) |

### 14.2 Better — play strength

| # | Item | Status |
|---|---|---|
| **A1** | The cost of a turnover was not modeled — the largest single item | ✅ §5.3 is now an expectation: `p·V − (1−p)·c_turnover`, scaled by unactivated players, possession and rush |
| **A2** | No clock anywhere | ✅ §5.1's `urgency` from `turns_left − turns_to_score`; also scales `w(EndTurn)` |
| **A3** | `advance` normalised by remaining distance, not by what the activation could achieve | ✅ §5.1 divides by `min(d_now, MA + 2)` |
| **A4** | No lateral term — whole columns tied (5 squares in §7.2, 10 in §7.3) | ✅ §5.1's `lane`, a real corridor-occupancy term backed by §3's `row_prefix` |
| **A5** | Block-dice weights hand-set and skill-blind | ✅ §2.4 computes `block_equity(n, BlockSkills)` exactly at startup; the brief's row survives as the pinned sanity check |
| **A6** | `BlockChoiceProperties`'s `E[best]` ignored skills | ✅ §6.4 reads the same `block_equity` — one function, two callers |
| **A7** | Interception ignored the possession swing | ✅ §6.19 scales by `possession_value`; decline lowered 0.35 → 0.20 |
| **A8** | Passing had no fumble term and no throw-away option | ✅ §6.5.3 subtracts `c_turnover` on incompletion and enumerates empty target squares |
| **A9** | Cage geometry ignored where the threat is | ✅ §5.1 weights the four corners by `threat_share` |
| **A10** | Blitz destination scoring re-scanned assists per square | ✅ §6.5.2 recounts only the top 3 by `p_arrive`, incrementally along the frontier |
| **A11** | The activated set was used but never stored | ✅ §3's `activated: BitSet22` — not a `HashSet`, per §9's determinism rule |

### 14.3 Faster

| # | Item | Status |
|---|---|---|
| **B1** | Cache `reachable` per `(positions_stamp, player)` | ✅ §9's rules; §6.5.1 tier 2 reuses it for the `Move` prompt that follows |
| **B2** | Two-tier player scoring | ✅ §6.5.1 — proxy for all eligible, full search for the top 3 |
| **B3** | Split the stamp | ✅ §3 (same change as D6) |
| **B4** | Integer milli-weights from the start | ✅ §9 — score in `i32`, convert only inside `sample_softmax` |
| **B5** | Size the option buffer for the real case | ✅ §9 — a reusable `Vec`, not a `SmallVec<[_; 32]>` that a 200-option `Move` spills every time |
| **B6** | Bound the threat map to what the actor can reach | ✅ §3 and §5.2 — built over the reach set's dilation, not the pitch |
| **B7** | Take the engine's own move squares for step 1 | ✅ §4.2 — `field_model.move_squares` supplies the first edge free |

### 14.4 Mechanical coverage

| # | Item | Status |
|---|---|---|
| **C1** | The coverage floor was a static table | ✅ §6.5.2 counts dispatches **live** and decays the floor once a step has fired — a coverage search, not a fixed prior |
| **C6** | Star specials at a flat 0.40 | ✅ §6.5.2 — they ride the live floor; §12 records that category E is a sampling artifact, not a work item |
| **C7** | `TeamSetup` had no legality validator | ✅ §6.21 states the rule: three on the LOS, ≤ 2 per wide zone, ≤ 11 placed, own half — and logs the drop |
| **C8** | Everything calibrated on bb2025 lineman | ✅ §10.4 runs per edition, with the difference table and the 30/30/30 gate |
| **C3** | Scoring — 5 dead steps | 📅 **Phase 4.** No extra work: the move engine unlocks `AssignTouchdowns`, `InitPunt`, `EndPunt`, `PuntDirection`, `PuntDistance` on its own |
| **C4** | Six prompts unactionable for data-model reasons | 📅 **Phase 8** — engine work: a legal-set enumerator for `ReRollForTargets`, an `Action` variant for `SelectPosition`, a spell list on `WizardSpell`, plus `Journeymen`, `SwarmingPlayers`, `PettyCash` |
| **C5** | `KickoffEventPlacement` has no legal-placement enumerator | 📅 **Phase 8** — the `legal_actions` helper for Quick Snap / Solid Defence / High Kick |
| **C2** | Inducements — 6 dead steps | 📅 **Phase 9** — engine + harness: both sides taught to buy in lockstep, the shape of the TTM / KTM / interception / bomb campaigns |

### 14.5 What this pass changed about the shape of the design

Four of the fixes are not local corrections — they change how the agent thinks:

- **§5.3 is now an expectation rather than a discount.** Weights are signed, and a bad option scores
  below zero instead of merely small. Everything about risk sequencing within a turn follows from
  this one line, with no special cases.
- **§9's budget is per prompt class, rolled up per game.** A flat per-prompt cap could not express
  that activations are rarer than moves and deserve more time; the measured mix (277 activations and
  263 moves per game) is now what the budget is built on.
- **The coverage floor became a search.** Counting live rather than reading a stale table is what
  makes "do not regress coverage" self-enforcing as mechanics come alive.
- **Declining is worth zero, not a formula.** Once §5.3 subtracts a turnover cost, "stop here" has a
  principled weight — `0.0` — and every hand-tuned decline constant in the movement family collapses
  into it. D10 is what happens when that is guessed instead: the guess beat every real option.

**And three of them were only findable by computing on real boards** (D10, D11, D12). Reading the
spec produced none of the three; running §6 over a genuine pitch produced all three inside an hour.
That is the argument for keeping §7 in this document and for §10.3's benchmark existing at all.

**The temperatures in §8 are provisional until §5.3 lands.** Signed weights with a wider spread make
any fixed temperature effectively sharper, and §8's values were measured on the old multiplicative
form. Re-run §7.7(a)'s sweep and judge by top-20 mass; §10.4 is the mechanism.

### 14.6 Deliberately not on this list

- **A search of any kind.** Out of scope per §1.2's scope note.
- **Learned weights.** §10.4's paired-seed loop is the calibration mechanism; a gradient method needs
  an objective the parity harness does not expose.
- **Opponent modelling beyond the threat map.** §5.2's one-turn lookahead is the whole budget.

---

## 15. Proposed — round two

Ten further improvements, found by reviewing §§1–14 *after* the backlog pass. None of these is a
restatement of §14: three are modelling errors the first review did not look for, one is a mistake
introduced *by* the §14 pass, three are performance wins that only exist now that the caching design
is settled, and three are coverage, calibration and test machinery the document currently assumes
and does not have.

*verified* means checked against the engine source.

| # | Group | Item | Effort |
|---|---|---|---|
| **P1** | Better | The threat map lets every opponent blitz; a team gets **one** | S |
| **P2** | Better | `lane` is applied to every intent but only means anything for a carrier | S |
| **P3** | Better | The `Screen` intent pulls players *toward* the ball — the opposite of screening | S |
| **P4** | Better | Standing up costs 3 MA and negatraits can void an activation; neither is priced | M |
| **P5** | Faster | The threat map depends on the opponent, who does not move during our turn | S |
| **P6** | Faster | The team re-roll is never marked as spent, so every path over-estimates | S |
| **P7** | Faster | The reach cache almost never hits, because any move invalidates it | M |
| **P8** | Coverage | Nothing rewards a rare *situation*, only a rare *action* | M |
| **P9** | Coverage | §8's temperatures are provisional and there is no mechanism to re-fit them | M |
| **P10** | Coverage | No test checks that the sampler produces the distribution the scorer intended | S |

---

### 15.1 Better

**P1. The threat map lets all eleven opponents blitz. A team gets one.** *(verified)*

§5.2 scores a square by how many opponents could "end adjacent to `sq`", and multiplies each by
`strength_factor` — the block dice they would get. That treats every opponent as capable of hitting
the square. They are not: a team may declare **one Blitz per turn** (`turn_data.blitz_used`, the
flag both existing agents already filter on). Everyone else can only *move* adjacent and mark.

So the current `exposure` conflates two very different futures:

| What an opponent can actually do to `sq` | Should cost |
|---|---|
| Already adjacent, or can reach adjacency — **and the blitz is unspent** | the full block-dice term |
| Can reach adjacency, blitz already spent or better used elsewhere | a *marking* penalty only |
| Is already adjacent right now | a block **without** needing the blitz |

**Fix:** compute the block term for the single best blitz candidate only — `max` over opponents, not
`sum` — and give everyone else a smaller, flat marking contribution. Everything else in §5.2 stays.

This is not a small correction. With the top-3 sum as written, three opponents who can each reach a
square produce roughly three times the threat of one, so `exposure` collapses toward `1/(1+3·k)` and
squares near the pack are priced as though the whole pack could hit them. The likely effect is that
the agent is systematically too timid near opposing players, which is exactly the sort of bias the
§10.4 loop would spend a long time chasing without finding the cause.

**P2. `lane` is applied to every intent, and only means something for a carrier.**

§5.1 now reads `V = base_intent · sideline · exposure · lane`, with `lane` measuring opponents in a
corridor between the square and **the endzone**. For a carrier that is the right geometry. For a
player whose intent is `Mark`, `Cage` or `Screen` it is not the relevant question at all — a marker
*wants* to be near opponents, and `lane` penalises it for being there.

This one is mine, introduced in the §14 pass. **Fix:** scope `lane` to the intents where corridor
geometry is the point (`Score`, `Pickup`), and leave the rest on `base · sideline · exposure`. Score
the intents first, then apply intent-appropriate modifiers — which also makes the modifier set part
of the intent definition rather than a global multiply.

**P3. The `Screen` intent pulls players toward the ball, which is the opposite of a screen.**

`Screen` is `0.30 · (1 − normalized_distance_to_ball)`, maximised by standing **on** the ball. A
screen is a line of players placed *between* the ball and where the threat is coming from, spaced so
opponents cannot slip through the gaps.

**Fix:** score a square by how much it obstructs the shortest opponent path to our carrier —
reuse the §5.2 threat BFS, which already computes those paths, and reward squares that appear on
many of them. Two useful properties fall out: the screen naturally forms on the threatened side, and
it spaces itself, because once a square is taken the paths through it are no longer shortest.

**P4. Standing up costs 3 MA, and a negatrait can void the whole activation. Neither is priced.**
*(verified)*

`mechanics/movement.rs` has `STAND_UP_COST = 3`, and `must_roll_to_stand_up(ma) = ma <= 3` — a MA-3
player must roll to stand at all. Two consequences the value model does not express:

- **A prone player's reachable set is computed from the wrong budget.** `reachable` caps at `MA + 2`;
  for a prone player it must cap at `MA − 3 + 2`, and for MA ≤ 3 the whole activation is gated behind
  a roll. A prone MA-6 lineman can reach five squares, not eight, and §6.5.1's "prone and marked →
  0.70" is choosing between options that were never available.
- **A negatrait failure evaporates the activation.** §6.5.1 multiplies `w_player` by 0.55 for an
  unresolved Bone Head / Really Stupid / Take Root / Wild Animal / Blood Lust, which correctly says
  "activate these later". But it does not discount the *value* of what they might do: everything a
  Bone Head player is offered should be multiplied by `p_pass_negatrait`, because with probability
  `1 − p` none of it happens.

**Fix:** `reachable` takes the real starting budget; `w_action` multiplies by the negatrait success
probability. Both are one-line changes to inputs that are already computed elsewhere in the engine.

---

### 15.2 Faster

**P5. The threat map depends on the opponent, and the opponent does not move during our turn.**

§3 splits `positions_stamp` from `acting_stamp`, which was the §14 fix. But `positions_stamp` still
changes every time **one of our own players** moves a square — and during our turn that is the only
thing that changes. So the threat map, the most expensive feature at 12 µs and ~4 ms per game, is
rebuilt on every single one of our moves to produce an almost identical answer.

The opponent's positions change only when we knock one down, push one, or one is removed. **Fix:**
key the threat map on an `opp_positions_stamp` covering opposing coordinates and states only. Our
own players do block paths, so the map is not perfectly invariant — treat our occupancy as a cheap
overlay applied at read time, or simply accept a one-move-stale map, which is well inside the
accuracy this heuristic needs. Expected saving: most of that 4 ms, for about ten lines.

**P6. The team re-roll is never marked as spent.** *(verified against §4.2)*

§4.2 models the team re-roll as "worth its full value on the **first** roll of the path and zero
afterwards". Within one path that is right. Across a turn it is wrong: once the re-roll is actually
consumed — on a dodge two activations ago, or on a block — every subsequent path still prices its
first roll as if a re-roll were waiting. A 4+ dodge is scored at 0.75 when it is really 0.50.

That also silently corrupts the reach cache, because two identical boards with different re-roll
state produce different correct answers and share a cache key.

**Fix:** put `team_rr_available: bool` in `tempo` (the engine already tracks `rerolls` and the
per-turn consumption), read it in `rr_dodge`/`rr_gfi`, and include it in the `reachable` cache key.
Cheap, and it removes a whole class of "why did it take that dodge" confusion from the traces.

**P7. The reach cache almost never hits.**

§9 caches `reachable` per `(positions_stamp, player)` so the activation scorer and the `Move` prompt
share work. But `positions_stamp` changes whenever *anyone* moves — including the player who just
moved — so within a turn of eleven activations the cache is cold nearly every time. The design
claims a saving it does not get.

**Fix, in increasing order of effort:**

1. Key on `(opp_positions_stamp, own_occupancy_hash, player, rr_state)` so unrelated moves elsewhere
   on the pitch do not invalidate a player's reach.
2. **Incremental repair.** A single square of movement changes reachability only near the two
   squares involved. Recompute the affected region and splice, instead of a full Dijkstra.
3. Failing both, accept it and delete the claim from §9 rather than budgeting for a saving that does
   not happen.

Do (1) first and measure; (2) only if the benchmark says it is still the hot spot.

---

### 15.3 Coverage and calibration

**P8. Nothing rewards a rare *situation*, only a rare *action*.**

§6.5.2's live coverage floor boosts actions whose `StepId` has not dispatched. That is the right
mechanism for mechanics, and it does nothing for *states*. Plenty of engine code is reachable only
from a board configuration rather than from a declaration: a throw-in from a specific sideline, a
touchback, a full cage, a player with three markers, a ball bouncing into the crowd.

**Fix:** hash a coarse board descriptor — ball zone, carrier marked-ness bucketed, players-in-crowd,
turn bucket, weather — into a few thousand buckets, count them per run, and add a small bonus
(capped well below the action floor, so it never distorts real play) to options that would land in
an unvisited bucket. It is the same self-correcting shape as C1, applied to the other axis, and it
is the cheapest available lever on the 33-dead-step number that does not require new engine work.

Keep it small and keep it logged: a novelty bonus that grows large enough to change real decisions
stops being a coverage tool and starts being a bug.

**P9. §8's temperatures are provisional and there is no mechanism to re-fit them.**

§8 now says the temperatures will move once §5.3's turnover cost lands, and §10.4 nominates the
paired-seed loop as the mechanism. But that loop measures **win rate**, which is an extremely noisy
and indirect way to discover that a temperature is leaving 78% of the probability mass outside the
top twenty options. The `Move` temperature was found by hand, on two boards, in a spreadsheet.

**Fix:** `scripts/fit_temperatures.py`. Run `FFB_TRACE=1` over N seeds, parse the `RUST_STEP` state
strings (which carry the full board — §7.1), replay the §6 scorer over each recorded prompt, and
solve for the `T` per prompt class that hits a target **top-20 mass**. It is exactly the procedure
§7.7(a) performed by hand, automated, and it turns the table from hand-tuned into measured. It also
gives every future weight change a cheap way to check it did not quietly flatten a distribution.

**P10. No test checks that the sampler produces the distribution the scorer intended.**

§10.2's invariants cover determinism, non-zero support, no panics, only-offered-options, and turn
termination. None of them checks the step between `score` and `act`: that sampling N times from a
fixed option set reproduces the softmax the scorer computed.

**Fix:** a chi-square goodness-of-fit test over ~10⁵ samples from a pinned option set. It is a dozen
lines, and it is the test that would have caught **D10** — the decline option quietly holding the
highest weight on every board — the moment it was written, instead of two rounds later when the
numbers were computed by hand on a real pitch.

---

### 15.4 What is deliberately still not here

Unchanged from §14.6, and worth restating because each has been tempting at least once:

- **A search of any kind** — out of scope per §1.2.
- **Learned weights** — §10.4 is the calibration mechanism; a gradient method needs an objective the
  parity harness does not expose. P9 is the closest thing that stays inside the budget.
- **Opponent modelling beyond one ply** — P1 and P3 make the existing one-ply map *correct*, which is
  a different thing from making it deeper.

---

## 16. The experiment — heuristics vs random over an identical action space

**The agent is implemented** (`crates/ffb-engine/src/agent/heuristic_agent.rs`, driven by
`ffb-parity --heuristic <temp_scale>`). The control arm is the *same agent* with `temp_scale` set to
`1e6`, which flattens every softmax so it samples uniformly over exactly the options it enumerated.
One parameter, one code path.

### 16.1 Four bugs found by measuring, not by reading

The first published version of this section reported 0.40 touchdowns per game against 0.07. **That
result was an artifact**, and the reader who distrusted it was right. Four defects, in the order they
were found:

| # | Defect | How it was found |
|---|---|---|
| **E1** | `w(EndTurn) = 0.05 + 0.90·(1 − best)` assumed activation weights near 1.0. The real `w_player · w_action` product runs ~0.15, so `EndTurn` scored 0.83 and won every turn | **11 activations per game** instead of ~270 |
| **E2** | The touchdown override was a hard `return` that bypassed the softmax — so it fired **at any temperature** | the "uniform" arm scored touchdowns it had never sampled |
| **E3** | `score_action` returned only the **argmax** target, baking a greedy choice into the option before sampling | code audit prompted by E2 |
| **E4** | `reachable` ignored `acting_player.current_move`, granting a fresh MA+2 budget on every re-prompt | an MA-6 lineman moving **15 squares**; max is 8 |

E2 and E3 are the same mistake: **a heuristic that the temperature knob cannot switch off**. Every
decision must go through the buffer. With all four fixed, the touchdown advantage vanished — both
arms scored 0.01 per game — which was the correct measurement of a genuinely broken agent, and the
thing that exposed the fifth and largest bug.

### 16.2 The bug that mattered: the agent had never valued the ball

`Board::ball_live` was `ball_in_play && !ball_moving`, and the `Pickup` intent was gated on
`ball_live && carrier.is_none()`.

In this engine **`ball_moving == true` means the ball is LOOSE on the ground**, not in flight —
`legal_actions`' own tests say so ("loose ball 4 steps away" sets it true; "carried ball (not
moving)" sets it false). So `ball_live` meant *carried*, and the Pickup gate read **"the ball is
carried AND nobody is carrying it"** — a condition that can never hold.

The consequences, visible in a decision trace of 754 movement choices:

- `Pickup` never fired, **not once**;
- `Cage` and `Screen` were gated on a carrier that was `None` 82% of the time;
- so `Retreat` — the flat 0.10 floor — won **642 of 746** non-carrier decisions;
- the carrier moved **8 times in 754 decisions**.

The agent had no ball-oriented behaviour at all. Fixing the semantics — plus a real pickup
probability (`AG + tackle zones on the ball square`) and a 0.92 activation priority for any player
who can reach a loose ball — is what produced the result below.

### 16.3 Two more bugs, found by adding a third arm

Turning the temperature *down* to near-argmax exposed two defects that sampling had been hiding:

| # | Defect | Symptom |
|---|---|---|
| **E5** | `reachable` returned its results straight out of a `HashMap`, whose iteration order Rust randomises per process | **the same seed produced different scores on consecutive runs** — every argmax tie-break varied |
| **E6** | `used_this_turn` was only a ×0.30 *preference*. A near-argmax policy re-picks a used player forever if they still outscore every unused one | the greedy arm hung on some seeds |

E5 is §9's own rule — *no `HashMap` iteration in any scoring path, also a determinism requirement* —
violated in the one function that matters most. Sorting the reach set by coordinate fixed it, and
three consecutive runs of a seed now produce byte-identical games.

E6 is the general lesson restated: **a preference is not a guarantee.** With sampling, a 0.30 factor
plus an ε-floor eventually moves on; with argmax it never does. Used players are now skipped outright
while any unused player is eligible, and the soft factor only applies when the engine re-offers
someone with nobody else left.

Both fixes change all three arms, so everything below is a single re-run of all of them.

### 16.4 Four play fixes

Measurement said the agent was rushing and fouling far past anything a coach would do — 94.8 GFI
rolls and 6.35 fouls per game. Four changes:

| Fix | What was wrong |
|---|---|
| **Rush aversion** — an explicit penalty per GFI square, `0.40` for a non-carrier and `0.10` for the carrier | The plain expectation makes a 5-in-6 rush for a small gain marginally positive, so the agent rushed on ~34% of its movement decisions. But a turnover forfeits the **rest of the drive**, not one square, and that compounding is invisible to a single-step mean. A rush has to clear a bar, not break even. The carrier pays little — pushing for a touchdown is exactly when variance is worth it |
| **Foul pricing** — `p(armour breaks) × victim value × referee risk × timing` | A flat `0.30` with no ejection risk and no notion of whether the victim was worth it |
| **`Blitz` split from `Block`** | Both were scored by the *same match arm*, so they were numerically identical and a near-argmax policy took whichever the engine offered first. `Blitz` now scores what a blitz is *for*: `p_arrive × dice` over every **reachable** opponent, discounted 0.85 when the victim is already adjacent, since spending the once-per-turn blitz on someone a plain block reaches is a waste |
| **Coverage devices confined to the sampling arms** | §6.5.2's live floor pushes any action that has not dispatched yet, to stop a stronger policy silently dropping a mechanic. It costs play strength *by construction* — with it on, fouls sat at ~3 per game **no matter how the foul weight was priced**, because the floor overrode a correctly-computed `0.003` with `0.35`. It is now off below `temp_scale 0.1` |

The last one is the trade §6.5.2 describes, made explicit: **the sharp arms measure how well the
weights play; the sampling arms measure coverage.** Turning it off lifted touchdowns from 1.00 to
1.67 per game on its own.

### 16.5 Result — four arms, 100 games each, bb2025 lineman v lineman

**Read these numbers correctly.** Each arm is **self-play**: both sides are the same agent at the
same temperature (`run_heuristic_game` builds a home and an away `HeuristicAgent` with the same
scale). And every figure is the **whole game, both teams combined** — so "1.80 touchdowns/game" is
0.90 per team, "11.5 GFI/game" is ~5.7 per team, "22.4 blocks" is ~11 per team.

So this table compares four separate *populations*, not four competitors. No arm here has ever
played another; §16.9 does that.


| Per game | UNIFORM `1e6` | SAMPLED `1.0` | **GREEDY `0.01`** | ARGMAX `0` |
|---|---|---|---|---|
| **Touchdowns** | 0.00 | 0.19 | **1.76** | 1.54 |
| Games with ≥1 touchdown | 0 | 17 | **90** | 85 |
| Most in one game | 0 | 2 | 4 | 4 |
| **GFI rolls** | 114.9 | 19.5 | **11.2** | **10.4** |
| **Fouls** | 0.93 | 1.02 | **0.37** | **0.34** |
| Activations | 81.2 | 134.7 | **204.8** | 193.4 |
| Squares moved | 518 | 702 | **989** | 915 |
| Dodge rolls | 18.7 | 27.0 | 33.2 | 35.3 |
| Ball pickups | 1.04 | 3.80 | 2.30 | 2.12 |
| Blitz declared | 9.07 | 10.71 | 7.95 | 9.06 |
| Block declared | 2.11 | 3.77 | 22.43 | 23.58 |
| — +2D | 0.49 | 1.13 | **10.69** | 9.36 |
| — +1D | 1.47 | 2.30 | 10.79 | 13.49 |
| — −2D | 0.15 | 0.34 | 0.83 | 0.66 |
| Pushbacks | 1.45 | 2.53 | 17.05 | 17.31 |
| Optimal die pick | 65.6% | 89.8% | **99.4%** | 99.0% |
| Knocked down | 20.2 | 10.1 | 21.5 | 22.1 |
| — stun / KO / cas | 17.9 / 2.21 / 1.66 | 10.1 / 1.46 / 0.75 | 22.4 / 2.68 / 1.83 | 23.4 / 3.02 / 2.00 |
| Distinct event types | 36 | 37 | 36 | 34 |

**Both targets met.** GFI 94.8 → **10.4–11.2** (asked: 0–10). Fouls 6.35 → **0.34–0.37** (asked:
0–2), and the ones it does take are worth taking (§16.7). Neither was bought with play strength —
touchdowns went the other way, 1.00 → **1.76**.

### 16.6 Turnovers, and what a turnover costs

| | UNIFORM | GREEDY | ARGMAX |
|---|---|---|---|
| Team-turns / game | 33.9 | 35.5 | 35.2 |
| **Turnovers / game** | **19.7** (58% of turns) | **10.3** (29%) | 10.5 (30%) |
| **Players still unactivated at a turnover** (of 11) | **8.45** | **7.16** | 7.40 |
| Players unactivated at a *clean* turn end | 8.81 | **4.45** | 4.65 |
| Turnover cause | GFI 1399, dodge 488, catch 59, pickup 27 | **dodge 811**, pickup 105, catch 63, **GFI 56** | dodge 858, pickup 85, GFI 58, catch 47 |

Two things stand out.

**The cause of turnovers has completely changed.** Rushing went from the dominant cause (1399) to
almost nothing (56); dodging is now the problem (811). That is the rush-aversion fix working, and it
names the next lever: the same treatment for dodges.

**A turnover still costs about two thirds of a turn.** 7.16 players are left unactivated when one
happens, against 4.45 at a clean end — so a turnover forfeits roughly 2.7 extra players' worth of
activation. That is what makes turnovers expensive and why `c_turnover` scaling with the unactivated
count is the right shape; the measurement says the coefficient could stand to be *larger*, not
smaller.

### 16.7 Foul efficiency — what an attempt actually buys

Counted per attempt, not per game. The first version of this table showed the arms fouling with no
regard for whether the foul would work; fixing that is below it.

| Per foul attempt | UNIFORM | SAMPLED | **GREEDY** | ARGMAX |
|---|---|---|---|---|
| Attempts (100 games) | 93 | 102 | **37** | 34 |
| Attempts / game | 0.93 | 1.02 | **0.37** | 0.34 |
| **Armour broken** | 45.2% | 43.1% | **75.7%** | 73.5% |
| **Stun** | 25.8% | 24.5% | **54.1%** | 47.1% |
| **KO** | 12.9% | 12.7% | 13.5% | 8.8% |
| **Casualty** | 6.5% | 5.9% | 8.1% | 17.6% |
| Referee spots it | 23.7% | 19.6% | 27.0% | 23.5% |
| **Ejected** | 20.4% | 16.7% | 18.9% | 23.5% |
| KO-or-casualty per ejection | 0.95 | 1.12 | **1.14** | 1.12 |

#### Why ejection rate does not move — and cannot

`step_referee.rs:111` decides it:

```rust
referee_spots_foul = armor[0] == armor[1];              // doubles on the armour roll
if !referee_spots_foul && ctx.is_armor_broken() {
    referee_spots_foul = injury[0] == injury[1];        // then doubles on the injury roll
}
```

**Ejection is doubles on the dice. Nothing the agent chooses touches it.** The expected rate is
`1/6 + p_break × 5/6 × 1/6`, i.e. **17–23%** depending only on how often the armour breaks — and all
four arms measured inside that band. The 20.4% vs 16.7% spread that looked like the sampled arm
"avoiding" ejections is 19/93 against 17/102, about **0.67σ**. It is noise, and it was always going
to be: there is no policy that fouls more safely.

If anything the causation runs the *wrong* way. A **better**-chosen foul breaks armour more often,
which opens the second doubles check, so it carries a marginally **higher** ejection risk. The only
levers are fouling less often, or carrying a bribe.

#### Foul assists — the real gap

The first pricing computed `p_break` **from armour value alone**. That was wrong, and it is exactly
the thing that makes one foul worth taking and another not: a foul's armour roll gets **+1 per net
offensive assist** (`foul_assist_armor_modifier`, ±1..7). Scoring from AV alone made every victim
look identical and every foul look bad, so the agent could not learn to foul the player it had three
team-mates standing over.

Now wired to the engine's own two calls — the same ones `InjuryTypeFoul::armour_roll` makes:

```rust
let off = UtilPlayer::find_offensive_foul_assists(g, pid, &t) as i32;
let def = UtilPlayer::find_defensive_foul_assists(g, pid, &t) as i32;
let p_break = /* P(2d6 + (off - def) > AV) */;
```

And restructured from a stack of multipliers into an expectation, because the stack was swallowing
the signal — a three-assist foul and an unassisted one both scored ≈ 0.01, so the agent simply never
fouled:

```rust
let p_eject = 0.167 + p_break * (5.0 / 6.0) * 0.167;
let w = (p_break * victim - p_eject * eject_cost) * timing;
```

**The effect is exactly what it should be.** The agent now fouls **less than half as often** (0.93 →
0.37 per game) and each attempt is far more productive:

| | before | after |
|---|---|---|
| Armour broken per attempt | 45.2% | **75.7%** |
| Stun per attempt | 25.8% | **54.1%** |
| KO-or-casualty per ejection | 0.95 | **1.14** |

#### Per foul attempt, at 400 games

The 100-game table above showed armour breaks nearly doubling while KO and casualty barely moved,
which is not how the rule works: assists modify the **armour** roll, the injury split is unmodified,
so **every** per-attempt rate should scale with the break rate. At 37 attempts that could not be
told apart from noise. At 400 games — 390 attempts against 161 — it can.

**All rates below are per foul attempt**, not per armour break (stun + KO + casualty sums exactly to
the break count):

| Per foul attempt | UNIFORM | **GREEDY** | ratio |
|---|---|---|---|
| Foul attempts (400 games) | 390 | 161 | |
| **Armour broken** | 41.3% ±2.5 | **67.1% ±3.7** | 1.62× |
| **Stun** | 24.6% ±2.2 | **39.1% ±3.8** | 1.59× |
| **KO** | 9.7% ±1.5 | **15.5% ±2.9** | 1.60× |
| **Casualty** | 6.9% ±1.3 | **12.4% ±2.6** | 1.80× |
| **KO or casualty** | 16.7% ±1.9 | **28.0% ±3.5** | 1.68× |
| Ejected | 17.7% ±1.9 | 18.6% ±3.1 | 1.05× |

KO per foul **1.6×**, casualty per foul **1.8×**, KO-or-casualty **1.68×** — all tracking the 1.62×
improvement in break rate, and all separated by well over 1σ. Errors are ±1σ on the proportion.

#### The conditional split is invariant, which is the check that the mechanism is right

If assists only touch the armour roll then the split of a *broken* armour into stun / KO / casualty
must be the same in both arms. It is:

| Conditional on the armour breaking | UNIFORM | GREEDY |
|---|---|---|
| stun \| broke | 59.6% ±3.9 | 58.3% ±4.7 |
| KO \| broke | 23.6% ±3.3 | 23.1% ±4.1 |
| casualty \| broke | 16.8% ±2.9 | 18.5% ±3.7 |

All three within 1σ. And the scaling prediction holds numerically: taking the uniform arm's
per-attempt rates and multiplying by the greedy arm's break-rate ratio predicts **KO 15.8%** against
an observed **15.5%**, and **casualty 11.2%** against an observed **12.4%**.

So the causal chain is exactly the one the rule describes, end to end: **more assists → more armour
breaks → proportionally more stuns, KOs and casualties per foul**, with the injury distribution
untouched. Nothing else in the foul path differs between the arms.

**Ejection is the one rate that does not move** — 17.7% against 18.6%, inside 1σ — for the reason
above: it is doubles on the dice, and no policy can lower it.

**What this does *not* say.** The KO and casualty rates here are set by an unmodified injury roll
against AV8. Getting more out of each *break* needs **injury**-roll modifiers — Dirty Player, Mighty
Blow, Piling On — which a skill-less lineman does not have. That is a roster property, and the foul
heuristic cannot be fully validated until it runs on a tier that has those skills.

### 16.8 What is still wrong

- **Greedy beats pure argmax, 1.80 to 1.45** — 91 scoring games against 84, ~2.6σ over 100 paired
  seeds, so it is real. Argmax takes more 1-dice blocks (13.5 vs 10.8) and fewer 2-dice (9.4 vs
  10.7): with zero exploration it locks onto a deterministic line and cannot get out. **A sliver of
  noise beats none**, which is worth knowing before fitting §8 — the target is not "as sharp as
  possible".
- **The sharp arms have stopped passing entirely** (0.00 passes/game, hand-offs 0.11–0.13, distinct
  event types down to 34). `p_complete` is a hard-coded `0.6` and the carrier's `c_turnover` carries
  a ×1.4, so every pass scores negative. That is a **coverage regression** as well as a play one, and
  it needs the real range-ruler probability from `PassModifierFactory`.
- **Dodges are now the dominant turnover cause** (811 of 1035). Rush aversion needs its dodge
  equivalent.
- **22–24 blocks per game is high.** Blocking is winning on raw weight against moving; the same
  scale question as `Block` vs `Blitz`, one level up.
- Still one tier, one edition, skill-less rosters, Rust-only. Nothing here is a parity result.

### 16.9 Head to head — greedy against uniform, 200 games

The self-play tables above never put the arms against each other. This does: home and away take
different temperature scales (`--heuristic 0.01 --heuristic-away 1e6`), and the whole thing is
played twice with the colours swapped so home advantage cannot flatter either side.

| 100 games each way | Greedy wins | Uniform wins | Draws | Greedy TDs | Uniform TDs |
|---|---|---|---|---|---|
| Greedy home, uniform away | 97 | 0 | 3 | 193 | 0 |
| Uniform home, greedy away | 93 | 0 | 7 | 159 | 0 |
| **Combined (200)** | **190** | **0** | **10** | **352** (1.76/game) | **0** (0.00/game) |

**190–0–10.** The uniform arm did not score a single touchdown in 200 games against a live opponent,
and never won one. The ten draws are all 0–0 games where the greedy side failed to convert.

This is the result the self-play tables were only ever a proxy for, and it is worth being precise
about what it does and does not establish. It shows the §6 weights beat uniform sampling over an
**identical action space and an identical code path** — the only difference between the two players
is one constructor argument. It does not show the agent is *good*: its opponent here samples
uniformly, which is the weakest possible baseline. Beating a real opponent, or the existing
`RandomAgent`, is a different measurement.

---

## 17. Full event table, and results per agent

Everything in §16 counted **both teams together**. This section splits it.

### 17.1 Every event type, all four arms, per game (both teams)

100 games each, current code. Attributable and game-level events alike.

| Event | UNIFORM | SAMPLED | GREEDY | ARGMAX |
|---|---|---|---|---|
| playerMoved | 517.99 | 702.39 | 993.18 | 912.04 |
| playerAction | 81.16 | 134.67 | 204.84 | 193.42 |
| goForItRoll | 114.94 | 19.46 | 11.16 | 10.43 |
| turnEnd | 33.86 | 34.01 | 35.43 | 35.25 |
| dodgeRoll | 18.74 | 26.99 | 32.43 | 34.71 |
| injury | 21.76 | 12.31 | 26.70 | 28.52 |
| playerFellDown | 20.17 | 10.05 | 21.06 | 22.01 |
| blockRoll | 2.11 | 3.77 | 22.02 | 23.11 |
| block | 2.11 | 3.77 | 22.02 | 23.11 |
| pushback | 1.45 | 2.53 | 16.75 | 16.94 |
| scatterBall | 3.78 | 8.37 | 5.97 | 5.26 |
| pickupRoll | 1.52 | 6.34 | 3.83 | 3.31 |
| kickoffResultEvent | 2.00 | 2.17 | 3.55 | 3.41 |
| kickoffScatter | 2.00 | 2.17 | 3.55 | 3.41 |
| catchRoll | 2.07 | 4.83 | 1.60 | 1.45 |
| ballPickedUp | 1.04 | 3.80 | 2.26 | 2.15 |
| **passRoll** | 3.14 | 5.30 | **0.00** | **0.00** |
| mvpRoll | 2.00 | 2.00 | 2.00 | 2.00 |
| winningsRoll | 2.00 | 2.00 | 2.00 | 2.00 |
| startHalf | 2.00 | 2.00 | 2.00 | 2.00 |
| **touchdown** | **0.00** | 0.19 | 1.76 | 1.54 |
| foul | 0.93 | 1.02 | 0.37 | 0.34 |
| refereeSpotsFoul | 0.93 | 1.02 | 0.37 | 0.34 |
| handOver | 0.37 | 1.47 | 0.09 | 0.08 |
| kickoffExtraReRoll | 0.32 | 0.33 | 0.68 | 0.63 |
| cheeringFans | 0.32 | 0.37 | 0.56 | 0.50 |
| weatherChange | 0.27 | 0.25 | 0.42 | 0.44 |
| quickSnapRoll | 0.24 | 0.31 | 0.43 | 0.38 |
| solidDefenceRoll | 0.15 | 0.11 | 0.30 | 0.28 |
| kickoffPitchInvasionStun | 0.15 | 0.14 | 0.19 | 0.34 |
| argueTheCall | 0.22 | 0.20 | 0.10 | 0.08 |
| kickoffTimeout | 0.07 | 0.10 | 0.16 | 0.24 |
| dodgySnackRoll | 0.08 | 0.12 | 0.24 | 0.13 |
| playerEjected | 0.19 | 0.17 | 0.07 | 0.08 |
| kickoffDodgySnack | 0.08 | 0.11 | 0.19 | 0.12 |
| throwIn | 0.05 | 0.11 | 0.07 | 0.17 |
| kickoffPitchInvasion | 0.08 | 0.07 | 0.10 | 0.14 |
| **TOTAL / game** | **840.3** | **995.0** | **1418.5** | **1330.4** |
| **Distinct types** | **36** | **37** | **36** | **36** |

**Only the sampled arm covers all 37.** Uniform misses `touchdown` — it never scores. Greedy and
argmax miss `passRoll` — they never pass. That is the coverage/strength trade in one line: the arm
that plays best has a hole the arm that plays worst does not, which is exactly why §6.5.2's coverage
floor exists and why it stays on in the sampling arms.

### 17.2 Per agent — self-play

Both sides are the same agent, so these two columns are two samples of one policy.

| Per agent, per game | greedy (H) | greedy (A) | uniform (H) | uniform (A) |
|---|---|---|---|---|
| **Touchdowns** | **0.94** | **0.82** | 0.00 | 0.00 |
| Activations | 106.5 | 98.4 | 40.0 | 41.2 |
| Squares moved | 515.8 | 477.4 | 251.4 | 266.6 |
| **GFI rolls** | **3.73** | **7.43** | 56.1 | 58.9 |
| Dodge rolls | 15.97 | 16.46 | 9.17 | 9.57 |
| Pickup attempts | 1.07 | 2.76 | 0.74 | 0.78 |
| Ball pickups | 0.63 | 1.63 | 0.51 | 0.53 |
| Passes | 0.00 | 0.00 | 1.93 | 1.21 |
| Blocks thrown | 11.94 | 10.08 | 1.06 | 1.05 |
| Pushbacks | 9.24 | 7.51 | 0.71 | 0.74 |
| Fouls | 0.18 | 0.19 | 0.49 | 0.44 |
| Injuries **suffered** | 12.68 | 14.02 | 10.47 | 11.29 |
| Knockdowns **suffered** | 9.81 | 11.25 | 9.68 | 10.49 |
| Ejections suffered | 0.04 | 0.03 | 0.10 | 0.09 |
| Record | 35 W | 25 W | 0 W | 0 W |
| | 40 draws | | 100 draws | |

Note the **home/away asymmetry in the greedy arm** — 0.94 vs 0.82 touchdowns and 3.73 vs 7.43 GFI
rolls. The two sides run the same policy on different RNG seeds, so this is either seed variance at
n=100 or a real positional asymmetry (the away team acts first from the opening kickoff). It is not
explained and worth resolving before any weight is tuned on a home-only measurement.

### 17.3 Per agent — against uniform

Genuine head-to-head, played both colours so home advantage cannot flatter either side.

| Per agent, per game | **greedy** (home) | uniform (away) | uniform (home) | **greedy** (away) |
|---|---|---|---|---|
| **Touchdowns** | **1.94** | **0.00** | **0.00** | **1.60** |
| Activations | 117.6 | 41.7 | 39.3 | 114.5 |
| Squares moved | 625.9 | 253.3 | 226.1 | 612.6 |
| **GFI rolls** | **2.57** | 55.7 | 49.9 | **3.11** |
| Dodge rolls | 9.37 | 9.61 | 9.99 | 8.04 |
| Pickup attempts | 2.45 | 1.02 | 0.76 | 3.17 |
| Ball pickups | 1.48 | 0.62 | 0.45 | 1.74 |
| Passes | 0.00 | 2.50 | 3.76 | 0.00 |
| Blocks thrown | 9.39 | 2.00 | 1.82 | 8.67 |
| Pushbacks | 7.54 | 1.31 | 1.25 | 6.62 |
| Fouls | 0.07 | 0.34 | 0.35 | 0.11 |
| **Injuries suffered** | **5.03** | **15.08** | **14.18** | **4.82** |
| **Knockdowns suffered** | **2.85** | **14.31** | **13.49** | **2.43** |
| **Record** | **97 W – 0 L – 3 D** | | | **93 W – 0 L – 7 D** |

Averaged over both colours:

| | greedy | uniform |
|---|---|---|
| Touchdowns / game | **1.77** | **0.00** |
| GFI rolls / game | **2.84** | **52.8** |
| Blocks thrown / game | **9.03** | 1.91 |
| **Injuries suffered / game** | **4.93** | **14.63** |
| Record over 200 games | **190 W – 0 L – 10 D** | 0 W – 190 L – 10 D |

Three things this shows that the combined tables could not.

**Uniform never scored once in 200 games against a live opponent.** Not a low rate — zero.

**Greedy takes a third of the punishment it hands out.** 4.93 injuries suffered against 14.63 —
because it blocks 4.7× as often (9.03 vs 1.91) and because it is not throwing its own players to the
floor with failed rushes: 2.84 GFI rolls against 52.8, an 18× difference. Most of uniform's damage is
self-inflicted.

**Greedy scores twice as much against uniform (1.77) as against itself (0.88).** Which is the right
shape — the self-play number is the harder test, and the one to track.

---

## 18. Runtime cost — measured

§9 budgeted **< 100 ms of agent time per game** and, per prompt class, tens of microseconds a
decision. Here is what it actually costs. All figures are release builds on one core, 100 games,
bb2025 lineman v lineman.

### 18.1 Clean wall clock, all logging off

`FFB_QUIET=1` disables the per-seed event dump, the per-seed `println!`, and every trace. Nothing is
written to disk and nothing is formatted.

| Arm | 100 games | per game | per decision |
|---|---|---|---|
| UNIFORM `1e6` | 79.5 s | 795 ms | 2402 µs |
| SAMPLED `1.0` | 120.9 s | 1209 ms | 2025 µs |
| GREEDY `0.01` | 123.6 s | 1236 ms | 1372 µs |
| ARGMAX `0` | 109.4 s | 1094 ms | 1245 µs |

For reference, with the event dump and per-seed line on: 84.1 / 125.5 / 153.7 / 138.7 s. So the
logging was costing **5–20%** — real, but not the story.

**The story is that this is 11–12× over the per-game budget and 25–48× over the per-decision one.**
That is a clean miss and it should be recorded as one.

### 18.2 Agent time versus engine time

`FFB_HEUR_TIME=1` brackets each `act()` and each `apply()` separately. Per-game wall clock confounds
them badly here, because the sharper arms produce ~70% more events per game and therefore do more
*engine* work for reasons that have nothing to do with scoring. 40 games per arm:

| Arm | decisions / game | agent ms / game | engine ms / game | **agent share** |
|---|---|---|---|---|
| UNIFORM | 331 | 682 | 36.0 | **95%** |
| SAMPLED | 597 | 1043 | 57.9 | **95%** |
| GREEDY | 901 | 1149 | 79.9 | **93%** |
| ARGMAX | 879 | 1046 | 77.9 | **93%** |

**The engine is 36–80 ms per game. The agent is 680–1150 ms.** Scoring is 93–95% of the total, so
every bit of the overrun is the agent's, and the engine — the thing doing the actual Blood Bowl — is
already comfortably inside what the whole budget allowed.

### 18.3 Is the split honest? Residual 0.03%

A fair challenge: does `agent_ns` really contain only the agent? A whole-loop timer settles it —
`agent + engine + residual` must equal the wall clock, so nothing can hide outside the two brackets.

| Arm | loop ms/game | agent | engine | **residual** |
|---|---|---|---|---|
| UNIFORM | 719.3 | 680.6 | 38.4 | **0.2 (0.03%)** |
| ARGMAX | 1090.8 | 1012.1 | 78.3 | **0.4 (0.04%)** |

So the accounting is complete: `agent_ns` brackets exactly `act()`, `engine_ns` exactly `apply()`,
and the loop guards (`is_finished`, `current_prompt`, `active_side`) plus the event `extend` are the
0.03%.

**One thing that split does include, and should be named.** `act()` calls into *engine* code —
`legal_block_targets`, `legal_pass_receivers`, `legal_throw_team_mate_targets`,
`ServerUtilPlayer::find_block_strength`, `UtilPlayer::find_*_foul_assists` — some of which run their
own searches. That work is counted as the agent's, which is right: the agent chose to ask. What it
does **not** include is engine *execution* — step dispatch, dice, state mutation — all of which is
inside `apply()`. So "93–95% agent" means 93–95% *deciding*, against 5–7% *doing*.

### 18.4 Why per-decision cost differs between arms

The arms enumerate and score identically — only the sampling differs — so an average that ranges
from 1245 to 2402 µs per decision needs explaining. Timing per **prompt class**, 40 games each:

| Prompt class | UNIFORM n/game | UNIFORM µs each | ARGMAX n/game | ARGMAX µs each |
|---|---|---|---|---|
| **ActivatePlayer** | 90.4 | **6725** | 206.3 | **4136** |
| **Move** | 150.5 | **481** | 492.9 | **320** |
| BlitzTarget | 9.5 | 11 | 9.1 | 11 |
| BlockChoice | 2.1 | 6 | 24.0 | 6 |
| ReRollOffer | 19.5 | 6 | 14.4 | 5 |
| Pushback | 1.4 | 3 | 18.1 | 3 |
| FollowUp | 1.4 | 3 | 17.9 | 3 |
| TeamSetup | 46.6 | 2 | 81.7 | 2 |
| everything else | ~6 | ≤1 | ~8 | ≤1 |

| Share of agent time | UNIFORM | ARGMAX |
|---|---|---|
| `ActivatePlayer` | **89.3%** | **84.3%** |
| `Move` | 10.6% | 15.6% |
| all 13 other classes combined | **<0.1%** | **<0.1%** |

**Two effects, both real.**

**The mix changes the average.** Argmax makes 879 decisions a game against uniform's 331, and the
extra ones are the *cheap* ones: it blocks 24 times a game against 2, and each block brings a
`BlockChoice`, a `FollowUp` and often a `Pushback` — all 3–6 µs. Averaging over a longer tail of
trivial decisions pulls the mean down without anything getting faster.

**And `ActivatePlayer` is genuinely cheaper in the sharper arm** — 4136 µs against 6725. That is the
`any_unused` filter interacting with turn length. Uniform turns over constantly, so it gets 2.7
activation prompts per team-turn and they cluster at the *start* of a turn with all eleven players
still eligible — eleven Dijkstras. Argmax gets 5.9 prompts per turn and reaches deep into them, where
most players have already acted and are filtered out, so the later prompts run two or three
Dijkstras instead of eleven.

**The optimisation target is unambiguous.** `ActivatePlayer` is 84–89% of all agent time at 4–7 ms a
prompt; `Move` is another 10–16%; the remaining thirteen prompt classes together are under a tenth of
one percent. At ~11 players and ~6.7 ms that is roughly **600 µs per player** just to build and score
one player's reach — for a 60-to-200 node search, which is where the naive Dijkstra below shows up.
Nothing outside these two prompts is worth touching.

### 18.5 Why — and it is exactly what was predicted

**None of P5, P6 or P7 is implemented.** §15 listed the caching work; §16 built the play logic
without it. The measured cost is the predicted cost of that decision, and the four causes are, in
expected order of size:

1. **`reachable` is called O(players) times per activation prompt, and never cached.** §18.4
   measures this directly: `ActivatePlayer` is 84–89% of all agent time. §6.5.1's Move
   branch runs a Dijkstra per eligible player, and the Blitz branch runs another — up to **22
   Dijkstras for a single `ActivatePlayer`**. This is D5, which §14 "fixed" in the *spec* by
   specifying a two-tier scorer, and which this build does not do.
2. **`Board::new()` is rebuilt on every `act()` call.** The tackle-zone map, the row prefixes and the
   carrier lookup are recomputed from scratch for every decision, including the many decisions that
   do not read them. This is what §3's `positions_stamp` exists to prevent.
3. **`threat_on` is O(players) per square, not a precomputed map.** Scoring a ~200-square `Move`
   prompt walks all 22 players per square — ~4400 iterations, each doing a distance and a strength
   comparison — and `path_share` walks them again for the `Screen` intent. §5.2 specifies a bounded,
   cached threat map; this build has neither.
4. **The Dijkstra itself is naive.** A `HashMap` for the visited set, a **linear scan** over the
   frontier to find the minimum (O(n²)), and a full `Vec` clone of the path on every improvement.

Nothing here is surprising and nothing here is deep. It is one caching layer, specified in three
items that were deliberately deferred, and the measurement now says what deferring them costs:
**roughly an order of magnitude.**

### 18.6 What it does not block

At ~1.2 s per game a 100-seed matrix run costs about two minutes, which is why none of this got in
the way of §16 or §17. It matters for two things and no others: a coverage sweep across 30 rosters ×
3 editions × 100 seeds would take hours rather than minutes, and any future search would be paying
this per node. Fix P5–P7 before either.

---

## 19. Against the targets

§10.3's benchmark table, plus the two rates asked for during §16, measured on the **greedy** arm,
100 games, both teams combined (the units §10.3 was written in).

| Metric | Target | Baseline (§0.5) | **Measured** | |
|---|---|---|---|---|
| Touchdowns / game | > 1.5 | 0.00 | **1.76** | ✅ |
| Mean squares per activation | > 3 | 0.95 | **4.85** | ✅ |
| Optimal block-die pick | > 90% | 62.4% | **99.4%** | ✅ |
| Skulls kept with a better die | < 5 | 23 | **0** | ✅ |
| Fouls / game | 0–2 | 6.35 | **0.37** | ✅ |
| GFI rolls / game | 0–10 | 114.9 | **11.16** | ≈ |
| Failed dodges / game | < 2 | 3.6 | **12.62** | ❌ |
| Completions / game | > 1 | ~0 | **0.00** | ❌ |
| Interceptions / game | > 0.1 | ~0 | **0.00** | ❌ |
| Agent time / game | < 100 ms | — | **1149 ms** | ❌ |
| Distinct steps dispatched | > 190/200 | 167/200 | *not measured* | — |

**Six of ten hit, one marginal, three missed, one unmeasured.**

### 19.1 One of those targets was the wrong metric

**"Failed dodges < 2 per game" is a bad target and I should not have written it.** It is trivially
satisfied by not moving: the uniform arm posts 5.3 because it barely goes anywhere. The greedy arm
posts 12.62 because it moves 993 squares a game against 518 and therefore dodges 32 times against
19. The **per-dodge** failure rate is 39% in both arms, because that is dice.

The metric that actually means something is **turnovers per team-turn**, and on that the agent is
where it should be:

| | UNIFORM | GREEDY |
|---|---|---|
| Turnovers / game | 19.7 | **10.3** |
| **% of turns ending in a turnover** | **58%** | **29%** |
| Players still unactivated at a turnover | 8.45 | 7.16 |

Replace the failed-dodge row in §10.3 with turnovers-per-turn.

### 19.2 The blitz is declared 7.8 times a game and never blocks

Auditing the event stream for whether a declaration produces a `block`:

| | declared / game | produced a block | rate |
|---|---|---|---|
| `Block` | 22.02 | 22.02 | **100%** |
| `BlitzMove` | 7.84 | **0.00** | **0.0%** |

Zero. Both sharp arms, 100 games each. **Every blitz the agent declares wastes the team's
once-per-turn blitz.** This is the same shape as the Kick-Team-Mate and Throw-Team-Mate bugs in this
repo's history: an action declared and then abandoned.

The decision trace shows the mechanism:

```
Away ActivatePlayer  -> Activate(away_03, Blitz, target=home_02)
Away BlitzTarget     -> SelectPlayer(home_03)          <- a DIFFERENT player
Away Move            -> Move(4 squares -> 14,11)
Away Move            -> Move(2 squares -> 14,13)
Away Move            -> EndPlayerAction                <- no block, ever
```

Two distinct defects, both mine:

1. **The folded target and the `BlitzTarget` answer disagree.** §6.5.2's Blitz branch picks a victim
   and folds it into the declaration; the engine then asks again via `BlitzTarget`, and that handler
   re-picks from `eligible_players` with a different rule. They chose `home_02` and `home_03`.
2. **The `Move` handler does not know it is in a blitz.** It scores destinations by `V` alone, so it
   walks the blitzer wherever the value model likes and never ends adjacent to the victim — after
   which there is nothing to block and it declines.

The fix is to give the blitz an explicit two-part plan: pick the victim once, then constrain the
movement to squares adjacent to that victim (`legal_blitz_move_targets` already computes exactly
that set), and stop treating the blitz's movement as ordinary movement.

This is now the top open item. Fixing it should also move the passing numbers, because a blitz that
lands is a knockdown the agent currently pays for and never receives.

### 19.3 The three remaining misses

- **Passing is dead** (0.00 completions, 0.00 interceptions). `p_complete` is a hard-coded `0.6` and
  the carrier's `c_turnover` carries a ×1.4, so every pass scores negative and is never chosen. Needs
  the real range-ruler probability from `PassModifierFactory`. It is also the reason the sharp arms
  cover 36 event types where the sampled arm covers 37.
- **Agent time is 11.5× the budget** — §18, and entirely the missing caching layer (P5–P7).
- **Dead-step coverage is unmeasured.** The 167/200 figure predates all of this; nobody has re-run
  the `StepId` inventory against the heuristic agent. It should move — scoring alone unlocks five
  category-C steps — but that is a prediction, not a measurement.

---

## 20. Ten ways to make it faster

1149 ms of agent time per game against a 100 ms budget (§18). The measurement says where every
microsecond is:

| | share of agent time | per prompt | per game |
|---|---|---|---|
| `ActivatePlayer` | **84–89%** | 4136–6725 µs | 853 ms |
| `Move` | 10–16% | 320–481 µs | 158 ms |
| all 13 other prompt classes | **< 0.1%** | ≤ 11 µs | < 1 ms |

So this is a list about two prompts. Savings marked *(est.)* are reasoned from the cost breakdown,
not measured.

### 20.1 Move → Move never makes sense — end the activation eagerly

**Measured: 35.7% of all `Move` prompts are a consecutive Move-then-Move with nothing in between** —
2049 of 5746 over ten games. Mean 2.07 move *answers* per moving activation, with a tail out to
seven. Every one of those is a full ~200-option re-plan that reaches the same place a single longer
path would have reached.

Moving twice is moving once to the final square. The activation should answer its first `Move`
prompt with the whole path and then **end immediately**, without scoring anything, unless the
activation state actually changed in a way that opens options it did not have before:

| Sequence | New options? |
|---|---|
| Move → Move | **No.** Same options, worse path. Never correct |
| Move → Pass / Hand-off | Yes — a throw is not a move |
| Move onto the ball → Move | Yes — it is a carrier now; the value model changed |
| Blitz: Move → Block → Move | Yes — the board changed underneath it |
| Stand up → Move | Yes — it could not move before |

So the rule is a small state machine on the activation, not a scoring decision: after a plain move
with no acquisition and no block, the next `Move` prompt is answered `EndPlayerAction` for free.
Removes ~2050 re-plans and makes the ~1780 end-of-activation answers free as well — together about
two thirds of all `Move` cost. **(est. −10% of total)**

### 20.2 Plan once at declaration; replay it afterwards

The `ActivatePlayer` scorer already ran `reachable` for the player it chose and already knows the
destination and the path. Then the `Move` prompt throws that away and computes it again. Cache the
winning `(player, action, destination, path)` from the activation decision and serve the following
`Move` prompt from it. One Dijkstra and one value sweep saved per activation, and it composes with
20.1. **(est. −8%)**

### 20.3 Two-tier activation scoring — the fix the spec already specifies

§6.5.1 says: cheap proxy for every eligible player, full search for the top three. The build does
neither — it runs `reachable` per player in the Move branch **and again** in the Blitz branch, up to
22 Dijkstras for one prompt. The proxy needs no search at all: carrier or not, marked or not, prone
or not, best block dice from where it already stands, distance to the ball, and the best of its eight
adjacent squares. That resolves every rule in §6.5.1 except "free mover". **(est. −55% of
`ActivatePlayer`)**

### 20.4 Rasterize the value field once per position change

`value_of` is called per square, ~200 times per `Move` prompt and again for every candidate in the
activation scorer. But **most of `V` does not depend on who is moving**: `threat`, `lane`,
`sideline`, the cage geometry, the distance to the ball and to the carrier are all properties of the
*square*. Only `advance`, `urgency` and the MA cap are mover-specific, and those are closed forms.

Precompute two `[f32; 26*15]` rasters (one per side) whenever positions change, and make per-square
scoring an array read plus three multiplies. **(est. −60% of what is left after 20.3)**

### 20.5 Rasterize the threat map specifically

The worst offender inside 20.4, and worth naming separately: `threat_on` walks **all 22 players for
every square**, so scoring a 200-square prompt costs ~4400 player visits — and `path_share` walks
them again for the `Screen` intent. One pass over the players fills the raster; every read after that
is O(1). This is P5 and B6, still unimplemented.

### 20.6 Give the Dijkstra real data structures

`reachable` currently uses a `HashMap<FieldCoordinate, …>` for the visited set, a **linear scan over
the frontier** to find the minimum (so O(n²) in the ~200 nodes), and clones the whole path `Vec` on
every improvement. Replace with a flat `[…; 390]` array indexed `y*26+x`, a `BinaryHeap`, and
back-pointers — reconstructing the path exactly once, for the destination actually chosen.
**(est. 3× inside `reachable`)**

### 20.7 Cache and lazily build the board features

`Board::new()` rebuilds the tackle-zone map, the row prefixes and the carrier scan on **every**
`act()` call — including the ~60 prompts a game (`BlockChoice`, `FollowUp`, `Pushback`, `ReRollOffer`)
that never read any of it. Key it on a positions stamp (P5/B3) and build each part on first use.
During one team's turn the opponent does not move at all, so most of it is invariant for the whole
turn.

### 20.8 Memoize `block_target_weight` per (attacker, defender)

It calls `ServerUtilPlayer::find_block_strength` twice, and that function runs a **nested
player × player loop** for the Guard-cancel rule — roughly 484 iterations per call. It is invoked
from the `Block` branch, the `Blitz` branch and the `BlitzTarget` handler, often for the same pair
several times in one prompt. A small memo table keyed on the pair, cleared with the positions stamp.

### 20.9 Make the action space whole plans, not fragments

The generalisation of 20.1, and the most valuable item here because it is a **correctness** fix as
much as a speed one. Enumerate an activation as `(player, plan)`:

```
Block(victim)                    Foul(victim)
MoveTo(sq)                       MoveTo(sq) + Pass(receiver)
Blitz(victim, via sq)            MoveTo(sq) + HandOff(receiver)
Pickup(ball) + MoveTo(sq)        StandUp + MoveTo(sq)
```

The engine's follow-up prompts are then **replayed from the plan** instead of re-decided: one
scoring pass per activation instead of the current 3.4.

And it makes two things expressible that currently are not:

- **move-then-pass**, which the fragmented design cannot represent at all — the reason passing is
  dead (§19.3);
- **blitz-to-a-specific-victim**, where the movement is constrained to squares adjacent to the chosen
  victim — the reason the blitz never blocks (§19.2).

Two bugs and a speed-up from one restructuring. This is the item to do first if only one gets done.

### 20.10 Prune before scoring, and skip the trivial

Two cheap wins on top:

- **Admissible bound.** With a sharp temperature, any option more than ~10·T below the maximum
  carries under 1e-4 of the probability. Compute an optimistic `p_arrive_upper × V_upper` per
  destination straight off the raster, fully score only the top K, and put the rest in a single
  sampled tail bucket that keeps the ε-floor's guarantee intact.
- **Short-circuit trivia.** Return immediately when an option set has one element instead of running
  softmax over it; reuse scratch buffers rather than allocating a `HashMap` and a path `Vec` per
  `reachable` call.

### 20.11 Expected total

The three structural items — 20.3, 20.4 and 20.6 — carry most of it, and they are all things §15
already listed and §16 deliberately skipped:

| | est. agent ms / game |
|---|---|
| now | **1149** |
| + 20.3 two-tier | ~520 |
| + 20.4 / 20.5 rasters | ~250 |
| + 20.6 Dijkstra structures | ~180 |
| + 20.1 / 20.2 eager end and plan reuse | **~130** |

That lands within ~30% of the 100 ms budget without 20.7–20.10, which are then the margin. And
20.9 is worth doing regardless of the clock, because it is what makes passing and blitzing work at
all.

---

## 21. All ten, measured

All ten of §20 are implemented, **agent-side only**. Every change is inside
`crates/ffb-engine/src/agent/heuristic_agent.rs`, which is the agent boundary; the engine's own
pathfinder, `legal_actions` and `util` are read but never modified, so engine parity is untouched.

### 21.1 The headline

100 games, greedy arm, both agents, all logging disabled:

| | before | after | |
|---|---|---|---|
| **agent — deciding** | 1149 ms/game | **81.1 ms/game** | **14.2×** |
| `ActivatePlayer` | 4136 µs | **192 µs** | 21.5× |
| `Move` | 320 µs | **75 µs** | 4.3× |
| `BlockChoice`, `Pushback`, `FollowUp`, `BlitzTarget` | ~11 µs | 5–11 µs | unchanged |
| **engine — executing** | — | 82.8 ms/game | |
| residual (harness) | — | 0.6 ms/game | 0.4% |
| **whole loop** | — | **164.5 ms/game** | |

The agent was 93–95% of the loop. It is now **49%** — marginally less than the engine that executes
its choices. Against the §10.3 budget of 100 ms it is now **inside**, with room.

Units, because they are easy to misread: both figures cover **one full game with both agents
summed**, and the two timers are disjoint — `agent_ns` wraps `agent.act()` and nothing else,
`engine_ns` wraps `engine.apply()` and nothing else. Per agent the decision half is ~40.5 ms/game;
per decision it is **97 µs to decide and 99 µs to execute**.

### 21.2 Play got better, it did not merely survive

| | before | after | target |
|---|---|---|---|
| Touchdowns / game | 1.76 | **2.15** | > 1.5 ✅ |
| **Blitz follow-through** | **0.0%** | **84.4%** | — ✅ |
| Fouls / game | 0.37 | **0.29** | 0–2 ✅ |
| GFI rolls / game | 11.16 | **1.02** | 0–10 ✅ |
| Squares / activation | 4.85 | 3.60 | > 3 ✅ |
| Distinct event types | 36 | **40** | — |
| Completions / game | 0.00 | 0.00 | > 1 ❌ |

Fewer squares per activation but more touchdowns: the movement is better aimed, not merely longer.

### 21.3 §20.9 is what fixed the blitz

The blitz was declared 7.8 times a game and produced a block **zero** times (§19.2). Two defects:
the folded target and the `BlitzTarget` answer disagreed, and the `Move` handler did not know it was
in a blitz. The plan model fixes both at once — the victim is chosen once, the path is constrained
to a square adjacent to it, and on arrival the agent sends the `Action::Block` that `StepInitMoving`
dispatches as the blitz. **0.0% → 84.4%**, worth 5.6 extra blocks a game that were previously paid
for and never received.

### 21.4 Three bugs that measurement caught and reasoning did not

**The "admissible bound" in §20.10 was admissible, but its *ranking* was not.** Sorting destinations
by `p_arrive` and keeping the top 24 keeps every one-square shuffle (p = 1.0) and discards the
six-square scoring runs (p ≈ 0.3) — exactly the moves that score. **Touchdowns fell 1.76 → 0.19.**
After §20.4's rasters the full value computation is a handful of multiplies over ~90 squares, so the
pruning was buying nothing and paying for it in touchdowns. Removed; the code now carries the
post-mortem so nobody re-adds it.

**`StepInitMoving` guards every folded dispatch and falls *through* when the guard fails**,
re-emitting the same `Move` prompt. Resending a rejected terminal action spun forever. Each is now
gated on the engine's own condition and latched so it is attempted at most once per activation.

**The all-players-used backstop went missing in the rewrite.** An activation that ends without
moving leaves the engine's eligible list unchanged, so `used_this_turn` is the only thing making
progress; without the backstop the driver livelocked. Both hangs presented identically — 200,000
decisions, `ActivatePlayer` and `Move` alternating — and neither was visible from reading the code.

### 21.5 One item measurement rewrote

§20.7 said "cache the board features". Caching alone made the cheap prompts *worse*: building the
rasters for a `BlockChoice` took it from 11 µs to **86 µs**, and `BlockChoice` reads none of them.
The fix is the three tiers §20.7 actually specified — prompts that read nothing about the board,
prompts that read only the cheap core (tackle zones, occupancy, who has the ball), and the two that
read the rasters. That is worth 7.6 ms/game on its own: `BlockChoice` 86 → 7.6 µs, `Pushback`
78 → 5.4 µs, `FollowUp` 80 → 4.6 µs.

### 21.6 Still open

- **Passing is dead** — 0.00 completions. The plan structure now *expresses* move-then-pass, and the
  engine dispatches it, but `pass_weight`'s distance-banded `p_complete` keeps every pass negative
  against the carrier's ×1.4 turnover cost. It needs the real range ruler, not a wider band.
- **Dead-step coverage is still unmeasured** against this agent; the 167/200 figure predates all of
  it. Event-type coverage moved 36 → 40, which is a hint, not the measurement.
- **The parity gate has never been run on this branch.**

---

## 22. Per-agent report

**Every number is one agent's own, pooled across home and away.** Nothing is combined across the two
teams, and the colour split is gone — it was noise, not signal.

Worth stating plainly, because §21 and §22 use different units: §21's `1.76 → 2.15` touchdowns is
**both agents added together**. Per agent that is **0.88 → 1.08**. The rate went *up* by 22%.

Three policies, all the *same program* with one parameter (§1):

| arm | `temp_scale` | what it is |
|---|---|---|
| **GREEDY** | `0` | argmax. No RNG consumed at all |
| **SAMPLED** | `1` | the §8 temperature table, with §6.5.2's coverage floor live |
| **RANDOM** | `1e6` | uniform sampling over the identical option set |

### 22.1 Touchdowns as raw counts

Rates are easy to misread at these magnitudes, so the self-play data as integers first:

| | touchdowns in 100 games | games with any touchdown | most by one agent in one game |
|---|---|---|---|
| **GREEDY** | **215** | 96 / 100 | 3 |
| **SAMPLED** | **31** | 28 / 100 | 2 |
| **RANDOM** | **5** | **5 / 100** | **1** |

The random arm scored **five touchdowns across the entire 100-game run**, never twice in a game, and
**195 of its 200 agent-games were scoreless**. So ~5% of random matches contain a touchdown — which
matches the ~7% prior this work started from, and is a useful check that the runs are sane.

### 22.2 Self-play — each policy against itself

| per agent per game | **GREEDY** | **SAMPLED** | **RANDOM** |
|---|---|---|---|
| agent-games | 200 | 200 | 200 |
| **Touchdowns** | **1.07** | 0.15 | 0.03 |
| Activations | 112.06 | 65.62 | 53.38 |
| Squares moved | 402.16 | 209.60 | 163.69 |
| Squares per activation | 3.59 | 3.19 | 3.07 |
| **ATTACK** | | | |
| Blocks thrown | 16.14 | 7.74 | 6.75 |
| Blitzes declared | 3.38 | 3.57 | 3.23 |
| Blitzes that blocked | 2.86 | 2.59 | 2.21 |
| **Blitz follow-through** | **84.6%** | 72.5% | 68.5% |
| Knockdowns inflicted | 7.46 | 3.37 | 2.69 |
| **Knockdowns per block** | **46.2%** | 43.6% | 39.8% |
| Injuries inflicted | 7.64 | 4.55 | 3.85 |
| KOs inflicted | 0.54 | 0.38 | 0.24 |
| Casualties inflicted | 0.54 | 0.33 | 0.23 |
| Fouls | 0.14 | 1.17 | 1.15 |
| Own players ejected | 0.02 | 0.23 | 0.20 |
| **DEFENCE** | | | |
| **Forced fumbles (from a block)** | **0.42** | 0.19 | 0.14 |
| Fumbles forced per block | 2.6% | 2.5% | 2.1% |
| Interceptions | 0.00 | 0.00 | 0.01 |
| Fumble recoveries | 0.00 | 0.04 | 0.04 |
| Own fumbles | 0.17 | 0.10 | 0.18 |
| **Own falls** | **1.96** | 2.33 | 3.04 |
| **BALL** | | | |
| Pickup attempts | 1.34 | 2.77 | 2.16 |
| Balls picked up | 0.85 | 1.66 | 1.35 |
| Pickup success | 63.2% | 59.9% | 62.7% |
| Passes thrown | 0.00 | 2.73 | 2.73 |
| Completions | 0.00 | 0.92 | 0.76 |
| Hand-offs | 0.11 | 1.19 | 0.83 |
| GFI rolls | 0.57 | 1.54 | 2.08 |
| Dodge rolls | 9.90 | 9.78 | 11.56 |
| **Skulls kept over a better die** | **0.00** | 0.10 | 0.33 |

### 22.3 Against random

| per agent per game | **GREEDY** | **SAMPLED** | **RANDOM** |
|---|---|---|---|
| agent-games | 200 | 200 | 400 |
| **Touchdowns** | **0.89** | 0.17 | 0.02 |
| Activations | 112.13 | 63.17 | 53.85 |
| Squares moved | 349.21 | 199.65 | 166.86 |
| Squares per activation | 3.11 | 3.16 | 3.10 |
| **ATTACK** | | | |
| Blocks thrown | 15.22 | 8.00 | 6.41 |
| Blitzes declared | 1.75 | 3.69 | 3.20 |
| Blitzes that blocked | 1.55 | 2.90 | 2.13 |
| **Blitz follow-through** | **88.3%** | 78.6% | 66.6% |
| Knockdowns inflicted | 7.68 | 3.43 | 2.52 |
| **Knockdowns per block** | **50.5%** | 42.9% | 39.3% |
| Injuries inflicted | 7.91 | 4.86 | 3.31 |
| KOs inflicted | 0.72 | 0.33 | 0.24 |
| Casualties inflicted | 0.71 | 0.28 | 0.20 |
| Fouls | 0.17 | 1.42 | 0.78 |
| Own players ejected | 0.04 | 0.20 | 0.14 |
| **DEFENCE** | | | |
| **Forced fumbles (from a block)** | **0.39** | 0.19 | 0.12 |
| Fumbles forced per block | 2.6% | 2.4% | 1.8% |
| Interceptions | 0.00 | 0.00 | 0.00 |
| Fumble recoveries | 0.03 | 0.04 | 0.01 |
| Own fumbles | 0.10 | 0.15 | 0.13 |
| **Own falls** | **1.54** | 2.70 | 2.60 |
| **BALL** | | | |
| Pickup attempts | 1.43 | 2.71 | 2.69 |
| Balls picked up | 0.81 | 1.68 | 1.59 |
| Pickup success | 56.8% | 62.0% | 59.0% |
| Passes thrown | 0.00 | 2.54 | 3.48 |
| Completions | 0.00 | 0.73 | 1.13 |
| Hand-offs | 0.10 | 1.09 | 1.24 |
| GFI rolls | 0.49 | 1.58 | 1.64 |
| Dodge rolls | 7.18 | 10.48 | 10.73 |
| **Skulls kept over a better die** | **0.00** | 0.08 | 0.30 |

Records over 200 games: greedy beats random **152–46–2**, sampled beats random **30–167–3**, and greedy beats sampled **126–65–9**.

The RANDOM column pools its games against both opponents, so it is 400 agent-games.

### 22.4 Runtime

| | GREEDY | SAMPLED | RANDOM |
|---|---|---|---|
| decisions / game | 418 | 246 | 205 |
| **µs per decision** | **97.0** | **92.2** | **95.6** |
| agent ms / game | **40.5** | 22.7 | 19.6 |
| engine ms / game | 41.4 | 20.2 | 16.9 |

All three cost the same **per decision** — 92–97 µs. That is the §1 contract holding: one program,
one parameter. The cheaper arms are cheaper *per game* only because they make fewer decisions, since
they end their turns sooner.

### 22.5 How the defensive numbers are derived

The engine emits **no `fumble` and no `interception` GameEvent**, so both are reconstructed from the
event sequence. The detectors, stated because the numbers are only as good as they are:

| | detector |
|---|---|
| **forced fumble** | a `blockRoll` by A, then a `playerFellDown` at coord C of a player *not* on A's team, then a `scatterBall` whose `from` is C. Credited to A |
| own fumble | the same shape, but the player who fell is on the acting team |
| **interception** | a successful `catchRoll` by an opponent of the thrower with **no** `scatterBall` between it and the `passRoll` — the ball was taken in flight |
| fumble recovery | a successful `catchRoll` by an opponent **after** a `scatterBall` — the ball was loose, not intercepted |
| completion | a successful `catchRoll` by a team-mate of the thrower, no scatter between |

**The interception row is a measurement gap, not a verdict.** The `Interception` prompt *does* fire —
0.36 times per game in the passing arm — so the mechanic is reachable and the agent is being asked.
But with no interception event in the stream the sequence proxy above is the only observable, and it
reads 0.00–0.01 everywhere. Measuring it properly needs an engine-side event, which stays out of
scope until parity is settled.

### 22.6 What the three arms say

**Greedy is the strength arm, and defence is the clearest part of the gap.** Against random it lands
**50.5% knockdowns per block** against 39.3%, forces **0.39 fumbles a game** against 0.12, and falls
over **1.54** times against 2.60. It never kept a Skull over a better die in 400 agent-games. Blitz
follow-through 88.3% against 66.6%.

**Sampled is the coverage arm, and it costs nearly all of the play strength.** 0.15 touchdowns in
self-play — closer to random (0.03) than to greedy (1.07) — and it loses to greedy 9–126. This is
**by construction, not a defect**: §6.5.2's coverage floor is live at any `temp_scale ≥ 0.1` and tops
out at 0.35, while genuine option weights sit near 0.15. The floor therefore *outranks* real play,
which is exactly what makes the arm explore. The bill is in the table: fouls 1.17 against greedy's
0.14, ejections 0.23 against 0.02.

**But it buys almost no coverage.** Distinct event types: greedy **40**, sampled **41**, random
**41** — and the one type greedy misses is **`passRoll`**. Every other mechanic these runs reach,
greedy reaches too. So the coverage arm currently justifies itself on a single event type, and that
one is dead for a reason already known. Fix passing and the case for a separate sampled arm at this
temperature largely disappears.

### 22.7 The two open items, now better evidenced

**Passing.** Greedy throws zero. The *identical enumeration* throws 2.5–3.5 a game once sampled, so
the receivers, the plan and the engine dispatch all work — the option is enumerated and then priced
out. `pass_weight`'s distance-banded `p_complete` cannot beat the carrier's ×1.4 turnover cost at any
range. The sampled arm's own numbers argue the weights are not simply too pessimistic: it completes
only 0.92 of every 2.73 throws, so a hard prior against passing is not wrong — it is just too hard.
This needs the real range ruler, not a constant.

**Ball contest.** Greedy loses the ball race to random: **1.43 pickup attempts a game against 2.69**,
and 0.81 secured against 1.59. It scores fewer touchdowns against random (0.89) than against itself
(1.07), which should not happen against a weaker opponent — and this is the mechanism. A ball it
never holds is a ball it cannot score with.

---

## 23. Wide and deep

> **Superseded by §25 for DEEP.** The deep mode measured here walked one square per decision, which
> was the wrong design; it was rebuilt to move full paths from a single search and now beats wide on
> both speed and strength. The wide figures and the self-play trap below still stand.

Two search shapes over **identical weights**. Same primitives, same value model, same temperature
knob — only the shape of the search differs, which is what makes them comparable at all.

| | what one decision is |
|---|---|
| **WIDE** | the whole joint action space: every player × action × target × destination, drawn once |
| **DEEP** | a chain — pick the player, then that player's action-and-target, then walk the move **one square at a time**, deciding again at every step |

Selected with `--mode wide|deep`, and `--mode-away` gives the two sides different shapes so they can
play each other.

### 23.1 Branching factor

One sampled game, seed 7:

| | prompts | decisions | options / decision (mean) | median | max | **options examined** |
|---|---|---|---|---|---|---|
| WIDE | 504 | 213 | **1055.3** | 1239 | 2188 | **224,772** |
| DEEP | 731 | 667 | **8.9** | 9 | 195 | **5,957** |

Deep faces a branching factor **119× smaller** and examines **37× fewer options** across the whole
game. It pays for that with 3.1× as many decisions.

### 23.2 Time

Greedy, 30 games, logging off:

| | whole loop | agent | agent / one agent | engine | decisions / game | µs / decision |
|---|---|---|---|---|---|---|
| WIDE | 271.5 ms | 192.4 | 96.2 | 78.5 | 859 | **223.9** |
| DEEP | **238.7 ms** | **126.4** | **63.2** | 111.3 | 1712 | **73.8** |

Deep is **3× cheaper per decision** and **34% cheaper in agent time** — but it makes twice as many
decisions, and the engine has to execute every one of them, so engine time rises 42%. The
**whole-loop** difference is only 12%. Most of what deep saves in the agent it hands to the engine.

### 23.3 Strength — and a trap worth naming

| per agent per game | WIDE greedy | DEEP greedy | WIDE sampled | DEEP sampled |
|---|---|---|---|---|
| **Touchdowns** | 1.02 | **1.26** | 0.21 | 0.01 |
| Activations | 113.94 | 111.05 | 68.98 | 52.95 |
| Squares moved | 409.55 | **507.95** | 281.18 | 212.22 |
| Squares / activation | 3.59 | **4.57** | 4.08 | 4.01 |
| Blocks thrown | **15.67** | 14.41 | 7.15 | 4.04 |
| Knockdowns inflicted | **7.38** | 6.89 | 3.21 | 1.85 |
| Injuries inflicted | **7.53** | 6.99 | 4.47 | 2.80 |
| Forced fumbles | 0.43 | **0.67** | 0.11 | 0.14 |
| Own falls | **2.17** | 3.03 | 4.12 | 6.59 |
| Balls picked up | 1.17 | **1.46** | 0.99 | 2.19 |
| GFI rolls | **0.62** | 5.10 | 19.21 | 16.45 |
| Blitz follow-through | 58.5% | 59.5% | 43.6% | 31.4% |

On self-play alone deep looks **better**: 1.26 touchdowns to wide's 1.02, and 4.57 squares per
activation to 3.59.

Then they play each other, 200 games, both colours:

| | record | touchdowns / game |
|---|---|---|
| **WIDE** | **168 – 24 – 8** | **1.86** |
| DEEP | 8 – 24 – 168 | 0.33 |

**Self-play scoring rate measures the pair, not the policy.** Deep scores more against itself because
both sides defend badly; against wide's defence its attack collapses from 1.26 to 0.33. Every
"touchdowns per game in self-play" number in this document carries that caveat, including the ones
used earlier to claim progress. Head-to-head is the only strength measure here.

Why deep loses, from the table: it moves *further* (508 squares to 410) but rushes eight times more
often (5.10 GFI to 0.62) and falls over more (3.03 own falls to 2.17). A one-step walk follows the
value gradient and cannot see that the last square of a run needs a rush it should not take — a
Dijkstra over the whole reachable set prices the whole route before committing to any of it. Deep
also throws fewer blocks and inflicts fewer injuries, so it neither scores against a real defence nor
grinds one down.

### 23.4 A correction, and a fairness fix

**The blitz follow-through figure reported in §21 and §22 was wrong.** The measuring script cleared
its pending declaration on `turnEnd`, which silently dropped every blitz that was the last action of
a turn — exactly the ones most likely to have failed. Counted directly:

| | declared / game | blocked | follow-through |
|---|---|---|---|
| TIER2=16, 6 destinations | 13.09 | 7.26 | **55.5%** |
| all destinations, flat draw | 12.35 | 6.99 | 56.6% |
| all destinations, nested draw | 12.46 | 7.24 | **58.1%** |

So the honest claim is **0% → ~58%**, not 0% → 84%, and nothing regressed across those builds — the
earlier number was an artefact of the counter, not a property of the agent.

Deep mode also needed a fairness fix before this comparison meant anything. Stage 2 runs with no
reach, and the blitz branch dropped any victim it could not path to — so deep could only blitz a
player it was **already adjacent to**, which flattered its follow-through to 97.6% by removing the
hard cases. Distant victims are now offered on a distance estimate, since the step-wise walk is
goal-seeking and approaches them; follow-through fell to 59.5%, level with wide, which is the number
to trust.

### 23.5 What each mode is for

Neither dominates, and they answer different questions.

- **WIDE** is the stronger player and the one to measure play strength with. It is also the shape a
  search algorithm wants a *policy prior* from, because it prices whole routes before committing.
- **DEEP** is the shape a tree search wants to *expand*: a branching factor of 9 instead of 1055 is
  the difference between a tractable tree and an intractable one, and it reaches the same action
  space one small decision at a time. Its weakness is purely the missing lookahead, which is exactly
  what a tree search would supply.

---

## 24. Fixes found by watching it play

Two viewers, one per mode, both driven by the same `FFB_HEUR_DUMP` stream:

- `docs/decision_replay_wide.html` — 515 prompts, 217 decisions, ~920 options each
- `docs/decision_replay_deep.html` — 859 prompts, 786 decisions, ~8.7 options each

Built with `scripts/build_decision_replay.py`. Everything below was found by reading them.

### 24.1 A blitz that has to move cannot land — an ENGINE limitation

Splitting every blitz declaration by whether the blitzer moved before blocking, over 100 games:

| blitz declared… | declared | blocked | rate |
|---|---|---|---|
| victim **already adjacent** | 759 | 728 | **95.9%** |
| **after any movement** | 505 | **1** | **0.2%** |

`StepInitMoving` accepts `Action::Block` during a `BlitzMove` and calls `dispatch_player_action`,
which publishes `DispatchPlayerAction` and jumps to `goto_label_on_end`. The sequence generator has
to act on that, and after movement it does not — the activation simply ends. The agent-side trace
confirms the agent is doing its part: every skipped case logged
`pa=Some(BlitzMove) blocked=false adj=true`, i.e. the block **was** sent.

That is 40% of all declared blitzes burning the team's once-per-turn blitz for nothing. It is an
engine defect, and parity comes first, so the agent adapts instead: **only declare a blitz against
an already-adjacent victim.** Movement *after* the block still works, so an adjacent blitz remains a
block plus a free reposition.

| | before | after |
|---|---|---|
| WIDE follow-through | 57.7% | **97.3%** |
| DEEP follow-through | 59.1% | **97.3%** |
| WIDE touchdowns (self-play) | 2.04 | **2.25** |
| DEEP touchdowns (self-play) | 2.52 | **2.68** |
| WIDE blocks / game | 31.3 | **37.2** |

This also reverses a judgement in §23: deep mode's original 97.6% follow-through was **correct
behaviour**, not the limitation I called it. Offering it distant victims made it worse, and the
"fairness fix" was fixing the wrong thing.

### 24.2 The blitz metric was wrong three times

Worth recording, because the same metric misled in both directions before it was right:

| attempt | error | read |
|---|---|---|
| 1 | cleared the pending declaration on `turnEnd`, dropping every blitz that was a turn's last action | **84%** — too high |
| 2 | treated the next `playerAction` as the window's end, but the engine emits `playerAction Blitz` for the dispatch itself, so each success scored as one failure plus one success | **58%** — too low, and declarations doubled |
| 3 | window runs to the next `playerAction` by a **different** player | **97.3%** — correct |

### 24.3 Passing: three pricing errors fixed, and it still does not throw

The range is now the engine's own ruler — `PassMechanic::find_passing_distance` over the edition's
`throwing_range_table`, which is asymmetric in dx/dy and returns nothing for Long/LongBomb in a
Blizzard — with the target from `minimum_roll_simple`. An out-of-range pass is no longer offered at
all. Three genuine errors went with it:

1. the receiver's square was valued with the **thrower's** `Mover`, so a pass was priced as though
   the thrower ran there himself and the receiver's gaining the ball counted for nothing;
2. the catch used the raw AG target, missing the **+1 for an accurate pass** — 3+ against 2+;
3. `c_turnover(carries_ball = true)` charged the carrier's ×1.4 premium **on top of** the
   incompletion, double-counting the very loss the incompletion represents.

Weights moved from −0.98…−0.65 to −0.70…−0.40. **Still negative, and passing remains 0.00 per
game** — and the trace says why:

```
dist=QuickPass tgt=4 pThrow=0.50 pCatch=0.83 pC=0.42 v=0.07 cto=0.75
```

`tgt=4` is correct: a human **lineman**'s PA is 4+, so his quick pass is a coin flip. The binding
term is `v=0.07` — the receiver's square is worth almost nothing, because the support intents
(cage, mark, screen) pull team-mates *toward the ball*, never downfield. There is never anybody
worth throwing to.

So this is no longer a pricing bug; it is a **positional** gap. Passing needs a receiver intent that
sends someone deep, which is a design change rather than a fix, and it is not attempted here.

### 24.4 Diagnostics kept

Three env-gated traces earned their place and stay in: `FFB_BLITZ_TRACE` (why a blitz dispatch was
skipped), `FFB_PASS_TRACE` (receivers and weights), `FFB_PASS_TRACE2` (the term-by-term breakdown of
one pass). Each of them settled a question that reasoning about the code had got wrong.

---

## 25. Deep, rebuilt — and it wins

**§23's deep mode was the wrong design and its numbers are superseded.** It walked one square per
decision, which minimises the branching factor but throws away all lookahead — and it lost
3–31–166. The economy deep is actually after is not a smaller move; it is **not paying to pathfind
eleven players in order to pick one**:

| stage | choice | options | search |
|---|---|---|---|
| 1 | which player | ~11 | none |
| 2 | that player's action and target | ~15 | none |
| 3 | the destination, as a **full path** | every reachable square | **one Dijkstra** |

Wide runs up to `TIER2` = 16 searches per activation prompt (~1800 a game) to build its joint
enumeration. Deep runs exactly one, after the player is settled (~114 a game). The move itself is a
whole path sampled from the complete reachable set, exactly as in wide.

### 25.1 Time

Greedy, 30 games, logging off:

| | whole loop | agent | per one agent | engine | µs / decision |
|---|---|---|---|---|---|
| WIDE | 328.6 ms | 229.9 | 115.0 | 98.1 | 255.0 |
| **DEEP** | **157.7 ms** | **75.0** | **37.5** | 82.1 | **82.6** |

Deep is **3.1× cheaper in agent time** and 2.1× cheaper over the whole loop — and at 75 ms/game it
is the first configuration in this document to come in **under the 100 ms/game budget** §10.3 set.

### 25.2 Strength

Head-to-head, 200 games, both colours:

| | record | touchdowns / game |
|---|---|---|
| **DEEP** | **72 – 79 – 49** | **1.14** |
| WIDE | 49 – 79 – 72 | 0.96 |

Consistent in both colours — deep 37–37–26 as home, 35–42–23 as away — so it is not a side effect.

**Deep is both faster and stronger.** That reverses §23 completely, and the reason is instructive:
the step-wise version was not "deep with less lookahead", it was deep with *no* route pricing at
all. Once stage 3 prices a whole path the way wide does, deep keeps every bit of wide's movement
quality and simply stops paying for fifteen searches it was going to discard.

Where wide is still ahead is the *joint* choice: it can see that player A's best move is worth less
than player B's, because it priced both. Deep commits to a player on a search-free proxy and can be
wrong about that. The head-to-head says that cost is smaller than the compute it saves.

### 25.3 One bug worth recording

The first cut of stage 3 scored **zero** touchdowns. `AgentPrompt::Move`'s `squares` is the set of
legal **next** squares — the adjacent ones — not the set of legal destinations. Wide checks
`path.first()` against it; the rewrite filtered *destinations* against it, so only one-square moves
survived and no carrier ever reached an endzone. The destination set must not be filtered by
`squares` at all; the path's first square is what has to be legal.

---

## 26. Passing, made possible

§24.3 left passing dead for a positional reason, not a pricing one: `v` for the receiver's square
was ~0.07 because cage, mark and screen all pull team-mates *toward* the ball, so there was never
anybody worth throwing to. Three changes, and they only work together.

### 26.1 A receiver intent

A non-carrier standing where he could **catch and then run it in next turn** is worth more than one
standing in a screen — and worth more still if he can actually catch. The square must be

- inside the range table (`dx < 14 && dy < 14`), so the throw is legal at all;
- within `MA + 2` of the endzone, so he could cover the rest himself; and
- **ahead of the ball**, which is what makes it a receiver position rather than a spot beside it.

Weighted `0.30 · p(catch) + 0.20 · closeness`, where `p(catch)` is the real AG−1 accurate-pass target
with a Catch re-roll if he has the skill.

That third condition was learned the hard way. Without it the intent fired across half the pitch and
pulled the whole team off the cage:

| | touchdowns | passes / game |
|---|---|---|
| before | 2.26 | 0.00 |
| receiver intent, unconstrained | **1.94** | 0.11 |
| …restricted to ahead-of-ball, and weakened | **2.26** | 0.08 |

### 26.2 A pass that rescues an unscoreable drive

When the carrier cannot reach the endzone in the turns the half has left, running is worth nothing
and a completion is the only path to a score. The expectation has to say so:

- if a receiver **can** still make it when the carrier cannot, the value of success rises to at
  least 0.85 — the pass converts a dead drive into a live one;
- and the cost of the turnover falls to 30%, because a drive that was going to score zero has
  little left to lose. The generic `c_turnover`, priced off how many players are still unactivated,
  cannot see that.

Measured over the pass options priced in one run: **56.4% of rescue passes are positive**, against
**0.4% of normal ones**. The agent passes when passing is the answer and not otherwise.

### 26.3 The run had to be repriced too

Even so, rescue passes lost — because the carrier's own move did not know the drive was hopeless
either. `urgency` gets this exactly backwards: it saturates at 1.0 precisely when the score becomes
impossible, *raising* the value of a pointless advance. Runs that cannot reach the endzone in time
are now damped to 25% of their advance value, which leaves the ground worth something for field
position but stops it competing with the only move that can still score.

| | touchdowns | passes / game |
|---|---|---|
| WIDE before | 2.26 | 0.00 |
| WIDE after | 2.14 | **0.30** |
| DEEP before | 1.89 | 0.00 |
| DEEP after | 1.82 | **0.19** |

### 26.4 It passes when it should, and only then

Every pass thrown across 100 games, by turn (8 is the last of the half):

| turn | 5 | 6 | 7 | 8 |
|---|---|---|---|---|
| passes | 7 | 16 | 5 | 2 |

**Zero before turn 5; 77% in turns 6–8.** That is the requirement met exactly — the agent throws
when the clock has made running useless, and never as a first resort. The cost is 0.12 touchdowns
in wide and 0.07 in deep, about one standard error at n = 100.

The pass options in the viewers now carry the arithmetic behind the decision, e.g.
`pass to away_07 · RESCUE: 5 turns to run it in, 3 left — he needs 2`.

### 26.5 A measurement trap worth recording

`FFB_HEUR_DUMP` writes with `fs::write` once per game, so the file holds **only the last seed**. Two
counts earlier in this work were quoted as "over 20 games" when they were one game. Trace output on
stderr does accumulate across seeds; the dump does not. The viewers only ever want one game, so this
stays as it is — but it is not a multi-game source.

---

## 27. Ball-moves: eight attempts, and an honest negative

**I did not get passing and hand-offs to improve the agent.** They are in, they behave as asked, and
they cost about 4% of touchdowns. This section is the trail, because the negative result is worth
more than another round of tuning.

### 27.1 How it was measured

Self-play touchdown rate cannot answer "did this help" — it measures the pair, not the policy
(§23.3). So a `wide-noball` mode was added: the identical agent with the Pass and HandOff branches
switched off. The variant then plays it head-to-head, both colours, and the result is reported in
standard errors on the decisive games.

That harness is the most useful thing to come out of this section, because it caught two false
positives that would otherwise have been shipped as wins.

### 27.2 The trail

| | change | games | result |
|---|---|---|---|
| v1 | receiver's square valued absolutely | 300 | −0.08 SE |
| v2 | only credit a move that *creates* a score | 300 | −0.08 SE |
| v3 | tempo: value the ground the throw buys | 300 → **800** | +0.48 → **−1.36 SE** |
| v4 | price the margin over running, not the position | 800 | −0.75 SE |
| v5 | `scores_now` as a probability, not a certainty | 800 | 0.00 SE |
| v6 | put ball-moves on the same absolute scale as moves | 800 | +0.61 SE |
| v7 | real FUMBLE / INACCURATE split from the engine | 800 → **3200** | +0.67 → **−2.55 SE** |
| v8 | narrowed to the scoring case alone | 3200 | −2.74 SE |
| v9 | activate the receiver after a ball-move | 3200 | **−1.46 SE** |

**Two readings at 800 games (+0.48, +0.67) reversed to −1.36 and −2.55 at scale.** At this effect
size 800 games is not enough to see the sign, and I reported the first of those as progress before
checking. Any future claim about this agent's strength needs the 3200-game harness.

### 27.3 What each attempt actually found

Three were real bugs, and are worth keeping regardless of the outcome:

- **v5.** `scores_now` tested `d ≤ MA + 2` and called it a touchdown — a ten-square dash needing
  *two rushes* scored 1.0 and was multiplied by a 1.6 payoff. The agent handed the ball off about
  three times a game believing it had scored. It is now `P(touchdown | he catches it)`.
- **v6.** A Move option is scored absolutely — where is the ball at end of turn — and ball-moves
  were being scored *marginally*, as the gain over running. A marginal +0.2 was beating an absolute
  +0.2 move. Not comparable quantities.
- **v9.** The receiver was activated before the turn ended only **78%** of the time. The other 22%
  moved the ball one square, spent the carrier, and stopped — the plan's premise never happened.
  Making the follow-up explicit halved the deficit, from −2.74 to −1.46.

### 27.4 Why it still does not pay

A hand-off buys about one square and costs a catch roll, a spent activation, and the risk of a
turnover. The benefit that justifies a real passing game is *positional and spans turns* — you
position a receiver, and the throw pays off over the next two turns. **This agent chooses each
activation greedily with no lookahead, so it cannot collect that.** The one non-speculative case —
a receiver who still has his activation and can run it in when the carrier cannot — is simply too
rare to cover the cost: 2.50 hand-offs a game convert 26% of the time, 0.41 passes convert 11%.

That is the same conclusion §24.3 reached qualitatively, now with a number on it. Making passing pay
needs a receiver-positioning model that puts somebody genuinely deep every turn, or a search that
can see two turns ahead — not another weight.

### 27.5 What is in the build, and how to turn it off

The behaviour asked for is there and verified over 200 games:

| | per game | in turns 5–8 |
|---|---|---|
| Passes | 0.41 | **90%** |
| Hand-offs | 2.50 | 69% |

Rare in open play, common when the clock has made running useless — and priced by the engine's own
range ruler and fumble grading throughout. The cost is ~4% of touchdowns against `wide-noball`,
which remains available as a mode for anyone who wants the stronger, ball-move-free agent.

---

## 28. Move-then-give: the play is real, the engine cannot offer it

The review note was exactly right about Blood Bowl:

> A move + handoff to a non-activated unit enables double movement (two players) with the ball. The
> same for passing; moving → passing → catching → moving (and potentially a handoff chain here)
> allows for really long plays that are necessary to score in turn 7 or 8.

`HandOverMove` and `PassMove` are precisely that: the carrier moves his full MA + 2 **first**, gives
the ball at the end of it, and the receiver — if he still holds his activation — then moves his own.
The ball travels `carrier's move + 1 + receiver's move` in one turn, about double what running can
manage, which is the only way to cross a pitch in the two turns left at the end of a half.

**This engine's `legal_actions` never offers it.** Instrumented over six games, the actions presented
for a ball carrier are only ever:

```
101 × HandOver        0 × HandOverMove
108 × Pass            0 × PassMove
```

and `legal_actions/mod.rs` contains **zero** occurrences of `HandOverMove` or `PassMove`. The
hand-off option is gated on a team-mate being **already adjacent**, with the receiver folded into
the declaration, and the engine resolves the give immediately — no movement phase is ever emitted.
`StepInitMoving` does contain the machinery to dispatch a give from a `Move` prompt, but nothing
reaches it, because no declaration ever produces the `…Move` player action.

Declaring it bare does not help: the other two agents both fold the receiver in, and the code
comments record why — *"Declaring a bomb with no target left StepInitPassing parked with an unset
thrower and NO prompt."* Measured, a bare `HandOff` declaration silently consumes the activation
and does nothing at all.

### 28.1 What that cost, and what it explains

Two more variants were measured against `wide-noball` before the cause was found:

| | change | games | result |
|---|---|---|---|
| v10 | run-up enumerated, receiver folded in | 3200 | **−8.63 SE** |
| v11 | …plus two real bugs fixed (below) | 3200 | **−7.18 SE** |

Both are far worse than v9's −1.46, and for a simple reason: the agent was **pricing the ball as
though the carrier had run up**, while the engine gave the ball away from where he stood. A
systematic over-valuation of every hand-off.

Two genuine bugs were found on the way and are worth recording even though the branch was reverted:

- **`w * p_arrive` is a sign error.** Right for a positive weight, backwards for a negative one:
  multiplying −0.6 by an arrival chance of 0.4 gives −0.24, so a *riskier* run-up made a *bad* plan
  look better. Risk has to be §5.3's `p·w − (1−p)·c` at every scale.
- **`pass_weight` looked the thrower up through `acting_player.player_id`**, which during activation
  enumeration is the previous player or nothing, so the lookup failed and every pass was silently
  dropped — passes measured 0.00 per game.

### 28.2 Where this leaves passing

This is the same shape as §24.1's blitz: the rules permit the play, the engine's declaration route
does not, and parity comes first so the engine is not touched. It is now the *second* confirmed case
where the agent's ceiling is set by the action set rather than by the policy.

It also finally explains §27's negative result. Every valuation there was trying to make an immediate
one-square give pay for a catch roll. It cannot, and no weighting will make it — the play that pays
is the one the engine does not offer. **Passing cannot be made to improve this agent until
`legal_actions` emits `HandOverMove` and `PassMove`.**

The build therefore keeps §27's state: ball-moves priced for what the engine can actually do, ~4%
of touchdowns worse than `wide-noball`, and behaving as asked — rare in open play, concentrated in
turns 5–8. The engine gap is the thing to fix, and it belongs in the parity backlog rather than in
the agent.

---

## 29. The move-variant declarations: found, half-fixed, one gap left

§28 concluded that move-then-give was unreachable. That was right about the symptom and **wrong
about the cause** — it is not that Java and Rust disagree, it is that Rust only ever exercised one
of the two declarations Java has.

### 29.1 Java has two declarations; Rust only ever sent one

`StepEndSelecting` — in **both** engines — routes them differently:

| declaration | sequence |
|---|---|
| `HAND_OVER` | Pass — give immediately, no movement |
| `HAND_OVER_MOVE` | **Move** — move, *then* give |

The real Java **client** always sends the move-variant (`SelectLogicModule`):

```java
case HAND_OVER: sendActingPlayer(player, PlayerAction.HAND_OVER_MOVE, false);
case PASS:      sendActingPlayer(player, PlayerAction.PASS_MOVE, false);
```

The Java **parity harness** does not — `declared = (action == BLITZ) ? BLITZ_MOVE : action` — so it
declares the immediate `HAND_OVER`/`PASS`. Rust's `pac_to_player_action` mirrors the harness, which
is why the parity suite agrees and why no agent could move before giving the ball. Rust was faithful
to the harness and the harness never used the play.

### 29.2 What was added

Two new `PlayerActionChoice` variants, `HandOffMove` and `PassMove`, mapped to
`PlayerAction::HandOverMove` / `PassMove` at all four sites plus the client encoder. **Nothing
existing changed**: the parity agents keep declaring `HandOff`/`Pass`, `legal_actions` offers the
same slots in the same order, and every current mapping is untouched, so the parity streams are
byte-identical. `ffb-engine` 7306/0, `ffb-client` 1532/0, `ffb-parity` 49/0.

And it works as far as the movement phase — verified in the sequence trace:

```
Activate(home_06, HandOffMove, target=home_04)
Move(6 squares -> 16,12)      ← the carrier now runs his move
HandOff(home_04)              ← and gives at the end of it
```

### 29.3 The gap that remains

The give then **parks the driver**: the game stops at 267 events instead of ~1500, and the A/B read
−28 SE, which is what a stalled game looks like rather than a bad policy.

`StepEndMoving::push_sequence_for_player_action` dispatches
`HandOver | HandOverMove | … → Pass::build_sequence(&PassParams::default())` — with **no target**.
`StepInitPassing`'s hand-over branch then requires `thrower_action == HandOver`,
`thrower_is_acting` and a catcher, none of which the move path establishes; the immediate path gets
them from `StepInitSelecting`. In Java the receiver rides on `ClientCommandHandOver` and is picked
up downstream; Rust's `Action::HandOff { receiver_id }` is discarded by `StepInitMoving`, which
matches Java's own handler but leaves Rust without the state Java gets from the command object.

**Half of that is now closed.** Java sets the thrower and catcher in
`StepInitPassing.handleCommand(CLIENT_HAND_OVER)`, because in Java the command reaches *that* step.
Rust's agent answers the Move prompt instead, so `StepInitMoving` is where the same state has to be
established — and it now is:

```
game.pass_coordinate = Some(receiver's square);
game.thrower_id      = acting player;
game.thrower_action  = Some(PlayerAction::HandOver);
publish StepParameter::CatcherId(receiver)
```

with the `CLIENT_PASS` equivalent alongside it — pass coordinate from the command, catcher derived
from that square, thrower from the acting player. That lifted a move-then-give game from **267
events to 967**: the driver no longer stalls on `StepInitPassing`'s no-thrower bail, which returns a
bare `StepOutcome::cont()` with no prompt and stops the game outright.

**What is still wrong:** the give does not resolve. No `handOver` event is emitted — the carrier
finishes his run and the turn simply ends. `StepInitPassing` is now reached, but returns before its
hand-over branch, which means `end_turn` or `end_player_action` is already set by the time it runs:
something upstream is ending the activation instead of handing the ball over.

That is a bounded question rather than an open-ended one — **which step publishes `EndPlayerAction`
between `StepInitMoving::handle_command` and `StepInitPassing::execute_step`** — and it is the next
thing to trace. `StepInitMoving` does publish `EndPlayerAction(true)` on its own move-stack-empty
path, which is the first suspect.

Until it resolves, the agent declares the immediate form the engine completes: 2.00 touchdowns, 1.93
hand-offs, 0.20 passes per game, 1340 events per game.

The agent therefore declares the immediate form again, which the engine completes: 2.10 touchdowns,
1.93 hand-offs and 0.15 passes per game, games finishing normally.

### 29.4 The loop

`scripts/ballmove_loop.sh <tag> [seeds]` runs one iteration: build, **check games still finish**,
then A/B against `wide-noball` over both colours and report standard errors.

The sanity check is the important part and was learned the hard way — a broken ball-move path parks
the driver, and the A/B reports that as a catastrophic loss without ever saying the games stopped.
1600 seeds is the floor: §27.2 records two readings at 800 games that reversed sign at 3200.
