# Parity Agent Contract (T3 Phase 2)

Single source of truth for the deterministic random agents on both sides:

- Rust: `crates/ffb-engine/src/agent/random_agent.rs` (`RandomAgent::act`, `compute_follow_up`)
- Java: `ffb-ai/src/main/java/com/fumbbl/ffb/ai/parity/ParityRunner.java`

Any change to either implementation MUST be reflected here and mirrored on the
other side in the same change set. The contract governs *which* RNG stream is
consumed *when*; the game dice (GameRng / Fortuna) are governed by engine rules,
not by this document.

## 1. RNG channels

| Channel | Algorithm | Seed | Pick formula |
|---|---|---|---|
| decisionRng | Xoshiro256** | `seed ^ 0xDEAD_BEEF_CAFE_0001` | `next_u64() % n` (Java: `Long.remainderUnsigned(nextLong(), n)`) |
| actionRng | Xoshiro256** | `seed ^ 0x00C0_FFEE_ACE0_0001` | same |

Both sides construct the agent once per game (`RandomAgent::new_parity(seed)` /
ParityRunner constructor).

## 2. decisionRng consumption order

1. **CoinChoice** — 1 call (`% 2 == 0` → heads).
2. **ReceiveChoice** — 1 call (`% 2 == 0` → receive).
3. **KickBall** — 2 calls per kickoff: `x_raw = % 13`, `y_raw = % 13`;
   kick coord = `(x_raw + 13, y_raw + 1)` when home kicks, `(x_raw, y_raw + 1)`
   when away kicks.
4. **Player pick at activation** — 1 call per pick over the remaining-eligible
   list (see §4). Picks that land on an inactive (just-unstunned) player are
   *rejected but still consume the call*; the agent re-picks in a loop
   (mirrors Java server SKIP_STEP).
5. **Blitz block re-offer** — when the engine re-offers the single blitzing
   player for the block step (eligible list = exactly that already-used player),
   1 call (`pick(1)`).
6. **Pass fallback coordinate** — only when the passer has no teammate on the
   pitch: 2 calls (`x = % 26`, `y = % 14 + 1`). Quirk: uses decisionRng, not
   actionRng. (Normal pass target selection is §3.)

## 3. actionRng consumption

Exactly **1 call per concrete target pick** in the follow-up computation
(move square, block target, blitz target, foul target, pass teammate,
hand-off receiver). **0 calls** when the target list is empty — the agent
returns EndTurn/deselect without drawing.

Stage B adds one more actionRng call per activation: the **action pick** (§5).

## 4. Eligibility algorithm

- Turn key = `(half, turn_nr, side_playing)`. On a new turn key: clear
  `used_this_turn`, snapshot the engine's eligible list (roster order, NOT
  sorted) as `eligible_this_turn`.
- Remaining = `eligible_this_turn` minus `used_this_turn` (Phase 2 additionally
  filters players whose action list is empty).
- Pick uniformly over remaining with decisionRng (§2.4); mark picked player
  used immediately (even if subsequently rejected as inactive).
- EndTurn when remaining is empty, when the turn mode is not Regular
  (kickoff Blitz!/QuickSnap: activate-then-deselect only), or when a turnover
  occurred this turn (`turnover_this_turn`).

## 5. Action choice per activated player

- **Stage A (current):** always `actions[0]` of the player's action list —
  Move for standing players, StandUp for prone players.
- **Stage B (milestone):** 1 actionRng call: `i = pick_action(actions.len())`,
  take `actions[i]`. Enables Block/Blitz/Pass/HandOver/Foul organically.

## 6. Target sorting (CHANGED from player-id sort)

All candidate target lists are sorted by **target coordinate `(x, y)`**
ascending before the pick — never by player id. (Rust ids `home_01..home_11`
and Java ids `teamLinemanParityHome1..11` sort differently as strings; ids must
never enter an ordering.)

- Move / StandUp targets: legal one-step squares, sort by `(x, y)`.
- Block / Blitz / Foul / Stab targets: opposing players, sort by the target
  player's coordinate `(x, y)`.
- Pass teammates: teammate coordinates on pitch, sort by `(x, y)`.
- HandOver receivers: adjacent teammates, sort by the receiver's coordinate
  `(x, y)`.

## 7. Deterministic dialog/prompt responses (0 dice unless stated)

| Prompt | Response |
|---|---|
| Team reroll offer | decline |
| Skill reroll offer (Dodge/SureFeet/etc.) | engine-internal, not an agent choice |
| Apothecary | decline |
| Follow-up after push | decline |
| Pushback square | the min-`(x, y)` on-pitch square |
| Block die choice | index 0 |
| Interception / pass interference | decline (no voluntary interference) |
| ArgueTheCall | ALWAYS argue (1 game d6: >5 keeps the player, <2 also bans the coach; the team turn ends either way). Java's runner cannot decline cleanly (dialog-clear loops the server on SW ejections), so both sides argue. |
| Touchback | give to the eligible player nearest the pitch center `(13, 8)` by squared distance (first on ties, in the engine's eligible order) |
| SkillUse / PilingOn | always use the skill |

## 8. Per-action RNG sequence tables

Filled in from instrumented traces (`FFB_TRACE=1` / `-Dffb.diceTrace=true`)
as Stage A/B debugging establishes them. Format: ordered list of
(channel, count, purpose).

### Move (standing)
| # | Channel | Purpose |
|---|---|---|
| 1 | decisionRng | player pick (§2.4) |
| 2 | actionRng | move-square pick (§3) |
| — | game dice | dodge/GFI/pickup per engine rules |

### StandUp (prone)
As Move; standing up may consume game dice (MA < 3 stand-up roll) per engine
rules.

### Block (trace-verified, seed 1)
| # | Channel | Purpose |
|---|---|---|
| 1 | decisionRng | player pick |
| 2 | actionRng | action pick |
| 3 | actionRng | block-target pick |
| — | game dice | n block dice; multi-die → BlockChoice index 0 (either chooser); declined reroll = 0 dice; knockdown armor 2d6 (+injury 2d6 when broken) per fallen player, attacker first |

### Blitz (trace-verified, seed 1)
| # | Channel | Purpose |
|---|---|---|
| 1 | decisionRng | player pick |
| 2 | actionRng | action pick |
| 3 | actionRng | blitz block-target pick (Rust: compute_follow_up Block branch; Java: SELECT_BLITZ_TARGET) |
| — | game dice | as Block. The agent's blitz blocks immediately (no pre-block move). Attacker down = turnover. |

### Foul (trace-verified, seed 1)
| # | Channel | Purpose |
|---|---|---|
| 1 | decisionRng | player pick |
| 2 | actionRng | action pick |
| 3 | actionRng | foul-target pick |
| — | game dice | armor 2d6 (+assists/DP mods); injury 2d6 when broken (2-7 = Stunned); referee spots on armor doubles or (broken) injury doubles → argue 1d6 (§7) → turnover |

### Pass / HandOver
Not yet exercised by a passing seed: 1 decisionRng (player) + 1 actionRng
(action pick) + 1 actionRng (teammate/receiver pick, coordinate-sorted);
pass fallback coordinate per §2.6. Game dice TBD on first occurrence.

## 9. Known engine-rule divergences being tracked (not agent contract)

- Rust dodge adds a DisturbingPresence modifier; Java's DodgeModifierFactory
  has `isAffectedByDisturbingPresence = false`. Inert for linemen. (task #12)
- Rust sets `blitz_used` on a plain Block (engine/mod.rs ~1031); Java only on
  Blitz. Both eligible computations currently mirror the Rust behavior —
  consistent-but-wrong; verify against `TurnData.setBlitzUsed` call sites in
  Stage B.
