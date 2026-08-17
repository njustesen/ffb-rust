# Parity state-hash widening

The parity gate compares a per-step **state hash**. Anything that hash does not encode is a
divergence both engines can carry forever while every matrix reports 30/30. This log records each
field added to it and what that field caught.

## What the hash encoded before this campaign

```
half, turn_home, turn_away, active team, score,
ball (x, y, in_play),
and per player: x, y, one coarse state label
```

That is all. **Not** encoded: team re-rolls remaining, the `blitz_used` / `pass_used` /
`hand_over_used` / `foul_used` flags, acting-player state, MA spent, SPP, casualty records, weather,
inducements — and the three casualty states all collapsed to a single `"Injured"` label.

The two implementations must produce byte-identical strings:

| side | file |
|---|---|
| Rust | `crates/ffb-model/src/util/state_hash.rs` |
| Java | `ParityRunner.java` — `stateString`, `stateHash`, `playerStateStr` |

`ParityRunner.java` is harness and co-editable. Everything under `ffb-common` / `ffb-server` is Java
**engine** code and is not.

## Method

One field group per iteration, never several at once — otherwise a red cannot be attributed to the
field that caught it. Each iteration: change both harnesses, rebuild the fat jar
(`/c/Users/Admin/bin/maven/bin/mvn -q -o -pl ffb-ai -am package -DskipTests`), `cargo build
--release`, `cargo test --workspace`, then all three edition matrices at seeds 1-100. Reds are the
point, not a setback: each is a divergence that was already shipping green. Root-cause every red in
the **Rust** engine only.

## Iteration 1 — split the collapsed casualty label

`BADLY_HURT` / `SERIOUS_INJURY` / `RIP` all hashed as `"Injured"`, so the gate could not tell a dead
player from a bruised one. Now `Bh` / `Si` / `Rip`.

**Result: bb2016 `khemri` and `khemri_fumbbl` went red at 92/100** (seed 6, step 9). bb2020 and
bb2025 stayed 30/30.

The whole divergence was one player: `h02: JAVA Si / RUST Bh`.

### The bug it caught

Khemri mummies have **Decay**: BB2016 rolls the casualty twice and keeps the worse. Java rolled
d6=3 (Badly Hurt), then the decay d6=5 (Serious Injury), and applied the worse. Rust rolled both —
`ctx.casualty_roll_decay` and `ctx.injury_decay` were both computed correctly — and then **never
read the second one**:

```rust
pub fn get_player_state(&self) -> Option<PlayerState> { self.injury }   // primary roll only
```

Java's `InjuryContext.getPlayerState()` (`InjuryContext.java:218-228`) returns the worse of the two
by raw state id, and all four of its outcome helpers — `isCasualty`, `isKnockedOut`, `isReserve`,
`isSeriousInjury` — route through it. Rust's helpers each read the raw `injury` field instead, as
did the `applyTo` site that writes the state into the field model.

Fixed by mirroring `getPlayerState()` and routing every helper and the apply site through it.
`khemri` bb2016: **92 → 100/100**. Regression test:
`injury.rs::get_player_state_takes_the_worse_of_injury_and_decay`.

This bug was invisible to every previous 30/30 gate and would have stayed invisible: the two states
it confuses hashed identically. It was found on the widening's first run.

## Iteration 2 — per-team turn-used flags

`blitz_used` / `foul_used` / `hand_over_used` / `pass_used`, home then away, as `f0100,0011`. The
hash carried no turn state at all, so a team that wrongly consumed (or wrongly kept) a once-per-turn
action still hashed identically.

**Result: 30/30 on all three editions — no divergence found.** A negative result, and a useful one:
those four flags are already in sync everywhere the suite reaches. The field earns its place as a
*regression* guard rather than a bug-finder. Today's `blitz_used` bug (a select-phase Foul Appearance
failure skipped the dispatch that sets the flag, letting a team blitz twice) was caught only
indirectly, because the stale flag happened to change which actions the agent was offered next; with
this field it would have been caught at the step it occurred.

`ttm_used` / `ktm_used` are absent: Java's `TurnData` exposes no accessor and `TurnData.java` is
engine code, not the co-editable harness.

## Remaining groups

1. ~~Per-team turn-used flags~~ — done, iteration 2.
2. **Per-team re-rolls remaining** (`TurnData.getReRolls()`).
3. **Acting-player state** — who is activated, MA spent — if the first two land clean.
