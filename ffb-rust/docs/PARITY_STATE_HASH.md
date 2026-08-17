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

## Iteration 3 — per-team re-rolls remaining

`" r3,4"`, home then away. **Result: human bb2025 dropped to 7/10 immediately** (seeds 1, 6, 10),
diverging on the final `game_end` hash: Java `r3,3`, Rust `r3,4`.

### The bug it caught

The extra re-roll appears the moment `StepApplyKickoffResult` resolves **Brilliant Coaching**:

```rust
// step_apply_kickoff_result.rs::handle_brilliant_coaching
game.turn_data_away.rerolls += 1;      // straight into the permanent pool
```

Java keeps that re-roll in a **separate one-drive bucket**. `TurnData` has
`reRollsBrilliantCoachingOneDrive`; both `bb2020` and `bb2025` `RollMechanic` spend it *first* when
a re-roll is used, and `getReRolls()` does not include it. Rust's `TurnData` carries the mirrored
field `rerolls_brilliant_coaching_one_drive` — and never reads or writes it.

So in Rust a Brilliant Coaching re-roll **never expires at the end of the drive**: the team keeps it
for the rest of the half. In the failing seeds it survived to the final whistle.

This never showed up before for two reasons: re-roll counts were not in the hash at all, and the
parity agent declines team re-rolls by contract, so the surplus rarely changes a die.

### What it caught — four defects, three editions

**1. Brilliant Coaching re-rolls never expired (engine).** Java grants the re-roll into the main
pool AND records it in a one-drive counter, then `StepEndTurn.removeReRollsLastingForDrive` zeroes
the counters and subtracts their sum back out at end of drive. Rust granted only into the pool and
left the mirrored `rerolls_brilliant_coaching_one_drive` permanently 0, so the team kept the re-roll
for the rest of the half. Fixed in `step_apply_kickoff_result.rs` (grant) and `step_end_turn.rs`
(new `remove_rerolls_lasting_for_drive`, both teams, Java's `!new_half || half > 1` guard).

**2. `rollExtraReRoll` is a d3, not a d6 (engine) — the big one.** `DiceRoller.java:193-195` defines
it as `rollDice(3)`, and bb2016 uses it for BOTH the Cheering Fans and Brilliant Coaching contests.
BB2025 is the edition that rolls a d6. Rust rolled a d6 in both.

The draw COUNT was right, so the shared dice stream never desynced — the only observable was which
team won the contest, a re-roll count nothing compared. **This shipped green through every 30/30
gate in the project's history.** It turned all 30 bb2016 rosters red at once (lineman seed 6: Java
d3 3/3 → tie, both gain; Rust d6 6/3 → home only). bb2016 went 0/30 → 29/30 on the fix alone.

**3. The new one-drive removal must not run for BB2016 (engine, self-inflicted).** Rust routes every
edition's `EndTurn` to the shared BB2025 step, and `bb2016/StepEndTurn.java` has no one-drive
handling *at all* — grep it for `OneDrive` or `setReRolls` and there is nothing. Since the mixed
`handle_pump_up` that BB2016 uses does fill the pump-up counter, the new code would have taken back
a re-roll Java keeps. Gated to `rules != Bb2016`. Caught by reading the Java twin, not by a test.

**4. The lineman parity team's fan factor (harness data).** `team_lineman_parity_{home,away}.xml`
carry `<fanFactor>5`; `make_lineman_team` in `ffb-parity` hard-coded 0. Fan factor feeds the
spectator count, which sets FAME, which decides the extra-re-roll contest — bb2016 lineman seed 65:
Java spectators 8000/11000 → `fameA=1` → a 3-3 tie where both teams gain; Rust 3000/6000 →
`fameA=2` → away only. The dice were byte-identical (`pos=44 sides=3 result=3`,
`pos=45 sides=3 result=2`); only the bonus differed. Fixed in the harness, not the engine.

### The pattern worth remembering

Three of the four trace back to one question: **which edition's file actually runs?** The d3 bug,
the BB2016 gating slip, and (in the coverage work the same day) the BB2016 event blackout and the
MVP/winnings/deviate blind spots are all the same shape — a twin exists per edition, the driver
routes to one of them, and the others drift. Ask that question first whenever an edition-specific
number looks wrong.

### Gate

bb2016 30/30, bb2020 30/30, bb2025 30/30, seeds 1-100, on the rebuilt jar.

## Iteration 4 — acting-player state

`" ap h03,2"` — WHO is activated and how much movement it has spent; `-` when nobody is activated.

The player is identified by its index in the same ordering the player parts use (sorted by squad
number, first 11 per team), because the raw ids differ between the engines
(`teamKhemriParity16Away10` vs `away_10`) and hashing those directly would diverge on every step
for reasons that have nothing to do with the engine. If the acting player is somehow not among the
first 11 of either squad, both sides emit `?,<move>` rather than `-`: reporting "nobody is acting"
for a player that IS acting would be exactly the kind of silent lie this campaign exists to remove.

Java's `ActingPlayer` already exposes `getPlayerId()` and `getCurrentMove()`, so this needed no
Java engine change — only the `ParityRunner` helper.

**Result: 30/30 on all three editions — no divergence found.** Like iteration 2, it lands as a
regression guard: the hash previously carried no activation state at all, so an engine could hold a
different player active, or a different MA spend, with nothing compared moving.

## Iteration 5 — effective player stats

Each player part gains `,MA/ST/AG/AV` **with modifiers applied**:
`h00:12,7,Standing,6/3/3/8`. Chosen because temporary stat modifiers feed armour and injury rolls
and have a bug history here (the Dodgy Snack `-MA/-AV` enhancement outliving its drive, elf seed 38:
effective AV 6 vs Java's 7).

**Result: bb2016 30/30, bb2025 30/30, bb2020 29/30** — `halfling` 97/100, first divergence seed 9
step 131.

### The bug it caught — missing stat-limit clamp

`h00` (the Treeman, MA2/ST6) ends the game with **AV 11 in Java, 12 in Rust**. Base AV 11 plus a
`+1` from the Iron Man prayer (`inducements/mixed/prayers/prayer_player_effect.rs:24`).

Java clamps: `Player.getStatWithModifiers` reads a `PlayerStatLimit` off the temporary modifier and
applies `min(max, sum)` / `max(min, sum)`. Rust's `*_with_modifiers` just sums the deltas.

The limits are edition-specific (`mechanics/{bb2016,mixed}/StatsMechanic.limit`):

| stat | bb2016 | bb2020 / bb2025 |
|---|---|---|
| MA | none | 1–9 |
| ST | none | 1–8 |
| AG | none | 1–6 |
| PA | none | 1–6 |
| AV | 1–10 | 3–11 |

Two details that matter for the port:

* the clamp applies **only when a temporary modifier for that stat is present** — Java takes the
  limit from the modifier stream, so a base stat above the cap is never clamped;
* the limits differ per edition, so applying one set universally would mis-clamp the other (a BB2016
  MA-1 player with Greasy Cleats should reach 0; the mixed limit would floor it at 1).

Fixed by carrying the limit on the modifier the way Java does: `temporary_stat_mods` becomes
`(source, stat, delta, limit_min, limit_max)`, `stat_limit(rules, stat)` reproduces
`StatsMechanic.limit`, and `add_temporary_stat_mod_limited` is used by the two production call sites
(the prayers, and the Dodgy Snack grant). Only two of the eleven `add_temporary_stat_mod` uses were
production code — the other nine were inside test modules, checked individually rather than assumed.

### A correction worth recording

The first version of this fix read BB2016's limit table off one line and wrote `AV => (1, 10),
_ => (0, 0)`. Java's switch **falls through** `MA`, `ST`, `AG` and `AV` to a single
`PlayerStatLimit(1, 10)`, leaving only PA unbounded (`bb2016/StatsMechanic.java:44-49`).

Worse, the regression test asserted the same misreading — *"BB2016 bounds only AV, so MA may reach
0"* — so it would have locked the bug in rather than caught it. A test derived from the same wrong
reading as the code is no test at all. It now pins the whole table for both editions explicitly,
against the Java source rather than against one inferred case.

### Gate

bb2016 30/30, bb2020 30/30, bb2025 30/30, seeds 1-100.

### Two phantom reds — a harness hazard, not engine behaviour

This iteration produced two reds that did not reproduce: `bb2025 lineman` (99/100) and
`bb2016 dark_elf_league_fumbbl` (33/100). Both came from running a parity command for a matchup
while a matrix gate was running that same matchup — both engines' logs share
`parity/<home>_vs_<away>/`, and `run_cross_matrix.py` documents that same-matchup runs are not
concurrency-safe. Re-run in isolation, both were 100/100.

**Never believe a red without reproducing it in isolation**, and never touch a matchup while a gate
is live.

## Iteration 6 — weather

`" wNice Weather"`. Nothing compared the weather, yet it gates real rules: Sweltering Heat faints
players at drive end, and a Blizzard puts Long Pass and Long Bomb out of range — a bug actually
fixed earlier in this project's history.

Both engines already spell the weather identically (`Weather.getName()` / `Weather::name()`), so
there is no hand-written enum mapping that could drift.

**Result: 30/30 on all three editions — no divergence found.** A regression guard.

### Two candidates rejected after checking

* **Ball carrier** — fully implied by the hashed player positions plus the ball coordinate. Adding
  it would grow the string and verify nothing new.
* **KO-box contents** — largely implied now that player states distinguish `Ko` / `Bh` / `Si` /
  `Rip` and off-pitch players report `-1,-1`.

The goal is coverage of *distinct* state, not field count.

## Iteration 7 — turn mode

`" tmregular"`. The hash recorded WHOSE turn it was but never WHICH KIND: a Blitz!, a Quick Snap, a
Pass Block window and a regular turn all hashed alike, though they route through different step
sequences and permit different actions.

Both engines already emit the same camelCase names across all 32 constants
(`TurnMode::name()` / `TurnMode.getName()`), so no hand-written mapping can drift.

**Result: 30/30 on all three editions — no divergence found.** A regression guard.

## Where the hash stands

| | before this campaign | now |
|---|---|---|
| half / turn / active team / score | yes | yes |
| ball x,y,in-play | yes | yes |
| per player: x, y, state | one coarse label — `Ko`, and `Injured` for all three casualty states | `Ko` / `Bh` / `Si` / `Rip` |
| per player: MA/ST/AG/AV with modifiers | — | yes |
| per-team turn-used flags | — | blitz / foul / hand-over / pass |
| per-team re-rolls remaining | — | yes |
| acting player + MA spent | — | yes |
| weather | — | yes |
| turn mode | — | yes |

## Diminishing returns

The genuinely unhashed state is close to exhausted. What is left is implied by fields already
compared (ball carrier, KO-box contents), absent from one engine, or client-only. Further fields
would grow the string without verifying anything new.

Seven field groups produced **seven defects**, six in the engine and one in harness data:

| field group | defects |
|---|---|
| casualty-state split | Decay took the first casualty roll, not the worse of two |
| turn-used flags | none — regression guard |
| re-rolls remaining | Brilliant Coaching never expired; `rollExtraReRoll` d6 vs Java's d3; my own one-drive removal wrongly applied to BB2016; lineman `fan_factor` 0 vs Java's 5 |
| acting-player state | none — regression guard |
| effective player stats | modified stats never clamped to their limits |
| weather | none — regression guard |
| turn mode | none — regression guard |

The `rollExtraReRoll` d6/d3 bug is the one to remember: it consumed the same number of dice as the
correct roll, so the shared stream never desynced and it survived every green gate in the project's
history. Its only footprint was a re-roll count nothing compared.

## Remaining groups

1. ~~Per-team turn-used flags~~ — done, iteration 2.
2. ~~Per-team re-rolls remaining~~ — done, iteration 3; four defects, listed above.
3. ~~Acting-player state~~ — done, iteration 4.
4. Candidates not yet taken: ball carrier id, weather, turn mode, temporary player stat modifiers
   (the Dodgy Snack -MA/-AV class), KO-box contents.
