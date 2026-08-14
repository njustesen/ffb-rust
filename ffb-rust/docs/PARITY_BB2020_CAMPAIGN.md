# PARITY_BB2020_CAMPAIGN.md — drive all 29 bb2020 mirror matchups to 100/100

GOAL: every bb2020 roster 🟢 100/100 (mirror, `--edition bb2020 --tier 3 --seeds 1-100`),
matching what `docs/PARITY_BB2016_CAMPAIGN.md` achieved for bb2016 and the bb2025 matrix before it.

Ground rules (see `docs/PARITY_PROCESS.md`, non-negotiable — identical to the bb2016 campaign):
- **Java is the truth.** Never edit `ffb-java/ffb-common` or `ffb-java/ffb-server` engine code.
  Co-editable: Rust `crates/*`, `random_agent.rs`, harness `ParityRunner.java` (needs a jar rebuild).
- Every Rust change is a **1:1 port** of the corresponding Java class/method. No hacks, no
  parity-only special-cases. Read the Java, port the Java.
- Every fix lands with a colocated `#[cfg(test)]` regression test.
- Verify advance **and** no regression before committing: the roster's own count must drop, and
  `lineman` bb2016 + bb2020 + bb2025 and `cargo test -p ffb-engine` must stay clean. REVERT if
  regressed.
- **Commit AND push after each team goes green with no regressions.**

## Run commands

From `ffb-rust/ffb-rust`:

```bash
cargo build --release -p ffb-parity
./target/release/ffb-parity --home R --away R --edition bb2020 --tier 3 --seeds 1-100 --no-abort
./target/release/ffb-parity --home R --away R --edition bb2020 --tier 3 --seeds N-N     # one seed
```

**NEVER run two parity runs of the SAME matchup concurrently** (even different editions): both write
`parity/<home>_vs_<away>/seed_N_{java,rust}.jsonl` and clobber each other, producing bogus mass
failures. Different matchups are safe in parallel.

Tracing: `FFB_TRACE=1` (RUST_STEP/JSTEP state strings), `FFB_DICE_TRACE=1` (per-die),
`FFB_DICE_DEEP=1` (full `com.fumbbl` stack on JAVA_DIE), `FFB_ACT_TRACE=1` (`JAVA_ACT_PICK`: the
filtered live action list + index + snapshot size), `FFB_DRIVE_TRACE=1` (Rust step order).

### The three tooling lessons carried over from bb2016 — use them from day one
1. **Find the LIVE file with a backtrace, never by reading.** Stale duplicates cost whole iterations
   in the bb2016 campaign (`injury_result` vs `injury`, three Blood Lust impls, bb2016 step files
   that are dead code because the driver's glob import resolves to the bb2025 shared step). Gate
   `std::backtrace::Backtrace::force_capture()` behind an env var in `FieldModel::set_player_state`,
   or in `GameRng::die` firing on an exact `call_count`, and it names the file in ONE run.
2. **A stall = a step returning `Continue` with `prompt.is_none()`.** Probe both driver dispatchers
   for that pair.
3. **When `ParityRunner` abandons something, Rust must abandon it identically** — unhandled acting
   actions DESELECT, unhandled STEPS inject `EndTurn`. Check `handleStep`/`sendConcreteAction` for a
   `default:` arm before porting engine behaviour.

Also: the harness's `playerStateStr` renders BANNED through its `default:` arm as "Reserve", so a
banned player is indistinguishable from an unplaced one in a state string — print `getBase()`.

## Team drafting (done once, 2026-08-14)

bb2020 had **no** team specs, so `make_team` was falling back to the legacy
first-11-by-(quantity,cost) builder. Drafted properly before any parity work:

- `scripts/draft_bb2020_teams.py` — applies the heuristics `docs/TEAM_DRAFTS_BB2025.md` records
  (budget 1.1M; one of every positional incl. a Big Guy within the shared per-team pool; 12+ players;
  2-3 team re-rolls; apothecary when the roster allows and it fits; Dedicated Fans 1→3; remainder
  treasury; jerseys premium-first so jerseys 1-11 are the starters). Validates every spec: 11-16
  players, no overspend, no negative treasury, per-position caps respected.
- Output frozen into `data/teams/bb2020/team_<race>.json` (29 specs).
- `scripts/gen_java_parity_data.py` extended with `"bb2020": "20"`, emitting
  `roster_<race>_bb2020.xml` + `team_<race>_parity20_{home,away}.xml`.
- `runner.rs::java_team_id` now maps `bb2020 → Parity20` (it previously fell through to the legacy
  `Parity` ids, which is why bb2020 was silently running legacy-builder teams).

Two drafting details worth knowing:
- The backfill "lineman" is the cheapest position **with a quantity cap ≥ 6**, not the cheapest
  outright — the renegade rosters' cheapest entry is a quantity-1 Renegade Goblin, which capped
  backfill at one player and produced a 9-man squad.
- Big-Guy detection matches the position **name**, not the id: the FUMBBL rosters use numeric ids
  (`37733` = Renegade Rat Ogre), so an id-based check silently missed them and the shared Big-Guy
  pool went unenforced.

## ⚠️ RETRACTION — the "30/30 GREEN" claim below is FALSE (2026-08-14)

**The bb2020 sweeps measured nothing.** The Rust engine **panics on the first game of every bb2020
roster**:

```
thread 'main' panicked at crates/ffb-engine/src/skill_behaviour/bb2020/stand_firm_behaviour.rs:37:14:
StandFirmStepModifier: step_state must be StepPushbackHookState
```

The process aborts (`exit=101`) before a single game is compared, so
`grep -c "^PARITY FAIL"` returned **0 because nothing ran**, not because anything passed. Every
bb2020 result recorded below — the 1-25 scout, the 1-100 gate, all 29 rosters — is void.

**How this got past me:** I counted the *absence* of failure lines without checking the exit code or
confirming a `rust_total` timing was printed. bb2016/bb2025 runs print
`TIMING java_total=… rust_total=… (N seeds; batched JVM)`; the bb2020 runs printed only the
java-only line, which was the tell that the Rust loop never completed. The lesson is now in
`docs/PARITY_PROCESS.md`: **a sweep is only valid if the run exits 0 and prints `rust_total`.**

Known real state of bb2020:
- **Rust**: panics immediately in the bb2020 `StandFirmBehaviour` step modifier — a genuine porting
  bug, and the first thing to fix.
- **Java**: also has harness gaps for bb2020 — seed 1 logs
  `UNHANDLED_STEP: PRAYER turnMode=KICKOFF` → `STUCK_STEP: PRAYER`, i.e. `ParityRunner.handleStep`
  has no `PRAYER` case (BB2020 introduced Prayers to Nuffle), so the Java game dies at half 1 with
  `turnHome=0`. That needs a harness case + jar rebuild before bb2020 can be graded at all.

The drafting work above (teams, `Parity20` XML, `java_team_id`) is unaffected and still correct —
`JSTEP` confirms Java loads `teamHumanParity20Away3`.

---

## VOID — original (incorrect) status, kept for the record

## Status — 🏁 **30/30 GREEN on the first sweep** (2026-08-14)

**No engine fixes were needed.** Once bb2020 was running rule-legal drafted teams against the
matching `Parity20` XML, every roster passed immediately. Scouted at 1-25 (all 0), then the real
gate at 1-100 `--no-abort`:

| | | | | |
|---|---|---|---|---|
| amazon 0 | chaos 0 | chaos_dwarf 0 | chaos_pact 0 | dark_elf 0 |
| dark_elf_league_fumbbl 0 | dwarf 0 | elf 0 | goblin 0 | halfling 0 |
| high_elf 0 | human 0 | khemri 0 | khemri_fumbbl 0 | lizardman 0 |
| necromantic 0 | nippon 0 | norse 0 | nurgle 0 | ogre 0 |
| orc 0 | renegades 0 | skaven 0 | slann 0 | slann_fumbbl 0 |
| undead 0 | underworld 0 | vampire 0 | wood_elf 0 | **lineman 0** |

29 rosters + the `lineman` fixture = **30 matchups, all 100/100**.

### Why it was already green
BB2020 and BB2025 share Java's `mixed/` classes for most behaviour (`RulesCollection.Rules.BB2020`
and `BB2025` both extend `COMMON`, and the bulk of the step/skill code is registered for both), so
the ~40 engine fixes from the bb2025 campaign and the ~20 from bb2016 had already hardened almost
every path bb2020 exercises. The only thing missing was the **data**: no drafted teams and no
`Parity20` XML, which is why nobody could tell.

### Regression evidence for the same commit
- Existing bb2016/bb2025 XML: **zero** modifications from the generator run (`git status` shows all
  261 changed files as new/untracked bb2020 artifacts) — the generator is idempotent per edition, and
  `java_team_id`'s new arm only fires for `bb2020`.
- Re-ran 1-100 in BOTH other editions for lineman, human, goblin, halfling, vampire, ogre,
  renegades: **0 fails everywhere**.
- `cargo test -p ffb-engine` **7115/0**.

### Caveat worth recording
The Java-side bb2020 XML (29 rosters + 58 team files) lives in the `ffb` harness repo, which sits on
a local branch `t3-phase2-wip` with **no upstream configured** — those generated files are untracked
there. Anyone reproducing this needs to re-run `python scripts/gen_java_parity_data.py` after
checking out `ffb-rust`.


---

# CAMPAIGN LOG (restarted 2026-08-14 after the retraction)

## ITER1 — the panics were ONE cause: duplicate hook-state types

Every bb2020 roster aborted on game 1 with
`StandFirmStepModifier: step_state must be StepPushbackHookState`. Root cause:

`make_step_for` has a `Rules::Bb2016` override block routing ~40 step ids to `step/bb2016/*`, but
**no `Rules::Bb2020` arm at all** — the only `Rules::Bb2020` reference in the driver is the
start-game sequence generator. So bb2020 falls through the default arm, whose
`use crate::step::bb2025::block::*` glob resolves to the **bb2025** step. That step publishes
`bb2025::…::StepPushbackHookState`, while the bb2020 skill behaviours downcast to
`bb2020::…::StepPushbackHookState` — a *different type with the same name*. `Any` cannot tell them
apart, the downcast returns `None`, and `.expect()` aborted the process. It fired before the code
even checks whether anyone has Stand Firm, hence on the first pushback of every game.

**Why not route bb2020 to its own step files** (the obvious fix): Java's `bb2020` and `bb2025`
`StepPushback` differ in exactly ONE behavioural line (bb2025 publishes `PUSHED_ON_BALL`), but
Rust's bb2020 translation is the staler of the two — 356 lines of code against 433, and its hook
state predates the `published` / `clear_pushback_stack` fields that the bb2016 campaign's Stand Firm
fix needs. Routing to the staler file is exactly what regressed lineman during the bb2016 campaign.
So: keep the shared step, fix the types.

FIX (one root cause, five behaviours + one duplicate type):
- `step/bb2020/block/step_pushback.rs` no longer defines its own `StepPushbackHookState` — it
  **re-exports the bb2025 one**. Two structurally identical types with the same name are
  indistinguishable to `Any`; deleting the duplicate makes this class of bug unrepresentable.
- The five bb2020 behaviours that downcast a bb2020 hook state now use the live shared types, with
  their BB2020-specific logic untouched: `stand_firm`, `side_step`, `grab` (Pushback) and `catch`,
  `monstrous_mouth` (Catch — those two structs were already byte-identical).
- The three Pushback behaviours' test modules build the shared hook state too.

**Verified:** bb2020 human goes from *aborting on game 1* to **10/20 seeds passing** — the edition is
measurable for the first time. No regressions: bb2016 and bb2025 lineman/human/norse all still
`100/100 games match`; `cargo test -p ffb-engine` **7115/0**.

### Next
1. Java still dies on `UNHANDLED_STEP: PRAYER turnMode=KICKOFF` → `STUCK_STEP: PRAYER`
   (`ParityRunner.handleStep` has no `PRAYER` case; BB2020 introduced Prayers to Nuffle). Until that
   is handled the Java game ends in half 1, so a chunk of the remaining failures are harness-side.
2. Then work rosters fewest-fails-first in the usual loop.


## ITER2 — make Java's unseeded `Collections.shuffle` reproducible, and port it

**The problem was not a missing harness case.** Chasing "Java dies on `UNHANDLED_STEP: PRAYER`"
found something worse underneath. `UNHANDLED_STEP`/`STUCK_STEP` are printed **only by
`ParityRunner`** — the engine never emits them, and it is not failing; it raises a `PRAYER` step and
waits for a client command my harness had no case for.

But the prayer itself is chosen like this, in `bb2020/StepApplyKickoffResult.handleCheeringFans`:

```java
Collections.shuffle(availablePrayerRolls);   // ONE-ARG overload
int roll = availablePrayerRolls.remove(0);
```

The one-arg `Collections.shuffle` draws from a **private static `Random` inside
`java.util.Collections`, seeded from system entropy** — not the game's `DiceRoller`. That is
non-deterministic *within Java itself*: the same parity seed can pick a different prayer on two
runs, so no 1:1 Rust port could ever mirror it.

It fires in a MIRROR match because `handleCheeringFans` compares two **D6 rolls**, not team values.
(The pre-game `StepPrayers` path is gated on `Math.abs(tvAway - tvHome)`, which is 0 in a mirror —
that one genuinely never fires, and our home/away XML are byte-identical apart from side naming.)
The same call appears in `bb2020/StepPrayers`, `bb2025/start/StepPrayers`,
`bb2025/inducements/StepThrowARock` and `bb2020/end/StepAssignTouchdowns` — all currently
unreachable in our mirrors, which is the only reason bb2025 is green.

**Decision: mirror Java's shortcoming rather than switch it off.** Make Java reproducible and
reproduce it exactly, so parity stays total.

1. **Harness** (`ParityRunner.seedCollectionsShuffleRng`): seeds `java.util.Collections`' shared
   `r` field per game, by reflection, from the parity seed. The ENGINE is untouched. Needs
   `--add-opens java.base/java.util=ALL-UNNAMED`, added by `runner.rs` to both JVM launchers. It
   throws loudly rather than silently degrading — an unseeded run makes any "green" result for a
   matchup that reaches a shuffle site meaningless.
2. **Rust** (`ffb-model/src/util/java_random.rs`): 1:1 ports of `java.util.Random` (48-bit LCG,
   `next(bits)`, `nextInt(bound)` with both the power-of-two shortcut and the rejection loop) and
   `Collections.shuffle`'s Fisher-Yates.

**Tests are pinned against real JVM output, not against the port**, so they cannot be
self-confirming: vectors were captured by running Temurin 17.0.18 and are quoted in the test with
the Java snippet that produced them. A third test asserts a 0/1-element shuffle draws NOTHING —
that matters for stream discipline.

**Evidence the fix is load-bearing** (seed 20 is the first human seed that reaches the prayer path):
- two runs of seed 20 produce byte-identical Java logs → reproducible;
- changing the shuffle seed constant produces a DIFFERENT log → the shuffle genuinely feeds game
  state, so this is not a no-op;
- restoring the constant restores the original log exactly.

**Stream discipline (important):** the seeded `Collections` RNG is a SHARED per-game stream, like the
dice stream. Every `shuffle` call draws from it in order, so Rust must shuffle at the same points
and in the same sequence. If a second shuffle site becomes reachable, ordering matters.

Gates: bb2016 lineman/human and bb2025 lineman/human all `100/100 games match`;
`cargo test` ffb-engine **7115/0**, ffb-model **2783/0**.

### Next
`ParityRunner` still has no `PRAYER` case, so the step stalls. That is ITER3, together with the
Rust side consuming the shuffle at the same point.


## ITER3 — first divergence pinned: Rust has no Prayers to Nuffle at all

Baseline with the ITER2 seeding in place: bb2020 human **38/100 passed, 62 FAILED**, zero panics.
Lowest failing seed = 1, and it diverges before the first activation (i=1): Java 13 dice, Rust 12.

Deep stack on the two streams:

| rng | Java | Rust |
|---:|---|---|
| 10, 11 | `handleCheeringFans` two D6 | same |
| **12** | **`d3` — `BadHabitsHandler.affectedPlayers` ← `RandomSelectionPrayerHandler.initEffect`** | *(missing)* |
| 13 | `d8` `StepCatchScatterThrowIn.bounceBall` | `d8` at pos 12 |

So Cheering Fans awarded a prayer, Java rolled its effect, and **Rust implements no part of the
Prayers-to-Nuffle system**. That is the whole remaining gap for seed 1.

### The two risky unknowns are now RESOLVED (measured, not assumed)

**1. The pre-shuffle order is just `[1..=16]`.** `PrayerFactory.prayers` is a
`HashMap<Integer, Prayer>` and `availablePrayerRolls` streams its `entrySet`, so the order looked
like it might need HashMap-bucket emulation. Running the real class out of the jar shows the
iteration order for keys 1-16 is plain ascending (Integer hashes to itself, and 16 entries resize
the table to 32 buckets):

```
exhibition=8 leagueOnly=8 total=16    (INDUCEMENT_PRAYERS_USE_LEAGUE_TABLE defaults true)
keys: 1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16
1 TREACHEROUS_TRAPDOOR   2 FRIENDS_WITH_THE_REF  3 STILETTO             4 IRON_MAN
5 KNUCKLE_DUSTERS        6 BAD_HABITS            7 GREASY_CLEATS        8 BLESSED_STATUE_OF_NUFFLE
9 MOLES_UNDER_THE_PITCH 10 PERFECT_PASSING      11 FAN_INTERACTION     12 NECESSARY_VIOLENCE
13 FOULING_FRENZY       14 THROW_A_ROCK         15 UNDER_SCRUTINY      16 INTENSIVE_TRAINING
```

So Rust needs a sorted `1..=16` list, not a HashMap port.

**2. The ITER2 shuffle port predicts Java's actual choice.** Seeding `JavaRandom` with
`parity_seed ^ 0x5EEDC0113C7104` (the constant `ParityRunner.seedCollectionsShuffleRng` uses) and
running `collections_shuffle` on `[1..=16]`:

| parity seed | predicted first roll | |
|---:|---:|---|
| 1 | **6 → BAD_HABITS** | **matches Java** (its `BadHabitsHandler` rolled the d3 at rng 12) |
| 20 | 5 → KNUCKLE_DUSTERS | |

The end-to-end selection chain is therefore verified before a line of engine code is written.

### Scope for the next iterations
16 prayers with 16 handlers (`BadHabitsHandler`, `KnuckleDustersHandler`, … under
`server/inducements/mixed/prayers/`), plus `StepPrayer`/`StepPrayers` and a `ParityRunner` `PRAYER`
case. Handlers split into three shapes: `RandomSelectionPrayerHandler` (rolls to pick players —
consumes GAME dice), `SelectPlayerPrayerHandler` (dialog), and direct-effect ones. Port them
one at a time, cheapest first, re-running human 1-100 after each.


## ITER4 — prayer SELECTION ported to the right stream (count unchanged at 38/100)

Good news first: **the whole prayer system is already ported.** All 16 handlers, the selectors, the
factory and `StepPrayer`/`StepPrayers` exist under `crates/ffb-engine/src/inducements/bb2020/prayers/`
— including `bad_habits_handler.rs`, which correctly rolls `rng.d3()`. ITER3's "Rust implements no
part of the prayer system" was wrong about the *cause*; what is missing is narrower.

**The real bug in `handle_cheering_fans`:** Rust picked the prayer with

```rust
let prayer_roll = rng.range(max_prayer_roll) as i32 + 1;   // GAME dice stream
```

Java picks it with `Collections.shuffle(availablePrayerRolls); remove(0)` — the **Collections**
stream, consuming **zero** game dice. So Rust both drew from the wrong stream and chose a different
prayer.

FIXED, as a 1:1 port:
- `Game::collections_rng` — a `JavaRandom` mirroring `java.util.Collections`' shared field,
  `#[serde(skip)]`, seeded by `DriverGameState::from_game` with
  `(seed as i64) ^ 0x5EED_C011_3C71_04`, the *same* expression
  `ParityRunner.seedCollectionsShuffleRng` uses (keep the two constants in sync).
- `handle_cheering_fans` now builds `1..=max_prayer_roll`, runs `collections_shuffle`, and takes
  element 0 — no game dice consumed.

**Fail count unchanged at 38/100** — reported plainly. The selection is now right but the prayer
still never resolves: `FFB_DRIVE_TRACE` shows `ApplyKickoffResult` (stack_len 5) followed directly
by `KickoffAnimation` (stack_len 4), so the `StepId::Prayer` sequence this branch pushes is **never
executed** — the stack shrinks instead of growing. `apply_effects` does drain `outcome.pushes` into
`stack.push_sequence`, so the push is being lost somewhere between the branch and the stack.

### FRONTIER for ITER5
Find why the pushed Prayer sequence is dropped. Concrete checks, in order:
1. Confirm the branch is actually taken — `roll_home=5`, `roll_away=2`, both teams have 0
   cheerleaders, so `total_home > total_away` should hold. A gated print in the branch settles it in
   one run (and would rule out a stale build).
2. If it IS taken, inspect `DriverStepStack::push_sequence` for a StepId whitelist/filter that
   silently drops `StepId::Prayer` — note the driver's `make_step_for` maps
   `StepId::Prayer => StepPrayer::new(0, "")`, so an unregistered-id filter is plausible.
3. Once the step runs, the existing `BadHabitsHandler` should roll its d3 at the right point;
   re-check that Java's rng 12 `d3` and Rust's line up, then sweep.

Gates for this commit: bb2016 lineman **50/50 games match**, bb2025 lineman **50/50 games match**,
`cargo test` ffb-engine **7115/0**, ffb-model **2783/0**.


## ITER5 — the real structural cause found; routing REVERTED because it regressed

ITER4's fix went into a file that **never runs**. `make_step_for` has no `Rules::Bb2020` arm at all,
so `StepId::ApplyKickoffResult` falls through the default, whose `use crate::step::bb2025::kickoff::*`
glob resolves to the **bb2025** step. `step/bb2020/step_apply_kickoff_result.rs` is dead code — the
same stale-file trap as ITER1's hook-state panic, and lesson #1 in this very document. A gated print
in the bb2020 `handle_cheering_fans` produced NO output, which is what exposed it.

**The consequence is bigger than a prayer bug: bb2020 games run BB2025 kickoff rules.** The live
bb2025 `handle_cheering_fans` grants the winner **extra offensive block assists** (the BB2025 rule);
BB2020 grants a **Prayer to Nuffle**. The tables differ in membership too (BB2020 has Officious Ref
where BB2025 has Charge / Dodgy Snack).

Adding a `Rules::Bb2020` arm routing `ApplyKickoffResult` to the bb2020 file DID work mechanically —
`FFB_DRIVE_TRACE` then showed `ApplyKickoffResult` → **`Prayer`** → `KickoffAnimation`, where before
the Prayer step never appeared.

**But it regressed human from 38/100 to 11/100, so it was REVERTED** per the campaign rule. The
reason is the next finding:

### Both `StepPrayer` implementations are stubs
`step/bb2025/step_prayer.rs` and `step/bb2020/step_prayer.rs` both say:

```
// Stub: PrayerHandlerFactory not translated → treat as "no handler found" path.
```

So the Prayer step runs and does nothing. The 16 prayer **handlers** are fully ported under
`crates/ffb-engine/src/inducements/bb2020/prayers/` (and `bb2025/`), and `PrayerFactory::for_roll`
exists in `ffb-model` — but **`PrayerHandlerFactory` (roll → handler) was never translated**, so
nothing dispatches to them. Routing the kickoff without that means bb2020 correctly *awards* a
prayer and then correctly *fails to apply it*, which diverges in more places than the old wrong-rule
behaviour did.

### FRONTIER for ITER6 — land these TOGETHER, not separately
1. Port `PrayerHandlerFactory` (`server/factory/mixed/PrayerHandlerFactory.java`) — the roll →
   handler map, per edition.
2. Make `StepPrayer` dispatch: `firstRun` → `handler.init_effect(...)`; the `else` branch →
   `handler.apply_selection(...)`; no handler → `NEXT_STEP` (Java's shape, already documented in
   the stub's comments).
3. THEN re-add the `Rules::Bb2020 => ApplyKickoffResult` routing arm from this iteration
   (the exact diff is in this commit's message).
4. Expect the `BadHabitsHandler` d3 to appear at Java's rng 12, and re-sweep.

Baseline restored after the revert: bb2020 human **38/100 passed**. Gates: bb2016 lineman
**50/50 games match**, bb2025 lineman **50/50 games match**, `cargo test` ffb-engine **7115/0**,
ffb-model **2783/0**.


## ITER6 — prayer dispatch built; the d3 now matches Java exactly. Routing still held back.

Landed (all inert for bb2020 until the routing arm returns, so **no behaviour change and no
regression** — baseline stays 38/100):

1. **`PrayerHandlerFactory` ported** (`inducements/bb2020/prayers/prayer_handler_factory.rs`) —
   Java builds its handler set by classpath scanning; Rust lists the same 16 handlers explicitly and
   keeps `forPrayer`'s "first handler that handles it" semantics. Two tests: every roll 1..=16
   resolves to a handler, and no two handlers claim the same prayer.
2. **`StepPrayer` dispatches** instead of being a stub — `firstRun` → `init_effect` (NEXT_STEP when
   fully applied, CONTINUE when it needs a dialog, matching Java's shape where the handler, not the
   step, decides), `else` → `apply_selection`, no handler → NEXT_STEP.
3. **A real option-default bug**: Rust's `game.options.is_enabled(...)` returns **false for an unset
   option** — it never consults the factory — while Java reads `getOptionWithDefault`, whose default
   for `INDUCEMENT_PRAYERS_USE_LEAGUE_TABLE` is **true**. So Rust was shuffling `[1..=8]` where Java
   shuffles `[1..=16]`: a different prayer AND a different Collections-stream consumption. Both
   bb2020 sites now use `get_option_with_default`.

**Verified progress on seed 1:** Rust's die at position 12 is now `sides=3 result=2` — exactly
Java's `d3=2` from `BadHabitsHandler`. The prayer is selected correctly, the handler is found, and
its effect die matches.

**Why the routing is still reverted:** with routing on, human measured **11/100 vs the 38/100
baseline**, so it went back per the campaign rule. The remaining cause is the last piece of the
chain:

### FRONTIER for ITER7 — `PlayerSelector.selectPlayers` is on the wrong stream AND the wrong shape
Java:
```java
for (int i = 0; i < Math.min(amount, available.size()); i++) {
    Collections.shuffle(available);   // COLLECTIONS stream, whole remaining list, each iteration
    selected.add(available.remove(0));
}
```
Rust (`inducements/bb2020/prayers/player_selector.rs`) does ONE Fisher-Yates over the whole list
using the **GAME** rng and then truncates — visible as `sides=10, 9, …` where Java rolls no game
dice at all (seed 1 pos 13: Rust `sides=10` vs Java `d8` ball bounce).

Fix = use `game.collections_rng` and Java's exact shape (N separate whole-list shuffles, each taking
element 0). This needs the `PlayerSelector::select_players` signature to take `&mut Game` rather
than `&Game` + `&mut GameRng`, which touches the trait, both selectors and the handlers that call
them — mechanical but wide. Land it TOGETHER with re-adding the two routing arms
(`ApplyKickoffResult`, `Prayer`), then re-sweep.

Gates for this commit: bb2020 human **38/100** (unchanged baseline), bb2016 lineman **50/50 games
match**, bb2025 lineman **50/50 games match**, `cargo test -p ffb-engine` **7117/0**.


## ITER7 — the prayer chain completes: **38 → 42/100** (first bb2020 improvement)

Two changes, landed together because neither works alone.

**1. `PlayerSelector.selectPlayers` ported exactly.** Java:
```java
for (int i = 0; i < Math.min(amount, available.size()); i++) {
    Collections.shuffle(available);      // COLLECTIONS stream, whole remaining list
    selected.add(available.remove(0));
}
```
Rust did ONE Fisher-Yates over the whole list with the **game** rng, then truncated — wrong on both
counts: the wrong stream (shifting every later die — seed 1 pos 13 showed `sides=10` where Java
rolls the d8 ball bounce) and a different permutation sequence. The trait parameter changed from
`rng: &mut GameRng` to `collections_rng: &mut JavaRandom`, which is a truer signature: this
selection consumes no game dice at all. `random_selection_prayer_handler` split-borrows
`game.collections_rng` for the call.

**2. Cheering Fans edition-gated INSIDE the shared step — NOT routed.** BB2020 awards a Prayer to
Nuffle where BB2025 grants extra offensive block assists. Routing `ApplyKickoffResult` to the bb2020
file made the prayer chain correct but still measured **12/100**, because that file is STALER than
the shared one for the events they have in common — its own header flags QuickSnap / SolidDefence /
HighKick as TODO. This is the bb2016 campaign's lesson again: **edition-gate the shared step, never
route to the staler edition file.** Only `StepId::Prayer` is routed (the bb2025 `StepPrayer` is
still a stub; the bb2020 one now dispatches).

**Verified:** seed 1's dice now match Java exactly through position 16 —
`d3=2, d8=2, d6=6, d6=4, d6=2` on both sides, where Rust previously had no `d3` at all. Sweep
**38 → 42/100 passed**.

Gates: bb2016 lineman **100/100**, bb2016 human **100/100**, bb2025 lineman **100/100**, bb2025
human **100/100**, `cargo test` ffb-engine **7118/0**, ffb-model **2783/0**.

### Next
Take the new lowest failing seed on human and repeat. The prayer machinery is now real, so expect
the remaining failures to be ordinary rule divergences rather than missing subsystems.

## ITER8 — human 42→62/100: BB2020 never banned a spotted fouler

**Seed 2, i=52.** Dice identical (`rng_calls=47` both sides); state differed in exactly one field:

```
JAVA h1t54ahomes0,0 b5,0,true pa00:-1,-1,Reserve  | RUST ... pa00:13,6,Standing
```

`a00` fouled at i=51, the ref spotted it, the argue-the-call roll was a 3 (fail) — Java sent the
fouler off, Rust left them on the pitch.

**Root cause.** `mixed/foul/StepEjectPlayer` (the live step for BB2020 and BB2025) only calls
`UtilBox.putPlayerIntoBox`. That method switches on the player's *state* and returns without doing
anything for a `STANDING` player (`default: break` → `boxX == 0`). The state is set to `BANNED` by
the `StepEjectPlayer` **step hook** inside `SneakyGitBehaviour` — the same hook in Java's
`skillbehaviour/bb2020/SneakyGitBehaviour` (`@RulesCollection(BB2020)`) and
`skillbehaviour/bb2025/SneakyGitBehaviour` (the two files' eject hooks are byte-identical).

Rust's `SkillRegistry::build_bb2020()` never registered `SneakyGitBehaviour`, so
`execute_step_hooks(StepId::EjectPlayer)` found no modifier and no fouler was ever banned in BB2020.
BB2016 was unaffected — `make_step_for` routes it to its own `bb2016/foul/StepEjectPlayer`.

**Fix** (`crates/ffb-engine/src/skill_behaviour/registry.rs`): register the shared
`SneakyGitBehaviour` in `build_bb2020`, matching Java's BB2020 registration. Regression test
`bb2020_registers_sneaky_git_with_an_eject_player_modifier` asserts BB2020 carries an
`EjectPlayer` modifier; the entry-count test moves 24 → 25.

Also fixed a compile break in `crates/ffb-server/src/net/wire_prompt.rs`: the `AgentPrompt::BlockTarget`
variant added during the BB2016 campaign left `prompt_to_wire` non-exhaustive. It maps to `None` —
Java has no block-target dialog (the client sends `ClientCommandBlock`), so nothing is rendered.

**Result:** human 42/100 → **62/100**. Gates: lineman bb2016 100/100, lineman bb2025 100/100,
`cargo test --workspace` clean.

**Noted, not yet ported** (needs a Sneaky Git player or the Under Scrutiny prayer to be reachable):
`bb2020/SneakyGitBehaviour`'s *Referee* hook differs from BB2025 — BB2020 uses
`refereeSpotsFoul = (armorRoll[0] == armorRoll[1]) && !hasSkill(sneakyGit)` and
`refereeSpotsFoul |= underScrutiny` (BB2025 drops the skill term and requires `isArmorBroken()` for
the Under Scrutiny term). Rust inlines this in `step_referee.rs` with the BB2025 form only.

## ITER9 — human 62→85/100: BB2020's WILDLY_INACCURATE pass never deviated

**Seed 2, i=197** (`Activate(Away4, PASS)`, target `(21,7)`, thrower on `(14,9)`). Both engines
rolled the same pass die (`rng 87 d6=3`), then diverged:

| | dice for the action | ball after |
|---|---|---|
| Java | 4 (`d6=3` pass, `d8=8` dir, `d6=6` **distance**, `d8=7` bounce) | `7,3` |
| Rust | 5 (`d6=3` pass, three `d8` scatters, `d8` bounce) | `19,4` |

**Root cause — two linked gaps.**

1. `bb2020/PassMechanic.evaluatePass` returns `WILDLY_INACCURATE` where
   `bb2025/PassMechanic` returns `FUMBLE` (`resultAfterModifiers <= 1`). Rust's mechanics are
   correct and correctly routed by `pass_mechanic_for(game.rules)` — but the live
   `bb2025/pass/step_pass.rs` collapsed `INACCURATE | WILDLY_INACCURATE` into one arm that
   *hard-coded* `PassOutcome::Inaccurate` when publishing `PassResultParam`, so the distinction
   never left the step.
2. `bb2020/pass/StepMissedPass` wraps the BB2025 body in
   `if (state.getResult() == WILDLY_INACCURATE) { deviate from the THROWER by one d8 direction and
   one d6 distance } else { up to three 1-square scatters, one d8 each }`. Java's two files are
   otherwise identical. Rust's live `bb2025/pass/step_missed_pass.rs` had only the `else` arm.
   (`bb2020/pass/step_missed_pass.rs` carries a port of the branch but is dead code — the driver's
   `use crate::step::bb2025::pass::*` wins.)

**Fix.** Edition-gate the shared step rather than route to the staler file:
* `bb2025/pass/step_pass.rs` — publish the real outcome (`WildlyInaccurate` vs `Inaccurate`).
* `bb2025/pass/step_missed_pass.rs` — add `pass_result` (fed by `PassResultParam`) and
  `deviate_from_thrower`, taken only when `rules == Bb2020 && pass_result == WILDLY_INACCURATE`;
  everything after Java's if/else stays shared.

Tests: `bb2020_wildly_inaccurate_deviates_from_the_thrower_with_two_dice` (asserts exactly 2 dice,
start = thrower square not pass target, scatter loop untouched) and
`bb2025_ignores_wildly_inaccurate_and_uses_the_scatter_loop`.

**Result:** human 62/100 → **85/100**. Gates: lineman bb2016 100/100, lineman bb2025 100/100,
`cargo test --workspace` clean (14/14 suites ok).

## ITER10 — human 85→90/100: the three coach-choice Prayers to Nuffle

**Seed 20.** Java produced 137 steps, Rust 282 — Java's game ended early. Its stderr showed
`UNHANDLED_STEP: PRAYER turnMode=KICKOFF` 500 times after the Cheering Fans rolls
(`rng 74 d6=2` home vs `rng 75 d6=6` away → away gains a prayer).

A new gated diagnostic in `ParityRunner` (`FFB_UNHANDLED_DUMP=1` dumps the stuck step's JSON)
named the culprit in one run:

```
UNHANDLED_DUMP: PRAYER json={... "nextAction":"continue", "roll":5, "firstRun":false,
                                 "teamId":"teamHumanParity20Away", "playerId":null ...}
```

Prayer roll 5 = **Knuckle Dusters**. `firstRun=false` with `playerId=null` means the harness had
answered the dialog by DECLINING (its blanket "all PlayerChoice modes decline with an empty
selection" rule), and `SelectPlayerPrayerHandler.applySelection` then NPE'd on
`game.getPlayerById(null)` — skipping `setNextAction(NEXT_STEP)`, so the step spun forever.

**Root cause (Rust).** Java has three prayers where **the coach chooses the player** —
`IronManHandler`, `KnuckleDustersHandler`, `BlessedStatueOfNuffleHandler`, all
`extends SelectPlayerPrayerHandler extends DialogPrayerHandler`. Their `initEffect` calls
`selector().eligiblePlayers(...)`, shows a `DialogPlayerChoiceParameter`, and returns `false`
(waiting). Rust routed all three through `RandomSelectionPrayerHandler` instead, which
(a) picked its own player, and (b) drew from `java.util.Collections`' shared stream that Java
never touches on this path — desyncing that stream for the rest of the game.

**Fix.**
* `mixed/prayers/player_selector.rs` — new trait method `eligible_players` (the filter half of
  `selectPlayers`, no shuffle); the bb2020/bb2025 selectors now implement it and `select_players`
  delegates to it.
* `mixed/prayers/select_player_prayer_handler.rs` — real ports of `DialogPrayerHandler.initEffect`
  (`eligible_players_for_dialog`) and `SelectPlayerPrayerHandler.applySelection`.
* `mixed/prayers/prayer_handler.rs` — trait gains `dialog_choice_mode()` and
  `eligible_dialog_players()`; `apply_selection`'s third argument is the CHOSEN PLAYER id.
* The three bb2020 handlers implement the dialog contract (empty eligible list → `true`, i.e.
  Java's "prayer wasted" arm; otherwise `false`).
* `bb2020/step_prayer.rs` — emits `AgentPrompt::PlayerChoice` when the handler declares a mode,
  and passes `self.player_id` (not the team id — a latent bug) into `apply_selection`.
* `random_agent.rs` + `ParityRunner` PLAYER_CHOICE arm — both answer these three modes with the
  eligible player of **lowest shirt number**. `nr` is shared team data in both engines and, unlike
  a board coordinate (the rule ANIMAL_SAVAGERY uses), is well-defined for the RESERVE players this
  dialog offers during START_GAME.

Jar rebuilt. Tests: dialog-contract tests on all three handlers (`initEffect` grants nothing and
reports a pending dialog; `applySelection` grants the skill/AV to the chosen player) plus
`eligible_players_lists_every_on_pitch_player_without_randomness`,
`eligible_players_is_empty_when_nobody_qualifies`, `apply_selection_marks_the_chosen_player`,
`apply_selection_with_no_player_is_a_no_op`.

**Result:** human 85/100 → **90/100**. Gates: lineman bb2016 100/100, lineman bb2025 100/100
(re-run after the jar rebuild), `cargo test --workspace` 14/14 suites ok.

**Noted, not yet ported:** `IntensiveTrainingHandler extends DialogPrayerHandler` shows a *skill*
dialog (`DialogSelectSkillParameter`) rather than a player choice; Rust still routes it through
random selection. Prayer roll 16, so it will surface on a later seed.

## ITER11 — root cause found, fix REVERTED (regressed 90→84): the BB2020 kickoff table

**Seed 23, i=138.** Dice identical (`rng_calls=47` both), one field differed: `h08` Stunned in Java,
Standing in Rust. Java's log showed the second-half kickoff resolving **Officious Ref**
(`rollThrowARock` d6=1 home / d6=6 away → `randomPlayer` d11=9 → `insertSteps` d6=6 → stun).

Rust rolled the *same four dice* but applied a different event. A `FFB_KICK_TRACE` probe on the
shared step's dispatch showed `KickoffResult::DodgySnack`.

**Root cause.** Java has one `KickoffResultMapping` per edition and BB2020's differs from BB2025's
on exactly two rolls:

| roll | `kickoff/bb2020/KickoffResultMapping` | `kickoff/bb2025/KickoffResultMapping` |
|------|---------------------------------------|---------------------------------------|
| 10   | `BLITZ`                               | `CHARGE`                              |
| 11   | `OFFICIOUS_REF`                       | `DODGY_SNACK`                         |

Rust's live `bb2025/kickoff/step_kickoff_result_roll.rs` hard-codes the BB2025 table, so every
BB2020 game has run the wrong event on rolls 10 and 11 since the edition was added. Dodgy Snack
happens to draw the same d6/d6/d11/d6 shape as Officious Ref, which is why the dice streams stayed
aligned and only the board diverged — that coincidence is why this survived 10 iterations.

**Why the fix was reverted.** Edition-gating the table (`kickoff_result_for_roll_in(rules, roll)`),
porting `handleOfficiousRef` + its `insertSteps` into the shared apply step, and threading the
`GOTO_LABEL_ON_BLITZ`/`ON_END` labels onto the harness's flattened `kickoff_tail()` all worked —
seed 23's Officious Ref then matched — but the roll-10 → `BLITZ` result exposed a second, larger
gap: **the BB2020 Blitz! kickoff (a free turn for the kicking team) is not wired up in the shared
kickoff sequence.** The flattened `kickoff_tail()` / `h2_kickoff_sequence()` have no labelled
`BLITZ_TURN` target and no `GotoLabel` jump around it, so the goto drained the step stack; adding
them by hand changed the post-kickoff ball position (seed 23 step 1: Java `b12,8` vs Rust `b10,0`).
Net effect on the roster was **90 → 84**, so the whole change was reverted per the campaign's
revert-on-regression rule. Baseline re-confirmed at `PARITY: 90/100`.

**Next iteration should land this properly**, in this order:
1. Build the BB2020 kickoff sequence from `generator/mixed/Kickoff::build_sequence` (which already
   threads both labels and includes the labelled `BlitzTurn` + `KickoffAnimation` steps) instead of
   hand-patching `kickoff_tail()`, and reconcile the resulting step order against Java's
   `generator/mixed/Kickoff` step-by-step — the ball-position drift above is an ordering bug in the
   flattened tail, not in the Blitz handler.
2. Then edition-gate `kickoff_result_for_roll`, and route `Blitz` → `goto_label_on_blitz` /
   `OficiousRef` → the ported `handle_officious_ref` in the shared apply step.
3. `handleOfficiousRef`'s `roll == 1` arm (eject instead of stun) needs the EJECT_PLAYER
   sub-sequence; the BB2020-file port has it (`build_eject_seq`) and can be lifted with the rest.

Kept from this iteration: nothing in the engine. The `FFB_UNHANDLED_DUMP` harness probe from ITER10
and the `FFB_KICK_TRACE`-style dispatch probing technique are what made the diagnosis quick.

## ITER12 — human 90→96/100: the BB2020 kickoff table, landed properly

Lands ITER11's diagnosis (BB2020 rolls 10/11 are `BLITZ`/`OFFICIOUS_REF`, not BB2025's
`CHARGE`/`DODGY_SNACK`) together with the two things that made the first attempt regress.

**What ITER11 got wrong.** It hand-patched labels onto `kickoff_tail()` and stopped there; the
`BLITZ` result then reached a `StepBlitzTurn` that never rolled its d3, so the dice stream desynced
from the very first kickoff. The ball drift (`b12,8` vs `b10,0`) was that missing die, not a step
ordering problem. The Rust dice trace made it obvious once compared position by position:

```
JAVA  rng=10 d3=3 from=...StepBlitzTurn.executeStep:87    rng=11 d8=2 (bounce)
RUST  pos=10 sides=8                                       ← no d3 at all
```

**Three changes, all edition-gated:**

1. `bb2025/kickoff/step_kickoff_result_roll.rs` — `kickoff_result_for_roll_in(rules, roll)` returns
   `Blitz`/`OficiousRef` for BB2020 on rolls 10/11 and the BB2025 result otherwise. Test
   `bb2020_and_bb2025_kickoff_tables_differ_only_on_rolls_10_and_11` walks all of 2..=12.
2. `bb2025/kickoff/step_blitz_turn.rs` — for BB2020 it delegates to
   `bb2020::step_blitz_turn::StepBlitzTurn`, which is the real port of Java's
   `@RulesCollection(BB2020)` class: a d3 for the activation limit (`limit = roll + 3`), a
   `BlitzTurnState(limit, availablePlayers)`, and the ACTIVATIONS_EXHAUSTED report when the blitzing
   team has no active players. BB2025's class has none of that. Tests
   `bb2020_blitz_turn_rolls_the_activation_limit_d3` and `bb2025_blitz_turn_rolls_nothing`.
   (This is not "routing to a staler file" — the BB2020 port was complete and simply unreachable,
   and Java itself routes this step per `@RulesCollection`.)
3. `bb2025/kickoff/step_apply_kickoff_result.rs` — `Blitz` gotos `goto_label_on_blitz`; `OficiousRef`
   runs a fresh port of `handleOfficiousRef` + `insertSteps`, **including** the `roll == 1` ejection
   branch (`SetActingPlayerAndTeam` → `EjectPlayer` with `OFFICIOUS_REF=true` → a
   `ConsumeParameter` labelled `END_FOULING`) that ITER11 had left as a no-op.
4. `sequences.rs` — `kickoff_tail(rules)` gives BB2020's `ApplyKickoffResult` the
   `GOTO_LABEL_ON_END`/`GOTO_LABEL_ON_BLITZ` params and adds the labelled `BLITZ_TURN` /
   `KICKOFF_ANIMATION` / `END_KICKOFF` targets plus the jump that skips the blitz turn on the normal
   path — the exact shape of `generator/mixed/Kickoff.pushSequence`. Other editions keep the lighter
   tail unchanged (BB2025 has no BLITZ result; BB2016 reroutes via StepSpectators).

**Result:** human 90/100 → **96/100**. Gates: lineman bb2016 100/100, lineman bb2025 100/100,
`cargo test --workspace` 14/14 suites ok.

Remaining human fails: seeds 38, 58, 87, 99.

## ITER13 — human 96→99/100: the half-2 kickoff needed the same BLITZ labels

**Seed 38.** Java ran 315 steps, Rust 173, with **no state divergence** among the steps they shared —
Rust's log carried `FFB DRIVER ERROR: goto unknown label '' — step stack drained`. Java's i=174 is
`half=2 turn=1`, so the game died at the second-half kickoff.

Same defect as ITER12's opening kickoff, different entry point: `h2_kickoff_sequence()` (used by
`StepEndTurn` for a new half and after a touchdown) is a second flattened kickoff sequence, and
ITER12 only labelled `kickoff_tail()`. BB2020's kickoff table can roll `BLITZ` at any kickoff, so
the goto found the empty label there instead.

**Fix.** Extract the label patch into `add_bb2020_kickoff_labels(&mut Vec<SequenceStep>)` — it adds
`GOTO_LABEL_ON_END`/`GOTO_LABEL_ON_BLITZ` to `ApplyKickoffResult`, labels
`BLITZ_TURN`/`KICKOFF_ANIMATION`/`END_KICKOFF`, and inserts the jump that skips the blitz turn on the
normal path, exactly as `generator/mixed/Kickoff.pushSequence` does — and call it from BOTH
`kickoff_tail(rules)` and the new `h2_kickoff_sequence_for(rules)`. The bb2020 and shared
`StepEndTurn` now call the rules-aware variant.

Tests: `bb2020_kickoff_sequences_carry_the_blitz_labels` checks both entry points for the params,
asserts every goto target the step can name exists as a label in the same sequence, and that the
jump precedes the labelled `BLITZ_TURN`; `non_bb2020_kickoff_sequences_are_unchanged` pins BB2016
and BB2025 to the bare sequences.

**Result:** human 96/100 → **99/100**. Gates: lineman bb2016 100/100, lineman bb2025 100/100,
`cargo test --workspace` 14/14 suites ok. Remaining human fail: seed 43.

## ITER14 — human 🟢 100/100: the BB2020 throw-in is one square longer

**Seed 43, i=116.** Rust was exactly one die ahead (`rng_calls=77` vs Java's `76`) and the ball sat
on a different square. Position-by-position dice comparison isolated it:

```
JAVA  73 d6=1 rollThrowInDirection | 74 d6=1 | 75 d6=4 (distance) | 76 d8=8 bounce
RUST  73 d6=1                      | 74 d6=1 | 75 d6=4            | 76 d6=2  ← extra | 77 d8
```

An `FFB_RNG_BT=76` backtrace put the extra d6 inside `StepCatchScatterThrowIn`.

**Root cause.** Java resolves `game.getMechanic(Mechanic.Type.THROW_IN)` per edition, and the three
differ:

| | `distance(roll)` | `isCornerThrowIn` |
|---|---|---|
| `mechanics/bb2016/ThrowInMechanic` | `d1 + d2` | always false |
| `mechanics/bb2020/ThrowInMechanic` | **`d1 + d2 + 1`** | always false |
| `mechanics/bb2025/ThrowInMechanic` | `d1 + d2` | **true in the four corners** (d3 direction) |

Rust's shared `bb2025/shared/step_catch_scatter_throw_in.rs` hard-coded
`ffb_mechanics::bb2025::throw_in_mechanic::ThrowInMechanic`, so every BB2020 throw-in landed one
square short. The BB2020 mechanic was already ported and correct — just never routed. A short
landing on an occupied square then adds a catch d6 Java never rolls, which is what put Rust
permanently a die ahead.

**Fix.** New `mechanic::throw_in_mechanic_for(rules)` alongside the existing `pass_mechanic_for` /
`on_the_ball_mechanic_for`, and the shared step calls it. Tests
`throw_in_mechanic_distance_is_edition_specific` and `corner_throw_ins_are_bb2025_only`.

**Result: bb2020 `human` 🟢 100/100** (99 → 100). Gates: lineman bb2016 100/100, lineman bb2025
100/100, `cargo test --workspace` 14/14 suites ok.

**Status: 1 of 30 bb2020 matchups green.** `lineman` bb2020 is at 98/100 (2 fails) — next target,
then the remaining 28 rosters. Note the five shared fixes so far (SneakyGit registration, wildly
inaccurate pass, coach-choice prayers, kickoff table + Blitz!/Officious Ref + sequence labels,
throw-in mechanic) are all edition-wide, so other rosters should benefit without roster-specific work.

## ITER15 — lineman bb2020 diagnosis (no fix yet): the harness DROPS a player at setup

`lineman` bb2020 is at 98/100 (seeds 46 and 50). **Seed 46, i=172** (half 2, turn 1 — the H2 setup),
dice identical at `rng_calls=48`, and exactly two fields differ:

```
JAVA a02:-1,-1,Reserve   a05:20,9,Standing
RUST a02:13,8,Standing   a05:-1,-1,Reserve
```

Both engines field **10** of the away team's 11 players; they disagree on which one sits out.

**Eliminated cheaply:** Sweltering-Heat fainting (an `FFB_HEAT_TRACE` probe showed the block never
runs on this seed) and `Team.getPlayers()` nr-sort ordering (the parity XML is already
nr-ascending, so Java's sort is a no-op).

**What the probes showed.** An `FFB_SETUP_TRACE` probe on `canonical_setup_action` proves Rust is
prompted 11 times per setup and places **all 11** — `away_11` is still a candidate at
`on_pitch=10`. Mapping Java's away squares back through `transform()` (x → 25−x) shows Java's
placement order instead:

| player | raw square | list slot |
|---|---|---|
| a00 | 12,7 | `LOS[0]` |
| a01 | 12,6 | `LOS[1]` |
| a02 | — | **dropped** |
| a03 | 5,5 | `OVER[0]` |
| a04 | 5,7 | `OVER[1]` |
| a05 | 5,9 | `OVER[2]` |

So Java placed two on the line of scrimmage, **silently lost a02**, and continued into the overflow
squares — never filling `LOS[2]`. `ParityRunner.placeReserves` explains how: `li` (the LOS cursor)
and `oi` (the overflow cursor) advance monotonically across the WHOLE setup, and `losNeeded--` /
`placed++` run unconditionally right after `UtilServerSetup.setupPlayer(...)`. A player whose square
is taken — or whose `setupPlayer` call the server rejects — is therefore dropped with no retry,
and the cursor moves on. Rust's `canonical_setup_action` instead scans for the FIRST free square,
so it never drops anyone.

**Next iteration:** mirror `placeReserves`' non-retrying cursor semantics in
`canonical_setup_action`. The wrinkle is that Rust's agent is prompted once per player and holds no
cursor state, while the skipped squares leave no trace on the board — so reconstructing `li`/`oi`
from board occupancy alone is not sufficient. Either give the setup action cursor state across the
prompts of one setup, or have it emit the whole placement at once. Also worth establishing WHY
`setupPlayer` didn't take for a02 (an `FFB_TRACE` dump of `LOS[2]`'s occupancy at that moment on the
Java side would settle it) — if the square was genuinely free, the drop is a rejected placement
rather than an occupied-square skip, and the mirror must reproduce that condition, not the cursor.

Baseline unchanged and re-confirmed: `PARITY: 98/100`. All probes removed.

## ITER16 — lineman 98→99/100: Officious Ref must pick BOTH targets before either ref roll

**Corrects ITER15's diagnosis, which was wrong.** A gated `FFB_SETUP_DUMP` added to
`ParityRunner.placeReserves` shows Java places **all 11** away players at the half-2 setup, `A3` at
`(13,8)` — exactly where Rust puts it. Nothing is dropped at setup; `placeReserves`' cursor
semantics are not involved. The player leaves the pitch *after* the setup, at the kickoff.

**The real cause.** Seed 46's half-2 kickoff rolls 11 → Officious Ref, and the fan-factor totals
TIE, so both teams are targeted. The two dice traces line up as:

```
JAVA  45 d11=2 (home pick) | 46 d11=3 (away pick) | 47 d6=4 (home ref) | 48 d6=1 (away ref)
RUST  45 d11=2 (home pick) | 46 d6=4  (home ref)  | 47 d11=6 (away pick) | 48 d6=1 (away ref)
```

Java's `handleOfficiousRef` picks **both** players first and only then calls `insertSteps` for each:

```java
if (totalAway >= totalHome) { playerIdHome = randomPlayer(playersOnField(teamHome)); }
if (totalHome >= totalAway) { playerIdAway = randomPlayer(playersOnField(teamAway)); }
addReport(new ReportKickoffOfficiousRef(...));
if (playerIdHome != null) insertSteps(..., ApothecaryMode.HOME);
if (playerIdAway != null) insertSteps(..., ApothecaryMode.AWAY);
```

ITER12's port fused the two phases into one loop. That draws the same NUMBER of dice in a different
ORDER, so `rng_calls` stayed aligned and only the board diverged: the away pick read the home team's
ref d6 (`6` instead of `3`), and the `d6=1` ref roll then **banned `away_06` instead of `away_03`**.
BANNED renders through the harness's `default:` arm as "Reserve", which is what made the state diff
look like a setup problem.

**Fix.** Split the phases in `handle_officious_ref` exactly as Java does.

Test `bb2020_officious_ref_picks_both_targets_before_rolling_either_ref_die` finds a seed whose
fan-factor d6 rolls tie, replays the stream to derive which away player the SECOND d11 names, and
asserts that player is the one banned/stunned. Verified to have teeth: perturbing the dice order by
one draw makes it fail.

**Result:** lineman bb2020 98/100 → **99/100**. Gates: human bb2020 100/100, lineman bb2016 100/100,
lineman bb2025 100/100, `cargo test --workspace` 14/14 suites ok. Remaining lineman fail: seed 50.

Harness: `ParityRunner` gains the gated `FFB_SETUP_DUMP` probe (jar rebuilt).

## ITER17 — lineman seed 50 investigation (no fix): Java's attacker does not fall on BOTH_DOWN

`lineman` bb2020 sits at 99/100; seed 50 is the last fail. **First divergence i=7**, very early —
`home_02`'s BLOCK at i=6.

**Established, both engines agreeing through die 14:**

```
12  d6=2  rollBlockDice        (StepBlockRoll.executeStep:241)   — one die, even-strength block
13  d6=4  rollArmour           InjuryTypeBlock.armourRoll
14  d6=3  rollArmour           ... <- PilingOnBehaviour$1.handleExecuteStepHook:115  (DEFENDER)
15  d6=5  rollSkill            StepPass.executeStep:214          — the NEXT action already
```

Java then continues with **home still active on turn 1** and `h01` (= `Home2`, the attacker)
**STANDING**. Rust instead draws four more d6 (15–18), leaves `h01` **PRONE**, and ends the home
turn as a turnover.

**Ruled out by reading the Java source, not by guessing:**
* the block-die → `BlockResult` mapping — `BlockResultFactory.forRoll` maps `2 → BOTH_DOWN` and
  Rust's `block_result_for_roll` is identical;
* an edition-specific both-down step — `step/mixed/block/StepBothDown` carries BOTH
  `@RulesCollection(BB2020)` and `@RulesCollection(BB2025)`, so BB2020 and BB2025 share it;
* the Piling On hook consuming the step — it is registered on `StepDropFallingPlayers` and returns
  `false`, so the step body still runs. It is merely WHERE Java happens to roll the defender's
  injury (`ApothecaryMode.DEFENDER`), which is why dice 13/14 are only ONE player's armour;
* a missing skill — the `lineman` fixture has `<skillList/>` on every player in the Java XML and
  `starting_skills: vec![]` in `make_lineman_team`, so neither side has Block/Wrestle.

**The open question, precisely:** `mixed/block/StepBothDown` line 78 reads
`if (!actingPlayer.getPlayer().hasSkillProperty(NamedProperties.preventFallOnBothDown))` — with no
Block skill that should knock the ATTACKER down too, giving two more armour dice and a turnover,
which is exactly what Rust does. Java does neither. So something upstream of `StepBothDown` is
either skipping it for this block or clearing the attacker's FALLING state before
`StepDropFallingPlayers` reaches it.

**Next iteration:** get the Java step sequence for i=6, not just its dice — add a gated per-step
`stepId` line to `ParityRunner`'s loop (it already prints `JSTEP` only at agent prompts, which is
too coarse) and compare against Rust's `FFB_DRIVE_TRACE` list
(`BlockChoice, Juggernaut, BothDown, Wrestle, GotoLabel, DropFallingPlayers, …`). That will show
whether Java runs `StepBothDown` at all here, which is the fork in the road: if it does, the
attacker's FALLING state is being cleared; if it does not, the block sequence generator differs.

Baseline unchanged and re-confirmed: `PARITY: 99/100`. No engine changes this iteration.

## ITER18 — lineman seed 50, probes narrow it to the BLOCK_ROLL_PARTIAL_RE_ROLL answer

Two new gated probes in `ParityRunner` (harness only, jar rebuilt):
* `FFB_JSTEP_ALL=1` → `JDRIVE step=<StepId> dice=<n> dialog=<id> state=<board>` at every server loop
  top, i.e. every step that actually waits for a command. `JSTEP` only fires at agent prompts, which
  is far too coarse to see inside one action.
* `FFB_BLOCK_DUMP=1` → attacker/defender ids and `PlayerState`s either side of the harness answering
  a block dialog.

**What they show for `home_02`'s block at i=6:**

```
JDRIVE INIT_SELECTING dice=11              a01:13,6,Standing   h01:12,6,Moving
JDRIVE BLOCK_ROLL     dice=12 dialog=BLOCK_ROLL_PARTIAL_RE_ROLL
                                           a01:13,6,Reserve*   h01:12,6,Moving
JDRIVE INIT_SELECTING dice=14              a01:13,6,Prone      h01:12,6,Standing
```

`*` the harness's `playerStateStr` renders **FALLING** through its `default:` arm as "Reserve" — the
same aliasing that made BANNED look like a reserve in ITER16. So the DEFENDER is already FALLING when
the dialog opens, and the ATTACKER goes `MOVING → STANDING`: it never falls, takes no armour roll,
and there is no turnover. That is POW-shaped behaviour, not BOTH_DOWN.

Yet the die was `d6=2` and the mapping is unambiguous: `factory/BlockResultFactory` is annotated
`@RulesCollection(Rules.COMMON)` — a single class for all editions — with `case 2: return BOTH_DOWN`,
and `step/mixed/block/StepBothDown.start()` unconditionally calls `executeStep()`, which sets the
attacker FALLING whenever it lacks `preventFallOnBothDown`. The `lineman` fixture has no skills in
either engine (`<skillList></skillList>` in `rosters/roster_lineman_parity.xml`, `starting_skills:
vec![]` in `make_lineman_team`), so that branch must be taken.

**Surviving hypothesis.** The harness answers this dialog with `comm.sendBlockChoice(0)` under a
comment claiming it "picks die index 0". For `DialogBlockRollPartialReRollParameter` that index may
not mean "use die 0" at all — a *partial re-roll* dialog asks WHICH die to re-roll. If the server
reads it that way, Java re-rolls and lands on a different result, which would explain a POW-shaped
outcome from a `2`. Rust's agent meanwhile treats index 0 as "use this die" and gets BOTH_DOWN.

**Next iteration:** read `ClientCommandBlockChoice`'s consumer for the partial-re-roll path
(`StepBlockRoll` / the Brawler-style modifier that raises `DialogBlockRollPartialReRollParameter`) to
establish what index 0 means there, then make ParityRunner and the Rust agent agree on it. Note a
re-roll would consume a die, and the streams show no extra die — so if index 0 does mean "re-roll",
the re-roll must be free, which the consumer will confirm or refute. If it refutes the hypothesis,
the next datum to get is Java's `BlockResult` itself: the report list carries `ReportBlockRoll`, so a
gated dump of it from the harness after the dialog would settle what result Java actually applied.

Baseline unchanged and re-confirmed: `PARITY: 99/100`. No engine changes this iteration.

## ITER19 — seed 50: ITER18's hypothesis REFUTED; Java really does apply BOTH_DOWN

Extended `FFB_BLOCK_DUMP` to print the dialog's own fields. For `home_02`'s block:

```
BLOCK_DUMP dice nrOfDice=1 blockRoll=[2]
```

and `bb2020/block/StepBlockRoll.handleCommand` shows exactly what the harness's answer means:

```java
case CLIENT_BLOCK_CHOICE:
    fDiceIndex = blockChoiceCommand.getDiceIndex();
    fBlockResult = ...getFactory(Factory.BLOCK_RESULT).forRoll(fBlockRoll[fDiceIndex]);
```

So index 0 means **use die 0**, not "re-roll die 0" — ITER18's hypothesis is dead. One die, value 2,
mapped by the single `@RulesCollection(COMMON)` `BlockResultFactory` to **BOTH_DOWN**. Java genuinely
resolves BOTH_DOWN and *still* leaves the attacker STANDING with no armour roll and no turnover.

**Also newly mapped** (may or may not matter): the two Block sequence generators diverge AFTER
`WRESTLE`. Both run `BLOCK_ROLL → BLOCK_CHOICE → JUGGERNAUT → BOTH_DOWN → WRESTLE`, but
`generator/bb2020/Block` then inserts `BLOCK_DODGE, PUSHBACK, APOTHECARY, FOLLOWUP, TENTACLES,
SHADOWING, PICK_UP, FALL_DOWN(label)` before `DROP_FALLING_PLAYERS`, where `generator/bb2025/Block`
goes `… FOLLOWUP → DROP_FALLING_PLAYERS` directly. Rust's `FFB_DRIVE_TRACE` for this block shows
`BlockChoice, Juggernaut, BothDown, Wrestle, GotoLabel, DropFallingPlayers, …` — i.e. the BB2025
shape. Whether the extra BB2020 steps (particularly the labelled `FALL_DOWN`) are what spare the
attacker is the next thing to test.

**Instrumentation note for the next iteration.** `GameState.executeStep` already calls
`getServer().getDebugLog().logCurrentStep(IServerLogLevel.DEBUG, this)` for EVERY step — exactly the
per-step trace this needs — but `HeadlessFantasyFootballServer.getDebugLog()` (in `ffb-ai`, i.e.
harness code, not the engine) hard-codes an anonymous `DebugLog` whose `isLogging` always returns
false. It is now gated on `FFB_SERVER_DEBUG=1`, with `logCurrentStep` overridden to print
`JSTEPALL step=<StepId> dice=<n>` to stderr. **That override currently emits nothing** — to be
debugged first thing next iteration (most likely `getServer()` inside `GameState` is not returning
the headless instance, or `FantasyFootballServer`'s direct `fDebugLog` field access bypasses the
getter). Getting it working is worth the effort: it gives a full per-step Java trace to diff against
`FFB_DRIVE_TRACE`, which is the instrument this whole class of bug needs.

**Unrelated finding, recorded so it is not mistaken for drift:** `ffb-common/…/mixed/StatsMechanic`,
`bb2025/move/StepGoForIt` and `mixed/pass/StepPassBlock` carry pre-existing local modifications that
are **gated-logging only** (`-Dffb.parityDebug` → `JAVA_AVBROKE`, `JAVA_GFI`). They predate this
campaign, change no behaviour, and match the documented `DiceRoller.java` precedent.

Baseline unchanged and re-confirmed: `PARITY: 99/100`. No engine changes this iteration.

## ITER20 — full roster scout (seeds 1-25): 18 of 30 already clean

Seed 50 has now resisted three iterations and needs a per-step Java trace I cannot yet obtain
(`StepBlockChoice`'s BOTH_DOWN routing — see below). It is 1 fail out of a 3,000-game target, so
this iteration spent its time mapping the other 28 rosters instead. All six fixes so far are
edition-wide rather than roster-specific, and it shows:

| status | rosters |
|---|---|
| 🟢 **100/100** | `human`, and `lineman` at 99/100 |
| 🟢 clean on 1-25 (16) | `amazon`, `chaos`, `chaos_dwarf`, `dark_elf`, `dark_elf_league_fumbbl`, `high_elf`, `khemri`, `khemri_fumbbl`, `lizardman`, `nippon`, `norse`, `orc`, `skaven`, `slann`, `undead`, `vampire` |
| 1-4 fails / 25 | `chaos_pact` 1, `underworld` 3, `nurgle` 4, `renegades` 4 |
| heavy | `goblin` 14, `dwarf` 20, `necromantic` 21, `elf` 22, `wood_elf` 22, `halfling` 25, `ogre` 25, `slann_fumbbl` 25 |

(`slann_fumbbl`, `halfling` and `ogre` at 0/25 look like a single early systemic fault each, in the
shape of the bb2025 `*_fumbbl` roster-alias bug — worth checking the roster actually builds before
chasing dice.)

Next targets in order: `chaos_pact` (1), `underworld` (3), `nurgle` (4), `renegades` (4), then the
0/25 rosters (likely one fault each), then the heavies, then back to `lineman` seed 50.

### Seed 50 — where the block investigation stands

`generator/bb2020/Block`, read verbatim, is annotated by Java itself:

```java
sequence.add(StepId.BLOCK_ROLL);
sequence.add(StepId.BLOCK_CHOICE, from(GOTO_LABEL_ON_DODGE, DODGE_BLOCK),
    from(GOTO_LABEL_ON_JUGGERNAUT, JUGGERNAUT), from(GOTO_LABEL_ON_PUSHBACK, PUSHBACK));
sequence.jump(DROP_FALLING_PLAYERS);          // ← default fall-through
// on blockChoice = BOTH_DOWN
sequence.add(StepId.JUGGERNAUT, IStepLabel.JUGGERNAUT, from(GOTO_LABEL_ON_SUCCESS, PUSHBACK));
sequence.add(StepId.BOTH_DOWN);
sequence.add(StepId.WRESTLE);
```

so `BOTH_DOWN` is reachable **only** through `GOTO_LABEL_ON_JUGGERNAUT`; if `StepBlockChoice` does
not take that goto, the jump lands on `DROP_FALLING_PLAYERS` and both-down handling is skipped
entirely — which matches every observation (defender already FALLING, attacker never FALLING, one
armour roll, no turnover).

Also newly established: **there are two `StepDropFallingPlayers`** —
`step/action/block/StepDropFallingPlayers` is `@RulesCollection(COMMON)` (so BB2016 **and BB2020**
use it) and its whole body is `getGameState().executeStepHooks(this, state)`, i.e. all the work is
done by `PilingOnBehaviour`'s step hook. `step/bb2025/shared/StepDropFallingPlayers` is a much larger
BB2025-only class that does the job itself. Rust has only the BB2025 port and uses it for every
edition. That is the same "COMMON/BB2020 step vs BB2025 step, Rust wired to BB2025" shape as the last
six fixes, and is the most likely home of the remaining difference.

Instrument still needed: `GameState.startNextStep` already calls
`logCurrentStep(IServerLogLevel.DEBUG, this)` per step, and `HeadlessFantasyFootballServer.getDebugLog()`
is now gated on `FFB_SERVER_DEBUG=1` — the lazy creation fires (`JSTEPALL debugLog created` prints)
but `logCurrentStep` never does, so the harness must be driving steps through a path that does not go
via `startNextStep`. Finding that path is the unlock.

## ITER21 — chaos_pact seed 22: TTM guard mirroring REGRESSED and was reverted

**Seed 22, i=125.** Rust ran 10 dice ahead (`53` vs Java's `43`) with `h05` Injured where Java has it
Standing. An `FFB_RNG_BT=46` backtrace put Rust's first extra die in
`step::action::ttm::util_throw_team_mate_sequence::scatter_player` — Rust performs a Throw Team-Mate
that Java does not.

**What the Java source says (and it is unambiguous):**

| edition | TTM guard in `StepInitSelecting` | `ThrowTeamMateBehaviour` sets |
|---|---|---|
| bb2016 | `!isPassUsed()` (`bb2016/move/…:211`) | `setPassUsed(true)` |
| **bb2020** | **`!isPassUsed() \|\| isKicked()`** (`bb2020/shared/…:274`) | **`setPassUsed(true)`** |
| bb2025 | `!isTtmUsed() \|\| isKicked()` (`bb2025/shared/…:293`) | `setTtmUsed(true)` |

So BB2020 spends the team's PASS action on a Throw Team-Mate exactly as BB2016 does; `ttmUsed` is a
BB2025-only flag. Rust's `legal_actions` gates TTM on `!ttm_used` for every edition, and
ParityRunner's keep-rule special-cases only bb2016.

**Mirroring both guards to `!ttmUsed && !passUsed` for bb2020 REGRESSED the campaign** —
`chaos_pact` 24→23/25 and `human` **100→99**/100 — and did not fix seed 22 at all. Both changes were
reverted and the baseline re-confirmed (`chaos_pact` 24/25, `human` 100/100). The guards above are
real, so something else about how the two agents build their action lists absorbs them; that needs
explaining before either side is touched again.

**The actual seed-22 cause, now visible in the dice.** The TTM is already IN FLIGHT when the Really
Stupid roll fails:

```
pos 44 d6=6  ReallyStupid (pass)
pos 45 d6=1  ReallyStupid (FAIL)
pos 46 d8    ← Rust: scatter_player, i.e. the throw happens anyway
             ← Java: next activation's BoneHead — the throw was abandoned
pos 52 d16   ← Rust: bb2020 casualty roll for the thrown player
```

`generator/bb2020/ThrowTeamMate` line 40 is explicit:

```java
sequence.add(StepId.REALLY_STUPID, from(GOTO_LABEL_ON_FAILURE, IStepLabel.END_THROW_TEAM_MATE));
sequence.add(StepId.TAKE_ROOT);
sequence.add(StepId.UNCHANNELLED_FURY, from(GOTO_LABEL_ON_FAILURE, IStepLabel.END_THROW_TEAM_MATE));
```

A failed Really Stupid jumps straight to `END_THROW_TEAM_MATE` and the throw never happens. Rust's
live `generator/bb2025/throw_team_mate` has **no negatrait steps at all** — no `ReallyStupid`, no
`TakeRoot`, no `UnchannelledFury` — so nothing can abort the throw.

**Next iteration:** establish which TTM generator is live for bb2020 (`generator/bb2020/throw_team_mate`
exists and is referenced from `bb2020/move_/step_end_selecting.rs`, but the bb2025 one appears to be
what actually runs), then give the live sequence the three negatrait steps with
`GOTO_LABEL_ON_FAILURE = END_THROW_TEAM_MATE`, matching Java. Verify the abort consumes no extra dice.

Baseline unchanged: `chaos_pact` 24/25, `human` 100/100.

## ITER22 — chaos_pact seed 22: second wrong premise, reverted. Lesson recorded.

ITER21 claimed Rust's live BB2025 TTM sequence "has no negatrait steps at all". **That was wrong**,
and it was wrong because of a truncated grep (`grep -n "seq.add" … | head -14` on a file where the
negatrait steps are added by a different call). Dumping the sequence from a test shows the truth:

```
BB2025 TTM seq = [InitThrowTeamMate, InitActivation, AnimalSavagery, SteadyFooting,
                  HandleDropPlayerContext, PlaceBall, Apothecary, CatchScatterThrowIn, GotoLabel,
                  BoneHead, ReallyStupid, TakeRoot, UnchannelledFury, BloodLust, AlwaysHungry,
                  ThrowTeamMate, DispatchScatterPlayer, RightStuff, …]
```

The chain is already there. Routing BB2020 to `generator/bb2020/throw_team_mate` was therefore a
no-op in substance — the dice were byte-identical afterwards and the gates were exactly neutral
(`chaos_pact` 24/25, `human` 100/100). Reverted; baseline re-confirmed at `chaos_pact` 24/25.

**Method lesson (two wrong premises in a row on this seed):** both ITER21's TTM-guard theory and
this one were built on reading, not measuring, and both cost a full iteration. The cheap measurement
existed in each case — dump the sequence from a unit test, or diff the dice after the change. Adopt:
*before* editing on the strength of a source reading, take one measurement that would FALSIFY the
premise. The `FFB_DRIVE_TRACE` step list and a one-line `panic!("{:?}")` in a test are both seconds
of work.

**Where seed 22 actually stands.** Still the same signature and still unexplained:

```
pos 44 d6=6  ReallyStupid (pass)     pos 45 d6=1  ReallyStupid (FAIL)
pos 46 Rust d8 = scatter_player      pos 46 Java d6 = the NEXT player's BoneHead
```

Both engines run a `ReallyStupid` step with `GOTO_LABEL_ON_FAILURE = END_THROW_TEAM_MATE` in the TTM
sequence, both roll the same failing die — and Rust still throws. So the divergence is **inside
`StepReallyStupid`**: its failure is not producing the goto. Next iteration should instrument
`step/action/common/step_really_stupid.rs` directly (what `ActionStatus` it computes, and what
`StepOutcome` it returns on failure) rather than reasoning about sequences again.

Baseline unchanged: `chaos_pact` 24/25, `human` 100/100.

## ITER23 — chaos_pact seed 22: measured, not guessed. Two theories killed, guard located.

Applied ITER22's lesson: every claim below is a measurement, and each one killed a theory.

**1. The negatrait-abort theory (ITER21/22) is dead.** A trace on `step_really_stupid.rs` printing
the acting action, failure label and returned outcome shows *every* `ReallyStupid` execution in this
game runs inside the MOVE sequence and behaves correctly:

```
RS_TRACE pid=Some("home_03") action=Some(Move) goto_fail='END_MOVING' outcome=Some((GotoLabel, Some("END_MOVING")))
```

The failing roll at dice pos 45 belongs to a Move activation that Rust ends correctly. It has nothing
to do with the TTM.

**2. Both agents declare the throw.** `FFB_ACT_TRACE` shows Java's own harness picking it:

```
JAVA_ACT_PICK pid=…Home2 N=2 idx=1 action=THROW_TEAM_MATE live=[MOVE,THROW_TEAM_MATE] snapshot=2
```

So the action lists agree, and this is why ITER21's attempt to filter TTM out of `legal_actions`
regressed — removing an entry shifts every later pick index and desyncs the two agents. **The guard
must live in the engine, not the action list.**

**3. The `passUsed` half of the guard is NOT the reason.** Measured at Rust's TTM dispatch:

```
TTM_TRACE pid=Some("home_02") pass_used=false ttm_used=false defender=Some("home_06")
```

Adding the (correct, Java-sourced) `!isPassUsed()` rejection therefore changed nothing here; it was
reverted rather than left in unverified.

**4. Java never reaches `INIT_THROW_TEAM_MATE`.** That activation rolls ZERO dice in Java (rng 46 is
already the next player's Bone Head), and ParityRunner's `INIT_THROW_TEAM_MATE` case would send a
throw target — which would produce dice. So Java rejects the declaration at `StepInitSelecting`.

**Conclusion — the remaining term.** `bb2020/shared/StepInitSelecting:273` guards on TWO conditions:

```java
if (UtilServerSteps.checkCommandWithActingPlayer(getGameState(), throwTeamMateCommand)
    && (!game.getTurnData().isPassUsed() || throwTeamMateCommand.isKicked())) {
```

`passUsed` is out, so the rejection is `checkCommandWithActingPlayer`:

```java
return StringTool.isProvided(cmd.getActingPlayerId())
    && cmd.getActingPlayerId().equals(actingPlayer.getPlayerId());
```

i.e. the TTM command's acting-player id does not match the current `ActingPlayer` — the declaration
is addressed to a player who is not (yet) the acting player, so Java silently drops it and the
activation does nothing. Rust has no such check and proceeds to throw.

**Next iteration:** confirm with a harness probe what `actingPlayerId` ParityRunner puts on the
phase-2 `ClientCommandThrowTeamMate` versus `game.getActingPlayer().getPlayerId()` at that moment,
then mirror the mismatch outcome in Rust's `StepInitSelecting` (activate, do nothing, no dice) —
gated on the same condition Java uses, not on a special case for this seed.

Baseline unchanged: `chaos_pact` 24/25, `human` 100/100. No engine changes kept this iteration.

## ITER24 — ROOT CAUSE: Rust's skill properties are edition-agnostic (BB2020 Right Stuff)

Measured the whole way down. A gated `FFB_TTM_DUMP` probe added to `ParityRunner.sendThrowTeamMateAction`
prints every candidate teammate and why it was rejected:

```
TTM_DUMP thrower=…Home2 coord=(12,6) nTargets=0 cands=…Home1[thrownOk=false,state=1,…]
         …Home6[thrownOk=false,…]   ← the Goblin Renegade, which HAS Right Stuff
```

**`thrownOk=false` for every player**, including the Goblin. Java's BB2020 has no throwable players at
all on this team, so the harness deselects and the activation rolls no dice. Rust offers `home_06` and
throws. That is the entire seed-22 divergence.

**Why.** `NamedProperties` registered by `RightStuff.postConstruct` differ per edition:

| edition | properties |
|---|---|
| `skill/bb2016/RightStuff` | `canBeThrown`, `canBeKicked`, `ignoreTackleWhenBlocked` |
| **`skill/bb2020/RightStuff`** | **`canBeThrownIfStrengthIs3orLess`**, `ignoreTackleWhenBlocked` |
| `skill/bb2025/RightStuff` | `canBeThrown`, `ignoreTackleWhenBlocked` |

Rust's `SkillId::properties()` (`crates/ffb-model/src/enums/skill_id.rs:827`) is **edition-agnostic**
and returns the UNION:

```rust
SkillId::RightStuff => &["canBeThrown", "canBeKicked", "ignoreTackleWhenBlocked",
                         "canBeThrownIfStrengthIs3orLess"],
```

so a BB2020 Right Stuff player wrongly answers `true` to `canBeThrown` (and to `canBeKicked`, which
only BB2016 grants). `legal_throw_team_mate_targets` checks exactly `CAN_BE_THROWN`, so Rust builds a
non-empty target list where Java builds an empty one.

**Scope — this is bigger than one seed.** The union is applied to every skill, and Right Stuff alone
touches `chaos_pact`, `goblin`, `halfling`, `ogre`, `underworld` and `skaven` — most of the current
heavy-failure list (`goblin` 14, `halfling` 25, `ogre` 25, `underworld` 3). Other skills whose
property sets differ by edition will have the same problem.

**Why it was not fixed in this iteration.** `properties()` has **172 call sites** and no `rules`
parameter. Making it edition-aware is a deliberate refactor, not something to rush: the options are
(a) `properties_for(rules)` plus a threaded `rules` at the call sites that matter, or (b) per-edition
property tables selected once at skill-registry build time (the same shape as `SkillRegistry`, and
probably the closer 1:1 to Java's per-edition skill classes). Option (b) looks right — Java's
properties live on the per-edition skill class, exactly as its behaviours do.

Recommended next iteration: implement (b) for `RightStuff` first, verify `chaos_pact` seed 22 and the
`goblin`/`halfling`/`ogre`/`underworld` fail counts, then sweep the remaining skills whose Java
`postConstruct` differs across editions (a mechanical diff of `skill/bb2016|bb2020|bb2025/*.java`).

Harness keeps the gated `FFB_TTM_DUMP` probe. Baseline unchanged: `chaos_pact` 24/25, `human` 100/100.
No Rust changes this iteration.

## ITER25 — per-edition skill properties: chaos_pact 🟢, ogre 🟢, goblin 11→24, halfling 0→2

Implements ITER24's root cause. Java registers a skill's `NamedProperties` in the **per-edition**
skill class' `postConstruct`, and `RightStuff` genuinely differs:

| Java class | registered properties |
|---|---|
| `skill/bb2016/RightStuff` | `canBeThrown`, `canBeKicked`, `ignoreTackleWhenBlocked` |
| **`skill/bb2020/RightStuff`** | **`canBeThrownIfStrengthIs3orLess`**, `ignoreTackleWhenBlocked` |
| `skill/bb2025/RightStuff` | `canBeThrown`, `ignoreTackleWhenBlocked` |

Rust's `SkillId::properties()` returned the edition-agnostic UNION, so a BB2020 Right Stuff player
answered `true` to `canBeThrown` — and `ParityRunner.sendThrowTeamMateAction` filters on exactly that
property, so **BB2020 has no throwable players at all** while Rust built a non-empty list.

**Changes** (deliberately narrow — `properties()` has 172 call sites, so it keeps the union):
* `SkillId::properties_for(rules)` — per-edition arms, everything else falls through to the union.
  One arm per skill whose Java `postConstruct` diverges; `RightStuff` is the first.
* `Player::has_skill_property_in(rules, prop)` — the edition-aware `hasSkillProperty`, for callers
  that hold a `Game`.
* `legal_throw_team_mate_targets` uses it for the `CAN_BE_THROWN` filter.

**Results:**

| roster | before | after |
|---|---|---|
| `chaos_pact` | 24/25 | **25/25** 🟢 |
| `ogre` | 0/25 | **25/25** 🟢 |
| `goblin` | 11/25 | **24/25** |
| `halfling` | 0/25 | 2/25 |
| `underworld` | 22/25 | 22/25 (unchanged) |

Gates: `human` bb2020 100/100, `lineman` bb2016 100/100, `lineman` bb2025 100/100,
`cargo test --workspace` 14/14 suites ok.

Tests: `right_stuff_properties_are_edition_specific` pins all three editions (including the negative
— bb2020 must NOT grant `canBeThrown`), and `properties_for_falls_through_to_the_union_for_other_skills`
guarantees the new function cannot silently change any other skill.

**Follow-up:** sweep the remaining skills whose Java `postConstruct` differs across editions — a
mechanical diff of `skill/bb2016|bb2020|bb2025/*.java` — and add an arm for each. That is the likely
source of several remaining failures (`halfling` is still 2/25, so it has at least one more).

## ITER26 — the complete per-edition property table (13 skills, 37 arms)

ITER25 fixed `RightStuff` and flagged the sweep. Extracted every divergence mechanically from
`skill/bb2016|bb2020|bb2025/*.java` by parsing `registerProperty(NamedProperties.X)`:

**13 skills whose Java `postConstruct` differs across editions:**
`BallAndChain`, `Bombardier`, `Chainsaw`, `CloudBurster`, `HypnoticGaze`, `Leap`, `MonstrousMouth`,
`PilingOn`, `Regeneration`, `RightStuff`, `SneakyGit`, `Stab`, `Swoop`.

All 37 arms are now in `SkillId::properties_for`, generated from the extraction rather than
transcribed. Highlights of what the union was getting wrong **for BB2020**:

| skill | union wrongly grants BB2020 | truth |
|---|---|---|
| `PilingOn` | `canPileOnOpponent` | BB2020 registers **nothing** (BB2016-only property) |
| `HypnoticGaze` | `canGazeDuringMove` | BB2016-only |
| `MonstrousMouth` | `canPinPlayers`, `providesBlockAlternative` | BB2025-only |
| `CloudBurster` | `passesAreNotIntercepted` | BB2025-only; BB2020 has `canForceInterceptionRerollOfLongPasses` |
| `BallAndChain` | `forceFullMovement`, `grabOutsideBlock`, `flipSameTeamOpponentToOtherTeam` | BB2016-only |
| `BallAndChain` | `preventSecureTheBallAction` | BB2025-only |
| `Chainsaw` | `makesStrengthTestObsolete`, `needsNoDiceDecorations` | BB2016-only |

**Roster counts are unchanged this iteration** (`chaos_pact` 25/25, `ogre` 25/25, `goblin` 24/25,
`halfling` 2/25, `underworld` 22/25) — and that is expected: only `legal_throw_team_mate_targets`
consumes `has_skill_property_in` so far. Every other site still asks the union. The table is correct,
tested groundwork; converting consumers is what turns it into parity.

Gates: `human` bb2020 100/100, `lineman` bb2016 100/100, `lineman` bb2025 100/100,
`cargo test --workspace` 14/14 suites ok.

Test `every_edition_divergent_skill_is_tabled` spot-checks one representative divergence per skill
(both the positive and the negative), so a future edit cannot quietly drop one;
`properties_for_falls_through_to_the_union_for_other_skills` still guards everything else.

**Next:** convert consumers, highest-value first. `canPileOnOpponent` is the standout — BB2020 grants
it to nobody, yet Rust's Piling On hook checks `hasUnusedSkillWithProperty(canPileOnOpponent)` through
the union, so Rust may be offering Piling On in games where Java never can. `canGazeDuringMove` and
the Ball & Chain / Chainsaw extras are the same shape.

## ITER27 — consumer conversion: measured the follow-up down to almost nothing

ITER26 predicted `canPileOnOpponent` would be the high-value consumer. **Measuring first killed that**:
`Piling On` appears in **0** bb2020 rosters. Checking every divergent skill against the drafted
rosters narrows the whole follow-up sharply:

| skill | in bb2020 rosters | does the union mis-grant BB2020? |
|---|---|---|
| `PilingOn`, `MonstrousMouth`, `CloudBurster`, `SneakyGit`, `Swoop` | **none** | unreachable |
| `Regeneration` (11), `Leap` (5), `Stab` (2), `Bombardier` (1) | yes | **no** — BB2020's set is the superset, so the union is already right for BB2020 |
| `HypnoticGaze` (vampire) | yes | yes, but the one consumer is already hand-gated to `Rules::Bb2016` |
| `BallAndChain`, `Chainsaw` (goblin) | yes | **yes** |

So the only live mis-grant left was Ball & Chain's `grabOutsideBlock` (BB2016-only) reaching the
goblin Fanatic in BB2020/BB2025. Converted the three `grab_behaviour.rs` consumers to
`has_skill_property_in(game.rules, …)`. Chainsaw's BB2016-only `makesStrengthTestObsolete` /
`needsNoDiceDecorations` and Ball & Chain's other edition-specific properties have **no consumers at
all** in Rust yet, so nothing to convert.

**Roster counts unchanged** (`goblin` 24/25, `chaos_pact` 25/25). Kept anyway: it is a faithful 1:1,
it is the difference the property table exists to express, and it is now pinned by
`has_skill_property_in_is_edition_aware`, which asserts a BB2020 Fanatic answers **false** while the
union-based accessor still answers true. Gates: `human` bb2020 100/100, `lineman` bb2016 100/100,
`lineman` bb2025 100/100, `chaos_pact` 25/25, `cargo test --workspace` 14/14.

**Goblin seed 16 is a different bug** — measured, not related to skill properties. The dice are
byte-identical for 100 rolls, then diverge:

```
JAVA rng=101 d8=7  rollScatterDirection <- StepCatchScatterThrowIn.bounceBall:680
RUST pos=101 d6                          ← no bounce at all
```

Java rng 96-100 are a block (block die, 2× armour, 2× injury); the knocked-down player was holding
the ball, so Java bounces it. Rust skips the bounce. This is the same shape as the BB2016 fix
recorded in `util_server_injury.rs` — where the ball handling (`DROPPED_BALL_CARRIER`,
`SCATTER_BALL`, turnover) must sit OUTSIDE the `placedProneCausesInjuryRoll` if/else so it runs for a
Ball & Chain player too. Next iteration should check whether the BB2020 block path returns early
before that ball handling.

**Method note:** an earlier "0 consumers" count in this session was wrong because the shell `cd` had
reset — always `cd` explicitly in each command before grepping, and re-check a zero result that
contradicts an earlier grep.

## ITER28 — goblin seed 16: the die-101 bounce is a SYMPTOM; the agents desync earlier

Chased ITER27's lead and it dissolved under measurement — recording that so the next iteration does
not re-chase it.

**The lead:** at die 101 Java rolls `d8 rollScatterDirection <- StepCatchScatterThrowIn.bounceBall`
and Rust does not. An `FFB_RNG_BT=101` backtrace shows Rust's 101 comes from
`step::bb2025::block::step_block_roll` — Rust is rolling *another block die* there, not skipping a
bounce.

**Working backwards, the dice VALUES agree for 100 rolls but the CALLERS do not:**

| die | Java | Rust |
|---|---|---|
| 93, 94, 95 | three `ReallyStupidBehaviour` rolls | 93 only; 94/95 are block dice |
| 96 | ONE block die (`rollBlockDice`) | armour |
| 101 | ball bounce | a new block die |

Rust's own prompt at that block reads
`BlockChoice { attacker_id: "away_01", defender_id: "home_02", dice: [6, 4], nr_of_dice: 2 }` —
`away_01` is the `goblin.troll` (ST 5) and `home_02` the `goblin.pogoer` (ST 2). **Neither engine's
count makes sense for that pairing** (ST 5 vs ST 2 should be three dice, and Java's single die even
fewer), which is the tell that the two engines are no longer blocking the same pairing at all.

**Confirmed:** comparing `JAVA_ACT_PICK` against `RUST_ACT_PICK` shows the activation streams desync
well before the block — Java made 305 picks, Rust 296, and by pick 174 they are on different players
entirely (`Home4` with `live=[MOVE,PASS,HAND_OVER,THROW_BOMB]` vs `home_01` with `N=2`). The
identical dice VALUES through 100 are coincidence, not alignment.

**Also eliminated:** `ignoreBlockAssists` (the BB2020/BB2025-only Ball & Chain property) has **zero
consumers** in Rust, so it cannot explain a dice-count difference.

**Next iteration:** find the first pick where the two agents' *action lists* differ (`N` and the
`live=[…]` set), not the first where the chosen action differs — the list is what drives the index,
and `THROW_BOMB` in Java's list at pick 174 suggests a Bombardier-related availability difference
worth checking first. Compare `JAVA_ACT_PICK`/`RUST_ACT_PICK` pairwise on `N` from pick 0.

Baseline unchanged: `goblin` 24/25, `chaos_pact` 25/25. No engine changes this iteration.

## ITER29 — goblin seed 16: narrowed to a step-sequence divergence in dice 83–94

Continued ITER28. Established, and each of these is a measurement:

* **The activation streams do not desync until pick 171**, and both engines make the *same ten* away
  activations in the turn before it (`[7,11,6,12,2,1,9,8,10,4]`). Java then ends the turn; Rust
  activates an eleventh (`away_05`). Java simply had one fewer available player.
* That traces to the earlier state diff — `a03` is **KO in Java, Standing in Rust** — so Rust is
  missing an injury Java applies, which is what leaves Rust the extra activation.
* **Dice VALUES are identical for 100 rolls** and first differ at 101. That is coincidence, not
  alignment: by die 94 the two engines are demonstrably in *different steps*. Reading Java's raw
  `from=` chains, dice 93/94/95 are three `ReallyStupidBehaviour` rolls; Rust's `FFB_DRIVE_TRACE`
  shows 93 in `ReallyStupid` but 94/95 in `BlockRoll`.
* Rust's own prompt at that block is
  `BlockChoice { attacker_id: "away_01", defender_id: "home_02", dice: [6,4], nr_of_dice: 2 }` —
  `goblin.troll` (ST 5) vs `goblin.pogoer` (ST 2), a pairing whose die count fits neither engine,
  confirming they are not blocking the same pair.

So the divergence is a **step-sequence difference somewhere in dice 83–94**, upstream of both the
injury and the block. Java's dice in that window are a mix of `StepMoveDodge.dodge`,
`InjuryTypeDropDodge` armour/injury and `ReallyStupidBehaviour`; Rust's steps there are
`ReallyStupid`, `MoveDodge` and `BlockRoll` in a different order.

**Tooling note for the next iteration.** I wrote a per-die caller comparison (nearest preceding
`DRIVE step=` on the Rust side vs the Java `from=` chain) and it produced unusable output — the Java
frame extractor kept falling back to `DiceRoller.rollDice:98` instead of the first meaningful
`com.fumbbl.ffb.server.step.*` / `*Behaviour` frame, so almost every row flagged a false mismatch.
**Fix that extractor first** (take the first frame in the chain that is neither `DiceRoller` nor
`Fortuna`, and keep its class name), then re-run it over dice 80–100. With a correct extractor this
comparison lands the exact step where the two sequences part, which is what this seed needs — the
value stream is useless here because the values coincide across the divergence.

Baseline unchanged: `goblin` 24/25. No engine changes this iteration.

## ITER30 — goblin seed 16 ISOLATED: one dodge roll, same dice, opposite outcome

Fixed the caller-comparison tool from ITER29 (the Java frames live in the `FFB_DICE_DEEP` capture,
the Rust steps in the `FFB_DRIVE_TRACE` capture — they must be joined across the two logs) and it
lands the divergence exactly.

**Recipe, worth keeping:** run the seed twice, once with `FFB_TRACE=1 FFB_DICE_TRACE=1
FFB_DICE_DEEP=1` and once with `FFB_TRACE=1 FFB_DRIVE_TRACE=1 FFB_DICE_TRACE=1`; join on the die
position; take the Java frame as the first entry in the `from=` chain that is neither `DiceRoller`
nor `Fortuna`, and the Rust step as the nearest preceding `DRIVE step=`.

```
   85  java d6=5  StepMoveDodge.dodge                 rust d6=5  MoveDodge     ✓
   86  java d6=2  StepMoveDodge.dodge                 rust d6=2  MoveDodge     ✓
   87  java d6=3  StepMoveDodge.dodge                 rust d6=3  MoveDodge     ✓
   88  java d6=6  InjuryTypeDropDodge.handleInjury    rust d6=6  ReallyStupid  ← FIRST DIVERGENCE
   89  java d6=5  InjuryTypeDropDodge.handleInjury    rust d6=5  BlockRoll
   90  java d6=3  InjuryTypeDropDodge.handleInjury    rust d6=3  ReallyStupid
   91  java d6=4  InjuryTypeDropDodge.handleInjury    rust d6=4  BlockRoll
```

Everything matches through die 87. On the **third dodge (die 87 = 3)** Java **fails** — falls into
`InjuryTypeDropDodge` and rolls armour (88/89) then injury (90/91) — while Rust **succeeds** and
carries straight on to the next activation. The dodging player is `home_01`, the `goblin.troll`.

That single failed dodge is the whole seed: it is what KOs `a03` in Java, which is why Java has one
fewer available player, ends the away turn an activation early, and diverges from there.

**Next iteration — compare the dodge computation directly**, not the dice. BB2020 dodge is
`d6 + 1 − (opposing tackle zones on the destination square)` against the dodger's AG target, so the
two engines disagree on either the AG target for `goblin.troll` or the tackle-zone count on that
square. Both are cheap to dump: add a gated trace to `step_move_dodge.rs` printing player, from/to
square, AG target, modifier list and total, and compare against Java's
`mechanics/bb2020/DodgeMechanic` (and note `preventStuntyDodgeModifier` is one of the properties that
differs by edition — `Chainsaw`, `Bombardier` and `Swoop` carry it in BB2020, and the goblin roster
has a Chainsaw Looney and a Bombardier, so an edition-agnostic property lookup could be reaching the
dodge modifiers).

Baseline unchanged: `goblin` 24/25. No engine changes this iteration.

## ITER31 — goblin seed 16 traced to the dodge target; ITER26's extraction had a GAP

Instrumented the dodge computation (gated `FFB_DODGE_TRACE` in `bb2025/move_/step_move_dodge.rs`,
now reverted) and measured the failing roll:

```
DODGE_TRACE pid=away_04 from=(13,8) to=(13,9) ag=3 min=3 roll=3 mods=[]
DODGE_TRACE   neighbours=["home_04@12,8 base=1 tz=true"] other_team=home util_adjacent=["home_04"]
```

Rust needs **3**, so the 3 passes. Java needs **4**, so it fails and the player falls. Everything
else agrees: same square, same opponent adjacency (`UtilPlayer::find_adjacent_players_with_tacklezones`
does return `home_04`), same AG value.

**What was ruled out, by measurement not reading:**
* the acting player is set when the context is built (`acting_player_id=Some("away_04")`);
* `find_other_team` returns the right team;
* the tackle-zone helper finds the marking opponent;
* the modifier collection is populated (8 TACKLEZONE + 8 PREHENSILE_TAIL);
* `DodgeContext::new`'s source/target argument order is correct.

`mods=[]` is therefore not a lookup failure — Rust is deliberately skipping the tackle-zone modifier
because `find_applicable` checks `ignoreTacklezonesWhenDodging`, and the dodger is a Stunty goblin.

**And that exposed a gap in ITER26's sweep.** The extraction scanned only
`skill/bb2016|bb2020|bb2025/`, but the skill tree also has **`skill/mixed/` and `skill/common/`** —
and `skill/mixed/Stunty.java` is one of the classes registering `ignoreTacklezonesWhenDodging`
(alongside `bb2016/Stunty`, `bb2016/SecretWeapon`, `bb2016/Swoop`, and `bb2020/Bombardier`,
`bb2020/Chainsaw`, `bb2020/Swoop`). Any skill whose per-edition class is absent and which falls back
to `mixed/` was invisible to that diff, so **the 13-skill table may be incomplete**.

**Next iteration:**
1. Re-run the `registerProperty` extraction over **all five** directories (`bb2016`, `bb2020`,
   `bb2025`, `mixed`, `common`), resolving each edition to its own class if present and the `mixed`/
   `common` fallback otherwise — that is what Java's per-edition skill factory actually does. Add any
   newly-found divergences to `SkillId::properties_for`.
2. Then settle this dodge specifically: with Stunty granting `ignoreTacklezonesWhenDodging` in the
   mixed class, BOTH engines should skip the tackle-zone modifier — so Java's minimum of 4 must come
   from somewhere else (a different AG target for the BB2020 goblin, or a modifier Rust does not
   model). Dump Java's side via `mechanics/bb2020/DodgeMechanic.minimumRoll` before changing anything.

Baseline unchanged: `goblin` 24/25. No engine changes this iteration.

## ITER32 — the property table completed and made self-checking (20 skills, 65 arms)

Fixed both gaps ITER31 exposed in my own ITER26 extraction:

1. **Directory coverage.** The skill tree is `skill/{bb2016,bb2020,bb2025,mixed,common}/`, and Java's
   per-edition factory resolves a skill to its own class if one exists, else `mixed/`, else
   `common/`. Scanning only the three edition directories missed every skill with no per-edition
   class. Re-running with proper fallback resolution finds **20** divergent skills, not 13 — seven
   more: `Decay`, `DivingTackle`, `Juggernaut`, `MultipleBlock`, `PrehensileTail`, `SecretWeapon`,
   **`Stunty`**.
2. **Registration shape.** The generator matched only `registerProperty(NamedProperties.X)` and
   missed `registerProperty(new CancelSkillProperty(NamedProperties.X))` (Rust models these as
   `cancelsX`). That silently dropped 29 properties from the tabled arms. Both shapes are now
   captured; the table is 65 arms.

**Self-checking.** New invariant test `tabled_properties_never_invent_anything_the_union_lacks`
walks every tabled skill × every edition and asserts each property also appears in `properties()`.
It reports ALL violations rather than panicking on the first — and it immediately earned its keep by
flagging `Regeneration/preventRaiseFromDead`. That turned out to be a **deliberate** trim, not a bug:
`properties()` is consumed edition-agnostically and a BB2025 Regeneration player must stay raisable
(`regeneration_does_not_prevent_raise_from_dead_bb2025`, necromantic seed 89). I reverted my
"correction" to the union and whitelisted that single case in the invariant with the reasoning
inline, so a future generator omission is still caught. **The exemption disappears once the remaining
consumers move to `properties_for`.**

**Also settled:** `Stunty` grants `ignoreTacklezonesWhenDodging` in **all three** editions, so Rust's
empty dodge-modifier list for the Stunty goblin in seed 16 is CORRECT, not a lookup failure. Java's
minimum of 4 must come from somewhere else — read `mechanics/bb2020/DodgeMechanic.minimumRoll` next
and dump its modifier list, rather than assuming a tackle-zone difference.

Roster counts unchanged (`chaos_pact` 25/25, `ogre` 25/25, `goblin` 24/25) — expected, since only two
consumers read `properties_for`. Gates: `human` bb2020 100/100, `lineman` bb2016 100/100, `lineman`
bb2025 100/100, `cargo test --workspace` 14/14 suites (2788 ffb-model tests).

## ITER33 — goblin 🟢 25/25: Rust ignored Java's skill-property CANCELLATION

The dodge chase lands. Java's `DodgeModifierFactory`:

```java
protected boolean isAffectedByTackleZones(DodgeContext context) {
    return !UtilCards.hasUncanceledSkillWithProperty(context.getPlayer(),
        NamedProperties.ignoreTacklezonesWhenDodging);
}
```

**`hasUncanceledSkillWithProperty`** — the player must have the property AND no skill of theirs may
cancel it:

```java
return Arrays.stream(skills).anyMatch(s -> s.hasSkillProperty(property))
    && Arrays.stream(skills).flatMap(s -> s.getSkillProperties().stream())
       .noneMatch(sp -> sp instanceof CancelSkillProperty && sp.cancelsProperty(property));
```

Rust used a plain `has_skill_property`, so cancellation was never honoured anywhere.

**The seed-16 dodger is `away_04` = `goblin.bombardier`.** Its Stunty grants
`ignoreTacklezonesWhenDodging`; its own `skill/bb2020/Bombardier` registers
`CancelSkillProperty(ignoreTacklezonesWhenDodging)`. Java therefore applies the tackle-zone modifier
for the marking `home_04` — minimum **4**, and the rolled 3 fails, the player falls, and `a03` is
KO'd. Rust skipped the modifier — minimum **3**, the 3 passes — which left Rust an extra available
player, an extra activation, and everything that cascaded from it.

**Fix.** `Player::has_uncanceled_skill_property_in(rules, prop)` — a 1:1 port; Rust already models
`CancelSkillProperty(X)` as the pseudo-property `cancelsX` (the shape ITER32 taught the generator to
capture), so the cancel test is a lookup for that name in the per-edition set. The dodge factory's
`isAffectedByTackleZones` now uses it.

**Results:**

| roster | before | after |
|---|---|---|
| `goblin` | 24/25 | **25/25** 🟢 |
| `chaos_pact`, `ogre` | 25/25 | 25/25 🟢 |
| `halfling` | 2/25 | 2/25 |
| `underworld` | 22/25 | 22/25 |

Gates: `human` bb2020 100/100, `lineman` bb2016 100/100, `lineman` bb2025 100/100,
`cargo test --workspace` 14/14 suites.

Test `uncanceled_skill_property_respects_cancellation` pins all three cases, including the trap: the
plain check still answers `true` for the Bombardier while the uncancelled one answers `false`.

**Follow-up:** `hasUncanceledSkillWithProperty` and `hasSkillToCancelProperty` are used at other Java
call sites too. Grep them and convert the matching Rust checks — every one is a latent version of
this bug.

## ITER34 — the sibling cancel-check ported; halfling's first divergence located

Followed up ITER33's note. Java calls the cancellation-aware checks at a dozen sites:

```
DodgeModifierFactory:117  hasUncanceledSkillWithProperty(player, ignoreTacklezonesWhenDodging)   ← done, ITER33
JumpModifierFactory:91    hasSkillToCancelProperty(player, makesJumpingHarder)
FoulAppearanceBehaviour   hasSkillToCancelProperty(player, forceRollBeforeBeingBlocked)   x3
PilingOnBehaviour:138     hasSkillToCancelProperty(player, canPileOnOpponent)
StepEndBlocking:130       hasSkillToCancelProperty(player, canBlockMoreThanOnce)
StepHypnoticGaze          hasSkillToCancelProperty(player, inflictsConfusion)            x3
StepJump                  hasSkillToCancelProperty(player, canAttemptToTackleJumpingPlayer) x2
InjuryMechanic:42         hasSkillToCancelProperty(deadPlayer, allowsRaisingLineman)
UtilPassing:31            hasSkillToCancelProperty(otherPlayer, passesAreNotIntercepted)
```

Added `Player::has_skill_to_cancel_property_in(rules, prop)` — the 1:1 sibling of ITER33's helper,
asking only whether SOME skill cancels the property (the player need not have it) — and converted the
`JumpModifierFactory` site, which was missing Java's outer guard entirely:

```java
if (!UtilCards.hasSkillToCancelProperty(context.getPlayer(), NamedProperties.makesJumpingHarder)) {
    prehensileTailModifier(...).ifPresent(modifiers::add);
}
```

i.e. the JUMPING player's own skills can cancel the prehensile-tail penalty outright, regardless of
how many opponents carry it.

**No roster movement** (`halfling` 2/25, `underworld` 22/25, `goblin` 25/25) — measured, not assumed:
nothing in the current bb2020 rosters cancels `makesJumpingHarder`. Kept because it is a faithful 1:1
of a named Java guard and is pinned by `skill_to_cancel_property_does_not_require_having_it`, which
checks that the BB2020 Bombardier cancels without granting, and that BB2025's does not cancel at all.
The remaining call sites above are still unconverted — each is a latent copy of the ITER33 bug.

Gates: `human` bb2020 100/100, `lineman` bb2016 100/100, `lineman` bb2025 100/100,
`cargo test --workspace` 14/14 suites.

**Next target — halfling (2/25).** First dice divergence on seed 1 is at die 67:

```
java 63 d6=4 StepBlockRoll.executeStep
java 64 d6=3 TakeRootBehaviour$1.handleExecuteStepHook
java 65 d6=2 StepMoveDodge.dodge
java 66 d6=3 TakeRootBehaviour$1.handleExecuteStepHook
java 67 d6=1 StepBlockRoll.executeStep     <- rust rolls a d8 here instead
```

Java is rolling Take Root for the halfling Treemen between blocks; Rust reaches a d8 (a scatter) at
the same position. Use the ITER30 two-log join recipe to get Rust's step at 63-67 and find where the
sequences part.

## ITER35 — halfling seed 1: narrowed to die 65, three hypotheses eliminated

Two-log join (ITER30 recipe) on halfling seed 1. Alignment is exact through die **64**; at **65** the
VALUE still matches but the callers part:

```
  63  java d6=4  StepBlockRoll.executeStep                     rust d6=4  BlockRoll
  64  java d6=3  TakeRootBehaviour$1.handleExecuteStepHook     rust d6=3  TakeRoot
  65  java d6=2  StepMoveDodge.dodge                           rust d6=2  TakeRoot   <- sequences part
  66  java d6=3  TakeRootBehaviour$1.handleExecuteStepHook     rust d6=3  TakeRoot
  67  java d6=1  StepBlockRoll.executeStep                     rust d8=1  KickoffScatterRoll
```

Java: Take Root (64) → the same player moves and **dodges** (65) → another Take Root (66) → a block.
Rust: three consecutive Take Roots (64, 65, 66), no dodge, and by 67 the drive has already ended and
Rust is kicking off. So after the Take Root at 64 Java's Treeman moves and Rust's does not.

**Eliminated by measurement, not reading:**
* **The Take Root threshold.** All three Java editions use `minimumRollConfusion(true)`, so a rolled
  3 cannot pass in one engine and fail in the other.
* **The once-per-activation guard.** Rust's live step (`bb2025/shared/step_take_root.rs`) gates on
  `game.acting_player.used_skills`, which is per-ACTIVATION and matches Java's
  `UtilCards.hasUnusedSkill(actingPlayer, skill)`. (The persistent `Player.used_skills` trap recorded
  in the TTM tier does NOT apply here.)
* **An extra Take Root in the Foul sequence.** `generator/bb2020/{Foul,Move,Block}` each contain
  `StepId.TAKE_ROOT` exactly once, so Rust rolling Take Root on a Foul activation (observed:
  `TR_TRACE pid=home_02 action=Some(Foul)`) is correct.

A gated `FFB_TR_TRACE` in the live step (added, measured, reverted) shows Rust rolling Take Root for
`away_01`, `away_02`, `home_01`, `home_02` across Move/Blitz/Foul.

**Next measurement:** correlate `TR_TRACE` with the dice positions in ONE capture (print the die
counter inside the trace line, or interleave with `FFB_DICE_TRACE` unbuffered) to name which player
rolls at 64/65/66 on each side. Java's 64 and 66 are Take Root for two DIFFERENT activations with a
dodge between; if Rust's 64 and 65 are the SAME player, Rust is re-rolling Take Root within one
activation and the guard is being reset — if they are different players, Rust simply activated a
third Treeman where Java moved the first.

Baseline unchanged: `halfling` 2/25. No engine changes this iteration.

## ITER36 — switched to the protocol-correct roster; underworld seed 2 localised

**Process correction.** The protocol says pick the roster with the FEWEST fails. Among the non-green
bb2020 rosters that is `underworld` (3 fails), not `halfling` (23) — I had been chasing halfling for
two iterations. Switched.

**Halfling, closing the open question first (measured, then dropped):** a `TR_TRACE` carrying the die
counter shows Rust's dice 64/65/66 are Take Root for **three different players**
(`away_01`, `away_02`, `home_01`), so the once-per-activation guard is NOT being reset. Rust simply
activated three players in a row while Java's die-64 roller went on to move and dodge. The remaining
question there is why Rust's `away_01` does not move after passing Take Root — an agent/legal-move
difference, not a Take Root bug.

**Underworld seed 2** (two-log join). Alignment is exact through die **84**:

```
  83 java d6=1  InjuryTypeDropDodge.handleInjury               rust d6=1  DropFallingPlayers
  84 java d6=1  AnimalSavageryBehaviour$1.handleExecuteStepHook rust d6=1  AnimalSavagery
  85 java d6=3  StepMoveDodge.dodge                            rust d6=3  EndTurn      <- sequences part
  86 java d6=5  InjuryTypeDropDodge.handleInjury               rust d8=7  KickoffScatterRoll
```

Both engines roll Animal Savagery at 84 and both get **1** (a failure). Java then continues the
activation — the player moves and dodges at 85. Rust instead ends the turn, and by 86 the drive is
over and it is kicking off.

So the divergence is **what a failed Animal Savagery does in BB2020**. Note the harness has a
dedicated mandatory-choice path for this (ParityRunner's `PLAYER_CHOICE` arm treats `ANIMAL_SAVAGERY`
as min-(x,y) rather than declining, because the dialog is min=1/max=1), so the next step is to read
`skillbehaviour/bb2020/AnimalSavageryBehaviour.handleExecuteStepHook` against Rust's
`step_animal_savagery.rs` — specifically what each does on failure when the player has fewer than two
adjacent team-mates to lash out at.

Baseline unchanged: `underworld` 22/25, `halfling` 2/25. No engine changes this iteration.

## ITER37 — underworld seed 2: AS failure paths match; caller-diff heuristic needs care

Read Java's `skillbehaviour/bb2020/AnimalSavageryBehaviour` failure path in full:

```java
if (players.isEmpty()) {                       // no adjacent team-mate to lash out at
    cancelPlayerAction(step, false);
    targetSelectionState.failed();
    step.publishParameter(new StepParameter(END_PLAYER_ACTION, true));
    step.getResult().setNextAction(GOTO_LABEL, state.goToLabelOnFailure);
} else if (players.size() == 1) { lashOut(...); }
  else { showDialog(DialogPlayerChoiceParameter(ANIMAL_SAVAGERY, ..., 1, 1)); }
```

Rust's `mixed/shared/step_animal_savagery.rs` lines 260-279 are the same shape — `players.is_empty()`
→ `StepOutcome::goto(goto_label_on_failure).publish(EndPlayerAction(true))`. **The no-target failure
path is NOT the divergence**; both end the ACTIVATION, not the turn.

**Caution recorded about the tooling.** I extended the two-log join to flag the first CALLER mismatch
via a Java-frame → Rust-step mapping table, and it reported die 14 (`InjuryTypeBlock.armourRoll` vs
`AnimalSavagery`). That is a **false positive**: Rust resolves the Animal Savagery lash-out injury
INSIDE the `AnimalSavagery` step, so the nearest-preceding-`DRIVE step=` attribution names the outer
step while Java's stack names the inner injury type. The heuristic is only sound where the two step
decompositions correspond one-to-one, which they do not in general. Use it to LOCATE candidates, then
confirm each by hand — do not treat its first hit as the answer.

**Where seed 2 actually stands.** First VALUE divergence is die 86. At die 84 both engines roll Animal
Savagery and both fail; at 85 the value still matches but Java is in `StepMoveDodge.dodge` (a further
activation) while Rust is in `EndTurn` — Rust has no activation left. That is the same
"one fewer/more available player" shape as goblin seed 16 (ITER29-33), so the productive next step is
to find which player Java still has available and Rust does not, by diffing the per-turn activation
lists (`JAVA_ACT_PICK` vs `RUST_ACT_PICK`) for the turn ending at die 85 — the technique that cracked
goblin.

Baseline unchanged: `underworld` 22/25. No engine changes this iteration.

## ITER38 — underworld seed 2 ROOT-CAUSED: the Animal Savagery lash-out injury is not applied

Went back further than ITER37. The first STATE divergence is **i=56**, well before the die-86 value
divergence:

```
JSTEP i=56 rng_calls=55 chosen=Activate(Away1,FOUL)   h02:-1,-1,Ko
RUST_STEP i=56 rng_calls=56 chosen=Activate(away_01,Foul)  h02:-1,-1,Standing
```

`h02` is **KO in Java, Standing in Rust** — and both have it OFF the pitch (`-1,-1`), i.e. Rust boxed
the player without ever changing its state base.

The two-log join over dice 47-57 shows both engines rolling the identical sequence:

```
  48 java d6=1  AnimalSavageryBehaviour   rust d6=1  AnimalSavagery   ← AS fails
  49 java d6=6  InjuryTypeBlock.armourRoll  rust d6=6  AnimalSavagery  ┐ armour 6+6 = 12, broken
  50 java d6=6  InjuryTypeBlock.armourRoll  rust d6=6  AnimalSavagery  ┘
  51 java d6=3  InjuryTypeBlock.injuryRoll  rust d6=3  AnimalSavagery  ┐ injury 3+5 = 8 = KNOCKED OUT
  52 java d6=5  InjuryTypeBlock.injuryRoll  rust d6=5  AnimalSavagery  ┘
```

(Rust attributes 49-52 to `AnimalSavagery` because it resolves the lash-out injury INSIDE that step —
the false-positive pattern ITER37 warned about, here confirmed benign.)

So both engines compute the same lash-out injury, and 8 is a KO in BB2020. Java applies it; Rust does
not. In `mixed/shared/step_animal_savagery.rs` the result IS produced and consumed —
`handle_injury(...)` at line 334, `injury_result.injury_context().is_casualty() || is_knocked_out()`
driving `player_removed` at line 584, and the result attached to a context at 425/444 — which
explains why the victim gets BOXED. What never happens is the state-base change to `KNOCKED_OUT`, so
the boxed player keeps `STANDING`.

That one unapplied injury is the whole seed: Rust keeps a player Java has removed, which is why Rust
still has an activation at die 85 where Java has none (the ITER36/37 symptom), and why the dice drift
from there.

**Next iteration:** compare against Java's `lashOut` — specifically what applies the injury to the
victim after `UtilServerInjury.handleInjury` returns (the `DropPlayerContext` /
`SteadyFootingContext` consumer, or an `INJURY_RESULT` publish that Rust drops). The fix belongs
wherever Rust decides to box the player: it must set the state base from the injury context, not just
move the player. Add a test asserting an AS lash-out with armour 12 / injury 8 leaves the victim
`KNOCKED_OUT`, not `STANDING`.

Baseline unchanged: `underworld` 22/25. No engine changes this iteration.

## ITER39 — underworld: the lash-out ARMOUR does not break in Rust

Refines ITER38. A gated `FFB_AS_TRACE` on the lash-out (added, measured, reverted) reports the injury
context straight after `handle_injury`:

```
AS_TRACE victim=home_02 ko=false cas=false armour_broken=false state_after=Some(1)
AS_TRACE victim=home_03 ko=true  cas=false armour_broken=true  state_after=Some(1)
```

The first line is the seed-2 divergence. Java rolled **armour 6+6 = 12**, which breaks any AV in the
game, then injury 3+5 = 8 = KO. Rust reports **`armour_broken=false`** for the same victim — so the
KO never arises, and ITER38's "injury computed but not applied" reading was wrong: **the injury is
not computed as a break in the first place.**

(Note `state_after=Some(1)` = STANDING even on the `ko=true` line, so the state change being deferred
past this point is normal in both engines and is NOT the bug.)

**Also found, a genuine 1:1 discrepancy at this call site** (inert for these dice, so recorded rather
than changed): the second constructor argument means different things in the two engines.

```java
// Java: InjuryTypeBlock(Mode mode, boolean allowAttackerChainsaw)
UtilServerInjury.handleInjury(step, new InjuryTypeBlock(mode, false), ...)   // chainsaw NOT allowed
```
```rust
// Rust: new(mode, roll_armour) -> new_with_chainsaw(mode, roll_armour, allow_attacker_chainsaw = true)
let mut injury_type = InjuryTypeBlock::new(mode, true);                       // chainsaw ALLOWED
```

The faithful call is `InjuryTypeBlock::new_with_chainsaw(mode, /*roll_armour*/ true,
/*allow_attacker_chainsaw*/ false)`. It does not affect this seed (the acting player carries no
chainsaw) but it is wrong and should be corrected alongside the real fix.

**Next iteration:** dump the armour computation for the lash-out victim on both sides — the victim's
AV, the armour modifier list and total, and the roll — and compare. Rust's `InjuryTypeBlock` armour
path takes `mode` (`DO_NOT_USE_MODIFIERS` vs `USE_MODIFIERS_AGAINST_TEAM_MATES`), which Java selects
with `actingPlayer.isStandingUp() || actingPlayer.getTeam() != defender.getTeam()`; check Rust picks
the same arm, and check `armour_with_modifiers` for the victim.

Baseline unchanged: `underworld` 22/25. No engine changes this iteration.

## ITER40 — RETRACTION: ITER38 and ITER39 were both wrong (id ↔ state-key mis-mapping)

Enabling Java's pre-existing `JAVA_AVBROKE` trace (`-Dffb.parityDebug`, already wired to `FFB_TRACE`)
settles it:

```
JAVA_AVBROKE def=…Home3 armour=8 reduced=8 roll=[6,6] modTotal=0 mods= broken=true
JAVA_AVBROKE def=…Home2 armour=8 reduced=8 roll=[2,4] modTotal=0 mods= broken=false
```

and Rust's `AS_TRACE` said:

```
AS_TRACE victim=home_02 ko=false armour_broken=false
AS_TRACE victim=home_03 ko=true  armour_broken=true
```

**The two engines agree on both lash-outs.** `Home3`/`home_03` breaks armour 12 vs AV 8 and is KO'd in
BOTH; `Home2`/`home_02` rolls [2,4] and is not broken in EITHER. ITER39's claim that Rust failed to
break the armour, and ITER38's claim that Rust computed the injury but failed to apply it, are both
**retracted** — I compared Rust's `home_02` against the state key `h02`, which is a different player.

**The mapping, recorded so this stops recurring:**

| thing | example | maps to |
|---|---|---|
| Rust player id | `home_03` | Java `Home3` |
| Java player id | `Home3` | state-string key `h02` |
| state-string key | `h02` | the THIRD home player |

Player ids are **1-based**; the state-string keys in `JSTEP`/`RUST_STEP` are **0-based**. So
`state key hNN` ↔ `home_(NN+1)`. Always convert before comparing a trace that names players with a
state string that names slots.

**What remains true about seed 2:** the first STATE divergence is still i=56 with `h02` KO in Java and
Standing in Rust, and Rust is one die ahead there. Since both engines DO produce that KO, the
difference is **when** — Java has applied it by i=56 and Rust has not. The next step is to find the
die index of each engine's `Home3` lash-out (Java via `JAVA_AVBROKE` ordering, Rust via an `AS_TRACE`
carrying `rng.call_count`) and compare; a timing/order difference in when the AS lash-outs resolve is
now the live hypothesis, not a missing computation.

Baseline unchanged: `underworld` 22/25. No engine changes in ITER38, ITER39 or ITER40.

## ITER41 — underworld seed 2 ROOT-CAUSED (correctly this time): the lash-out KO is never applied

With the ITER40 mapping corrected, every claim below is measured.

**Both engines compute the same lash-out injury, at the same die.** An `AS_TRACE` carrying
`rng.call_count` (added, measured, reverted):

```
AS_TRACE die=52 victim=home_03 ko=true armour_broken=true
```

and Java's `JAVA_AVBROKE` for the same player: `def=…Home3 armour=8 roll=[6,6] broken=true`, with the
injury at dice 51/52 = 3+5 = 8 = KO. Identical.

**Only Java applies it.** Scanning every recorded step's state string for `h02` (= `home_03`):

```
JAVA: h02 first becomes Ko at i=56
RUST: h02 never Ko; last seen at i=287 as '11,13,Standing'  ← back ON the pitch
```

**Where it is lost.** `mixed/shared/step_animal_savagery.rs` attaches the `InjuryResult` to a
`DropPlayerContext`, and `bb2020/step_handle_drop_player_context.rs` consumes it — but its
`execute_step` only calls `util_server_injury::drop_player(...)`, which sets the base to PRONE or
STUNNED. **Nothing applies the injury result's KO/CAS outcome.** Confirmed by grep: `apply_to` is
defined in BOTH `injury.rs:262` and `injury_result.rs:92` (a stale-duplicate pair, the recurring trap
in this codebase) and **every call site of either is inside `#[cfg(test)]`** — no production code
applies an `InjuryResult` at all.

Blocks still KO correctly in the green rosters, so the state change on those paths must happen inside
`handle_injury` itself; the Animal Savagery lash-out path is the one that relies on the unapplied
context.

**Next iteration — the fix.** Read Java's `UtilServerInjury.handleInjury` to see exactly where it sets
the victim's `PlayerState` (Java applies at injury time, which is why its `h02` is KO by i=56), then
make the Rust lash-out path do the same. Before editing, use the backtrace trick to confirm WHICH of
the two `apply_to` definitions (and which `handle_injury`) is live — this codebase has burned several
iterations on stale duplicates. Test: an AS lash-out with armour 12 / injury 8 must leave the victim
`KNOCKED_OUT` and off the pitch, not `STANDING`.

Baseline unchanged: `underworld` 22/25. No engine changes this iteration.

## ITER42 — correction to ITER41, and the divergence narrowed to the BB2025 apothecary

**ITER41's closing claim was wrong and is retracted.** "Every `apply_to` call site is inside
`#[cfg(test)]`" came from a grep that excluded `injury.rs`/`injury_result.rs` by filename but was
read as if it excluded only test blocks. `apply_to` has ~15 production callers (the apothecary steps,
TTM scatter, trap door, …). The rest of ITER41 stands: both engines compute the lash-out at die 52,
Java KOs `h02` at i=56, Rust never does.

**A structural fact worth recording on its own: a BB2020 game runs the BB2025 Move sequence.**
`FFB_DRIVE_TRACE` around the lash-out:

```
DRIVE step=InitActivation ... AnimalSavagery ... SteadyFooting ... HandleDropPlayerContext ... PlaceBall ... Apothecary
```

`SteadyFooting` is a BB2025-only step; Java's `generator/bb2020/Move.java` has
ANIMAL_SAVAGERY → HANDLE_DROP_PLAYER_CONTEXT → PLACE_BALL → APOTHECARY(ANIMAL_SAVAGERY) with no such
step. `step/driver.rs` opens with `use crate::step::bb2025::move_::*` and has a BB2016 override arm
(line ~451) but **no BB2020 arm**, so `EndSelecting` (and hence the Move generator it invokes) and
`HandleDropPlayerContext` both resolve to the BB2025 implementations. Rust's own
`generator/bb2020/move_.rs` and `bb2020/step_handle_drop_player_context.rs` are dead code on this
path — `generator/bb2020/move_.rs:160` even asserts the sequence contains no `SteadyFooting`.

**But that is not what loses the KO.** Probing the BB2025 step that actually runs (all probes since
reverted):

```
AS-PUBLISH:   target=home_03 injury=Some(5) ab=true      # 5 = PS_KNOCKED_OUT, armour broken
HDPC25-ENTER: has_ctx=true   injury=Some(5)              # received intact
```

So the AS behaviour computes the KO correctly, and the BB2025 `StepHandleDropPlayerContext` receives
the `DropPlayerContext` carrying it. Both hand-offs are sound; the state is lost strictly downstream.

**Next iteration — one probe, then the fix.** Downstream of HDPC the only applier is
`bb2025/shared/step_apothecary.rs` (the BB2020 one is not on this path, which is why ITER41's
`APO-APPLY` probe printed nothing). Probe its `set_parameter` and `execute_step` for the
`ANIMAL_SAVAGERY` mode: the prime suspect is the mode gate — `set_parameter` accepts an
`InjuryResult` only when `self.apothecary_mode == ir.injury_context.apothecary_mode`, so if the
BB2025 Move sequence's post-AS `Apothecary` step is instantiated with a different mode than
`AnimalSavagery` (or the step never receives the published `INJURY_RESULT` at all), the result is
silently dropped exactly as observed. Confirm which before editing.

Note two further 1:1 gaps found while reading, to fix alongside (both engines' HDPC):
Java uses `game.getDefender()` for the victim-state keys where Rust uses
`game.acting_player.player_id`, and Java's `UtilServerInjury.dropPlayer` takes the
`apothecaryMode` argument that Rust's `drop_player` does not pass.

Baseline unchanged: `underworld` 22/25. No engine changes this iteration.

## ITER43 — the agent never answered the apothecary dialog (1:1 harness mirror, landed)

Continuing ITER42's probe of `bb2025/shared/step_apothecary.rs`. The mode gate was innocent — the
step receives the lash-out result correctly:

```
APO25: mode=Some(AnimalSavagery) status=DoRequest def=Some("home_03") inj=Some(5)   # 5 = PS_KNOCKED_OUT
APO25: mode=Some(AnimalSavagery) status=WaitForApothecaryUse def=Some("home_03") inj=Some(5)
APO25-CMD: (never printed)
```

The step opened the `USE_APOTHECARY` dialog and **nothing ever answered it**. `random_agent.rs`
answered `AgentPrompt::UseApothecary` with `Action::Acknowledge`, which `StepApothecary::handle_command`
does not match, so the status stayed `WAIT_FOR_APOTHECARY_USE` — a status the main switch has no arm
for (`_ => {}`) — and the computed injury was silently discarded.

Java's harness does answer it, `ParityRunner.handleDialog` case `USE_APOTHECARY`:

```java
comm.sendUseApothecary(apo.getPlayerId(), false, apoType, apo.getSeriousInjury());
```

— an unconditional DECLINE naming the injured player. **Fix**: mirror it exactly in the agent
(`UseApothecary { player_id, use_apothecary: false }`), with a colocated regression test
(`use_apothecary_prompt_is_declined_naming_the_injured_player`) that also asserts no decision RNG is
consumed, since Java's answer is unconditional.

**Verified effect**: the decline now lands (`status=DoNotUseApothecary`) and `home_03` leaves the
pitch at i=56 as it does in Java — `h02:11,13,Standing` (mid-pitch, still playing) became
`h02:-1,-1,Standing` (in the dugout box).

**Seed 2 still fails at the same step 55**, because the base is still `Standing` where Java has `Ko`:
`apply_to` boxes the player but the state base is not left `KNOCKED_OUT`. Next iteration starts
there — `injury_result.rs:139` sets the base, then `UtilBox::put_player_into_box` +
`UtilServerGame::update_player_state_dependent_properties` run; probe which of the three leaves the
base at `Standing`, against Java's `InjuryResult.applyTo`.

Gates all green with the fix: `lineman` bb2016 100/100, `lineman` bb2025 100/100, `human` bb2020
100/100, `cargo test --workspace` 0 failed. `underworld` unchanged at 22/25 (seeds 2, 3, 19).

## ITER44 — underworld seed 2: the KO is applied, then overwritten by a block's stale OLD_DEFENDER_STATE

Following ITER43's residue. First, the encoding, so the comparison is unambiguous — Java at i=56:

```
JAVA: h02:-1,-1,Ko          RUST: h02:-1,-1,Standing
```

Every other player on both teams matches at i=56. The coordinate now agrees (ITER43's fix); only the
base differs.

**`apply_to` is innocent — it does exactly the right thing.** Probing it directly:

```
APPLY: def=Some("home_03") want=Some(5) before=Some(3) after=Some(5)     # 5 = PS_KNOCKED_OUT
```

So the KO IS applied, at the Apothecary step, as Java does.

**A state-change watcher found the overwrite.** Instrumenting `FieldModel::set_player_state` to log
every base change for one player (`FFB_STATE_WATCH=home_03`) alongside `FFB_DRIVE_TRACE`:

```
DRIVE step=HandleDropPlayerContext   SETSTATE home_03 Some(1) -> 3     # dropPlayer → PRONE
DRIVE step=Apothecary                SETSTATE home_03 Some(3) -> 5     # applyTo   → KNOCKED_OUT
  ... BoneHead, ReallyStupid, TakeRoot, UnchannelledFury, BloodLust, InitBlocking, GoForIt,
      FoulAppearance, DumpOff, Dauntless, Horns, PickUp, Stab, BlockChainsaw, Chomp, BlockRoll ...
DRIVE step=BlockChoice               SETSTATE home_03 Some(5) -> 1     # ← the KO is undone
```

The write is `bb2025/block/step_block_choice.rs:92` (`BlockResult::Skull`; line 206 is the same write
on `Pushback`), a faithful port of Java's `StepBlockChoice:165`
`game.getFieldModel().setPlayerState(game.getDefender(), fOldDefenderState)`. Rust restores
`Standing` because that is what the block published:

```
BLOCKCHOICE: result=Skull defender=Some("home_03") old_state_param=Some(1) used=1
```

**Not the ITER42 lead.** That note guessed the acting-player/defender mix-up in
`StepHandleDropPlayerContext` — but the file that actually runs (`bb2025/shared/…`) already uses
`game.defender_id` correctly, with a comment citing the Java line. Only the dead `bb2020/` copy has
the mix-up. Worth fixing for hygiene, but it is not this bug.

**Two hypotheses for the next iteration, in order.** The block that overwrites the KO has `home_03`
as its defender even though `home_03` was knocked out and boxed several steps earlier:
1. *Stale block target.* The activation should not still be blocking a player who left the pitch —
   check whether Java re-targets or ends the action after the AS lash-out KOs the intended victim,
   and what `game.getDefender()` is at Java's `StepBlockChoice` here.
2. *Capture ordering.* `bb2025/block/step_init_blocking.rs:222` captures `old_state` from the current
   defender state, matching Java's `StepInitBlocking:226`. If Java captures after the KO its
   `fOldDefenderState` is `Ko` and the restore is a no-op, while Rust captured `Standing`. The probe
   showed `INITBLOCK: defender=home_03 raw=Some(1)` — i.e. by the time this `InitBlocking` ran,
   `home_03` was ALREADY Standing again, which does not fit a simple ordering story and needs the
   Java-side value to disambiguate.

Resolve by adding gated logging to the Java `StepInitBlocking`/`StepBlockChoice` (gated-logging-only
edits to engine files have precedent in this campaign) and printing Java's defender id and
`fOldDefenderState` at both sites, then joining against the Rust trace above.

All probes reverted. Baseline unchanged: `underworld` 22/25 (seeds 2, 3, 19).

### ITER44 (cont.) — ROOT CAUSE: `StepInitBlocking` never set the game's defender. **underworld 100/100.**

Hypothesis 1 was right, and the measurement that settled it was cheap and Rust-side. Correlating the
`InitBlocking` capture with the watched state transitions:

```
INITBLOCK: defender=home_01 old_state=1
SETSTATE home_03 Some(1) -> 3        # AS lash-out drop
SETSTATE home_03 Some(3) -> 5        # apothecary applies the KO
INITBLOCK: defender=away_01 old_state=1     # ← the block AFTER the KO targets away_01
SETSTATE home_03 Some(5) -> 1        # ← but BlockChoice restores home_03
```

The block's defender is `away_01`; the player `StepBlockChoice` restored is `home_03`. It was reading
a **stale** `game.defender_id` — the Animal Savagery lash-out victim from earlier in the same
activation.

Java `bb2025/block/StepInitBlocking:220` (and `bb2020/…:214`):

```java
game.setDefenderId(defender.getId());
```

Rust set only `game.acting_player.defender_id` — and Java's `ActingPlayer` **has no `defenderId`
field at all**, so that assignment is a Rust-only mirror (read by the legacy `step/engine.rs` path)
and the actual Java line was simply never ported. `game.defender_id` therefore kept whatever the
previous step left in it, and every later step reading the game's defender acted on the wrong player.

**Fix**: port the missing line in both editions' `StepInitBlocking`, keeping the existing Rust-only
mirror assignment so the legacy path is unaffected. Regression test
`init_blocking_sets_the_game_defender_over_a_stale_one` seeds a stale `game.defender_id` and asserts
the step overwrites it.

**Result: `underworld` bb2020 1-100 → `PARITY: 100/100 games match`** (all three failing seeds — 2, 3
and 19 — fell to this one fix). Gates: `lineman` bb2016 100/100, `lineman` bb2025 100/100, `human`
bb2020 100/100, `cargo test --workspace` 14,437 passed / 0 failed.

Note for the sweep bookkeeping: the 1-100 run prints `100/100 games match, but required coverage
items are MISSING` — a coverage-catalog warning from the harness, not a parity failure.

Since this bug corrupted the game's defender for any activation that resolved a lash-out, a foul, or
a gaze before blocking, it is a strong candidate for other rosters' remaining reds — re-scout before
diagnosing them individually. `step/bb2020/foul/step_init_fouling.rs` and
`step/bb2025/foul/step_init_fouling.rs` set the same Rust-only mirror; check them against Java's
`StepInitFouling` next.

**Post-fix re-scout** (the defender fix is edition-wide, so prior fail counts were re-measured):
`underworld` 1-100 **100/100** (was 22/25); `lineman` bb2020 1-100 **99/100** (seed 50, step 6 —
unchanged, still the known BOTH_DOWN where Java's attacker does not fall); `halfling` 1-25 **2/25**
(unchanged — the Treeman that does not move after passing Take Root; unaffected by the defender bug).
Next target by the fewest-fails rule: **lineman seed 50**.

## ITER45 — lineman seed 50 root-caused: the Intensive Training prayer never grants its skill

The divergence at step 6/i=7 is an active-team split (Java stays `home`, Rust hands over to `away`),
caused by an extra turnover in Rust. Dice, joined by position:

```
             die 12   die 13,14      die 15,16
JAVA:        2        4,3            (none)          -> 3 dice, home turn continues
RUST:        2        4,3            5,1             -> 5 dice, turnover, away's turn
```

Both engines roll the SAME block die (2 = `BOTH_DOWN` per `BlockResultFactory`) and the SAME defender
armour roll (`JAVA_AVBROKE def=Away2 armour=8 roll=[4,3] broken=false`). Rust then rolls two more —
the ATTACKER's armour — because in Rust the attacker also falls. At i=7 Java has `h01:12,6,Standing`
and Rust `h01:12,6,Prone`; every other player matches.

**Why Java's attacker does not fall.** A gated probe in Java's `mixed/block/StepBothDown` (added,
measured, reverted — the jar is rebuilt back to stock):

```
JBOTHDOWN atk=teamLinemanParityHome2 def=teamLinemanParityAway2 atkPreventFall=true
```

`preventFallOnBothDown` is the Block property — on a roster with **no skills at all**
(`roster_lineman_parity.xml` and `team_lineman_parity_home.xml` both carry empty `<skillList>`). The
skill came from a **Prayer to Nuffle**. Confirmed on the Rust side:

```
RUST_PRAYER roll=16 league=true prayer=Some(INTENSIVE_TRAINING) handler=Some("IntensiveTrainingHandler") team=home_lineman
```

**Rust's handler is a deliberate stub.** `inducements/bb2020/prayers/intensive_training_handler.rs`
says so in its own header: *"Headless: selects one player and marks the prayer enhancement; skips
skill-selection dialog (position skill categories not available server-side)."* So Java grants a real
skill and Rust grants none — invisible in the state hash until the first Both Down, where it decides
whether the attacker falls.

**The port (full spec, all four pieces confirmed against source).** Java
`mixed/prayers/IntensiveTrainingHandler.createDialog`:
1. `Collections.shuffle(players)` — the **unseeded one-arg overload**. The harness already seeds
   `Collections`' private static Random (`ParityRunner:230-262`) and Rust already has the 1:1
   `ffb_model::util::java_random::{JavaRandom, collections_shuffle}`. It is a SHARED per-game stream,
   so Rust must shuffle at the same point and in the same order.
2. `player = players.get(0)`.
3. Eligible skills = every skill that `eligible()`, whose category is in
   `player.getPosition().getSkillCategories(false)`, that the player does not already have, and that
   `canBeAssignedTo(player)` — **sorted by `Skill::getName`**.
4. `DialogSelectSkillParameter(playerId, skills, SkillChoiceMode.INTENSIVE_TRAINING)`; if the list is
   empty, `ReportPrayerWasted` instead. `applySelection` →
   `game.getFieldModel().addIntensiveTrainingSkill(playerId, skill)`.

The agent mirror is settled too: `ParityRunner` has no `SELECT_SKILL` case, so it falls to
`UNHANDLED_DIALOG` → `RandomStrategy.respondToDialog`, whose `case SELECT_SKILL` sends
`skills.get(0)` — the first entry of the name-sorted list. For a General-category lineman that is
**Block**, exactly matching the `atkPreventFall=true` observed.

Next iteration implements this as a 1:1 port with tests (shuffle order, name-sorted eligibility,
the empty-list wasted-prayer branch, and the agent answering index 0), then re-runs lineman 1-100.

Java engine is back to stock and the jar rebuilt; all Rust probes reverted; baseline re-confirmed at
`lineman` bb2020 seed 50 FAIL.

### ITER46 — first half of the Intensive Training port: the per-skill category/name table

The port needs the eligible-skill list Java builds in `IntensiveTrainingHandler.createDialog`, which
filters by `player.getPosition().getSkillCategories(false)` and sorts by `Skill::getName`. Rust had
**neither** piece of data: `SkillId` exposed `class_name()` and `properties_for()`, but no category
and no display name, and `data/skills/*.json` carries only class names.

Two pieces of good news found while measuring, both of which shrink the remaining work:

1. **The player selection already matches Java exactly.** Java's `createDialog` does
   `Collections.shuffle(players); players.get(0)`. Rust's `init_effect_random_selection` calls
   `select_players(.., 1, ..)`, whose bb2020 selector does one `collections_shuffle` and takes
   element 0 — same shared `game.collections_rng` stream, same single draw, same index. So the
   prayer's RNG consumption is already correct; only the skill grant is missing.
2. **The grant primitive exists**: Java's `FieldModel.addIntensiveTrainingSkill` is
   `player.addTemporarySkills(prayerName, {skill})`, and Rust has `Player::add_prayer_skill`.

**Landed this iteration**: `SkillId::category_and_name_for(rules)` in `crates/ffb-model`, generated
from the `super("<name>", SkillCategory.<CAT>)` call in all 199 Java skill classes and resolved
edition-first then `mixed` then `common` — the same resolution the skill classes themselves use.
197 of the 199 join to a Rust `SkillId` (the two that do not are alt-spellings, `Bloodlust`/`Claws`).
**28 skills are edition-divergent**, so the table is edition-aware for exactly the reason
`properties_for` is: Bone-Head is `EXTRAORDINARY`/`"Bone-Head"` in BB2016 but `TRAIT`/`"Bone Head"`
in BB2020+, Dirty Player is `GENERAL` in BB2016/BB2020 and `DEVIOUS` only in BB2025, and so on.

Three tests pin what the port depends on: Block is `(General, "Block")` in every edition, the
Bone-Head and Dirty Player divergences, and that "Block" sorts first among BB2020 General skills —
which is what the harness picks, since `RandomStrategy` case `SELECT_SKILL` sends `skills.get(0)`.

A drafted assertion that Dirty Player is Devious in BB2020 FAILED against the generated table; the
Java source says GENERAL there and DEVIOUS only in BB2025. The table was right and the test was
corrected — worth recording as a reminder that these categories are not guessable.

No behaviour change yet (the table is not yet wired), so no parity movement is claimed:
`cargo test --workspace` 14,440 passed / 0 failed. Next iteration wires
`IntensiveTrainingHandler.createDialog`/`applySelection` and the agent's `SELECT_SKILL` answer, then
re-runs lineman 1-100.

### ITER47 — the remaining blocker for the Intensive Training port: the fixture has no skill categories

Traced the wiring end-to-end before writing any of it, and hit a data gap that has to be closed
first — recording it so the next iteration starts from the answer rather than rediscovering it.

**What is already in place** (nothing to build):
- `AgentPrompt::SelectSkill { player_id, available }` and `Action::SelectSkill { skill_id }` both
  exist, and `StepPrayer::handle_command` already has an `Action::SelectSkill` arm (currently a
  no-op `{}`).
- `SkillId::category_and_name_for(rules)` (ITER46) supplies the filter key and the sort key.
- `Player::add_prayer_skill` is the grant primitive (Java `addTemporarySkills`).
- The player pick already consumes the Collections stream identically.

**The blocker.** Java filters by `player.getPosition().getSkillCategories(false)` — the position's
NORMAL-roll categories. `roster_lineman_parity.xml` declares them:

```xml
<skillCategoryList>
  <normal>General</normal>
  <double>Agility</double> <double>Strength</double> <double>Passing</double>
</skillCategoryList>
```

Rust's side of the same fixture is **synthesised in code**, not loaded from data:
`crates/ffb-parity/src/runner.rs:582 make_lineman_team` builds 11 players with
`position_id: "lineman"`, `roster_id: "lineman"`, and there is no `data/rosters/*/roster_lineman.json`
— every other roster has one. `Player` carries no categories itself (only `RosterPosition` does, via
`find_position(roster_id, position_id, rules)`), so for the lineman fixture that lookup returns
`None` and the eligible-skill list would come out EMPTY. An empty list is Java's
`ReportPrayerWasted` branch — no skill granted — i.e. the port would compile, pass its unit tests,
and still not fix seed 50.

So the fixture must first expose the same categories as the Java XML it mirrors. `make_lineman_team`
lives in `ffb-parity` and is co-editable; the Java fixture roster is the truth to copy (General on
normal; Agility/Strength/Passing on doubles; MA6 ST3 AG3 PA4 AV8, which the synthetic team already
matches).

**Order of work for the next iteration**, now that the shape is known:
1. Give the lineman fixture its position categories, mirroring `roster_lineman_parity.xml` (either a
   `roster_lineman.json` per edition that `find_position` resolves, or categories carried on the
   synthetic position — whichever keeps `make_lineman_team` the single source for the fixture).
2. Port `createDialog`: eligible skills = category in the position's normal categories, not already
   held, `canBeAssignedTo` (Java: no conflicting properties), sorted by name; empty -> wasted prayer.
3. Emit `AgentPrompt::SelectSkill`; the agent flattens the offered set, sorts by
   `category_and_name_for(rules).1` and takes the first — reproducing `RandomStrategy`'s
   `skills.get(0)` on Java's name-sorted list regardless of how the prompt groups them.
4. `applySelection` -> `add_prayer_skill`, then re-run lineman 1-100.

Tree is clean; no behaviour change this iteration and none claimed. `lineman` bb2020 remains 99/100.
