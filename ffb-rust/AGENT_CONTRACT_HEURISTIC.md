# Heuristic Agent Contract

Single source of truth for the **heuristic** agent on both sides:

- Rust: `crates/ffb-engine/src/agent/heuristic_agent.rs` (+ `agent/det_math.rs`)
- Java: `ffb-ai/src/main/java/com/fumbbl/ffb/ai/parity/heuristic/` *(not written yet)*

Companion to `AGENT_CONTRACT.md`, which governs the **random** parity agent. That contract is about
which RNG stream is consumed when; this one is about that **plus** the arithmetic, because the two
implementations must agree on every scored weight, not merely on the number of draws.

Any change to either implementation MUST be reflected here and mirrored on the other side **in the
same change set**. Campaign log: `docs/PARITY_HEURISTIC_CAMPAIGN.md`. Design and measurement
history of the policy itself: `docs/HEURISTIC_AGENT.md`.

> **Status:** the Rust side satisfies this contract as of ITER2. The Java side does not exist yet;
> this document is written first, deliberately, so the port is checkable against a spec rather than
> against a 3,657-line moving target.

---

## 1. RNG channels

| Channel | Algorithm | Seed | Notes |
|---|---|---|---|
| heuristic | Xoshiro256\*\* | `seed ^ 0x4845_5552_4953_5449` (`"HEURISTI"`) | every scored decision |
| fallback | Xoshiro256\*\* | `seed` (plain) | the embedded `UniformAgent`, for prompts the heuristic does not score |

Both are seeded via SplitMix64, exactly as `Xoshiro256StarStar.java` already implements for the
random agent. They are independent of the engine's `GameRng` (game dice), so the agent never
perturbs engine rolls.

The unit draw is:

```
unit() = ((next_u64() >> 11) as f32) / (1u64 << 53) as f32
```

Java: `(float)(rng.nextLong() >>> 11) / (float)(1L << 53)` — an **unsigned** shift, and the
`long -> float` conversion is round-to-nearest-even in both languages, so it agrees bit for bit.

The uniform escape draw is `next_u64() % n`, i.e. Java `Long.remainderUnsigned(rng.nextLong(), n)`.

---

## 2. Draw counts — exact, and load-bearing

The number of draws per decision is **not constant**. It depends on the option count and on
`temp_scale`, and both sides must reproduce it exactly or the streams desynchronise.

`EPS = 0.02`. `temp_scale` is the constructor argument (`--heuristic <scale>`): `0.0` = argmax,
`1.0` = the sampled arm, `1e6` = uniform over the same option set.

### `pick(t_base)` — used by `sample()` and by every DEEP stage

| condition | draws | behaviour |
|---|---|---|
| `n <= 1` | **0** | return 0 without a softmax |
| `temp_scale <= 0.0` | **0** | `argmax()` — first strict maximum wins |
| `0 < temp_scale < 0.1` | **1** | `eps` is 0, so `eps > 0.0 && self.unit() < eps` **short-circuits and never draws**; one `unit()` for `r` |
| `temp_scale >= 0.1` | **exactly 2** | one `unit()` for the eps probe, then either `next_u64() % n` (probe hit) or a second `unit()` for `r` |

The cumulative array is **unnormalised** (`acc` is the running sum of `exp((w-max)/t)`), the draw is
`r = unit() * acc`, and the selection is `cum.partition_point(|c| c < r).min(n-1)` — i.e. the first
index whose cumulative sum is `>= r`, clamped. The trailing `.min(n-1)` is load-bearing.

### `softmax_pick(w, t_base)` — used only by WIDE `ActivatePlayer`

| condition | draws |
|---|---|
| `n == 0` or `n == 1` | **0** |
| `temp_scale <= 0.0` | **0** (argmax) |
| otherwise | **exactly 1** |

Here the distribution **is** normalised, there is no eps escape, and the fall-through default when
the cumulative walk does not trigger is `pick = n - 1`.

### Per decision

| decision | draws |
|---|---|
| WIDE `ActivatePlayer` | 0 or 2 — two `softmax_pick` calls, group at `T=0.18` then child at `T=0.10` |
| WIDE `Move` | **0** — pure plan replay; a broken plan re-plans with `best_move`, which is argmax and also draws nothing |
| DEEP `ActivatePlayer` | two `pick` calls, `T=0.18` then `T=0.14` → 0/2/4 |
| DEEP `Move` | one `pick` per square, `T=0.12` |
| every other scored prompt | one `sample(T)` → one `pick(T)` |

`record_distribution` / `record_probs` (`FFB_HEUR_DUMP`) consume **no** RNG — verified. The parity
runner must nonetheless leave `FFB_HEUR_DUMP` unset.

---

## 3. Temperatures, by call site

There is no `TempTable` type; the temperature is a literal at each call site. `docs/HEURISTIC_AGENT.md`
§8 describes an intended table — **the code is authoritative, not §8**.

| prompt / stage | `t_base` |
|---|---|
| WIDE `ActivatePlayer`, group | 0.18 |
| WIDE `ActivatePlayer`, child | 0.10 |
| DEEP `ActivatePlayer`, player | 0.18 |
| DEEP `ActivatePlayer`, action+target | 0.14 |
| DEEP `Move`, per square | 0.12 |
| `BlitzTarget` | 0.15 |
| `BlockChoice` | 0.12 |
| `Pushback` | 0.15 |
| `FollowUp` | 0.30 |
| `ReRollOffer` | 0.20 |
| `SkillUse` | 0.20 |
| `Interception` | 0.20 |
| `KickBall` | 0.10 |
| `Touchback` | 0.20 |
| `CoinChoice` | 1.00 |
| `ReceiveChoice` | 0.30 |

Effective temperature is `t = max(t_base * temp_scale, 1e-6)`.

---

## 4. Arithmetic

**All scoring is `f32`.** IEEE-754 requires `+ - * /` on `binary32` to be correctly rounded; Java 17+
is unconditionally strict-FP (JEP 306) and Rust does not contract to FMA. Given identical inputs the
two languages therefore produce identical bit patterns, and no fixed-point conversion is needed.

Rules that keep that true:

1. **`exp` and `ln` go through `det_math`, never the platform libm.** `agent/det_math.rs` /
   `DetMath.java` are built only from correctly-rounded f32 primitives and exponent surgery, with a
   fixed polynomial degree and a fixed evaluation order, so they are identical by construction.
   `crates/ffb-engine/src/agent/testdata/det_math_golden.txt` pins 348 `(input bits, output bits)`
   vectors; **both** sides assert on that file. Regenerating it is a policy change.
2. **No other transcendentals.** No `sqrt`, `powf`, `powi`, `log2`, `log10` anywhere in the agent.
   `ceil` is exact in both and is allowed.
3. **No out-of-range float-to-int casts.** Rust saturates, Java does not. The one conversion that
   could approach a boundary — the Dijkstra key increment — is written
   `(-ln_f32(p) * KEY_SCALE).clamp(0.0, 1.0e9) as i64 as u32` and pinned by a test over the worst
   case the search can reach.
4. **Maxima are explicit loops, not library calls.** Rust's `f32::max` returns the non-NaN operand
   while Java's `Math.max` propagates NaN. Weights are asserted finite; the Java fold must be a
   plain `>` loop.
5. **`partial_cmp(...).unwrap_or(Equal)`** maps to a comparator returning 0 for incomparable values.

---

## 5. Ordering — player ids must never enter one

Rust ids are `home_01..home_11` (a star carries its own, e.g. `morgNThorg`); Java's are
`teamLinemanParityHome1..11`. They sort differently, so **no ordering may be keyed on a player id**
(the same rule as `AGENT_CONTRACT.md` §6).

The canonical key is **`(side, jersey nr)`** — `canon_key` in Rust — which is the key both state-hash
implementations already use (`state_hash.rs::collect_player_parts` sorts by `p.nr`;
`ParityRunner.addPlayersFromTeam` uses `Comparator.comparingInt(Player::getNr)`). `side` is 0 for
home, 1 for away.

Sites governed by this rule:

| site | ordering |
|---|---|
| WIDE `ActivatePlayer` tier-1 candidates | `canon_key`, then rank by `w_player * max(proxy, 0.05)` desc with `canon_key` as tie-break |
| DEEP `ActivatePlayer` players | `canon_key` |
| `build_plans` blitz foes | `canon_key` |
| `build_plans` hand-off team-mates | `canon_key` |
| `BlitzTarget` eligible list | `canon_key` |
| `Touchback` eligible list | `canon_key` |
| `Pushback` squares | `(x, y)` |
| reach output `order` | ascending flat cell index (`sort_unstable`) |
| `top_moves` | weight desc, ascending flat index as tie-break |

All six player-ordering sites are **single-sided** (acting team, opponents, or team-mates only), so
within a side `(side, nr)` reproduces lexicographic `home_NN` order exactly — which is why adopting
it was a byte-exact no-op on every non-star roster.

`block_memo`'s key is `canon_pack = (side << 16) | nr`, not a hash of the id.

### Container iteration

`field_model.player_coordinates` is a `HashMap` with a randomly-seeded hasher. Iterate it directly
**only** where the accumulation is commutative. Audited:

- `positions_stamp` — safe (`wrapping_add`; and it is only a cache key, so its value need not even
  agree between the two engines).
- `Features::build` — safe (one distinct square per player, integer increments, and a constant-addend
  f32 sum whose result depends only on the count).
- `build_support` — safe (integer counters, and `max`).
- **`build_threat` — NOT safe.** It writes `threat_str` under a strict `>` against `threat_reach`, so
  equal-reach opponents tie and the first writer wins. Must iterate `canon_players`. Before this was
  fixed, all 20 human seeds produced different event streams between two runs of the same binary.

`seen_action`, `seen_bucket`, `used_this_turn`, `block_memo` are keyed lookups only and may be maps.

---

## 6. Frozen constants

No policy value may come from the environment — it cannot be mirrored.

| constant | value |
|---|---|
| `EPS` | 0.02 |
| `HOPELESS_DAMP` | 0.25 (was `FFB_HOPELESS_DAMP`; frozen ITER1) |
| `STAND_UP_COST` | 3 |
| `TIER2` (tier-2 candidate cap) | 16 |
| `THROW_SPOTS` | 6 |
| `GIVE_SPOTS` | 2 |
| `KEY_SCALE` | 4096.0 |
| board | `W=26, H=15, XMAX=25, YMAX=14, CELLS=390` |

`FFB_HEUR_DUMP`, `FFB_HEUR_TIME`, `FFB_SEQ`, `FFB_GIVE_TRACE`, `FFB_BALLMOVE` and the other trace
vars are diagnostics only and must not alter a decision.

---

## 7. Agent instances

For the **parity** arm, one shared agent drives both coaches, seeded `seed ^ 0x4845_5552_4953_5449`
— mirroring `RandomAgent::new_parity(seed)` and `ParityRunner`'s single-object shape.

This differs deliberately from the **experiment** arm (`run_heuristic_game`), which builds two
agents, home `seed` and away `seed ^ 0xFFFF_FFFF`, so the two sides can play different modes
head-to-head. Both configurations are legitimate; they are not interchangeable, and a parity run
must use the shared one.

Likewise the parity arm constructs the game with
`GameState::new_with_options(..., BASELINE_SETUP_OPTIONS)` — matching Java's `HeadlessGameSetup` —
**not** the experiment arm's `new_full_pregame` + `INDUCEMENTS=true`, which no Java path mirrors.

---

## 8. Modes

`Mode::{Wide, WideNoBall, WideNoPass, WideNoHandOff, Deep}`, selected by `--mode`
(`wide|wide-noball|wide-nopass|wide-nohandoff|deep`; any unrecognised string is `Wide`).

The three `No*` variants are A/B controls, implemented as two guards in `build_plans` — `WideNoBall`
suppresses both `HandOff` and `Pass`, `WideNoPass` only `Pass`, `WideNoHandOff` only `HandOff`.
Everything else about them is `Wide`.

---

## 9. Scope for the lineman tier

The lineman fixture is skill-less, star-less and has nothing throwable, so these are **structurally
unreachable** and must be stubbed to throw loudly (`UNPORTED_PROMPT`) rather than answered:
`SkillUse` (measured: 0 over 100 seeds), `PuntTarget` (measured: `Punt` never declared),
`ThrowTeamMateTarget`, `SwoopTarget`, every star special, `MultiBlockTargets`, `BloodlustAction`,
bombs, and the whole inducement family.

The reachable `UniformAgent` fallback surface is six branches: `ApothecaryChoice`, `UseApothecary`,
`PlayerChoice` (`reason: "pileDriver"`), `KickoffEventPlacement`, `KickoffReturn`, and the
`Acknowledge`/`EndTurn` catch-all.

`ArgueTheCall` needs a **scored always-argue arm** rather than the fallback's coin flip:
`AGENT_CONTRACT.md` §7 records that `ParityRunner` cannot decline cleanly, because clearing that
dialog loops the server on Sent-Off ejections.

## 10. The turn loop — harness rules both agents inherit

The heuristic replaced `RandomAgent`'s pick loop wholesale, but the loop it sits in is
`ParityRunner`'s, and three of that loop's rules are part of the contract. Each was rediscovered the
hard way during the amazon campaign (`docs/PARITY_AMAZON_CAMPAIGN.md`, ITER19–29).

**Phase 1 / phase 2.** At `INIT_SELECTING` with no acting player (phase 1) the harness picks a
player and injects `CLIENT_ACTING_PLAYER`. On the NEXT iteration, with the acting player set
(phase 2):

```java
if (tier <= 2 || game.getTurnMode() != TurnMode.REGULAR) {
    justDeselected = true;
    inject(new ClientCommandActingPlayer(null, null, false));   // deselect, no move
} else { sendConcreteAction(game, gameState); }
```

Rust's prompt model merges the two Java commands into `ActivatePlayer` + `Move`, so phase 2 is
answered at the **Move prompt**: in any non-REGULAR turn mode (`PassBlock`, `KickoffReturn`,
kickoff `Blitz`, …) the agent returns `EndPlayerAction` and sets `just_deselected`. The window's
one activation is recorded and never moves. It is `!= Regular`, not `== PassBlock`.

**The latch.** `justDeselected` ends the next phase-1 visit's turn (`if (remaining.isEmpty() ||
justDeselected) { justDeselected = false; usedThisTurn.clear(); EndTurn }`) — so the original
team's turn ends right after its window closes. Both agents carry the flag with the same name.

**One activation per window.** `if (mode != REGULAR && !usedThisTurn.isEmpty()) { justDeselected =
true; EndTurn }` — checked BEFORE the pick. Together with phase 2 this is what closes a window:
pick → deselect → EndTurn.

**Live, filtered eligibility.** The heuristic branch recomputes `computeEligiblePlayers(game)` at
every activation (it does NOT reuse the random path's turn-start `eligibleThisTurn`) and runs
`filterStaleActions` over the result (`agent::filter_stale_actions` in Rust). Both halves, or the
positional action pick reads a different list.

**`PlayerChoice` and the long tail** are delegated to the embedded `RandomAgent`, whose arms are
coordinate-sorted or `actionRng`-picked to match `ParityRunner`'s handlers; the heuristic must not
grow its own arm for a prompt the harness answers by contract.
