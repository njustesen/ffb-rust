# Lizardman — heuristic-agent parity campaign

**Goal**: lizardman v lizardman, HeuristicAgent both sides, per-step state-hash parity 100/100,
seeds 1-100, tier 3, editions bb2016/bb2020/bb2025 × scales 1.0/0/1e6 (nine gates), plus random
controls and the closed-roster regression set. Started 2026-09-05, after khemri_fumbbl
(`ff1a6ee00`).

## Surface

The richest roster since human, and the first in several races that is NOT structurally
pre-hardened:

- **Kroxigor**: Bone Head (a Select-sequence negatrait), Loner 4, Mighty Blow, Prehensile Tail,
  Thick Skull.
- **Saurus**: bb2025 adds **Juggernaut + Unsteady** (Unsteady gates SecureTheBall via
  `preventSecureTheBallAction`); plain in bb2016/bb2020.
- **Skink / Chameleon Skink**: Dodge, Stunty, plus On the Ball and Shadowing.
- **Star player Boa Kon'ssstriktr** at nr 2 — Dodge, Fend, Hypnotic Gaze, Loner 4, Prehensile Tail,
  **Safe Pair of Hands**, Sidestep, Look Into My Eyes. This is the one that mattered.

## Baseline (2026-09-05, measured on `ff1a6ee00`)

| edition | @1.0 | @0 | @1e6 |
|---|---|---|---|
| bb2016 | **100** | **100** | **100** |
| bb2020 | **100** | **100** | **100** |
| bb2025 | 99 (seed 59) | **100** | 97 (seeds 18, 41, 81) |

Seven gates green, four reds — **all in bb2025, and @0 green**. Argmax spends no sampler draws, so
an argmax-green/sampled-red split is a draw-count divergence, not a resolution one.

## ITER1 — one fix closed all four reds

`first_state_divergence.sh` put seed 59's real split at **i=100**, not the reported step 100: the
same player `away_07` declared **Block** in Rust where Java declared **BLITZ_MOVE**. `FFB_CANDSUM`
then showed the candidate sets were *identical* (k=115, n=1458 both sides) while the **draw totals
differed** — Rust 315, Java 317. `FFB_DRAWS` named the missing pair:

```
JDRAW cls=SKILL_USE total=315 skill=SafePairOfHands pid=teamLizardmanParity25Home2
```

Java raised a Safe Pair of Hands dialog that Rust never raised. `Home2` is a **positional index**,
not a nr — it is the star **Boa**, who carries Safe Pair of Hands. He had fallen at (17,3) holding
the ball.

**Root cause: an edition-twin conflation.** Java has three `StepFallDown` twins and only bb2025 opts
in:

```java
bb2016: dropPlayer(this, player, ApothecaryMode.ATTACKER)          // 3-arg -> false
bb2020: dropPlayer(this, player, ApothecaryMode.ATTACKER)          // 3-arg -> false
bb2025: dropPlayer(this, player, ApothecaryMode.ATTACKER, true)    // 4-arg -> TRUE
```

`bb2025/move_/step_fall_down.rs` passed `false`, and its comment asserted *"the THREE-arg overload,
so eligibleForSafePairOfHands is FALSE (this file used to pass true)"* — the file had been corrected
**toward its bb2020 twin**. With `false`, no `DROPPED_BALL_CARRIER` was published, so
`StepPlaceBall` bailed at its `playerId == null` guard and the dialog never appeared; Java spent two
sampler draws answering it and the streams split for the rest of the game.

**Fix**: pass `true`. Test `falling_carrier_is_eligible_for_safe_pair_of_hands`, verified failing
with `false` and passing with `true`.

Two of my own mid-investigation assumptions were wrong and are recorded so they are not repeated:
the silent `RPLACEBALL` probe looked like the dead-file trap but was an early return *above* the
probe; and I first compared against `bb2020/shared/StepPlaceBall.java` before noticing a **separate
bb2025 twin** exists. On this codebase, always confirm which twin is live before concluding.

## Gates after the fix (all nine re-measured)

| edition | @1.0 | @0 | @1e6 |
|---|---|---|---|
| bb2016 | **100** | **100** | **100** |
| bb2020 | **100** | **100** | **100** |
| bb2025 | **100** | **100** | **100** |

Random controls: bb2016 **100/100**, bb2020 **100/100**, bb2025 **100/100**.
`cargo test -p ffb-engine` **7419/0** (+1 new test).

Closed-roster regressions (bb2025 @1.0, seeds 1-100) — the fix is in a step every race runs:
khemri, human, high_elf, goblin, amazon, dark_elf, **all 100/100**.

Coverage harvested ×3. bb2025 skill uses: 123 Dodge, 69/2 Juggernaut.

**🏁 lizardman CLOSED.** Frontier empty.
