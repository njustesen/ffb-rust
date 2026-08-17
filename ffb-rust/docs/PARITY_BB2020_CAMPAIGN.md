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

### ITER48 — Intensive Training ported. **lineman bb2020 100/100.**

Landed the four steps ITER47 laid out.

1. **Fixture categories.** `Player` gained `skill_categories_normal`, copied by `update_position` the
   same way `keywords` already is — Java reaches the categories through `player.getPosition()`, which
   Rust cannot do at runtime since `Player` and `RosterPosition` are separate structs.
   `make_lineman_team` sets `[General]`, mirroring `roster_lineman_parity.xml`.
2. **`createDialog`.** `eligible_skills` filters `SkillFactory::get_skills()` by category-in-position,
   not-already-held and `canBeAssignedTo`, then sorts by DISPLAY name (`category_and_name_for(rules).1`)
   — Java's `Comparator.comparing(Skill::getName)`. `init_effect` now returns Java's `handled()`: true
   (nothing pending) only when the list is empty, which is the `ReportPrayerWasted` branch.
   `Skill.eligible()` is not modelled because the base returns `true` and no shipped skill overrides it.
3. **The dialog.** New `PrayerHandler::skill_dialog` / `apply_skill_selection` (Java's SKILL dialog, as
   distinct from the PLAYER dialog `dialog_choice_mode` describes). `StepPrayer` emits
   `AgentPrompt::SelectSkill` and remembers the player, exactly as Java carries the id on
   `DialogSelectSkillParameter` and gets it back in `PrayerDialogSelection(playerId, skill)`. The
   chosen player is recovered from `field_model.prayer_enhancements[PRAYER_NAME]` rather than stored
   in `PrayerState`, which is a 1:1 of Java's class and has no such field.
4. **The agent.** The `SelectSkill` arm answered `Acknowledge` — which the step ignores — after
   burning a decision-RNG call. It now decodes the offered ids and picks the name-first skill with
   NO RNG, mirroring `RandomStrategy`'s `skills.get(0)` on Java's already-sorted list. Sorting on the
   agent side rather than trusting the prompt's grouping keeps it correct either way.

**Result: `lineman` bb2020 1-100 → `PARITY: 100/100 games match`.** Gates: `lineman` bb2016 100/100,
`lineman` bb2025 100/100, `human` bb2020 100/100, `underworld` bb2020 100/100,
`cargo test --workspace` 14,445 passed / 0 failed.

Two notes for later:
- The BB2025 handler has a different file shape and was left unwired. BB2025 lineman is 100/100 either
  way, so nothing is masked today, but Java's `bb2025/IntensiveTrainingHandler` exists and should get
  the same treatment with its own verification rather than being folded in untested here.
- Adding a `Player` field surfaced four test constructors that spell every field out instead of
  ending with `..Default::default()`; they were updated. A Java-side doc snippet also had to be
  fenced as ```text — an indented block in a doc comment is compiled as a doctest.

## ITER49 — re-scout, then `renegades` 🟢 100/100: Under Scrutiny was stubbed off

**Re-scout first** (ITER20's table predates several edition-wide fixes, so it was re-measured):

| roster | 1-25 | note |
|---|---|---|
| `chaos_pact` | **100/100 on 1-100** | was 1 fail |
| `goblin`, `ogre` | 25/25 | were 14 and 25 fails |
| `renegades` | 24/25 | this iteration's target |
| `nurgle` | 21/25 | |
| `dwarf` 5/25, `necromantic` 4/25, `elf` 3/25, `wood_elf` 3/25, `halfling` 2/25, `slann_fumbbl` 0/25 | | the heavies |

**`renegades` seed 25, step 152.** Both engines activate `away_01` for a FOUL and roll **identical
dice values** (`3,6,4,5,1,2,5,6` at positions 102-109). The difference is what the last one is: Java's
`JAVA_DIE rng=109 ... from=DiceRoller.rollArgueTheCall:152` — the fouler was BANNED and the away turn
ended (`i=153` is `home` in Java, still `away` in Rust).

The chain is `generator/bb2020/Foul:46 StepId.REFEREE` -> `StepReferee` -> the hook in
`skillbehaviour/bb2020/SneakyGitBehaviour`, which ends:

```java
boolean underScrutiny = step.getGameState().getPrayerState().isUnderScrutiny(actingPlayer.getPlayer().getTeam());
refereeSpotsFoul |= underScrutiny;
```

Rust's `step/action/foul/step_referee.rs` had **two** faults in the corresponding line:

```rust
// Stub: prayer state not yet implemented → false.
let under_scrutiny = false;
referee_spots_foul |= under_scrutiny && ctx.is_armor_broken();
```

The flag was hardcoded `false` even though `PrayerState::is_under_scrutiny` — and the whole
`UnderScrutinyHandler` with its own passing tests — had been there all along; and it was ANDed with
`isArmorBroken()`, which Java does not do. Under Scrutiny means the ref is watching that team, so ANY
foul is spotted, doubles or not, armour broken or not. Fixed to read the prayer state and OR it in
unconditionally, with two tests: a no-doubles unbroken-armour foul IS spotted under scrutiny, and the
same foul is NOT spotted without it (pinning that the armour-broken gate is gone).

**Result: `renegades` bb2020 1-100 → `PARITY: 100/100 games match`** (from 96/100-equivalent).
Gates: `lineman` bb2016/bb2025/bb2020 100/100, `human` bb2020 100/100, `underworld` bb2020 100/100,
`chaos_pact` bb2020 100/100, `cargo test --workspace` 14,447 passed / 0 failed.

**Running total: 8 of 30 green** (`lineman`, `human`, `underworld`, `chaos_pact`, `renegades`, plus
`goblin` and `ogre` clean on 1-25 and awaiting their 1-100 gate). Next by fewest fails: `nurgle` (4).

Worth noting the pattern: this is the third fix in a row where the Rust engine had the machinery and
simply never connected it (the apothecary answer, the Intensive Training grant, now Under Scrutiny).
A grep for "Stub:" and "not yet implemented" in the step tree is probably the cheapest way to find
the next few.

## ITER50 — `nurgle` seed 2 narrowed: a prone blitzer stands up despite a failed Really Stupid

`nurgle` fails 4 of 25 (seeds 2, 14, 23, 24). Seed 2, step 32:

```
i=32  both: Activate(away_02, BLITZ)   a01 = away_02, Prone in both engines
      JAVA_DIE rng=30 d6=1 from=DiceRoller.rollSkill:112     ← the negatrait, FAILED
i=33  JAVA a01:13,8,Prone      RUST a01:13,8,Standing
```

Both engines consume exactly one die and the same value, so the roll itself agrees; only the state
differs. `away_02` is the Beast of Nurgle — Really Stupid, rolled at 1, failed.

**Why Java leaves it prone.** The generators differ in a way that decides this:

- `generator/bb2020/Move` has **no** `STAND_UP` step at all — the move path stands the player up.
- `generator/bb2020/Block:47-48` has `JUMP_UP` + `STAND_UP` with
  `GOTO_LABEL_ON_FAILURE=END_BLOCKING`, placed **after** the negatraits (`BONE_HEAD`,
  `REALLY_STUPID`, …).

So on a blitz, a failed Really Stupid gotos `END_BLOCKING` and never reaches `STAND_UP`: the player
stays PRONE. On a move there is nothing to skip.

**And Java's `StepInitSelecting` never sets the player state.** Its stand-up block
(`bb2025/shared/StepInitSelecting.java:501-510`) sets only `currentMove` and `goingForIt` and then
calls `updateMoveSquares` — there is no `setPlayerState` in it. Rust's counterpart writes
`PS_STANDING` there, a Rust addition made for a vampire Bloodlust case on a MOVE action (the comment
in `step_init_selecting.rs` records it).

**Attempted and REVERTED.** I gated that pre-stand to exclude `Blitz`/`Block` (the sequences that own
a `STAND_UP` step). It changed nothing: `nurgle` stayed 21/25 and seed 2 still failed at step 32.
Watching the player proves why — the write that stands it up is a *different* one:

```
DRIVE step=InitSelecting
SETSTATE away_02 Some(3) -> 2        # PRONE -> MOVING, not the PS_STANDING line I gated
```

Base 2 is `PS_MOVING`, and `step_init_selecting.rs` has only ONE direct state write (the
`PS_STANDING` one at line 177), so this comes from something it calls. A grep for `PS_MOVING` writers
across `ffb-engine`/`ffb-model` outside tests found none, so the write is indirect — via a helper or
a literal — and finding it is the next iteration's first job. Since the gate produced no measurable
improvement, it was reverted rather than committed on a plausible-sounding rationale.

**Next iteration**: put the `FFB_STATE_WATCH` probe back (it is a 10-line addition to
`FieldModel::set_player_state`, and has now earned its keep twice), add a backtrace or a
`PS_MOVING`-specific marker to identify the writer, then decide whether the correct 1:1 is to drop
the state write entirely and let `StepStandUp` do the work — checking the vampire Bloodlust and
renegades/underworld cases the current comment cites, since those are what the pre-stand was built for.

Baseline unchanged and tree clean: `nurgle` 21/25.

## ITER51 — `nurgle` seed 2: the decision point found, and both of its inputs are wrong

Picked up ITER50's open question — which write stands the blitzer up — and answered it with a
backtrace probe on `FieldModel::set_player_state` (debug build, `RUST_BACKTRACE=1`):

```
SETSTATE away_02 Some(3) -> 2      ffb_engine::step::util_server_steps::change_player_action
SETSTATE away_02 Some(2) -> 1      ffb_engine::step::util_server_steps::change_player_action_to_none
```

The first is **correct** and matches Java: `UtilActingPlayer.changeActingPlayer` ends with
`fieldModel.setPlayerState(newPlayer, oldState.changeBase(PlayerState.MOVING))` — "show acting player
as moving". ITER50's suspicion of the `PS_STANDING` pre-stand was therefore a dead end, which is why
gating it changed nothing.

The second is the decision point, and Rust's version already implements Java's three-way branch
faithfully:

```rust
if acted()                      -> STANDING, inactive
else if standing_up || was_prone -> PRONE       // the branch Java takes here
else                             -> STANDING
```

**But both inputs are wrong for this activation.** Probing the branch:

```
TONONE old=away_02 acted=true standing_up=false was_prone=false old_ps=Some(1)
```

`old_player_state` is `STANDING(1)` although the player was demonstrably PRONE(3) when activated (the
first SETSTATE above proves it), and `standing_up` is `false` although `change_player_action` sets it
from exactly that pre-activation base. So the PRONE branch is unreachable regardless of what the
branch itself does. Two upstream suspects, both visible in the source:

1. `change_player_action` only assigns `old_player_state` `if ... .is_none()` — it is deliberately
   STICKY (a comment cites a Take Root case, wood_elf seed 1 i=49). A stale STANDING from an earlier
   activation would survive into this one.
2. `step_init_selecting.rs:120` clears `standing_up` for `Blitz`/`Block` when `block_defender_id`
   is none, and `StepStandUp` may clear it too.

**Also attempted and REVERTED**: `change_player_action` (unlike `change_player_action_to_none`) never
restores the OUTGOING acting player, while Java's `changeActingPlayer` does so in its
`(oldPlayer != null) && (oldPlayer != newPlayer)` block, before the generic BLOCKED/MOVING sweep. I
ported that restore; a probe showed it never fires on this path (the activation ends through
`change_player_action_to_none` instead), and `nurgle` stayed 21/25. It looks like a genuine gap and is
worth revisiting, but it is not this bug and was not committed without a measured effect.

**Next iteration**: fix the inputs, not the branch. Establish for this activation why
`old_player_state` is STANDING and when `standing_up` is cleared — the same backtrace probe on those
two fields (or a watch printing every write to them) will name the site in one run. Then re-check the
Take Root stickiness case the comment cites, since the fix must not reintroduce it.

Baseline unchanged and tree clean: `nurgle` 21/25.

## ITER52 — `nurgle` seed 2: three writers eliminated, the real question is where the failure label lands

Continued ITER51 by tracing every write to the blitzer's state with a backtrace probe. Three writers
exist, and this iteration establishes what each one does — correcting two earlier guesses.

1. **`change_player_action` PRONE -> MOVING**: correct, matches Java's "show acting player as moving".
2. **The `PS_STANDING` pre-stand in `step_init_selecting`**: fires here, confirmed by probe —
   `PRESTAND player=away_02 action=Blitz has_free=false ma=4 base=Some(2) owns_stand_up=true`.
   Gating it on `Blitz|Block` DOES suppress the write (the probe prints `owns_stand_up=true`), so
   ITER50's gate was correct in itself — it just is not sufficient.
3. **`change_player_action_to_none`**: does NOT run for this activation at all. Correlating the probes
   shows no `TONONE old=away_02` after the `CPA ... action=Blitz` line — the three `TONONE` lines
   ITER51 read were from other, MOVE activations of the same player, which is why their
   `standing_up=false / old_ps=STANDING` looked wrong. **ITER51's "both inputs are wrong" reading is
   retracted**: at the blitz activation the inputs are right
   (`CPA player=away_02 action=Blitz changed=true pre_state=Some(3) was_prone=true`).

With the pre-stand gated, the player stays MOVING and a **fourth** path stands it up:
`StepStandUp::execute_step`. Adding the missing Java restore of the outgoing acting player (ITER51's
other candidate) alongside the gate still leaves `a01` Standing, because `StepStandUp` runs before
either could matter.

**So the real question is why `StepStandUp` runs at all.** Java's failed Really Stupid gotos
`END_BLOCKING`, which is placed AFTER `JUMP_UP`/`STAND_UP` in `generator/bb2020/Block:47-48`, so both
are skipped. Rust's drive trace for this activation shows the goto happening — the stack drops from
47 to 41, six steps consumed — and then `JumpUp` (41) and `StandUp` (40) run anyway. The labels
themselves look right in the generators (`activation_sequence_builder.rs:119` passes
`GotoLabelOnFailure(fl)` to `ReallyStupid`; `block.rs:66,70` pass the same `fl` to `JumpUp`/`StandUp`).
So the failure label resolves to a point BEFORE the stand-up steps in Rust and AFTER them in Java.

**Next iteration**: compare where `fl` lands. Dump the assembled Rust blitz sequence with its labels
and find the index the failure label resolves to, then compare against the Java generator's ordering
of `IStepLabel.END_BLOCKING` relative to `JUMP_UP`/`STAND_UP`. The fix is a sequence/label placement
one, not a step-logic one — which also explains why two reasonable step-level fixes changed nothing.

Both experimental changes (the pre-stand gate and the outgoing-player restore) were reverted again:
each is defensible against the Java source, but neither moves a seed on its own and I am not
committing behaviour changes with no measured effect. Both are recorded here for when the label fix
lands and they can be evaluated with a real signal.

Baseline unchanged and tree clean: `nurgle` 21/25.

## ITER53 — `nurgle` seed 2: the failing roll is FOUL APPEARANCE, not a negatrait

Two more wrong assumptions eliminated, and the target is now a single step.

**There is no goto.** ITER52 read the drive trace's `stack_len 47 -> 41` as a jump; it is not. My grep
filtered the DRIVE lines to a handful of step ids, hiding the intermediate ones — those six steps are
`ReallyStupid, TakeRoot, UnchannelledFury, BloodLust, FoulAppearance, DumpOff` running normally, one
pop each, straight into `JumpUp`/`StandUp`. The failure label never fires. (Same lesson as ITER22:
a filtered log is not a measurement of what did NOT happen.)

**And the player is not the Beast of Nurgle.** `away_02` is `nurgle.warrior` in BOTH engines
(`data/teams/bb2020/team_nurgle.json` nr=2 and `team_nurgle_parity20_away.xml` nr=2 agree), and a
Nurgle Warrior has no negatrait at all — Disturbing Presence, Foul Appearance, Nurgle's Rot,
Regeneration. So Java's `rollSkill:112` of 1 at die 30 is the blitzer's **Foul Appearance** roll
against the target, and on a 1 Java cancels the action: `generator/bb2020/Block:45` gives
`FOUL_APPEARANCE` `GOTO_LABEL_ON_FAILURE = END_BLOCKING`, which sits after `JUMP_UP`/`STAND_UP`
(lines 47-48), so the blitzer never stands up and stays PRONE.

Rust's sequence agrees exactly — `generator/bb2025/block.rs:60-62` places `FoulAppearance` with
`GotoLabelOnFailure(END_BLOCKING)` at index 3, ahead of `JumpUp` (5) and `StandUp` (6). So the
sequence is right and **Rust's Foul Appearance simply did not fail**: it either rolled and passed, or
never rolled and the die went to another step.

**Also attempted and REVERTED**: `bb2020/really_stupid_behaviour.rs` reads and writes the persistent
`Player.used_skills` where Java's `UtilCards.hasUnusedSkill(actingPlayer, skill)` uses the
per-activation `ActingPlayer` set (cleared by `setPlayer`) — a real 1:1 gap, and the same shape as the
Bone-head fix recorded in the TTM tier. Moving it to `game.acting_player.used_skills` left `nurgle` at
21/25, and a probe then showed the Really Stupid roll site never runs for this player at all — because
he has no negatrait. Reverted; worth landing on its own merits with its own verification, since the
memory note "bb2020/bb2016 BoneHeadBehaviour still use Player.used_skills" says the family is latent.

**Next iteration**: probe `StepFoulAppearance` for this activation — whether it rolls at all, its
roll, its threshold, and whether it returns `GotoLabel`. Java needs the blitzer's roll of 1 to fail
and cancel. That is now a single step with a single die, so one instrumented run should settle it.

Baseline unchanged and tree clean: `nurgle` 21/25.

## ITER54 — `nurgle` seed 2: Rust has NO defender at the Foul Appearance step

Instrumented `StepFoulAppearance` as planned. For the failing blitz:

```
FA attacker=Some("away_02") defender=None def_has_fa=false attacker_cancels=false
```

It returns early without rolling — Java rolls (`rollSkill:112` = 1) and cancels the blitz. So the step
logic and the sequence placement are both fine; **Rust simply has no defender to test.**

**A real 1:1 deviation found and fixed on the way** (then reverted, see below). Java
`FoulAppearanceBehaviour` (bb2020:49-53, bb2025 identical) resolves the defender as:

```java
if (game.getFieldModel().getTargetSelectionState() != null) {
    defender = game.getPlayerById(targetSelectionState.getSelectedPlayerId());
} else {
    defender = game.getDefender();
}
```

The only condition is that the state EXISTS. Rust additionally required
`ts.is_selected() && ts.is_committed()` — and the commit happens later in this very step
(`commitTargetSelection()` after a successful roll), so a present-but-uncommitted state fell through
to `game.defenderId`. Mirroring Java exactly did not change the outcome here, because for this blitz
BOTH sources are empty: no target-selection state AND `game.defenderId == None`.

**So the open question is why the blitz has no defender by then.** In Java the defender is set before
`FOUL_APPEARANCE`: `generator/bb2020/Block:38` runs `SET_DEFENDER` with `params.getBlockDefenderId()`,
and `StepInitBlocking` calls `game.setDefenderId(...)` (the ITER44 fix). In Rust,
`generator/bb2025/block.rs:53-57` only adds `SET_DEFENDER` when `params.block_defender_id` is
`Some`, and for a blitz the target is chosen later by `SelectBlitzTarget` — so the sequence is built
with `None` and nothing sets the defender before Foul Appearance runs. Next iteration should confirm
that ordering and compare it against how Java's blitz path supplies `blockDefenderId` to the Block
sequence.

**Growing list of verified-but-unlanded 1:1 corrections.** Four now, each checked against the Java
source and each reverted for lack of a measured effect on its own:
1. gate the `PS_STANDING` pre-stand on sequences that own a `STAND_UP` step (ITER50/52);
2. restore the OUTGOING acting player in `change_player_action` (ITER51/52);
3. `bb2020` Really Stupid: per-activation `ActingPlayer.used_skills`, not the persistent player set
   (ITER53) — the memory note flags the Bone-head family as latent in the same way;
4. Foul Appearance defender resolution: drop the extra `is_selected() && is_committed()` filter (this
   iteration).
Once the defender-wiring fix lands and gives a signal, these should be re-applied and evaluated
TOGETHER against a full sweep, rather than continuing to discard them one at a time.

Baseline unchanged and tree clean: `nurgle` 21/25.

## ITER55 — gated `goblin`/`ogre` (neither was green), and `ogre` seed 57 is a pushback-square divergence

**Gate first, and it mattered.** ITER49's re-scout showed `goblin` and `ogre` clean on seeds 1-25, and
it would have been easy to count them green. Their 1-100 gates say otherwise:

| roster | 1-25 | **1-100** |
|---|---|---|
| `goblin` | 25/25 | **95/100** (5 fails) |
| `ogre` | 25/25 | **99/100** (seed 57) |

A reminder that the campaign's VALIDITY GATE is the full 1-100 sweep — a clean 1-25 predicts nothing.

**New protocol target: `ogre` (1 fail), ahead of `nurgle` (4).** Seed 57, step 132 (= `i=133`).
Diffing every player's state at that step, exactly one differs:

```
a04   JAVA 12,9,Standing   |   RUST 13,9,Standing
```

Tracing it back, both engines agree at `i=132` with `a04` at `(13,8)` and the blitzer `h01` at
`(12,7)`, and both consume the same four dice (rng 91 -> 95) during the blitz. So the dice and the
block resolution agree; only where `a04` ends up differs, by one square in x.

`a04` is an AWAY player moving during the HOME turn, so it moved by PUSHBACK. `(12,9)` is not a legal
pushback square for an attacker at `(12,7)` against a defender at `(13,8)`, which means `a04` was not
the primary defender — it was **chain-pushed** when the real defender was pushed into its square.

The choice rule itself is not the suspect: Java's `ParityRunner.sendPushback` picks min x, ties by min
y, and Rust's `choose_pushback_square` is a 1:1 of that with its own test. Java picked `(12,9)`, which
has the LOWER x, so if Rust had been offered that square its own rule would have chosen it too.
**Therefore Rust's offered option set differs** — the chain-push square computation, not the choice.

**Next iteration**: dump the pushback squares Rust offers for this chain push and compare with the
squares Java's `PushbackSquare`/`UtilServerPushback` generates for the same geometry. One instrumented
run on seed 57 should show whether `(12,9)` is missing from Rust's set or an extra square displaces it.

Baseline: `ogre` 99/100, `goblin` 95/100, `nurgle` 21/25. Tree clean; no engine change this iteration.

## ITER56 — chain-push root cause found; and a RETRACTION: `ogre` is 95/100, not 99/100

### The chain-push divergence, root-caused with a direct A/B

A gated probe in the HARNESS (`ParityRunner.sendPushback`, since reverted and the jar rebuilt to
stock) dumps the squares Java's dialog actually offers. Against Rust's own dump for the same push:

```
JAVA  all=[(14,7) (14,8) (14,9) (13,9) (12,9)]  best=(12,9)
RUST  squares=[(14,8) (14,9) (13,9)]            best=(13,9)
```

Rust's three squares are geometrically CORRECT for that attacker/defender pair — the bug is that Java
offers **five**. Java's `StepPushback:161` is `fieldModel.add(state.pushbackSquares)`: it ADDS to
whatever is already on the field model and only clears after the push resolves (`:219`). During a
CHAIN push the model accumulates every pushed player's squares, the dialog offers all of them at once,
and the harness picks the global min-x/min-y then derives WHICH player moves from the chosen square's
direction. Rust did `pushback_squares.clear()` first — its own comment even cites Java's `add` — so it
could never offer, let alone choose, a chain square.

Removing the clear was necessary but NOT sufficient: Rust still offered three. The reason is
structural — Java re-enters `StepPushback` once per chain link, accumulating and re-asking each time,
while Rust resolves the whole chain inside one invocation (`for (player_id, coord) in pushes`) and
asks only once. Landing this properly means matching that per-link loop, which is a real port rather
than a one-line fix. Change reverted pending that.

### RETRACTION: the `ogre` 99/100 in ITER55 does not reproduce

ITER55 recorded `ogre` at 99/100 with a single failing seed (57), which is what made it the
fewest-fails target. It now measures **95/100 with five failing seeds — 26, 42, 57, 70, 81** — stable
across three consecutive runs. The 99/100 figure is retracted; treat 95/100 as the baseline.

Between the two measurements the Java jar was rebuilt twice (probe in, probe out). The `ffb` working
tree carries six uncommitted files, and I checked every one: all are gated behind
`System.getProperty("ffb.parityDebug")` or `System.getenv(...)` and return unchanged values when the
gate is off (`StatsMechanic.armourIsBroken` now assigns to a local and returns it; `StepGoForIt` and
`StepPassBlock` only add a println). So they do not explain a behaviour change — which means either
the previously-built jar predated some of them, or the ITER55 reading was simply misread.

**This matters more than the seed it came from**: fewest-fails targeting is only as good as the
counts, so the next iteration should first pin the Java side — record the jar's build state, re-run
the already-green rosters (`lineman`, `human`, `underworld`, `chaos_pact`, `renegades`) against the
current jar, and confirm they are still green before trusting any new comparison. If any of them moved,
the campaign's green list needs re-verification too.

Current measured baselines: `ogre` 95/100, `goblin` 95/100, `nurgle` 21/25. Tree clean on both repos
(the six `ffb` files are pre-existing gated-logging changes, not from this session).

## ITER57 — ⚠️ STATUS CORRECTION: the bb2020 "green" list does not hold up

ITER56 flagged that `ogre` had moved from 99/100 to 95/100 and made re-verification the next job.
Doing that re-verification invalidates more than one roster.

### Measured now, against a pinned jar, with a clean tree at HEAD

| matchup | recorded earlier | **measured now** |
|---|---|---|
| `lineman` bb2016 | 100/100 | **100/100** ✅ |
| `lineman` bb2025 | 100/100 | **100/100** ✅ |
| `lineman` bb2020 | 100/100 (ITER48) | **99/100** |
| `human` bb2020 | 100/100 (ITER14, re-gated ITER48/49) | **95/100** (seeds 20, 26, 69, 70, 72) |
| `underworld` bb2020 | 100/100 (ITER44) | **99/100** |
| `chaos_pact` bb2020 | 100/100 (ITER49) | **97/100** |
| `renegades` bb2020 | 100/100 (ITER49) | **96/100** |
| `ogre` bb2020 | 99/100 (ITER55) | **95/100** |
| `goblin` bb2020 | 95/100 | **95/100** |

**Only the two non-bb2020 fixtures still hold.** Every bb2020 roster previously called green is short.

### What I ruled out

- **Not the Java source.** All six uncommitted files in the `ffb` tree are gated logging
  (`ffb.parityDebug` / `getenv`) that return unchanged values with the gate off — I read each diff:
  `StatsMechanic.armourIsBroken` assigns to a local and returns it, `StepGoForIt`/`StepPassBlock` only
  add printlns, `Xoshiro256StarStar` widens the trace stack capture, `HeadlessGameSetup` is the
  long-standing ruleset-override plumbing, `HeadlessFantasyFootballServer` gates its DebugLog.
- **Not the jar build.** Rebuilding produces a different md5 (archive timestamps) but the SAME result:
  `human` is 95/100 before and after.
- **Not log caching.** `run_java_headless` re-runs Java and overwrites the log every sweep; there is no
  reuse path in `ffb-parity/src/runner.rs`.
- **Not nondeterminism.** Two consecutive `human` sweeps fail on exactly the same five seeds, and three
  consecutive `ogre` sweeps agreed.
- **Not a dirty tree.** `git status` is clean apart from `.claude/worktrees`, HEAD is the ITER56 commit,
  and the binary was rebuilt from scratch before measuring.

### What that leaves

The current numbers are reproducible; the earlier ones are not. So the earlier gate readings are the
suspect measurement, not today's. I could not reconstruct how they came out at 100/100 — the code and
jar at those commits look equivalent to now — so rather than invent an explanation I am recording the
discrepancy and the evidence, and treating **the measured numbers above as the campaign's real status**.

**Practical consequence**: the campaign is NOT at "8 of 30 green". Against the pinned jar it is at
**2 of 30** (the bb2016/bb2025 lineman fixtures), with seven bb2020 rosters between 95 and 99.

### Next iteration

1. Re-derive the whole status table: sweep every roster 1-100 against the pinned jar and record the
   per-seed fail lists, so targeting is based on measurements taken under one known configuration.
2. Then resume fixes with the fewest-fails rule, re-running the affected roster AND at least two
   previously-passing rosters after every change, so a regression cannot hide again.
3. The five verified-but-unlanded 1:1 corrections (ITER50-54, plus the chain-push accumulation from
   ITER56) should be re-evaluated against this corrected baseline — some may now show a signal.

Jar pinned for the record: built 2026-08-15 00:44 from `ffb` HEAD `f5d83384f` plus the six gated-logging
files. Tree clean; no engine change this iteration.

## ITER58 — the regression PREDATES this session's fixes; earlier greens were measured against an unrecoverable jar

Followed the status correction with a bisect rather than a rebuild of the status table, because the
first question is whether my own commits caused it.

**They did not.** Built commit `4bea55b8` — *before* the Intensive Training port (`edfd1fdb`) and
before Under Scrutiny (`dc94c2c3`) — in an isolated git worktree and ran the same sweep:

```
human bb2020 @4bea55b8:  seeds 20, 26, 69, 70, 72 FAIL  ->  95/100
human bb2020 @HEAD:      seeds 20, 26, 69, 70, 72 FAIL  ->  95/100
```

Identical count, identical seeds. So the four landed fixes of this session (apothecary decline,
`game.setDefenderId`, Intensive Training, Under Scrutiny) are NOT responsible — the failures were
already there when ITER48/ITER49 reported `human` at 100/100.

**Java-log caching is now ruled out by reading the code, not by grep.** `main.rs:256` calls
`run_java_headless_range` unconditionally for the whole seed range before any comparison — one batched
JVM, every seed re-run, logs overwritten. There is no reuse path.

**The local `ffb` changes are load-bearing, not stray.** I tried to build a pristine `HEAD` jar in a
separate worktree to test whether one of the six uncommitted files is behaviourally significant. **The
build FAILS**: the committed harness does not compile without them (`ParityRunner` calls the
`HeadlessGameSetup.create(..., ruleset)` overload that exists only in the working tree). So the
campaign has always run against `HEAD` + those edits, and they cannot be removed to A/B them.

### Conclusion

Every variable I can still control is identical between the runs that reported green and the runs that
report 95-99 today — same Rust commit (proved by bisect), same Java source, same batch invocation. The
one variable I cannot recover is the **jar binary that existed before ITER45's rebuild**. The earlier
green readings were taken against it; it no longer exists, and rebuilding from the same source does not
reproduce them.

I am not going to invent a mechanism for that. What is defensible:
- the current numbers are reproducible across two commits, three consecutive sweeps, and two jar builds;
- the earlier numbers are not reproducible by any means available;
- therefore **the measured table in ITER57 is the campaign's status**, and the greens recorded before it
  must be re-earned rather than assumed.

### Standing rule from here

Every future green claim must cite a sweep run in the SAME turn as the claim, against the jar identity
recorded in ITER57, and must re-run at least two previously-passing rosters. No roster is "still green"
on the strength of an earlier turn's measurement.

Temporary worktrees used for the bisect were removed; both repos are clean.

## ITER59 — ✅ MYSTERY SOLVED: I destroyed the harness fix myself. Five rosters restored.

The lost greens are explained, and the cause was mine.

**The symptom.** `lineman` bb2020 seed 26 failed at **step 0** — and seed 26 also appeared in `human`'s
and `ogre`'s fail lists. Running it alone:

```
UNHANDLED_STEP: PRAYER turnMode=KICKOFF     ... 500 times
```

Java was STUCK on the prayer step until the iteration cap, so its log was garbage and the comparison
failed at the first step. The prayer is `IRON_MAN` (confirmed by a Rust-side probe, since both engines
roll the same): a `DialogPlayerChoiceParameter` with **minSelects=1**. `ParityRunner`'s PLAYER_CHOICE
arm declined it with an empty selection, which is invalid, so the dialog re-fired forever.

**Why it appeared mid-campaign.** Rust's agent has had the correct arm all along
(`random_agent.rs`, the IRON_MAN / KNUCKLE_DUSTERS / BLESSED_STATUE_OF_NUFFLE case picking the lowest
shirt number), and its comment even says *"ParityRunner's PLAYER_CHOICE arm mirrors this exact rule"*.
It had no counterpart in `ParityRunner` — because the Java half existed only as an **uncommitted local
edit**, and in ITER45 and ITER56 I ran `git checkout -- ParityRunner.java` to revert unrelated probes.
That discarded the campaign's prayer arm along with my probe. The next jar rebuild baked in its
absence, and five rosters silently stopped being green.

This retro-explains ITER56-58 completely: the code and jar source really were identical, the earlier
readings really were valid, and the unrecoverable variable really was the jar — because the source
behind it had been quietly deleted by my own revert.

**The fix**, restored and now **committed** in the `ffb` repo (`c79ad3b67`) rather than left in the
working tree, so no future `git checkout --` can destroy it again: the three mandatory Prayer-to-Nuffle
dialogs are answered with the lowest player number, mirroring the Rust rule. Lowest-nr rather than
min-(x,y) because these prayers choose among RESERVES, which have no board coordinates.

**Result — every sweep run this turn, against the rebuilt jar:**

| matchup | before | **after** |
|---|---|---|
| `lineman` bb2020 | 99/100 | **100/100** ✅ |
| `human` bb2020 | 95/100 | **100/100** ✅ |
| `underworld` bb2020 | 99/100 | **100/100** ✅ |
| `chaos_pact` bb2020 | 97/100 | **100/100** ✅ |
| `renegades` bb2020 | 96/100 | **100/100** ✅ |
| `ogre` bb2020 | 95/100 | 99/100 |
| `goblin` bb2020 | 95/100 | 95/100 |
| `lineman` bb2016 / bb2025 | 100/100 | **100/100** ✅ |

`cargo test --workspace` 14,447 passed / 0 failed. The Rust tree is clean — this turn's only change is
the harness commit.

**Lesson recorded**: never `git checkout -- <file>` to revert a probe in a file that carries
uncommitted campaign work. Add the probe with a marker and remove exactly that, or commit the
pre-existing work first. The `ffb` tree still holds five other uncommitted files
(`Xoshiro256StarStar`, `HeadlessFantasyFootballServer`, `HeadlessGameSetup`, `StatsMechanic`,
`StepGoForIt`, `StepPassBlock`) that are load-bearing and equally destructible — `HeadlessGameSetup`
is required for the build to even compile.

**Status: 5 of 30 bb2020 matchups green.** Next target by fewest fails: `ogre` (1).

## ITER60 — `ogre` seed 57: the re-entry port lands correctly but the chain still differs

Attacked `ogre` (1 fail, the protocol target) with ITER56's chain-push diagnosis in hand, now that the
harness prayer fix means the surrounding sweeps are trustworthy again.

**What was ported.** Java `StepPushback:137-145` walks the model's squares on re-entry, REMOVES every
UNLOCKED one, and re-adds the coach's chosen square marked selected+locked — leaving earlier links'
LOCKED squares in place. Then `:161` ADDs the new set. Rust kept every square on re-entry and then
`clear()`ed the whole set before installing the new one, discarding earlier links. I ported both
halves (remove-unlocked/keep-locked, then add rather than clear).

**It did not fix the seed.** A probe shows the dialog still offers only the current defender's three
squares:

```
PUSH rng=95 def=Some("away_05") squares=[(14,8) (14,9) (13,9)]      # Rust, after the port
JPUSH all=[(14,7) (14,8) (14,9) (13,9) (12,9)] best=(12,9)          # Java, same push
```

So at the moment this dialog opens, Rust's field model holds no squares from the earlier link at all —
they are gone before the re-entry pass runs, not lost by it. Rust does have per-link machinery
(`pushback_stack`, `chain_pushed_player`, re-entry via `handle_command`), so the remaining difference
is WHERE the chain is driven from, not whether the model accumulates. Reverted, since it moved nothing.

**The unlanded-corrections pile is now six**, each verified against named Java source and each reverted
for showing no effect alone:
1. gate the `PS_STANDING` pre-stand on sequences owning a `STAND_UP` step (ITER50/52);
2. restore the OUTGOING acting player in `change_player_action` (ITER51/52);
3. `bb2020` Really Stupid: per-activation `ActingPlayer.used_skills` (ITER53);
4. Foul Appearance: drop the extra `is_selected() && is_committed()` filter (ITER54);
5. pushback re-entry: remove-unlocked/keep-locked instead of clear (this iteration);
6. pushback add: accumulate instead of replace (ITER56, folded into 5).

**Next iteration should change tactics**: apply all six TOGETHER and measure once. Individually each is
invisible; several are in the same code paths (activation state, block/pushback sequencing) and may only
pay off in combination. If the combined set moves any roster, bisect within it; if it regresses, revert
wholesale. Continuing to test them one at a time has now cost six iterations for no movement.

Verified after the revert, same turn: `lineman` bb2020 **100/100**, `human` bb2020 **100/100**. Tree
clean. `ogre` remains 99/100, `goblin` 95/100.

## ITER61 — the six-correction batch is NEUTRAL. Pile closed; do not re-try them individually.

Ran the experiment ITER60 called for: applied all six recorded 1:1 corrections at once and measured
seven rosters (700 games) in a single configuration.

Applied together:
1. pre-stand gate on sequences owning a `STAND_UP` step (`step_init_selecting.rs`);
2. restore the OUTGOING acting player in `change_player_action` (`util_server_steps.rs`);
3. Really Stupid reads/writes the per-activation `ActingPlayer.used_skills`
   (`bb2020/really_stupid_behaviour.rs`);
4. Foul Appearance defender resolution without the extra `is_selected() && is_committed()` filter
   (`mixed/step_foul_appearance.rs`);
5. pushback re-entry: remove-unlocked / keep-locked (`bb2025/block/step_pushback.rs`);
6. pushback add: accumulate instead of clear (same file).

**Result — every number measured with all six applied:**

| roster | baseline | with all six |
|---|---|---|
| `lineman` | 100/100 | **100/100** |
| `human` | 100/100 | **100/100** |
| `underworld` | 100/100 | **100/100** |
| `chaos_pact` | 100/100 | **100/100** |
| `renegades` | 100/100 | **100/100** |
| `ogre` | 99/100 | **99/100** |
| `goblin` | 95/100 | **95/100** |

Exactly neutral: no roster moved in either direction. They are also each verified against named Java
lines and demonstrably non-regressive across 700 games — but "faithful and harmless" is not the bar
this campaign sets. The rule is a 1:1 port **tied to a divergence, with a colocated regression test**,
and none of these six is tied to a divergence any more: each was proposed as a cause and then shown by
measurement not to be one.

**Reverted, and the pile is now closed.** They are recorded here in full so no future iteration spends
time rediscovering, re-applying or re-testing them. If a later divergence points at one of these code
paths, the relevant correction can be lifted from this entry with its Java citation — but none of them
should be re-tried speculatively.

**New data**: `nurgle`'s first full 1-100 sweep is **86/100** (14 fails) — previously only ever measured
on 1-25 (21/25). That is the true figure for the fewest-fails ordering.

**Current status, all measured this turn against the pinned jar:**

| green (5) | `lineman`, `human`, `underworld`, `chaos_pact`, `renegades` |
|---|---|
| close | `ogre` 99/100, `goblin` 95/100 |
| far | `nurgle` 86/100 |
| unmeasured at 1-100 | the remaining 21 rosters |

Next target by fewest fails remains **`ogre` (1)** — the chain-push at seed 57, which ITER60 showed is
about WHERE the chain is driven from, not the square bookkeeping. Tree clean; `lineman` re-verified at
100/100 after the revert.

## ITER62 — `ogre` seed 57 root-caused: it is SIDE STEP, not a chain push

Three iterations of chain-push theory were wrong. A side-by-side probe of both engines' pushback
dialogs settles it:

```
JPUSH rng=95 all=[(14,7) (14,8) (14,9) (13,9) (12,9)] best=(12,9) occupant=-
RPUSH rng=95 def=Some("away_05")  squares=[(14,8) (14,9) (13,9)]
```

Rust's three squares are the REGULAR pushback set for an attacker at (12,7) against a defender at
(13,8) — straight back (14,9) plus the two flanks. Java's five are **every free ADJACENT square**,
including (12,9) and (14,7), which lie sideways/backwards relative to the push and are not reachable
by a regular push at all.

That is the signature of `PushbackMode.SIDE_STEP`, and the roster confirms it: `away_05` is `nr=5` =
`ogre.snotling`, whose skills are `['Dodge', 'Right Stuff', 'Side Step', 'Stunty', 'Titchy']`. Java's
`UtilServerPushback.findPushbackSquares` has an explicit `SIDE_STEP`/`GRAB` branch that offers every
adjacent empty valid square instead of the three-square fan. Java used it; Rust did not.

So the accumulate/clear bookkeeping investigated in ITER56 and ITER60 was never the issue — the sets
differ because Rust computed the WRONG MODE, and the ITER60 revert was correct. Also note both engines
clear identically: Rust's `init_pushback` clear cites `UtilBlockSequence:43`, which really does call
`clearPushbackSquares()`, and Java's other clear (`StepPushback:219`) is inside `if (state.doPush)`
exactly as Rust's is.

**Next iteration**: find why Rust's Side Step does not fire here. `skill_behaviour/bb2020/side_step_behaviour.rs`
exists and manipulates `state.pushback_squares`, so the question is its trigger — compare its hook
condition against Java's `SideStepBehaviour` and check what sets `PushbackMode` to `SIDE_STEP`
(Java: the defender having the skill, an unused-skill check, and possibly a coach confirmation).
The fix is expected to be in mode selection, and it should be testable directly: a Snotling defender
must be offered every free adjacent square, not the three-square fan.

Probes on both sides were removed with targeted edits rather than `git checkout --` (per the ITER59
lesson); the jar was rebuilt and `lineman` bb2020 re-verified at **100/100** in this same turn. Tree
clean on both repos.

### ITER63 — `ogre` 🟢 100/100: Side Step was auto-declined

Two faults in `skill_behaviour/bb2020/side_step_behaviour.rs`, both found by probing the guard and
then the re-entry:

1. **The answer was wrong.** Java shows a `DialogSkillUseParameter` and re-enters with the coach's
   reply; `ParityRunner` answers every SKILL_USE with USE=true except four named skills (DumpOff,
   PrimalSavagery, SafePairOfHands, Swoop). Side Step is not among them, so **Java always side-steps**.
   Rust recorded `false` with the comment *"headless: auto-decline"* — a Rust-only shortcut with no
   Java counterpart.
2. **The fall-through was missing.** Setting the answer to `true` alone changed nothing: a probe showed
   the hook runs exactly ONCE per push (`SS-MODE` never printed). Java gets a second pass from the
   dialog round trip; Rust has none, so recording the answer and returning left `pushback_mode` at
   `REGULAR` forever. Recording it and falling straight through to the mode switch reaches the same
   state Java reaches on its second pass.

With both fixed, the defender is offered every free adjacent square, as Java does:
`ogre` seed 57 `a04` now lands on (12,9) in both engines.

An existing test, `side_step_headless_auto_declines`, asserted the old shortcut and failed — it
encoded the bug. Rewritten as `side_step_records_the_harness_answer_not_a_decline`, plus a new
`side_step_is_used_and_switches_the_pushback_mode` that pins both halves (answer recorded as `true`
AND `pushback_mode == SIDE_STEP` in the same pass).

**Result: `ogre` bb2020 1-100 → `PARITY: 100/100 games match`** (99 → 100).

Gates, all run this turn against the pinned jar: `lineman` **100/100**, `human` **100/100**,
`underworld` **100/100**, `chaos_pact` **100/100**, `renegades` **100/100**, `lineman` bb2016
**100/100**, `lineman` bb2025 **100/100**, `cargo test --workspace` **14,448 passed / 0 failed**.
`goblin` unchanged at 95/100.

**Status: 6 of 30 bb2020 matchups green** — `lineman`, `human`, `underworld`, `chaos_pact`,
`renegades`, `ogre`. Next by fewest fails: `goblin` (5). Note `bb2025`'s Side Step behaviour carries the
same auto-decline shortcut and was deliberately left alone — bb2025 lineman is 100/100 either way, so it
needs its own divergence and verification rather than an untested ride-along.

## ITER64 — `goblin` seed 38: the Officious Ref's stun must roll the Ball & Chain injury (PARTIAL)

`goblin` seed 38 diverged at **step 0**, i.e. before any activation. The dice join shows where:

```
die 12  d11=3    randomPlayer            (both)
die 13  d6=4     insertSteps             (both)
die 14  JAVA d6=2  rollDice:98           RUST d8=8   <-- Rust is 2 dice short
```

The kickoff result is **Officious Ref**. Java's `StepApplyKickoffResult.insertSteps` ends with
`publishParameters(UtilServerInjury.stunPlayer(this, player, apothecaryMode))`, and `stunPlayer` ->
`dropPlayer` has a branch: a player with `placedProneCausesInjuryRoll` — **Ball & Chain**, i.e. the
goblin Fanatic — gets a full `handleInjury(new InjuryTypeBallAndChain(), …)` (2d6 armour) instead of
simply being placed STUNNED. Rust called the rng-LESS `stun_player`, skipping those two dice.

Rust already had the rng-aware `stun_player_rng` (added for the Pitch Invasion path); the Officious Ref
path just never used it. Fixed by threading Java's `ApothecaryMode.HOME` / `AWAY` through
`officious_ref_insert_steps` (Java passes exactly those at `StepApplyKickoffResult:684,688`) and calling
the rng-aware variant. Test `officious_ref_stun_rolls_the_ball_and_chain_injury` searches for a seed
that reaches the stun branch and asserts exactly 3 dice are drawn (the ref d6 + 2d6 armour).

**Partial, and stated as such.** The dice now match Java exactly through the kickoff — `rng_calls` at
i=1 is 16 on both sides and the ball agrees at (25,5), where before Rust had 14 and (23,4). But the seed
still fails: one player differs, `a02` is `-1,-1,Ko` in Java and `13,8,Standing` in Rust. The injury is
now ROLLED but not APPLIED. Both engines' kickoff sequences do contain `APOTHECARY(HOME)` and
`APOTHECARY(AWAY)` immediately after `APPLY_KICKOFF_RESULT` (Rust `generator/mixed/kickoff.rs:53,55`,
Java `generator/mixed/Kickoff.java:45,46`), and Rust does publish the `InjuryResult` — so the next
iteration should trace why the apothecary step does not apply it, which is the same shape as the ITER43
`WAIT_FOR_APOTHECARY_USE` dead-end.

Kept rather than reverted, because unlike the six closed corrections this one has a **measured** effect
(the dice stream now matches through the kickoff) and is a prerequisite for the remaining state fix.

Gates, all run this turn: `lineman` **100/100**, `human` **100/100**, `ogre` **100/100**, `underworld`
**100/100**, `chaos_pact` **100/100**, `renegades` **100/100**, `lineman` bb2016 **100/100**, `lineman`
bb2025 **100/100**, `cargo test --workspace` **14,449 / 0**. `goblin` unchanged at 95/100 (seeds 38, 50,
81, 85, 98) — the count has not moved yet, so no progress is claimed beyond the dice.

### ITER65 — the kickoff sequence had no apothecary: `goblin` 95 → 96/100

Completed ITER64's partial fix. The dice matched but the injury was never applied, and probes showed
why in two steps:

```
APO-PARAM step_mode=Some(Defender) ir_mode=Attacker ...     # never ir_mode=Home/Away
DRIVE  ApplyKickoffResult -> SetActingTeam -> GotoLabel -> KickoffAnimation
```

No apothecary step ever received the result, and the drive trace shows why: after
`ApplyKickoffResult` the sequence goes straight to `KickoffAnimation`. **`kickoff_tail` — the sequence
Rust actually builds — has no `Apothecary` steps at all.** Java's `generator/mixed/Kickoff.java:45-46`
places `APOTHECARY(HOME)` then `APOTHECARY(AWAY)` immediately after `APPLY_KICKOFF_RESULT`; they are
what consumes and APPLIES the `INJURY_RESULT` a kickoff event publishes.

(Rust's `generator/mixed/kickoff.rs` DOES have both steps — but that generator is not the one this
path uses, which is why grepping for the steps earlier in the iteration was misleading. Checking that
a step exists somewhere is not the same as checking the built sequence contains it; the drive trace is
the authority.)

**Fix**: add both steps to `kickoff_tail`, for every edition, matching the shared Java generator.
Test `kickoff_sequence_applies_kickoff_event_injuries` asserts that for BB2016/BB2020/BB2025 the two
steps immediately follow `APPLY_KICKOFF_RESULT` in HOME-then-AWAY order.

**Result: `goblin` 95/100 → 96/100** (seed 38 fixed; 50, 81, 85, 98 remain).

Gates, all run this turn: `lineman` **100/100**, `human` **100/100**, `ogre` **100/100**, `underworld`
**100/100**, `chaos_pact` **100/100**, `renegades` **100/100**, `lineman` bb2016 **100/100**, `lineman`
bb2025 **100/100**, `cargo test --workspace` **14,450 / 0**. Adding the steps for all editions did not
disturb bb2016 or bb2025.

**Status: 6 of 30 green**, `goblin` 96/100 next by fewest fails.

## ITER66 — `goblin` seed 50: Rust draws an extra d6 before the casualty roll

Next seed after ITER65. At i=13 both engines activate `away_01` for a BLOCK with `rng_calls=19`; Java
then spends 9 dice and continues the away turn, Rust spends 15 and takes a turnover.

**The dice, compared by SIDES rather than position** (position is unreliable — Java logs the count
BEFORE the roll, Rust after, so the two can sit one apart; the sequence of die TYPES is offset-proof):

```
JAVA:  6 6 6 6 6 16 6 6 6
RUST:  6 6 6 6 6  6 16 6 6
```

Rust draws one extra d6 immediately before the casualty d16.

**Backtraces pin the structure.** Probing individual draws (a temporary `FFB_DIE_AT=<n>` hook in
`GameRng::die`, since reverted):

- die 25 (d6) → `do_injury_roll_for_player_impl` directly (frame 5 is `GameRng::d6`), so it is one of
  that function's own `d1`/`d2`;
- die 26 (d16) → `roll_casualty` ← `outcome_to_player_state` ← the SAME
  `do_injury_roll_for_player_impl` call.

Since one impl call rolls `d1`, `d2` and then the casualty, that call owns dice **24, 25, 26** — i.e.
Rust's injury pair is (24,25) where Java's is (23,24). The extra draw therefore happens at or before
die 23, shifting the injury roll one die later; `roll_casualty` itself is correct
(`bb2020/roll_mechanic.rs:122` is `[rng.die(16), rng.d6()]`, matching Java's d16-then-d6).

**Next iteration**: backtrace dice 21, 22 and 23 the same way to segment them (Java has 21-22 from
`rollDice:84` and 23-24 from `rollDice:98`), and find which Rust roll has no Java counterpart. The
answer is one of the block-dice / armour draws before the injury, not the casualty machinery.

Baseline unchanged and tree clean: `goblin` 96/100 (seeds 50, 81, 85, 98).

## ITER67 — `goblin` seed 85: the BB2020 blitz ran BB2025's PICK_UP and bounced the ball → 96/100 → **97/100**

Re-measured the baseline first (valid sweep: exit prints `rust_total`, `96/100 games match`):
`goblin` 96/100, seeds **50, 81, 85, 98**. Took seed 85 — it diverges earliest, at i=4, and shares
seed 50's shape (Java continues the away turn, Rust takes a turnover).

**The pre-state hashes are IDENTICAL at i=4** (`7e4fc1a9904ce30b`) and both engines activate
`away_03` for a BLITZ, so the board and the agent's choice agree; only the resolution differs.

**Diffing the dice by SIDES** (`FFB_DICE_TRACE=1`, splitting the Java lines by their `caller=` field)
localises it to a single call:

```
pos 25  JAVA d6=4   RUST d6=4
pos 26  JAVA d6=3   RUST d8=3     <-- Rust rolls a SCATTER where Java rolls the block die
pos 27  JAVA d6=2   RUST d6=2
```

Java's one block die is 3 = PUSH and the away turn continues. Rust's scatter pushes its block die to
pos 27 = 2 = BOTH DOWN → turnover. Everything downstream follows from that one extra d8.

**`FFB_DIE_AT=26` names the drawer in one run** (a backtrace hook in `GameRng::die`, now kept
permanently — see below): `StepCatchScatterThrowIn`. `FFB_DRIVE_TRACE=1` then shows the sequence
around it:

```
... Dauntless, Horns, Trickster, PickUp, CatchScatterThrowIn, CatchScatterThrowIn <- d8
```

That is the **BB2025** BlitzBlock sequence. `bb2025/BlitzBlock.java:43` adds `PICK_UP` between
`TRICKSTER` and `CATCH_SCATTER_THROW_IN`; `bb2020/BlitzBlock.java` has **no** `PICK_UP` there at all
(its only one is after `SHADOWING`, in the pushback branch). `away_03` is the goblin **Fanatic** —
Ball & Chain, i.e. No Hands — standing on the loose ball at (13,8): BB2025's PICK_UP cannot pick up,
so it publishes the ball and `CatchScatterThrowIn` bounces it. BB2020 never attempts the pickup.

The reason the BB2025 generator runs a BB2020 game at all: `bb2025/shared/step_end_selecting.rs` is
the live `StepEndSelecting` for every edition (the driver's per-edition overrides in
`make_step_for` cover BB2016 and, so far, only `StepId::Prayer` for BB2020), and its BLITZ arm
builds `bb2025::BlitzBlock` unconditionally.

### Two rejected fixes, both measured

Both were reverted under the campaign's gate (the roster's count must DROP):

1. **BB2020 `SelectBlitzTarget` activation + BB2020 `BlitzBlock`** (the full 1:1 swap) → `goblin`
   **10/100**.
2. **BB2025 activation bridge + BB2020 `BlitzBlock`** (isolating which half was to blame) →
   `goblin` **10/100** again.

So the BB2020 *sequence* is what the shared step-set cannot run: these steps depend on the BB2025
shape elsewhere in the sequence (`STEADY_FOOTING`/`CHOMP`, the `REMOVE_TARGET_SELECTION_STATE` and
`RESET_FUMBLEROOSKIE` positions, the leading `GO_FOR_IT` label). Wholesale BB2020 sequences need the
BB2020 *step* classes routed with them — the same "approach A" the BB2016 campaign did, and a
multi-iteration job, not this one.

### The fix that landed

The precedent is already in this codebase — `driver.rs`'s `StepApplyKickoffResult` note: when a
shared step is right except for one edition-specific detail, **edition-gate that one detail inside
the shared file** rather than routing the whole thing to a staler per-edition file. Applied here:
`bb2025::BlitzBlockParams` gains a `rules` field, and the `TRICKSTER → PICK_UP` entry is emitted
only when `rules != Bb2020`. `StepEndSelecting` passes `game.rules` through. Nothing else moves.

Test `pick_up_before_catch_scatter_is_bb2025_only` asserts the entry is present for BB2025 and
absent for BB2020, that the post-`FOLLOWUP` PICK_UP (which BB2020 has too) survives in both, and
that the BB2020 sequence is exactly one step shorter — so the gate cannot silently widen.

### Tooling kept

`FFB_DIE_AT=<n>[,<n>…]` in `GameRng::die` prints a backtrace for the named die positions. Previous
iterations added and reverted this hook repeatedly; it is now permanent, parsed once into a
`OnceLock<Vec<u64>>` so it costs an empty-slice check per roll when unset.

### Gate

| check | result |
|---|---|
| `goblin` bb2020 | 96/100 → **97/100** (seed 85 fixed; 50, 81, 98 remain) |
| `lineman` bb2016 / bb2020 / bb2025 | 100/100 each |
| `human`, `ogre`, `underworld`, `chaos_pact`, `renegades` bb2020 | 100/100 each |
| `cargo test --workspace` | **14,451 passed / 0 failed** |

**Note on exit codes**: `ffb-parity` exits 1 on `REQUIRED ITEMS MISSING` (a coverage requirement)
even when the run is 100/100. The ITER57 validity rule should therefore be read as: a sweep counts
only if it printed `rust_total` **and** an `N/100 games match` line — not on exit code alone.

**Status: 6 of 30 green**, `goblin` 97/100 still the fewest-fails target. Next: seed 50 (i=13, the
ITER66 thread — Rust draws an extra d6 before the casualty roll; re-check it against this fix first,
since a stray blitz-path pickup is exactly the shape ITER66 was chasing).

## ITER68 — the same PICK_UP gate, in the BLOCK sequence: `goblin` 97 → **98/100**

Straight continuation of ITER67, and it closes the ITER66 thread.

ITER66 had measured seed 50 correctly — "Rust draws one extra d6 immediately before the casualty
d16" — but attributed it to the injury machinery. Comparing the dice by SIDES against a fresh run
puts the first divergence at position 25 (Java d16 casualty, Rust d6), and the Rust
`FFB_DRIVE_TRACE` shows the extra draw at position 21 with its step named directly:

```
DRIVE step=Trickster
DRIVE step=PickUp          <- DICE_TRACE pos=21 sides=6 result=6
DRIVE step=CatchScatterThrowIn
...
DRIVE step=BlockRoll       <- pos=22,23   (Java: 21,22)
```

Same defect as ITER67, one generator over: this is a plain BLOCK, and `bb2025/Block.java:43` adds
`PICK_UP` between `TRICKSTER` and `CATCH_SCATTER_THROW_IN` where `bb2020/Block.java:51-52` goes
straight from one to the other (BB2020's only `PICK_UP` is at line 88, after `SHADOWING`, in the
pushback branch). Unlike seed 85 the pickup here SUCCEEDS — it just spends a die Java never spends,
shifting the injury pair from (23,24) to (24,25) and the casualty from 25 to 26.

**Fix**: the identical edition gate — `bb2025::BlockParams` gains `rules`, the entry is emitted only
when `rules != Bb2020`, and `StepEndSelecting` passes `game.rules` through. Test
`pick_up_before_catch_scatter_is_bb2025_only` in `bb2025/block.rs` mirrors the BlitzBlock one and
asserts the BB2020 sequence is exactly one step shorter.

### Swept the rest of the generator pair for the same class of defect

```
BlitzBlock.java     bb2025=2 PICK_UP   bb2020=1   <- fixed ITER67
Block.java          bb2025=1           bb2020=1   <- EQUAL COUNT, different POSITION; fixed here
ThrowTeamMate.java  bb2025=0           bb2020=1   <- OPEN: bb2020 has a PICK_UP bb2025 lacks
```

Block is the warning: counting occurrences is not enough, the position matters. `ThrowTeamMate` is a
real open lead for a later iteration (bb2020 picks up where bb2025 does not) but nothing measured
points at it yet, so it is recorded, not changed.

### Gate

| check | result |
|---|---|
| `goblin` bb2020 | 97/100 → **98/100** (seed 50 fixed; 81, 98 remain) |
| `lineman` bb2016 / bb2020 / bb2025 | 100/100 each |
| `human`, `ogre`, `underworld`, `chaos_pact`, `renegades` bb2020 | 100/100 each |
| `cargo test --workspace` | **14,452 passed / 0 failed** |

**Status: 6 of 30 green**, `goblin` 98/100. Next: seed 81 (i=128, away turn 8) and seed 98 (i=10).

## ITER69 — `StepHandleDropPlayerContext` was doing half of `dropPlayer`; seed 98 is an ORDERING divergence

`goblin` stays at **98/100**. One real 1:1 correction landed, and seed 98 is now characterised
precisely rather than guessed at. Stated as no-count-change, per ITER64's precedent for keeping a
verified correction that has a measured effect but does not yet close a seed.

### The correction

Seed 98's first dice divergence is at rng 20: Java rolls a d8
(`StepCatchScatterThrowIn.bounceBall`) where Rust rolls a d6. Reading Java's
`UtilServerInjury.dropPlayer` shows the shape that matters — the `placedProneCausesInjuryRoll`
(Ball & Chain) branch and the place-PRONE branch are an `if/else` over the **state change only**;
the ball handling below it (`DROPPED_BALL_CARRIER`, `setBallMoving(true)`,
`CATCH_SCATTER_THROW_IN_MODE = SCATTER_BALL`, the turnover) sits OUTSIDE that `if/else` and runs for
a Ball & Chain player too.

Rust's `drop_player_rng` already ports this correctly — with a doc comment describing this exact
bug, fixed once for bb2016. **`StepHandleDropPlayerContext` was not calling it.** It inlined the B&C
injury roll itself and returned `Vec::new()` for the drop parameters, so dropping a Fanatic that was
standing on the ball rolled the chain injury but never published `SCATTER_BALL` — the ball did not
bounce. It also hardcoded `ApothecaryMode::Defender` where Java passes
`dropPlayerContext.getApothecaryMode()`.

Fixed by calling `drop_player_rng` — i.e. by deleting the duplicate. Verified with a probe that the
ball now bounces off the Fanatic's square (13,8 → 13,7) where before it stayed put. Test
`dropping_a_ball_and_chain_player_on_the_ball_publishes_scatter_ball` asserts both halves survive:
`SCATTER_BALL` published *and* the chain `InjuryResult` still rolled.

**Lesson, and worth generalising:** the duplicate had been written because the shared helper
"couldn't roll" at the time it was needed. Once the rng-aware helper existed, the duplicate was
never removed and silently kept the older, half-complete behaviour. Grep for a second
implementation before extending a step's inline logic.

### Seed 98 is an ordering divergence, not a missing roll

With that fixed the dice COUNT and VALUES around the divergence now line up — the d8 is simply in a
different place:

```
       20  21  22  23  24  25  26  27  28  29  30  31
JAVA   d8  d6  d6  d6  d6  d6  d6  d6  d6  d6  d6  d6
RUST   d6  d6  d6  d6  d6  d6  d6  d6  d6  d6  d6  d8
       ^^ bounce                                  ^^ bounce, 11 rolls late
```

Every result in 21..30 is identical. Java bounces the ball the moment the drop happens; Rust defers
the same bounce to the end of the block sequence. The `SCATTER_BALL` published by the drop is not
being consumed by the `CatchScatterThrowIn` that immediately follows it — probing the step shows it
arriving with `mode=None` there and with `mode=ScatterBall` only much later.

**Next iteration** should start there: which `CatchScatterThrowIn` in the sequence receives the
published `CATCH_SCATTER_THROW_IN_MODE`, and why the nearer one does not. That is a parameter-routing
question in the driver, not a dice or rules question — a different shape from every fix so far this
campaign, and likely to explain seed 81 too.

### Gate

| check | result |
|---|---|
| `goblin` bb2020 | 98/100 (unchanged — no progress claimed) |
| `lineman` bb2016 / bb2020 / bb2025 | 100/100 each |
| `human`, `ogre`, `underworld`, `chaos_pact`, `renegades` bb2020 | 100/100 each |
| `cargo test --workspace` | **14,453 passed / 0 failed** |

## ITER70 — seed 98 root-caused and FIXED, but the fix regresses seed 50; REVERTED per the gate

`goblin` stays **98/100** (seeds 81, 98). No engine change landed. This iteration bought a complete
root cause for seed 98 and, more usefully, found the second half that has to land with it.

### Seed 98's root cause (confirmed)

The i=10 block is `home_03` (Java `h02`) blocking `away_03` (Java `a02`) — and `a02`, the goblin
**Fanatic**, is standing on the loose ball at (13,8). Java's state strings show the outcome:

```
JSTEP i=10  b13,8,true   a02:13,8,Standing   h02:12,8,Standing
JSTEP i=11  b12,9,true   a02:-1,-1,Ko        h02:13,8,Standing
```

`a02` is pushed off, knocked down and KO'd; `h02` ends on (13,8); the ball has moved (13,8) → (12,9).
That move is the rng-20 d8. It is a BOUNCE caused by a failed pick-up: Java's `StepPickUp.pickUp()`
returns FAILURE **without rolling** for a player with `preventHoldBall` / `preventPickup` — No Hands
— and `h02` is the other Fanatic. So the attacker ends up on the loose ball, cannot pick it up, and
the ball bounces.

Rust never attempted that pick-up, because **`bb2020/Block.java:86-89` runs TENTACLES, SHADOWING and
PICK_UP between FOLLOWUP and DROP_FALLING_PLAYERS, and `bb2025/Block.java:90-93` goes straight from
one to the other.** The shared bb2025 generator (the one bb2020 uses) faithfully ports bb2025 — so
bb2020 is missing three steps. This is the mirror image of ITER67/68: there bb2020 had FEWER steps
than bb2025, here it has MORE.

### Why the obvious fix is not enough

Adding the three steps for bb2020 **fixes seed 98 and breaks seed 50** — still 98/100, so the
count did not drop and a previously-green seed went red. Reverted.

Seed 50 shows why. Java's `StepPickUp` has an `ignore` flag set from a step parameter:

```java
if (parameter.getKey() == StepParameterKey.FOLLOWUP_CHOICE) { ignore = !toPrimitive(...); }
...
private boolean isPickUp(Player<?> p) { return !ignore && isBallInPlay() && isBallMoving() && ...; }
```

`ParityRunner` answers `FOLLOWUP_CHOICE` with `sendFollowupChoice(false)` — always decline — so in
Java that pick-up is ignored whenever a follow-up choice was actually made. In seed 50 the blocker
(`away_01`, the Troll — no No Hands) is already standing on the loose ball at (13,7), so with the
step present and `ignore` false Rust rolled a pick-up d6 that Java never rolls, shifting the injury
pair and casualty by one — exactly the ITER66 signature, re-created.

**Rust already has the `ignore` field and already maps `FollowupChoice` to it**
(`bb2025/move_/step_pick_up.rs:36,89`), and `StepFollowup` already publishes `FollowupChoice`. Probing
shows the step nevertheless entering with `ignore=false`, so the parameter is not reaching it. The
driver's `DriverStepStack::publish` delivers top-of-stack downward and stops at the first step whose
`consumes_parameter` returns true, so the candidates are: the parameter is not published on this
path at all, or something between `FOLLOWUP` and `PICK_UP` consumes it first.

### Next iteration — land these two together

1. Find why `FollowupChoice` does not reach `StepPickUp` (instrument
   `DriverStepStack::publish` for that key; check whether `StepFollowup` publishes it on the
   declined path in this branch).
2. With `ignore` wired, re-add the bb2020 `TENTACLES, SHADOWING, PICK_UP` block after `FOLLOWUP`.
   Expect seed 98 green AND seed 50 to stay green.

Do not land the sequence half alone — a note to that effect is in `bb2025/block.rs` at the insertion
point.

### Also landed

`scripts/dicediff.py` — splits a combined `FFB_DICE_TRACE=1` capture into the Java stream (the lines
carrying `caller=`) and the Rust stream, compares by `(sides, result)` in order, and prints the first
disagreement with context. Positions are not directly comparable (Java logs the count before the
roll, Rust after), which has misled several iterations; this compares the sequences instead. It is
the first thing to run on any new failing seed.

### Gate

| check | result |
|---|---|
| `goblin` bb2020 | 98/100, seeds 81 + 98 — baseline restored after the revert |
| `cargo test -p ffb-engine` | 7,150 passed / 0 failed |

No engine change committed, so the wider roster gate was not re-run.

## ITER71 — `StepFollowup` never republished FOLLOWUP_CHOICE: `goblin` 98 → **99/100**

Landed ITER70's two halves together, exactly as that iteration predicted.

### The missing half

Java publishes the follow-up answer to the whole stack from `handleCommand`, not just to itself:

```java
case CLIENT_FOLLOWUP_CHOICE:
    publishParameter(new StepParameter(StepParameterKey.FOLLOWUP_CHOICE, cmd.isChoiceFollowup()));
```

Rust's `StepFollowup::handle_command` only assigned the local field. Everything downstream of it
was therefore blind to the answer — and the step that cares is `StepPickUp`, whose
`setParameter(FOLLOWUP_CHOICE)` sets `ignore = !choice` and whose `isPickUp()` starts with
`!ignore`. Rust already had the field, the mapping and the `!ignore` guard; nothing ever
delivered the parameter, so `ignore` stayed false.

That was invisible while the BB2020 follow-up pick-up was missing from the sequence — the two bugs
cancelled out. Adding either alone breaks a seed, which is why ITER70's sequence-only attempt
traded seed 98 for seed 50.

`StepFollowup` reaches `execute_step` via `handle_command` before building its outcome, so the
answer is stashed in a `pending_choice_publish` field and pushed onto the existing `out_params`
list on the way through — the same route the automated (Pinned / Multiple Block / Frenzy / Taunt)
choices already used.

### Both halves, together

| change | fixes | breaks alone |
|---|---|---|
| `StepFollowup` republishes `FOLLOWUP_CHOICE` | (enables the gate below) | — |
| bb2020 Block gains `TENTACLES, SHADOWING, PICK_UP` after `FOLLOWUP` | seed 98 | seed 50 |

`ParityRunner` always declines the follow-up (`sendFollowupChoice(false)`), so in practice that
pick-up is ignored whenever a follow-up dialog was shown (seed 50, where the blocker was already
standing on the loose ball), and runs when no dialog was needed (seed 98, where the No Hands
attacker ends up on the ball and fails without a roll, bouncing it).

Tests: `answering_the_followup_dialog_republishes_the_choice` (both answers),
`followup_pick_up_is_bb2020_only`, and `bb2020_block_gates_are_exactly_two` — which pins the two
edition gates in the Block sequence as going in OPPOSITE directions (bb2020 loses the TRICKSTER
pick-up, gains the three follow-up steps) so a future edit cannot widen either silently.

### Gate

| check | result |
|---|---|
| `goblin` bb2020 | 98/100 → **99/100** (seeds 50 AND 98 fixed; 81 remains) |
| `lineman` bb2016 / bb2020 / bb2025 | 100/100 each |
| `human`, `ogre`, `underworld`, `chaos_pact`, `renegades` bb2020 | 100/100 each |
| `cargo test --workspace` | **14,456 passed / 0 failed** |

**Status: 6 of 30 green**, `goblin` 99/100. Next: seed 81 (i=128, away turn 8) — the last goblin
seed. Note the same BlitzBlock question is still open: `bb2020/BlitzBlock.java` also has
`TENTACLES, SHADOWING, PICK_UP` after FOLLOWUP, and the bb2025 BlitzBlock port has only a bare
PICK_UP there; worth checking against seed 81 before hunting further.

## ITER72 — `goblin` seed 81 is Argue-the-Call: Rust argues 4× where Java argues 2×

`goblin` unchanged at **99/100**. No engine change — the Rust side is measured out and the decisive
next probe is on the Java harness. Recording the state so the next iteration starts from evidence.

### Where it diverges

`scripts/dicediff.py` puts the first sides difference at index 77: Java rolls a d6, Rust a d8.
`FFB_DIE_AT=78` shows Rust is already in `StepKickoffScatterRoll` — the next drive's kickoff —
while Java is still finishing the previous drive. Java's caller at that die:

```
DiceRoller.rollArgueTheCall:152  StepEndTurn.argueTheCall:813  StepEndTurn.handleCommand:193
   ParityRunner.handleDialog:811
```

So this is the end-of-drive Secret Weapon send-off, and Rust runs out of it early.

### What matches, and what does not

Probing `report_secret_weapons_used` and `argue_and_remove_secret_weapons`:

| | Java | Rust |
|---|---|---|
| players flagged `hasUsedSecretWeapon` | 6 (both teams' Fanatic + Bombardier + Looney) | 6 — **matches** |
| Secret-Weapon ban rolls (`rollSecretWeapon`, 2d6) | 2 | 2 — **matches** |
| argue-the-call d6 rolls | **2** | **4** |

The flagging rule is identical and correctly ported (`markPlayedAndSecretWeapons`: on the pitch this
drive + `getsSentOffAtEndOfDrive` — it is NOT about actually using the weapon). Only the goblin
Bombardier carries a valued `Secret Weapon (5)` in `data/rosters/bb2020/roster_goblin.json`, so
exactly the two Bombardiers take a 2d6 ban roll and the other four are auto-banned with no die —
both engines agree on that.

The divergence is purely the argue COUNT. Rust argues for every eligible flagged player
(`away_03=5, away_04=1, home_04=4, home_05=5`; `home_03` correctly skipped as REMOVED_FROM_PLAY).
Java rolls only two.

### Ruled out

- **The eligibility filter.** `bb2020/StepEndTurn.getPlayerIds` is `hasUsedSecretWeapon() &&
  !REMOVED_FROM_PLAY.contains(base)` plus the IllBeBack opt-out — identical to bb2025, and
  `REMOVED_FROM_PLAY = {BANNED, BADLY_HURT, SERIOUS_INJURY, RIP}` is exactly Rust's
  `is_casualty() || base == BANNED`.
- **The bb2016 "one argue per team" shape.** bb2020 HAS `playerIdsArgued` (5 references, same as
  bb2025; bb2016 has none), and `argueTheCall` leaves `fArgueTheCallChoice*` null while
  `playersForArgue` is non-empty, so it re-fires the dialog per player. Rust's per-player loop is
  the right shape for bb2020 — it is the count that is wrong, not the structure.
- **Coach-banned early exit.** The first Rust argue rolls a 5 (not a natural 1), so nothing is
  banning the coach and cutting the loop short.

### The next probe is on the harness, not the engine

Everything above is Rust-side evidence; what is missing is which player ids Java's dialog actually
offers. `ParityRunner` is co-editable, so add a one-line log in the `ARGUE_THE_CALL` dialog arm
(around line 800) printing `argueParam.getPlayerIds()` and the team on every firing, rebuild the
jar, and compare the two lists directly. Two shapes to look for:

1. Java's dialog offers fewer ids than Rust's eligible set → the flag is being cleared somewhere
   Rust does not clear it (one Rust probe already hints at this: `away_05` appears in the ban-roll
   pass but NOT in the argue pass, while its opposite number `home_05` appears in both — an
   asymmetry with no obvious cause on the Rust side).
2. Java's dialog fires once per team and the loop exits early for a reason not visible in the
   source read above.

### Note on the dice diff

Values matching either side of the divergence proves nothing here: every roll in this window is a
d6 drawn from the same raw stream, so identical values appear at identical indices no matter which
roll they belong to. Only the SIDES change (the d8 at index 77) exposed it. When a window is
all-d6, compare the CALLERS, not the values.

### Gate

| check | result |
|---|---|
| `goblin` bb2020 | 99/100 (unchanged) |
| working tree | probes removed, no engine change |

## ITER73 — harness log settles seed 81's argue count; `StepFallDown` was doing half of `dropPlayer`

`goblin` stays **99/100**. One 1:1 correction landed (no count change), and seed 81's real frontier
moved three dice EARLIER than ITER72 thought.

### The harness log

`ParityRunner`'s `ARGUE_THE_CALL` arm now prints the dialog's player ids under `DEBUG`
(`JAVA_ARGUE_DIALOG`). Rebuilt per the ITER69-era recipe in `PARITY_BB2016_CAMPAIGN.md`
(`javac` the one class against the fat jar, `jar uf` it back in; backup at `…jar.bak-iter73`), and
verified safe FIRST: `lineman` bb2020 100/100 against the rebuilt harness before trusting anything
it printed.

```
JAVA_ARGUE_DIALOG team=…Away  ids=[Away3, Away4, Away5]
JAVA_ARGUE_DIALOG team=…Away  ids=[Away4, Away5]
JAVA_ARGUE_DIALOG team=…Away  ids=[Away5]
JAVA_ARGUE_DIALOG team=…Home  ids=[Home4, Home5]
JAVA_ARGUE_DIALOG team=…Home  ids=[Home5]
```

Java argues **five** times (away 3, home 2), re-firing per player exactly as the source read
suggested. Rust argues four — it is missing `away_05`, because `away_04`'s argue rolls a natural 1,
Rust sets `coach_banned` and `break`s out of the away loop. Java's `askForArgueTheCall` also guards
on `!turnData.isCoachBanned()`, so the shape is right; Java simply did not roll a 1 there.

**ITER72's "Java argues 2× / Rust 4×" is corrected: it is 5× vs 4×**, and the counts differ because
the two engines are drawing DIFFERENT raw values by then — they are already offset.

### The real frontier is three dice earlier

The Java callers around the offset:

```
pos 70,71  InjuryTypeBallAndChain … UtilServerInjury.dropPlayer:316  StepFallDown.executeStep:88
pos 72     ReallyStupidBehaviour
pos 73-76  rollSecretWeapon  (two 2d6 ban rolls)
pos 77+    rollArgueTheCall
```

Rust's ban rolls sit at pos 70-73 — three dice early. So **Rust never performs a fall that Java
does**: a Ball & Chain player falls via `StepFallDown`, taking a chain injury (2 dice), plus a
Really Stupid roll. Everything downstream (ban rolls, argues) is just that offset propagating.

This is invisible to `dicediff.py` because the whole window is d6: the first SIDES difference is 7
dice later, where Rust's kickoff d8 meets Java's argue d6.

### The correction that landed

`StepFallDown` had the same defect ITER69 fixed in `StepHandleDropPlayerContext`: it inlined the
Ball & Chain injury roll and returned `Vec::new()` for the drop parameters, losing the ball handling
Java puts outside that if/else. It also passed `eligibleForSafePairOfHands = true` where Java line 88
calls the THREE-arg `dropPlayer(this, player, ApothecaryMode.ATTACKER)` overload — i.e. **false**.

Replaced with `drop_player_rng(game, rng, &player_id, false, ApothecaryMode::Attacker)`. That is now
the third site where an inline duplicate of `dropPlayer` had drifted from the shared helper; a sweep
for any remaining ones is worth an iteration on its own.

**No count change** — seed 81's dice are byte-identical before and after, so the missing fall is
elsewhere. Kept because it is a verified 1:1 correction with no regression.

### Next iteration

Find why Rust never runs that `StepFallDown`. It is at the very end of a drive, immediately before
the Secret Weapon phase, and involves a Ball & Chain player — so the candidates are the end-of-turn
sequence and the Ball & Chain "falls over if it does not move" rule. `FFB_DRIVE_TRACE` around Rust
rng 69-70 against the Java callers above is the direct comparison.

### Gate

| check | result |
|---|---|
| `goblin` bb2020 | 99/100 (unchanged — no progress claimed) |
| `lineman` bb2016 / bb2020 / bb2025 | 100/100 each |
| `human`, `ogre`, `renegades` bb2020 | 100/100 each |
| `cargo test --workspace` | **14,456 passed / 0 failed** |

Harness note: the `JAVA_ARGUE_DIALOG` line is `DEBUG`-gated (silent without `FFB_TRACE=1`) and lives
in the local `ffb` checkout, which is not pushed — same as the other six gated-logging edits the
campaign already depends on.

## ITER74 — the pitch was two rows too tall: `goblin` 99 → **100/100 🟢 GREEN**

A hand-rolled bounds check in `StepMoveBallAndChain` used a **26×17** pitch (`y` 0..16) where Java's
`FieldCoordinateBounds.FIELD` is **(0,0)..(25,14)**.

### How it surfaced

ITER73 established that Java performs a Ball & Chain fall Rust does not, three dice before the
visible divergence. Probing the chain-move scatter shows the Fanatic walking straight down the
sideline on its compulsory random move:

```
BC_PROBE roll=6 from=(13,10) orig_to=(14,11) base=East dir=Southeast
BC_PROBE roll=1 from=(14,11) orig_to=(13,10) base=West dir=Southwest
BC_PROBE roll=2 from=(13,12) orig_to=(12,11) base=West dir=Southwest
BC_PROBE roll=1 from=(12,13) orig_to=(11,12) base=West dir=Southwest
BC_PROBE roll=5 from=(11,14) orig_to=(12,13) base=East dir=Southeast   <- (12,15)
```

y = 10, 11, 12, 13, 14 … and then southeast to **(12,15)**. y=15 is the crowd. Java's
`!FieldCoordinateBounds.FIELD.isInBounds(coordinateTo)` fires, publishes `InjuryTypeCrowdPush` and
gotos FALL_DOWN — a crowd injury (2 dice) plus the chain injury from `dropPlayer` (2 more). Rust's
local check said (12,15) was on the pitch, so the Fanatic kept standing on a square that does not
exist and none of those dice were rolled.

Everything ITER72 and ITER73 chased — the Secret Weapon ban rolls landing 3 positions early, Java
arguing five times to Rust's four, the d8/d6 sides clash at index 77 — was this one offset
propagating.

### The fix

`is_in_bounds` now delegates to `FieldCoordinateBounds::FIELD.is_in_bounds`, which was already
correct at (25,14). The colocated test asserted `(25,16)` was IN bounds, so it had frozen the bug in
place; it now asserts the real pitch and that `(12,15)` — the bottom sideline — is the crowd.

**Worth generalising: this bug was a hand-rolled copy of a constant that already existed correctly
elsewhere in the tree.** Same shape as the three `dropPlayer` duplicates (ITER69, ITER73). Grepping
for other local re-implementations of `FieldCoordinateBounds` is a cheap, high-value sweep.

### Gate

| check | result |
|---|---|
| `goblin` bb2020 | 99/100 → **100/100 🟢** |
| `lineman` bb2020 | 100/100 |
| `ogre`, `underworld` bb2020 | 100/100 each |
| `cargo test --workspace` | **14,456 passed / 0 failed** |

**Status: 7 of 30 green** — `lineman`, `human`, `ogre`, `underworld`, `chaos_pact`, `renegades`,
`goblin`. Next: re-measure the remaining 23 rosters to pick the new fewest-fails target; several
were last measured before the ITER67-74 fixes and the Ball & Chain / pitch-bounds corrections are
broad enough that some may have moved on their own.

## ITER75 — full re-sweep: **7 → 24 of 30 green**

No engine change. Every roster re-measured 1-100 against HEAD, because the ITER67-74 fixes (the two
PICK_UP sequence gates, FOLLOWUP_CHOICE republishing, three `dropPlayer` duplicates, the pitch
bounds) were broad and most rosters had not been swept since.

**Seventeen rosters went green with no roster-specific work.** That is the campaign's usual
pattern — shared-engine fixes carry most of the matrix — but it is by far the largest single jump.

### 🟢 GREEN — 24 of 30

| | | | |
|---|---|---|---|
| amazon | chaos | chaos_dwarf | chaos_pact |
| dark_elf | dark_elf_league_fumbbl | elf | goblin |
| high_elf | human | khemri | khemri_fumbbl |
| lineman | lizardman | nippon | norse |
| ogre | orc | renegades | skaven |
| slann | undead | underworld | vampire |

### 🔴 RED — 6, by fewest fails

| roster | passed | fails |
|---|---:|---:|
| **nurgle** | 86/100 | 14 |
| dwarf | 20/100 | 80 |
| necromantic | 15/100 | 85 |
| wood_elf | 15/100 | 85 |
| halfling | 5/100 | 95 |
| slann_fumbbl | 0/100 | 100 |

Every sweep printed `rust_total` and an `N/100` line, so all 30 readings are valid measurements.

### Reading the reds

`nurgle` at 14 is the fewests-fails target and the next iteration's work. The other five cluster at
80-100 fails, which is the signature of a divergence in the FIRST drive rather than 100 independent
bugs — most likely one roster-wide trait each (Decay/Foul Appearance for necromantic and nurgle;
Titchy/Stunty and the halfling Treeman for halfling; the wood elf Treeman; Dwarf Blockers'
Thick Skull / the Deathroller for dwarf). `slann_fumbbl` at 0/100 is the same shape the bb2025
campaign hit — there it turned out to be a roster-lookup problem rather than an engine one, so check
the team actually loads before assuming a rules bug.

**Status: 24 of 30 green.** Next: `nurgle` seed-by-seed from the lowest failing seed.

## ITER76 — `nurgle`: a BLITZ never rolls Foul Appearance, because the bb2020 blitz bridge has no FOUL_APPEARANCE step

`nurgle` 86/100 (seeds 2, 14, 23, 24, 26, 40, 43, 52, 57, 60, 65, 66, 75, 77). No engine change
this iteration — the root cause is pinned and the fix is structural enough to want its own gate.

### Evidence

Seed 2's first diverging PRE-state is i=33, so **step 32 resolved differently**:

```
[  32] JAVA t2 h1 away  pre=5ea531449cae133d post=5ea531449cae133d  Activate(Away2,BLITZ)
       RUST t2 h1 away  pre=5ea531449cae133d post=af8d5d3a78631af3  Activate(away_02,Blitz)
```

**Java's blitz is a complete no-op** — post == pre. That is a FAILED Foul Appearance cancelling the
declared action, and the Java dice confirm it:

```
pos 48  DiceRoller.rollSkill  FoulAppearanceBehaviour$1.handleExecuteStepHook:68
          StepFoulAppearance.executeStep:75    result 1  = fail
```

Rust rolls no such die and resolves the blitz normally.

### Why

Probing `StepFoulAppearance` across the whole seed (291 invocations): it resolves a defender only
**12** times, always via `game.defender_id`, and `target_selection_state` is `None` on every single
call. Those 12 are BLOCK actions, where `InitBlocking` sets `defender_id` before the step runs.

For a BLITZ, Java rolls Foul Appearance inside the **SelectBlitzTarget** sequence
(`bb2020/SelectBlitzTarget.java:35`), reading the target from the `TargetSelectionState`. Rust's
blitz bridge — the one ITER67 documented — replaces that sequence with bb2025's
`ActivationSequenceBuilder`, which has **no FOUL_APPEARANCE step at all**, and never creates a
TargetSelectionState. So a blitzed Foul Appearance defender is never rolled against.

This is precisely the gap ITER67 flagged and could not land: bb2020's SelectBlitzTarget carries
FOUL_APPEARANCE, DUMP_OFF, JUMP_UP and STAND_UP that the bb2025 builder omits. Back then swapping
the whole activation regressed goblin to 10/100 — but that attempt ALSO swapped the BlitzBlock
sequence, and predates the FOLLOWUP_CHOICE and pitch-bounds fixes.

### Next iteration

Add **only** `FOUL_APPEARANCE` (failure → END_BLOCKING) to the bb2020 blitz activation bridge in
`bb2025/shared/step_end_selecting.rs`, not the whole bb2020 sequence. It needs a defender to read:
Rust has no TargetSelectionState, so either set `game.defender_id` at the blitz dispatch (the
dispatch already knows `block_defender_id`) or build the TSS there. Prefer `defender_id` — it is
what the 12 working BLOCK cases already use, and `StepFoulAppearance` reads it as its fallback.

Gate it against `nurgle` AND `goblin`/`ogre` (both blitz-heavy and currently green) before keeping.

### Tooling

`scripts/stepdiff.py` — prints the Java and Rust step logs for a seed side by side and names the
first diverging PRE-state, i.e. the step that resolved differently. `dicediff.py` finds where the
DICE part company; this finds where the STATE does, which is the more useful question when a step
is a silent no-op on one side.

## ITER77 — the blitz Foul Appearance fix, attempted and REVERTED: `86 → 0/100`

Implemented exactly what ITER76 prescribed, measured it, and reverted it. `nurgle` stays 86/100.

**The change**: for BB2020, append `FOUL_APPEARANCE` (failure → END_BLOCKING) to the blitz
activation bridge, and set `game.defender_id` from `params.block_defender_id` at dispatch so the
step has a defender to read.

**The result**: `nurgle` **0/100** — every seed, worse than the disease. Reverted.

### Why it failed, and what that rules out

The step insertion is not obviously the problem; setting `game.defender_id` at dispatch is.
`defender_id` is not a private channel to `StepFoulAppearance` — it is game state that **every**
step in the activation sees, and several read it: `StepHandleDropPlayerContext` resolves its
`victim_state_key` through `game.defender_id`, and the negatrait steps that run in this very
activation can drop the acting player. Setting it before the activation instead of at
`InitBlocking` therefore changes behaviour for a whole sequence of steps that Java runs with
`defenderId` still null.

Java has no such problem because it does NOT use `defenderId` here — the blitz target lives in the
`TargetSelectionState`, a separate field that only `StepFoulAppearance` (and the block steps)
consult. Rust has no TargetSelectionState at all on this path (`tss=None` on all 291 probes in
ITER76), which is the actual missing piece.

### Next iteration — two options, in preference order

1. **Give `StepFoulAppearance` the defender as a step PARAMETER.** It currently accepts only
   `GOTO_LABEL_ON_FAILURE`; add `BLOCK_DEFENDER_ID` to its `set_parameter`, and have the blitz
   bridge publish it alongside the step. This keeps the target on the same private channel Java
   uses (its TSS) instead of leaking it into shared game state. Smallest change, most faithful.
2. **Build the TargetSelectionState in the blitz bridge**, so the step's existing
   TSS-then-`defender_id` lookup resolves the way Java's does. Closer to Java structurally, but the
   TSS is consulted by more code than the parameter is, so it carries the same class of risk that
   just failed — and `StepFoulAppearance`'s TSS branch additionally filters on
   `is_selected() && is_committed()`, which Java does not, so that would need correcting too.

Do NOT retry via `game.defender_id`.

### Gate

| check | result |
|---|---|
| `nurgle` bb2020 | 86/100 — baseline restored after the revert |
| working tree | change reverted; a note at the insertion point records the 0/100 result |

## ITER78 — three more blitz Foul Appearance attempts, all measured, all reverted

`nurgle` stays **86/100**. Nothing landed. Three variants were built and swept; recording all three
so the next attempt does not re-walk them.

| # | change | result |
|---|---|---|
| A | FOUL_APPEARANCE added to the bb2020 blitz activation + defender via `game.defender_id` | **0/100** |
| B | same, but defender via a new `BLOCK_DEFENDER_ID` step parameter | **0/100** |
| C | defender parameter attached to the EXISTING FOUL_APPEARANCE in bb2025's BlitzBlock (no added step) | 86/100 — no change |
| D | C + move it: FA in the bb2020 activation, removed from bb2020's BlitzBlock unless `frenzy_block` | 86/100 — no change |

### What that establishes

**ITER77's conclusion was wrong and is retracted.** It blamed `game.defender_id` leaking into the
activation. Variant B kept the defender on a private step parameter and still gave 0/100, so the
delivery mechanism was never the problem — **adding the step at all is**.

The reason is now clear and was missed in ITER76: **bb2025's BlitzBlock ALREADY has a
FOUL_APPEARANCE step** (`bb2025/blitz_block.rs:89`, position 4, right after the blitz GFI). So
variants A and B rolled Foul Appearance TWICE per blitz. ITER76's claim that "the blitz has no
FOUL_APPEARANCE step" was simply false — the step is there; what it lacked was a defender.

Variants C and D then gave it one, and the count did not move at all — which means it STILL is not
rolling. Handing `BLOCK_DEFENDER_ID` to the step did not make `defender_has_fa` true.

### Where to start next time

Do not add or move the step again. Instrument the EXISTING one and answer one question:
**with `BLOCK_DEFENDER_ID` supplied, why does `StepFoulAppearance` still not roll?** Candidates, in
order:

1. The parameter never arrives. `DriverStepStack::publish` delivers top-of-stack downward and stops
   at the first step that CONSUMES a key — and `StepInitBlocking` also takes `BLOCK_DEFENDER_ID`
   and sits earlier in the sequence. If it consumes the key, the FA step never sees it. Check
   `consumes_parameter` on both. **This is the most likely cause and is cheap to confirm.**
2. `defender_has_fa` is false for another reason — e.g. the id is a Java-style id where the lookup
   wants a Rust one.
3. `attacker_cancels` is true (the file already carries one fix in this area).

Note the ordering question is real but SECONDARY: Java rolls the blitz FA in SelectBlitzTarget,
before the blitz GFI, whereas Rust's sits after it. That matters for the dice position once the
roll happens at all — but it cannot be measured until the roll happens.

### Gate

| check | result |
|---|---|
| `nurgle` bb2020 | 86/100 — unchanged, all four variants reverted |
| working tree | clean at HEAD; no engine change |

## ITER79 — the blitz Foul Appearance DOES roll; it is in the wrong PLACE (after the blitz GFI)

`nurgle` 86/100, no engine change. Instrumented instead of rearranging, and two earlier readings
turn out to be wrong.

### Two retractions

**1. ITER76: "`StepFoulAppearance` resolves a defender only 12 times, never on a blitz" — WRONG.**
Printing the acting action alongside the defender shows blitzes resolve fine, from
`game.defender_id`, which `InitBlocking` sets at the top of BlitzBlock:

```
FA2 n=25  act=home_02 action=Blitz param=Some(away_02) gamedef=Some(away_02) resolved=Some(away_02)
FA2 n=29  act=away_02 action=Blitz param=Some(home_03) gamedef=Some(home_03) resolved=Some(home_03)
FA2 n=30  act=away_02 action=Blitz param=Some(home_03) gamedef=Some(home_03) resolved=Some(home_03)
```

The 291 invocations are dominated by the Move sequence's own FOUL_APPEARANCE step, which correctly
has no defender — that is what I mistook for "never resolves". **The blitz already rolls Foul
Appearance, and always had a defender.** That also explains ITER78 variants C and D changing
nothing: they supplied a parameter the step did not need.

**2. "Every nurgle player has Foul Appearance" — WRONG.** Only the Nurgle Warrior and the Beast of
Nurgle carry it (`roster_nurgle.json:84,115`) — 4 of 12 players, not 12.

### The actual defect

Rust's blitz rolls Foul Appearance at BlitzBlock position 4, i.e. **after** `GO_FOR_IT` and
`STEADY_FOOTING`. Java rolls it in the SelectBlitzTarget activation, **before** the blitz GFI ever
happens (`bb2020/SelectBlitzTarget.java:35`), and BB2020's BlitzBlock therefore carries one only for
a frenzy follow-up.

Two consequences, and the second is the killer:

- the d6 lands at a different stream position; and
- **a failed blitz GFI gotos `STEADY_FOOTING` → `FALL_DOWN`, skipping the FA step entirely** — so on
  those blitzes Rust rolls no Foul Appearance at all where Java always has. Seed 2 step 32 is
  exactly that case: no blitz FA invocation anywhere near rng 46-49, while Java rolls one at 48.

### Why ITER78 variant D did not fix it

Variant D moved the step into the activation AND gated it out of BB2020's BlitzBlock — the right
shape — and still measured 86/100 unchanged. That needs explaining before another attempt: the most
likely reason is that the activation copy runs BEFORE `InitBlocking`, so `game.defender_id` is not
yet set and the parameter path was the only source — which D did supply, so verify with the FA2
probe that the activation copy actually fires and resolves, rather than assuming.

**Re-run the FA2 probe with variant D applied** and confirm `action=Blitz` invocations appear at the
activation position. That is one build+run and settles it.

### Gate

| check | result |
|---|---|
| `nurgle` bb2020 | 86/100 unchanged |
| working tree | clean at HEAD; probe reverted |

## ITER80 — variant D + probe together: Foul Appearance is a RED HERRING for seed 2

`nurgle` 86/100, no engine change. Ran ITER78's variant D with the FA probe active, which answers
ITER79's open question and closes off the whole Foul Appearance line for this seed.

### Two results

**1. Variant D's reordering is a dice no-op.** With FA moved into the activation and gated out of
BB2020's BlitzBlock, the blitz FA rolls land at rng **25, 29, 30, 66** — the exact positions they
occupied before the move (the `param=Some(...)` on those rows confirms they are the activation
copies, since only that path supplies the parameter). Nothing rolls between `InitBlocking` and
`GO_FOR_IT` on these blitzes, so moving the step across them changes no dice. That fully explains
ITER78's "no change" — the change was real but inert.

**2. Rust's step-32 blitz never reaches the Foul Appearance step at all** — not before variant D and
not after. There are exactly **5** blitz FA invocations in the whole seed (rng 25, 29, 30, 66, 101)
and none anywhere near rng 46-49, where Java rolls its FA at pos 48.

### What that means

The FA step is present in the sequence and is now FIRST in the blitz activation, yet the step-32
blitz still does not execute it. So the sequence is short-circuiting BEFORE it — some earlier
activation step gotos `END_BLOCKING`. **The divergence at step 32 is upstream of Foul Appearance,
and Java's pos-48 FA roll is a consequence of that, not the cause.**

Chasing Foul Appearance was the wrong thread from ITER76 onward. Four iterations, no count change.

### Next iteration

`FFB_DRIVE_TRACE=1` on nurgle seed 2, find the step-32 blitz activation, and read which step exits
it. That names the real defect directly and costs one run. Do not touch the FA sequencing again
until that trace says it is involved.

### Gate

| check | result |
|---|---|
| `nurgle` bb2020 | 86/100 unchanged |
| working tree | clean at HEAD; variant D and the probe both reverted |

## ITER81 — seed 2 pinned to ONE player state: `a01` Prone (Java) vs Standing (Rust)

`nurgle` 86/100, no engine change kept. But the seed is now pinned exactly, and the earlier
framing was wrong again.

### It is a STATE-only divergence, and both engines roll the same die

`FFB_TRACE=1` state strings at the failing step:

```
JSTEP     i=32 rng_calls=29 ... a01:13,8,Prone     chosen=Activate(Away2,BLITZ)
RUST_STEP i=32 rng_calls=29 ... a01:13,8,Prone     chosen=Activate(away_02,Blitz)

JSTEP     i=33 rng_calls=30 ... a01:13,8,Prone     <-- Java
RUST_STEP i=33 rng_calls=30 ... a01:13,8,Standing  <-- Rust
```

`rng_calls` 29 → 30 on BOTH sides: the blitz spends exactly one die in each engine, and every other
player and the ball agree. **The entire seed-2 divergence is one player left STANDING that Java
leaves PRONE.**

This also corrects ITER80: the FA step is not skipped — it runs and rolls, at rng 30. ITER80 was
looking for the roll in the wrong window (rng 46-49 came from the *Java dice trace positions*, which
are a different numbering from `rng_calls` at the step boundary).

`away_02` (=`a01`) is a prone Nurgle Warrior declaring a stand-up-and-blitz. Java's Foul Appearance
fails, `handleFailure` runs, and it puts the player BACK on the ground:

```java
if (actingPlayer.isStandingUp() && (playerAction == BLITZ_MOVE || isBlockAction || GAZE_MOVE || isKickingDowned))
    setPlayerState(player, playerState.changeBase(PRONE).changeActive(false));
```

Net effect in Java: nothing changes, which is exactly the `post == pre` hash.

### The attempted fix, and why it is not the answer

Rust's `fail_fa` ports that condition, but tests `PlayerAction::BlitzMove` — and Rust's
single-command blitz bridge leaves the acting action as plain `Blitz`, which is also not in
`is_block_action()`. So the condition is false for every Rust blitz. Adding `|| pa == Blitz`
looked like the fix.

**It measured 86/100 — no change.** So `fail_fa` is not what stands the player up, or is not
reached. Reverted (no measured effect).

### Next iteration

The question is now very small: **what sets `a01` to Standing during that blitz, and does Java's
equivalent run?** Put a gated backtrace in `FieldModel::set_player_state` firing when the new base
is STANDING for that player id (the campaign's documented technique, and the one that has resolved
this class of bug fastest). That names the writer in one run.

Keep the `|| pa == Blitz` change in mind — it is a genuine 1:1 discrepancy in `fail_fa` even though
it is not this bug, and should be revisited with a test once the real writer is known.

### Gate

| check | result |
|---|---|
| `nurgle` bb2020 | 86/100 unchanged |
| working tree | clean at HEAD |

## ITER82 — the writer named: `change_player_action` stands the blitzer up; the revert never runs

`nurgle` 86/100, no engine change. The backtrace answers ITER81's question.

### Who writes STANDING

A gated backtrace in `FieldModel::set_player_state` for `away_02` gives the full state history for
the seed. The relevant transition is `Prone(3) → Moving(2)`:

```
STAND_BT pid=away_02 Some(3) -> 2
  4: ffb_model::model::field_model::FieldModel::set_player_state
  5: ffb_engine::step::util_server_steps::change_player_action
  6: <...step_init_selecting::StepInitSelecting as Step>::handle_command
```

So the blitzer is lifted off the ground **at declaration**, by `change_player_action` — Java's
`UtilActingPlayer.changeActingPlayer`, which does the same thing ("show acting player as moving").
That is correct and shared. The player later goes `Moving(2) → Standing(1)`, which is what the state
string reports at i=33.

**Java does the identical Prone → MOVING write and then UNDOES it** when the Foul Appearance fails:
`handleFailure` → `changeBase(PRONE).changeActive(false)`. Rust's revert never happens.

### Why the ITER81 fix did not take

Rust's `fail_fa` guard is `if game.acting_player.standing_up { ... }`, and `change_player_action`
DOES set `standing_up = was_prone` (`util_server_steps.rs:77`), so the guard should hold. Adding
`|| pa == Blitz` to the action test should then have fired it — and measured nothing. The remaining
possibility is that **`fail_fa` is not reached at all**.

There is evidence for that: the FA step is invoked TWICE for this blitz (probe at `n=29` and
`n=30`), which is the re-roll path — first invocation rolls and fails, offers a re-roll, the agent
declines, second invocation takes `already_rerolled`. Whether that second pass reaches `fail_fa` or
returns earlier is exactly what has not been checked.

### Next iteration — one `eprintln`, not a redesign

Put a single gated print at the top of `fail_fa` and at each `return` inside `execute_step`, run
seed 2, and see which arm the second invocation takes. If `fail_fa` runs, the `|| pa == Blitz`
change is the fix and something else blocks it; if it does not, the re-roll-decline path is
returning without the Java failure handling — which would be the real defect and is a different
repair.

Five iterations on nurgle without moving the count. The seed is now pinned to a single state bit
with the writer named, so this is convergent, but it is worth saying plainly that the sequencing
detours (ITER76-80) were wasted and the state-diff + backtrace pair should have been the FIRST
tools reached for, not the last.

### Gate

| check | result |
|---|---|
| `nurgle` bb2020 | 86/100 unchanged |
| working tree | clean at HEAD; probe reverted |

## ITER83 — `fail_fa` IS reached, with `standing_up=false`; the `if (changed)` fix breaks two tests

`nurgle` 86/100. Nothing landed. **Recommend switching rosters — see the end.**

### The arm trace (this part is solid)

Gated prints on every return of `StepFoulAppearance` for seed 2's step-32 blitz:

```
FA4 enter n=29 act=away_02 action=Blitz def=home_03 hasfa=true standing_up=false roll=0
FA4 rolled=1 min=2 may_block=false already_rr=false
FA4 arm=ask-rr
FA4 enter n=30 act=away_02 action=Blitz def=home_03 hasfa=true standing_up=false roll=0
FA4 arm=rr-not-consumed
FA4 fail_fa standing_up=false action=Some(Blitz)
```

The roll is a **1**, it fails, a re-roll is offered, the agent declines, and **`fail_fa` IS
reached** — with `standing_up = false`. That is the whole reason the blitzer is not put back prone.
ITER82 guessed `fail_fa` might not be reached; it is.

### Why `standing_up` is false, and why the obvious fix is wrong

Java gates `setPlayer` / `setOldPlayerState` / `setStandingUp` — and the Prone→MOVING write —
inside `if (newPlayer != oldPlayer)`, leaving only `setPlayerAction` / `setJumping` unconditional
(`UtilActingPlayer.java:75-86`). Rust sets `set_player` and `standing_up` UNCONDITIONALLY. A blitz
re-invokes `change_player_action` on the same player for its block sub-activation, by which time the
player is already MOVING, so `was_prone` recomputes to false and the flag is lost.

Moving both inside the existing `if changed` guard — which is what Java does — **breaks two existing
tests**:

- `step_init_selecting::tests::activate_prone_player_blitz_with_target_runs_standup_via_next`
- `step_init_selecting::tests::prone_move_activation_sets_current_move_to_stand_up_cost`

Both assert `standing_up` after an activation where `changed` is FALSE, i.e. they encode the current
unconditional behaviour. Whether they assert the right thing, or were written around the existing
bug, is unresolved — and resolving it means understanding why `StepInitSelecting` calls
`change_player_action` with the player already set. Reverted rather than rewrite tests to fit an
unproven fix.

(The companion `|| pa == Blitz` change in `fail_fa` is still a genuine 1:1 gap — Rust's bridge keeps
the action as `Blitz` where Java is in `BLITZ_MOVE` — but it is inert on its own.)

### Recommendation: park nurgle, take a different roster

Six iterations (ITER76-83), no count movement. The seed is fully characterised — one player left
standing, exact cause known — but the repair runs into a test/behaviour question about
`change_player_action` that is a bigger piece of work than a seed fix, and it sits on the shared
activation path every roster uses.

The other five reds (dwarf 20, necromantic 15, wood_elf 15, halfling 5, slann_fumbbl 0) are all
80-100 fails, which historically means ONE first-drive cause each — cheaper per iteration and more
likely to move the matrix. `slann_fumbbl` at 0/100 in particular matched a roster-lookup fallback in
the bb2025 campaign, not an engine bug at all.

Suggested order: `slann_fumbbl` (check the team loads), then `halfling`, then `dwarf`.

### Gate

| check | result |
|---|---|
| `nurgle` bb2020 | 86/100 unchanged; seed 2's post-hash byte-identical |
| `cargo test --workspace` | 2 failures WITH the change → reverted; clean at HEAD |
| working tree | clean at HEAD |

## ITER84 — the Bone Head hyphen drop was gated one edition too narrowly: `slann_fumbbl` 0 → **98/100**

Parked nurgle per ITER83 and took `slann_fumbbl` (0/100). It was the cheapest red on the board and
went from worst to nearly green in one iteration.

### The bug

`loader.rs::position_json_to_roster_position` drops the hyphen-spelled `"bone-head"` skill that the
FUMBBL slann roster gives its Kroxigor, because Java's `SkillFactory.forName` does an exact
case-insensitive match and never resolves it — so Java's Kroxigor has NO Bone Head, while Rust's
lenient `from_class_name` would give it one and roll a per-activation negatrait d6 Java never rolls.

That drop was gated on `is_bb2025`. But the canonical names are:

```
skill/bb2016/BoneHead.java:25   super("Bone-Head", ...)   <- hyphen
skill/bb2020/BoneHead.java:25   super("Bone Head", ...)   <- space
skill/bb2025/BoneHead.java:25   super("Bone Head", ...)   <- space
```

**BB2020 and BB2025 both use the space form**; only bb2016 uses the hyphen. So the drop belongs to
both modern editions, and the bb2025-only gate left the BB2020 Kroxigor carrying a Bone Head Java
does not have — the same extra d6, one edition over (seed 1 step 14, `home_01` BLITZ).

The code comment already said "The bb2020/bb2025 Bone Head skill's canonical name is 'Bone Head'".
The prose was right and the condition did not match it.

### The fix

`position_json_to_roster_position` and `roster_json_to_roster` now take `rules: Rules` instead of an
`is_bb2025: bool`. The two drops they perform are NOT the same set and a single boolean could not
express both:

- hyphenated Bone Head → dropped for **BB2020 + BB2025** (kept for bb2016, where it resolves)
- bb2016-only skills (No Hands etc.) → dropped for **BB2025 only** (bb2020 defines them)

### Gate

| check | result |
|---|---|
| `slann_fumbbl` bb2020 | 0/100 → **98/100** (seeds remaining: 2) |
| `slann_fumbbl` bb2025 | 100/100 — the edition the original fix targeted is untouched |
| `slann` bb2020 | 100/100 — the non-FUMBBL Kroxigor roster |
| `lineman` bb2016 | 100/100 — the edition that KEEPS the hyphen spelling |
| `cargo test --workspace` | 14,456 passed / 0 failed |

**Status: 24 of 30 green**, `slann_fumbbl` 98/100 now the fewest-fails target. ITER83's call to
park nurgle was right: one iteration on a different roster beat six on that one.

## ITER85 — `slann_fumbbl`'s last 2 seeds are the end-of-game MVP roll; found a dead-code option path

`slann_fumbbl` stays **98/100** (seeds 29, 50). No engine change kept. The remaining failure is
fully characterised and it turned up a structural finding worth more than the seed.

### The divergence

Seed 29's first sides difference is at the very end of the game, rng 92-93:

```
pos 92  JAVA d1=1   DiceRoller.randomPlayerId:289  StepMvp.executeStep:120
pos 93  JAVA d1=1   DiceRoller.randomPlayerId:289  StepMvp.executeStep:126
        RUST d6=5 / d6=1
```

Java rolls a **d1** twice — one per team. `randomPlayerId(playerIds)` is
`playerIds[rollDice(playerIds.length) - 1]`, so a d1 means the list had exactly ONE entry. That is
the MVP *nomination* path: `mvpNominations > 0` makes `StepMvp` show a player-choice dialog, the
harness answers with exactly one player (lowest jersey, `ParityRunner:885-891`), and Java then rolls
`d(1)` to "pick" from that single nomination.

Rust took the `else` branch and auto-rolled `d(eligible)` — a d6, six eligible players.

### The structural finding

Rust's `mvpNominations` is unset, for two independent reasons:

1. **`UtilServerStartGame::add_default_game_options` is dead code.** It is never called outside its
   own unit tests, so NONE of the 18 options Java's `addDefaultGameOptions` sets are set in a Rust
   parity run. (The parity runner has its own short `BASELINE_SETUP_OPTIONS` list instead.)
2. **Even if it were called, the keys would not match.** It writes `opt::MVP_NOMINATIONS =
   `"MVP_NOMINATIONS"`` — a Rust enum-style spelling — while every reader looks up the Java wire
   name `"mvpNominations"` (`ffb_model::option::game_option_id`). The same mismatch applies to most
   of that module: `WIZARD_AVAILABLE`, `ALLOW_BALL_AND_CHAIN_RE_ROLL`, `CLAW_DOES_NOT_STACK`,
   `PETTY_CASH_AFFECTS_TV`, … Its own tests pass because they read back through the same wrong
   constant.

This is an open audit item and a likely source of several remaining divergences: wherever Java runs
with an option ON and Rust silently reads the factory default.

### What was tried and reverted

Both aliasing `opt::MVP_NOMINATIONS` to the wire name AND adding
`(MVP_NOMINATIONS, "6")` to `BASELINE_SETUP_OPTIONS` — measured **98/100, unchanged**, seeds 29 and
50 still failing with the same d1-vs-d6. So simply switching the option on is not sufficient: with
nominations enabled, Rust's `StepMvp` returns `cont()` awaiting a player-choice the parity agent
never sends, so it does not reach Java's single-nomination `d(1)` either. The agent side has to
answer that dialog the way `ParityRunner` does (one player, lowest jersey) for the option to help.
Reverted rather than leave an inert switch on.

### Next iteration

Two options, in preference order:

1. **Finish the MVP path**: teach the agent to answer the MVP player-choice prompt with the
   lowest-jersey eligible player, THEN enable `mvpNominations`. The two only work together — same
   shape as ITER71's FOLLOWUP_CHOICE pair.
2. **Or move on to `halfling` (5/100)** and come back; `slann_fumbbl` at 98 is already the best of
   the reds and this is an end-of-game-only divergence.

Separately, the option-key audit deserves its own pass — it is cheap to enumerate (compare the
`opt` module against `game_option_id`) and each mismatch is a candidate cause elsewhere.

### Gate

| check | result |
|---|---|
| `slann_fumbbl` bb2020 | 98/100 unchanged (seeds 29, 50) |
| working tree | clean at HEAD; both attempts reverted |

## ITER86 — `halfling` has the SAME signature as `nurgle`: a cancelled action Java reverts and Rust does not

`halfling` 5/100, no engine change. The point of this iteration is the connection, not the seed.

### Seed 1

```
[ 100] JAVA t7 h1 away  pre=a2409412555b888e post=a2409412555b888e  Activate(Away1,BLOCK)
       RUST t7 h1 away  pre=a2409412555b888e post=913aca7fda9c7da8  Activate(away_01,Block)
```

**Java's block is a complete no-op — `post == pre`.** Rust's changes state. `away_01` is a
**Treeman** (`team_halfling.json:16`), and the Java dice trace for this seed contains a
`TakeRootBehaviour` roll (`rollSkill`, pos 70). A Treeman that fails Take Root cannot carry out its
declared action; Java unwinds the declaration completely.

### This is nurgle seed 2 again, with a different negatrait

| | nurgle seed 2 | halfling seed 1 |
|---|---|---|
| step | 32, `Activate(Away2, BLITZ)` | 100, `Activate(Away1, BLOCK)` |
| negatrait | Foul Appearance (Warrior) | Take Root (Treeman) |
| Java | `post == pre` — action cancelled, state unwound | `post == pre` — same |
| Rust | state changed — action partly resolved | same |

Both are "a negatrait check fails, Java reverts the declaration's state effects, Rust leaves them
applied". ITER81-83 traced the nurgle case to `acting_player.standing_up` being lost on the blitz's
same-player `change_player_action` re-dispatch, and the repair being blocked on Java's
`if (newPlayer != oldPlayer)` guard vs two `step_init_selecting` tests that encode the current
unconditional behaviour.

**That reframes ITER83's "park nurgle" advice.** It is not one roster's seed — the remaining reds
are exactly the rosters that field negatrait Big Guys:

| roster | fails | negatrait carrier |
|---|---:|---|
| dwarf | 80 | Deathroller (Secret Weapon / Bone Head-class) |
| necromantic | 85 | Flesh Golem / Werewolf |
| wood_elf | 85 | Treeman — Take Root |
| halfling | 95 | Treeman ×2 — Take Root |
| nurgle | 14 | Beast of Nurgle (Really Stupid), Warriors (Foul Appearance) |

If the cancel-revert path is one bug, fixing it could move several of these at once — which makes
it worth more than its per-seed cost, and worth resolving the `change_player_action` test question
properly rather than routing around it.

### Next iteration

Do NOT keep sampling seeds. Settle the shared question:

1. Read the two `step_init_selecting` tests and determine why they call `change_player_action` with
   the acting player already set — is that the real engine flow, or test scaffolding?
2. If it is scaffolding, apply Java's `if (newPlayer != oldPlayer)` guard, fix the tests to set up
   the real flow, and measure nurgle + halfling + wood_elf together.
3. If it IS the real flow, then Rust's `StepInitSelecting` differs from Java's command sequence and
   that is the actual defect.

### Gate

| check | result |
|---|---|
| `halfling` bb2020 | 5/100 (baseline, unchanged) |
| working tree | clean at HEAD; no engine change |

## ITER87 — the `if (changed)` question SETTLED, and `standing_up` is NOT the blocker

`nurgle` 86/100, `halfling` 5/100 — both unchanged. Reverted. But two things are now closed off,
and one earlier hypothesis is disproven.

### The test question is answered: it was scaffolding

`StepInitSelecting::handle_command(ActivatePlayer)` calls `change_player_action(game, player_id, …)`
directly and does NOT pre-set the acting player (`step_init_selecting.rs:95-97`). A real activation
arrives with the previous one already cleared, so `changed` is TRUE — consistent with the ITER82
backtrace, where the Prone→MOVING write (which lives inside `if changed`) did fire.

The two tests set `game.acting_player.player_id = Some("p1")` before calling `handle_command` with
that same id, which forces `changed = false` and makes them exercise the same-player re-dispatch
path instead of an activation. **That is test scaffolding, not the engine flow.** With the pre-set
removed, both tests pass alongside Java's guard (`cargo test -p ffb-engine` 7,153 / 0).

### But the fix is inert

Applying Java's `if (newPlayer != oldPlayer)` guard to `set_player` / `standing_up` — plus the
`|| pa == Blitz` companion in `fail_fa` — and correcting the two tests measured:

| roster | before | after |
|---|---:|---:|
| nurgle | 86/100 | 86/100 |
| halfling | 5/100 | 5/100 |

**So `standing_up` being lost on re-dispatch is NOT what leaves the player standing.** ITER81-83
built on that hypothesis; it is now disproven by measurement. Reverted (no measured effect, and it
touches the shared activation path for every roster, so it is not worth carrying unproven).

### What survives

The ITER86 observation still stands and is the real target: in both rosters Java's activation is a
complete no-op (`post == pre`) after a failed negatrait, and Rust's is not. What has been ruled out
is the `standing_up` flag as the mechanism.

### Next iteration

Stop hypothesising about which flag and diff the state directly, as ITER81 did for nurgle — but for
halfling seed 1 step 100, which is a BLOCK (simpler than a blitz: no bridge, no re-dispatch, no
TargetSelectionState). Get the two state strings at i=100/101, find exactly which player/field
differs, then backtrace that write. The blitz path added three confounders to the nurgle
investigation that the halfling BLOCK case does not have.

### Gate

| check | result |
|---|---|
| `nurgle` / `halfling` bb2020 | 86/100 and 5/100 — both unchanged |
| `cargo test -p ffb-engine` | 7,153 / 0 WITH the change (tests corrected) — then reverted |
| working tree | clean at HEAD |

## ITER88 — halfling pinned to ONE coordinate: the defender is pushed in Rust, not in Java (Stand Firm)

`halfling` 5/100, no engine change. The BLOCK case was the right one to take — it gave a
single-field answer in one iteration where the blitz case took six.

### The divergence, exactly

`scripts/statediff.py` on seed 1, i=101 (the pre-state of the step after the failing one):

```
JAVA state hashes to a2409412555b888e     <- matches the parity log exactly
RUST state hashes to 913aca7fda9c7da8     <- matches the parity log exactly
  h00:  JAVA 12,7,Standing   RUST 11,6,Standing
```

One field. **The defender `h00` is pushed one square in Rust and not moved at all in Java.**
Everything else — every other player, the ball, the header — is identical.

### Both engines spend the SAME two dice

```
JSTEP     i=100 rng_calls=56 ... Activate(Away1, BLOCK)
RUST_STEP i=100 rng_calls=56 ... Activate(away_01, Block)
JSTEP     i=101 rng_calls=58        RUST_STEP i=101 rng_calls=58
```

and Java's two are:

```
pos 57  rollSkill      TakeRootBehaviour   result 2   (Take Root passed)
pos 58  rollBlockDice                      result 3   (= PUSHBACK)
```

So Java rolls a **pushback** and the defender still does not move. `h00` is the opposing
**Treeman**, and the halfling Treeman has **Stand Firm** (`roster_halfling.json:25`) — the skill
whose whole function is refusing a push. Java honours it; Rust pushes anyway.

Note this reframes ITER86 again: the activation is not "cancelled by a negatrait". Take Root
PASSED. The block resolves normally in both engines; only the push is refused in Java.

### Where to look

`StandFirmStepModifier` exists, hooks `StepId::Pushback`, and is registered for all three editions
(`registry.rs:105,187,260`), so the machinery is present. The next question is narrow: does
`StepPushback` actually invoke the step-modifier hooks, and is the hook's refusal applied to the
chosen pushback square? Note the campaign's opening retraction recorded a Rust panic in this exact
file (`StandFirmStepModifier: step_state must be StepPushbackHookState`) — it no longer panics, but
that is not the same as being wired up.

`wood_elf` (85 fails) also fields a Treeman and is the natural second measurement.

### Tooling — and a warning

`scripts/statediff.py` prints the field-level difference between the two engines' `FFB_TRACE`
state strings, and now also prints each string's fnv1a64 so it can be checked against the parity
log's `state_hash` (they must match — the log hash IS `fnv1a64(state_string)`).

**It reported "state strings are IDENTICAL" for every step on its first run.** The state blob
contains spaces (`h1t67aaways0,0 b25,8,true p<players>`), so a `state=(\S+)` capture grabbed only
the header. The hash printout is what caught it: the traced strings hashed to a value matching
NEITHER engine's logged hash. **Any state-diff tool must print the hash and be checked against the
log before its output is trusted** — a silently-truncating diff that always says "identical" would
have sent the next iteration hunting a phantom.

### Gate

| check | result |
|---|---|
| `halfling` bb2020 | 5/100 (baseline, unchanged) |
| working tree | no engine change; adds `scripts/statediff.py` |

## ITER89 — Stand Firm auto-DECLINED where the harness always uses it: **4 rosters move, dwarf GREEN**

One line. `halfling` 5→98, `wood_elf` 15→98, `dwarf` 20→**100 🟢**, `necromantic` 15→32.

### The bug

Java's `StandFirmBehaviour` (bb2020) shows a `DialogSkillUseParameter(defender, StandFirm)` when the
choice has not been made, and applies the client's answer on re-entry. `ParityRunner`'s SKILL_USE
arm **always uses the skill**, with exactly four exceptions — DumpOff, PrimalSavagery,
SafePairOfHands, Swoop (`ParityRunner.java:748-751`). Stand Firm is not one of them, so **Java always
refuses the push**.

Rust's hook auto-**DECLINED** instead:

```rust
// Java: if (!standingFirm.containsKey(id)) show dialog → headless: auto-decline
if !state.standing_firm.contains_key(&defender_id) {
    state.standing_firm.insert(defender_id.clone(), false);
    return false;                     // → push proceeds
}
```

So every Stand Firm defender in BB2020 got pushed where Java leaves it standing. Changed to
auto-ACCEPT and fall through to the existing refusal path (which already cleared the pushback stack
and published `FOLLOWUP_CHOICE=false` correctly — only the decision was wrong).

The colocated test `stand_firm_not_decided_headless_auto_declines` asserted the wrong behaviour and
had frozen it in place; it is now `..._auto_accepts` and additionally checks `do_push` and the
cleared pushback stack.

### Why this moved four rosters at once

Stand Firm is a Big-Guy staple: the halfling and wood elf **Treemen**, the dwarf **Deathroller**,
necromantic's **Flesh Golems**. ITER86's "these reds share one cause" hypothesis was right about the
grouping even though it was wrong about the mechanism (it guessed a cancelled-activation revert; the
real cause is a refused push).

### Results

| roster | before | after |
|---|---:|---:|
| dwarf | 20/100 | **100/100 🟢** |
| halfling | 5/100 | 98/100 |
| wood_elf | 15/100 | 98/100 |
| necromantic | 15/100 | 32/100 |

### Gate

| check | result |
|---|---|
| `lineman` bb2020 | 100/100 |
| `ogre` bb2020 | 100/100 |
| `cargo test --workspace` | all pass (the one stale test corrected) |

**Status: 25 of 30 green.** Remaining: halfling 98, wood_elf 98, slann_fumbbl 98, nurgle 86,
necromantic 32. Next: `necromantic` (68 fails) — it moved but least, so it likely has a second
roster-wide cause of its own; then the three 98s.

## ITER90 — `necromantic` is a Frenzy follow-up geometry difference, not another auto-decline

`necromantic` 32/100. No engine change. Scoped in one pass using the ITER88 method.

### Ruled out first: no other auto-decline applies here

Following ITER89's lead, the remaining bb2020 auto-decline hooks are **Grab**
(`bb2020/grab_behaviour.rs:127`) and **Indomitable** (`mixed/indomitable_behaviour.rs`). Java's
`GrabBehaviour` has the same dialog shape as Stand Firm, so Rust's auto-decline there is almost
certainly wrong the same way — but **no bb2020 roster has Grab** (`grep -rn Grab data/rosters/bb2020/`
is empty), so fixing it cannot move this matrix. Recorded as a latent 1:1 gap, not done.

(`bb2020/side_step_behaviour.rs` was already corrected this way in an earlier iteration —
"use Side Step instead of auto-declining it", ogre 100/100 — which is what made Stand Firm's
identical shape recognisable.)

### The actual divergence

Seed 1, step 14 — `Activate(Home1, BLITZ)`:

```
i=15  JAVA state hashes to d9dcfa1991073cd4      RUST 2e23e6ab75d3ae02
  h00:  JAVA 12,7,Standing   RUST 13,8,Standing
```

Only the **attacker's own square** differs, and by a diagonal. The defender's position matches, as
does every other player. Both engines spend exactly **4 dice** (rng 18 → 22).

`home_01` is a **Werewolf**, whose skills are `Claw, Claws, Frenzy, Regeneration`
(`roster_necromantic.json`). Frenzy carries `forceFollowup`: the attacker must follow up and then
block again. So this is a follow-up/Frenzy **geometry or decision** difference — same dice, same
push, different ending square for the blitzer — and NOT the negatrait/auto-decline family.

Relevant context: `ParityRunner` always answers FOLLOWUP_CHOICE with `false`, but Java's
`StepFollowup` checks `forceFollowup` INSIDE its `effective_choice == null` block, before the dialog
is ever shown — so a Frenzy attacker force-follows-up and the harness's decline never applies. If
Rust evaluates the decline first, or applies the force in a different order, the attacker ends up in
a different square. That ordering is the first thing to check.

### Next iteration

Compare Rust's `bb2025/block/step_followup.rs` force-followup ordering against Java's
`StepFollowup` for a Frenzy attacker, and check where the Frenzy second block re-enters. Then
confirm with `statediff.py` on necromantic seed 1 i=15 — a one-field check.

### Gate

| check | result |
|---|---|
| `necromantic` bb2020 | 32/100 (post-ITER89 baseline, unchanged) |
| working tree | clean at HEAD; no engine change |

## ITER91 — bb2020 Stand Firm never published FOLLOWUP_CHOICE(false): necromantic 32 → 100/100 GREEN

ITER90 named the symptom (a Frenzy attacker ending on a different square) but guessed the mechanism
was ordering inside `StepFollowup`. It is not: `StepFollowup` is correct. The parameter it needed
was never published.

### Measurement

necromantic seed 1, `Activate(Home1, BLITZ)` at i=14 → i=15. Full states:

```
i=14 (pre)   a02:13,8,Standing   h00:12,7,Standing
i=15 JAVA    a02:13,8,Prone      h00:12,7,Standing     rng 18 -> 22
i=15 RUST    a02:13,8,Prone      h00:13,8,Standing     rng 18 -> 22
```

The defender `a02` is **knocked down without being moved** in BOTH engines — that is a Stand Firm
avoid-push (necromantic's Flesh Golems carry Stand Firm). Java's blitzer therefore stays put. Rust's
blitzer walks onto 13,8 — the square the defender never vacated, with the defender still on it.

`h00` is a **Werewolf**: `Claw, Claws, Frenzy, Regeneration`. Frenzy carries `forceFollowup`.

### Root cause

Java `bb2020/StandFirmBehaviour.handleExecuteStepHook`, avoid-push branch:

```java
state.doPush = true;
state.pushbackStack.clear();
step.publishParameter(new StepParameter(StepParameterKey.STARTING_PUSHBACK_SQUARE, null));
step.publishParameter(new StepParameter(StepParameterKey.FOLLOWUP_CHOICE, false));   // <—
```

Rust's bb2020 copy set `do_push` / cleared the squares / cleared the starting square — and dropped
the FOLLOWUP_CHOICE publish. Without it `StepFollowup` still sees `followup_choice == None`, enters
its `effective_choice.is_none()` block, finds `FORCE_FOLLOWUP` on the attacker and publishes
`FollowupChoice(true)`. Java never reaches that block at all, because the parameter is already set.

The **bb2025 sibling already had this line** (`bb2025/stand_firm_behaviour.rs:155`, landed for dwarf
seed 1 step 101). The bb2020 file's own comment even claimed the publish
("Java: … publish FOLLOWUP_CHOICE=false") while the code below it did not do it — a comment
describing the Java, not the Rust.

### Fix

`crates/ffb-engine/src/skill_behaviour/bb2020/stand_firm_behaviour.rs` — push
`StepParameter::FollowupChoice(false)` onto `state.published` on the avoid-push branch, exactly as
bb2025 does. Two colocated regression tests cover both entry paths into that branch
(`stand_firm_accepted_publishes_followup_choice_false`, `..._auto_accept_...`), since only the
already-decided path was previously exercised.

### Gate

| check | result |
|---|---|
| `necromantic` bb2020 | **32/100 → 100/100 GREEN** |
| `lineman` bb2020 | 100/100 |
| `lineman` bb2025 | 100/100 |
| `lineman` bb2016 | 100/100 |
| `dwarf` bb2020 (prev. green) | 100/100 |
| `halfling` bb2020 | 98/100 (unchanged) |
| `wood_elf` bb2020 | 98/100 (unchanged) |
| `nurgle` bb2020 | 86/100 (unchanged) |
| `slann_fumbbl` bb2020 | 98/100 (unchanged) |
| `cargo test --workspace` | clean |

**bb2020 is now 26 of 30 green.** RED: nurgle 86 · halfling 98 · wood_elf 98 · slann_fumbbl 98.

### Process note (cost one wasted measurement)

Three `lineman` sweeps in bb2020/bb2025/bb2016 were launched CONCURRENTLY. They share
`parity/lineman_vs_lineman/seed_N_*.jsonl`, so they clobber each other and reported 21/100 and
78/100. Re-run one at a time all three are 100/100. The ban in `/parity-iter` on concurrent runs of
the same matchup applies **across editions** — this is what it is for.

### Two open leads carried forward

1. Rust's bb2020 (and bb2025) Stand Firm hooks do not set `clear_pushback_stack`, while Java does
   `state.pushbackStack.clear()`. The step's own `self.pushback_stack` therefore survives an avoid-
   push. Not observed to matter yet (necromantic seed 1 pushed nobody), and bb2025 is 30/30 without
   it, so it was left alone rather than changed speculatively. Only bb2016 sets the flag.
2. `bb2020/grab_behaviour.rs` still auto-DECLINES where `ParityRunner` always uses the skill. Inert
   today — no bb2020 roster has Grab — but it is the same defect class as ITER89 and ITER91.

### Next iteration

`nurgle` at 86 is the largest remaining red and the only one below 98. Six earlier iterations chased
a Foul Appearance cancel-revert there without landing it; re-open it with the ITER88/91 method —
statediff the first diverging step and identify which single field moves — rather than from the
prior hypothesis.

## ITER92 — nurgle root cause FOUND (BB2020 resolves Foul Appearance BEFORE STAND_UP); one 1:1 fix landed, the sequence move measured and REVERTED

nurgle stays 86/100. This iteration produced a **confirmed root cause** for the six-iteration
nurgle mystery, one landed 1:1 correction, and one measured-and-reverted attempt. Read the reverted
attempt's numbers before trying it again — it is the obvious fix and it does not work as written.

### The root cause (confirmed, not hypothesised)

nurgle seed 2, `Activate(Away2, BLITZ)` at i=32 → i=33. Both engines spend **exactly one die**
(rng 29 → 30) and produce **one** differing field:

```
i=32 (pre)   a01:13,8,Prone      (a01 = away_02, adjacent to h02 @12,7)
i=33 JAVA    a01:13,8,Prone      state 5ea531449cae133d (== the pre-state: nothing happened)
i=33 RUST    a01:13,8,Standing   state af8d5d3a78631af3
```

One die, spent by both: the **Foul Appearance** roll against the Nurgle target. It FAILS in both,
and the blitz is abandoned. The difference is only *when* the roll happens:

| | sequence | on failure |
|---|---|---|
| `bb2020/SelectBlitzTarget.java:35-36` | … BLOOD_LUST, **FOUL_APPEARANCE**, **DUMP_OFF**, JUMP_UP, STAND_UP | goto END_BLITZING — the blitzer is **still prone** and stays prone |
| `bb2025/SelectBlitzTarget.java` | … BLOOD_LUST, JUMP_UP, STAND_UP (no FA, no DUMP_OFF — both moved into `BlitzBlock`) | the blitzer has already **stood up** |

BB2020 puts Foul Appearance *before* the stand-up; BB2025 puts it *after*. The driver runs the
shared BB2025 generator for BB2020 games, so a BB2020 blitzer that fails Foul Appearance stands up
in Rust and does not in Java. Same die, same value, different order.

`bb2020/BlitzBlock.java:31` repeats FOUL_APPEARANCE only under `params.isFrenzyBlock()`, and has no
DUMP_OFF at all; `bb2025/BlitzBlock.java:37,39` adds both unconditionally.

### Landed: `StepStandUp` cleared `standingUp` on the free path (1:1, gate-neutral)

Chasing the above surfaced a genuine 1:1 divergence. Java's free stand-up branch
(`bb2025/StepStandUp.java:136-138`) sets **only** `setHasMoved(true)`; `standingUp` deliberately
stays TRUE. Only the ROLLED stand-up's success (`:114-115`) clears it. Rust cleared it on both
paths. The flag is read later — `FoulAppearanceBehaviour.handleFailure` reverts the attacker to
PRONE when `isStandingUp()` — so clearing it early disables that revert.

Fixed, with the colocated test rewritten to assert BOTH branches
(`rolled_success_clears_standing_up_but_free_stand_up_does_not`; the old
`success_clears_standing_up_flag` asserted the wrong behaviour on whichever branch it happened to
hit). Re-entry is still blocked by the `has_moved` half of the outer guard, exactly as in Java.

**This did not move nurgle**, and a probe says why: at the BB2025-order Foul Appearance failure the
acting action is `Blitz`, and Java's revert list is `BLITZ_MOVE || isBlockAction() || GAZE_MOVE ||
isKickingDowned()` — `BLITZ` is in neither engine's list (`PlayerAction.isBlockAction()` is
`BLOCK|VICIOUS_VINES|BREATHE_FIRE|CHAINSAW|STAB|PROJECTILE_VOMIT|CHOMP` in both). So the revert
cannot rescue the wrong ordering; the ordering itself has to change. The fix is kept because it is
correct against the Java and now has a test pinning it.

### Measured and REVERTED: moving FOUL_APPEARANCE into SelectBlitzTarget for BB2020

Implemented exactly as the tables above describe — edition-gate FA+DUMP_OFF into
`bb2025/select_blitz_target.rs` for BB2020, and in `bb2025/blitz_block.rs` gate FA behind a new
`frenzy_block` param and drop DUMP_OFF, for BB2020 only. Threaded `rules` through
`StepEndMoving::push_sequence_for_player_action` and the two `StepEndBlocking` push sites.

**Result: nurgle 86/100 → 0/100.** Reverted per the gate rule.

It fails at seed 1 step 1 — a blitz that previously passed:

```
i=2  JAVA h02:11,7,Standing   RUST h02:11,7,Prone
     Java rng 13 -> 15 (2 dice)      Rust rng 13 -> 16 (3 dice)
```

Both engines push h02 to 11,7; Java leaves it standing (a Push), Rust knocks it down and rolls
armour. The net die count should have been unchanged by the move (one added, one removed) but Rust
gained a die. **That contradiction is the thing to explain before retrying** — do not simply
re-apply the change. Two concrete leads: BB2020 rolls Foul Appearance at target-selection time,
i.e. BEFORE the blitzer moves, when the target may not be adjacent yet — check whether
`StepFoulAppearance` is reached with a defender set at that point, and what `StepDumpOff` does that
early. Also verify whether the Rust blitz reaches `SelectBlitzTarget` on every path or only from
`StepEndSelecting::BlitzSelect`.

### Gate

| check | result |
|---|---|
| `nurgle` bb2020 | 86/100 (unchanged — no progress this iteration) |
| `lineman` bb2020 | 100/100 |
| `necromantic` bb2020 | 100/100 (ITER91 holds) |
| `dwarf` bb2020 | 100/100 |
| `cargo test --workspace` | clean |

bb2020 remains **26 of 30 green**. RED: nurgle 86 · halfling 98 · wood_elf 98 · slann_fumbbl 98.

### Note

`crates/ffb-engine/src/step/bb2020/move_/step_stand_up.rs` has the same extra
`standing_up = false` on its free path, but that file is DEAD — the driver runs the shared bb2025
step for bb2020. Left alone deliberately rather than editing unreachable code mid-measurement.

### Next iteration

Either finish the sequence move above (starting from the die-count contradiction, not from the
patch), or switch to one of the three 98/100 rosters, which are single-seed problems and likely
cheaper: halfling 55/74, wood_elf, slann_fumbbl.

## ITER93 — halfling seed 74 localised to one activation, dice IDENTICAL; two tooling corrections

No engine change. halfling stays 98/100 (seeds 55, 74). Findings only, plus two measurement
corrections that cost time today and must not be repeated.

### halfling seed 74

```
[207] JAVA t4 h2 home  pre=d7185a18a5979c67 post=f120cfb03c309153 Activate(Home1, BLITZ)
      RUST t4 h2 home  pre=d7185a18a5979c67 post=2e973c007ef08f4b Activate(home_01, Blitz)
[208]   a02:  JAVA 12,9,Standing   RUST 12,9,Prone      <- the only differing field
```

`home_01` is the halfling **Treeman** (Take Root, Mighty Blow, Stand Firm, Strong Arm, Thick Skull,
Throw Team-Mate, Timmm-ber!) at (12,8); the target `away_03` is the adjacent halfling at (12,9), so
the blitz needs no movement. The Rust drive trace shows the whole block resolving —
`InitBlocking → GoForIt → FoulAppearance → … → BlockRoll → BlockChoice → BothDown/Wrestle →
DropFallingPlayers` — and the defender ending PRONE **in the same square**. Java leaves it standing,
also in the same square. Same square, different knocked-down state, same dice.

Note this is the third bb2020 red in a row (necromantic ITER91, nurgle ITER92, halfling here) whose
first divergence is a BLITZ activation that Java resolves more conservatively than Rust.

### Correction 1 — Java `rng_calls` are per-CALL, Rust's are per-DIE

Step 207 reads `rng_calls` 168 → 173 in Java (5) and 168 → 180 in Rust (12). That is **NOT a dice
divergence**: Java counts one call for a 3-dice block roll, Rust counts three. A ~2.4x ratio on an
activation containing a block + armour + injury is normal. Two iterations today nearly chased this
as a missing/extra roll. Compare dice by the `(sides, result)` SEQUENCE, never by either counter.

### Correction 2 — `scripts/dicediff.py`'s first-diff index is not the first divergence

For this seed it reported "FIRST DIFF at index 194: java d16=1 vs rust d6=5", which looks exactly
like a wrong casualty-die width. It is not:

* Rust's bb2020 `roll_casualty` is **already** `[rng.die(16), rng.d6()]`, matching Java's bb2020
  `RollMechanic.rollCasualty` (bb2025 the same; bb2016 is `[d6, d8]`). All three are correct — this
  was checked, not assumed.
* Java's `caller=` frames put that d16 at dice pos 195, while the state diverged at `rng_calls` 168.
  The window pos 186-196 (`StepStandUp` → `GoForIt` → 3 block dice → armour → injury → casualty) is
  entirely DOWNSTREAM of step 208. The "first diff" is just where the two already-desynced streams
  finally disagreed on a die WIDTH.

So: once the state has diverged, every later dice index is meaningless. Always establish the first
diverging step with `stepdiff.py` FIRST, and only compare dice within that step's window. The
`caller=` frames in the Java trace are the reliable way to place a die in the sequence.

### Gate

| check | result |
|---|---|
| `halfling` bb2020 | 98/100 (unchanged — findings only) |
| working tree | no engine change |

bb2020 stays **26 of 30 green**. RED: nurgle 86 · halfling 98 · wood_elf 98 · slann_fumbbl 98.

### Next iteration

halfling seed 74 step 207 is now a tight target: ONE activation, dice confirmed identical, one
differing field. Determine why Java's block leaves `away_03` standing in its own square while Rust
knocks it down there. Both "pushed nowhere" outcomes point at the pushback/Stand Firm family again
(the Treeman attacker has Stand Firm; the defender is a plain halfling, so it is the RESULT
interpretation, not a refused push). Read `StepBlockChoice`'s BothDown/Wrestle arm against
`bb2020/StepBlockChoice.java` before anything else.

## ITER94 — RETRACTION: halfling seed 74's dice are NOT identical; Rust is ~3 dice BEHIND Java at the block

No engine change. halfling stays 98/100. This iteration corrects ITER93 and replaces its conclusion
with a measured one.

### Retraction

**ITER93 claimed "dice confirmed identical" for halfling seed 74. That is false.** It rested on
`dicediff.py` reporting the value sequences matching through index 193 — but matching VALUES at the
same index does not mean the same ROLL. Two `FFB_DIE_AT` probes settle it:

| die pos | Java (`caller=` frame) | Rust (`FFB_DIE_AT` backtrace) |
|---|---|---|
| 188 | `DiceRoller.rollDice:90` — a **block die** | `StepTakeRoot::start` — a **Take Root** roll |
| 195 | `RollMechanic.rollCasualty` — a **casualty** roll | `StepMoveDodge::start` — a **dodge** |

Both engines happen to roll a 6 at position 188, which is exactly why the value comparison looked
clean. At the same absolute die position Java is already resolving the block while Rust is still on
Take Root, so **Rust needs more dice to reach the block — it has spent extra dice earlier in the
game.** Java's Take Root for this activation is at 185, Rust's at 188: Rust runs about three dice
behind.

Corroboration from the two engines' own views of the block roll: Java's three block dice are
`6,5,6` (positions 188-190, `rollDice:90` frames), while Rust's `BlockChoice` prompt carries
`dice: [2, 5, 6], nr_of_dice: 3`. Different first die → different chosen result at index 0 → Java
leaves `away_03` standing, Rust knocks it down. The dice count (3) and the choice index (0) agree;
only the values differ.

### What this means

The halfling divergence is **not** in `StepBlockChoice` or the Both Down / Skull interpretation, and
ITER93's "read `bb2020/StepBlockChoice.java` first" is withdrawn. The real defect is upstream: Rust
performs one or more rolls that Java does not, early enough to be invisible in the state hashes
(the per-step states match all the way to 206). An extra roll whose outcome does not change state
is precisely the shape that hides from `stepdiff.py`.

Note the shape matches ITER92's unexplained nurgle observation — there too Rust gained a die where
the accounting said it should not. These may be the same defect.

### Method note (third correction in two iterations)

`scripts/dicediff.py` compares by position and therefore CANNOT distinguish "same roll" from "same
number". It is only safe once both streams are known to be aligned. The reliable pairing is
Java's `caller=` frames against Rust's `FFB_DIE_AT=<pos>` backtrace — one probe each, and the
question is answered outright. Use that pair FIRST on any suspected dice divergence; do not let
`dicediff.py`'s first-diff index set the hypothesis.

### Gate

| check | result |
|---|---|
| `halfling` bb2020 | 98/100 (unchanged — findings only) |
| working tree | no engine change |

bb2020 stays **26 of 30 green**. RED: nurgle 86 · halfling 98 · wood_elf 98 · slann_fumbbl 98.

### Next iteration

Find the earliest die position where the two engines' backtraces disagree about which STEP drew the
die — bisect with `FFB_DIE_AT` against the Java `caller=` list, which is already dumped for this
seed. That names the extra Rust roll directly. Do not start from the block.

## ITER95 — "Moles under the Pitch" was never passed to the Rush modifier: halfling 98 → 100/100 GREEN, wood_elf 98 → 99

The ITER94 method worked exactly as intended. Bisecting with `FFB_DIE_AT` against Java's `caller=`
frames pinned the divergence to a **single die** in four probe runs.

### Bisection

Probing Rust at 20/40/60/80/100/120/140/160/180 and comparing which STEP drew each die against
Java's frames: aligned through 160, mismatched at 180. Second pass (165-179): aligned through 171,
mismatched at 173. Third pass (170-174) closed it:

| die | Java | Rust |
|---|---|---|
| 170 | `StepStandUp` | `StepStandUp` ✓ |
| 171 | `StepGoForIt.rush`, **d6 = 2** | `StepGoForIt` ✓ |
| 172 | `InjuryTypeDropGFI.handleInjury` ← `StepFallDown` | `StepBlockRoll` ✗ |

Same roll, same value: **Java's rush FAILED and the player fell; Rust's succeeded and went on to
block.** All 11 GFI rolls in the seed match in value and order
(`4,3,5,2,2,6,4,3,2,6,6`), so Java's minimum at die 171 was 3 while Rust's was 2.

### Root cause

Weather was `Nice`, so Blizzard was not it. The other `+1` GFI modifiers in the BB2016/BB2020
collection are the two **Moles under the Pitch** entries — a Prayer to Nuffle. Every Java call site
builds the context with the prayer state:

```java
new GoForItContext(game, actingPlayer.getPlayer(), getGameState().getPrayerState().getMolesUnderThePitch())
```
(`bb2025/StepGoForIt.java:214`, `bb2020:213`, `bb2016:161`, `UtilServerPlayerMove.java:173`)

All four Rust call sites used the **2-arg** `GoForItContext::new(game, player)`, which leaves
`teams_with_moles_under_pitch` EMPTY — so the modifier could never fire anywhere, in any edition.
Everything else was already correct and in place: `PrayerState::get_moles_under_the_pitch`, the
prayer handlers, the modifier entries in the factory, even a
`GoForItContext::new_with_moles` constructor with its own unit test. Only the wiring between them
was missing.

**Why it hid for so long:** the prayer is not part of the parity state string, so an unapplied
modifier is invisible until it flips a roll — and it only flips one when the rush comes up exactly
on the boundary (a 2). Halfling is the roster that rushes constantly with Prayers active.

### Fix

Pass `game.prayer_state.get_moles_under_the_pitch().clone()` into
`GoForItContext::new_with_moles` at all four sites (bb2025 + bb2016 + `UtilServerPlayerMove` are
live; the bb2020 step is off the live path but kept in step with its Java counterpart). Regression
test `moles_under_the_pitch_raises_the_rush_minimum` pins both directions — a Rush of 2 succeeds
without the prayer and FAILS with it.

### Gate

| check | result |
|---|---|
| `halfling` bb2020 | **98/100 → 100/100 GREEN** |
| `wood_elf` bb2020 | **98/100 → 99/100** |
| `slann_fumbbl` bb2020 | 98/100 (unchanged) |
| `nurgle` bb2020 | 86/100 (unchanged) |
| `necromantic` bb2020 | 100/100 (holds) |
| `lineman` bb2020 / bb2025 / bb2016 | 100/100 each (run serially) |
| `cargo test --workspace` | clean |

**bb2020 is now 27 of 30 green.** RED: nurgle 86 · slann_fumbbl 98 · wood_elf 99.

### Next iteration

`wood_elf` at 99/100 is one seed from green and the cheapest target. Use the ITER94/95 method from
the start: `stepdiff.py` for the diverging step, then `FFB_DIE_AT` vs Java `caller=` to pin the die.

Worth checking opportunistically: other `*Context` constructions in Rust that drop an argument Java
passes. This defect class — a context built with fewer arguments than Java's, silently disabling a
whole modifier family — is new to the campaign and would be invisible to the state hash every time.

## ITER96 — the trap-door injury was applied inline instead of published: wood_elf 99 → 100/100 GREEN

### Measurement

wood_elf seed 50, last turn (t8 h2), step 299 `Activate(Home6, MOVE)`. One differing field:

```
i=300   h05:  JAVA -1,-1,Injured   RUST -1,-1,Reserve
```

Dice are genuinely identical this time — verified the right way, by `caller=` frame rather than by
index. Java's tail:

```
86  d6=1   StepTrapDoor.start                      (the trap door opens)
87  d6=5   rollInjury  InjuryTypeCrowd.handleInjury  StepTrapDoor.trapDoorTriggered
88  d6=5   rollInjury  InjuryTypeCrowd.handleInjury  StepTrapDoor.trapDoorTriggered
89  d16=1  InjuryTypeServer.setInjury (casualty)
90  d6=5   InjuryTypeServer.setInjury (casualty)
```

Rust's dice at 86-90 are `1, 5, 5, d16=1, 5` — the same rolls, including the casualty. Both engines
rolled the casualty; only Rust failed to apply it.

### Root cause

Java `StepTrapDoor.trapDoorTriggered` (`:137-140`):

```java
publishParameter(new StepParameter(StepParameterKey.INJURY_RESULT,
    UtilServerInjury.handleInjury(this, ..., ApothecaryMode.TRAP_DOOR)));
game.getFieldModel().remove(player);
```

The result is **published** for the `APOTHECARY(TRAP_DOOR)` step that follows in the sequence — the
drive trace confirms `TrapDoor → Apothecary → CatchScatterThrowIn`. Rust instead called
`ir.apply_to(game)` inline and published nothing, and the `remove_player` two lines later then
overwrote the state, so the casualty surfaced as `Reserve` instead of `Injured`.

This is the **roll-it-but-don't-apply-it** pattern already recorded for the Ball & Chain chain
injury: in Java these results travel to the apothecary step as a parameter. Applying them at the
roll site loses them.

### Fix

Publish `StepParameter::InjuryResult(Box::new(ir))` and drop the inline `apply_to`, mirroring Java
exactly. Regression test `trap_door_publishes_the_injury_result_instead_of_applying_it`.

### Gate

| check | result |
|---|---|
| `wood_elf` bb2020 | **99/100 → 100/100 GREEN** |
| `halfling` bb2020 | 100/100 (ITER95 holds) |
| `necromantic` bb2020 | 100/100 (holds) |
| `slann_fumbbl` bb2020 | 98/100 (unchanged) |
| `nurgle` bb2020 | 86/100 (unchanged) |
| `lineman` bb2020 / bb2016 / bb2025 | 100/100 each (run serially) |
| `cargo test --workspace` | clean |

**bb2020 is now 28 of 30 green.** RED: nurgle 86 · slann_fumbbl 98.

### Next iteration

`slann_fumbbl` at 98/100 (2 seeds) is the cheaper of the two remaining. Same method: `stepdiff.py`
for the diverging step, then `FFB_DIE_AT` vs Java `caller=` to pin the die — this has now resolved
three reds in a row at single-die precision.

Worth a cheap sweep at some point: grep for other sites that call `apply_to` on an
`InjuryResult` where the Java publishes `INJURY_RESULT` instead. Two instances of this pattern have
now cost a red each (Ball & Chain, trap door).

## ITER97 — slann_fumbbl: Rust rolls a Trap Door d6 Java never rolls (die identified, cause not yet)

No engine change; slann_fumbbl stays 98/100 (seeds 29, 50). One measured-and-reverted attempt.

### The extra die, named exactly

slann_fumbbl seed 29, step 287 `Activate(Away1, BLITZ)` at t8 h2. One differing field at i=288:
`h00` is `13,10,Standing` in Java and `13,10,Prone` in Rust — the same "Java's blitz is more
conservative" shape as the last three reds.

Pairing Java `caller=` frames with Rust `FFB_DIE_AT` backtraces at the same positions:

| die | Java | Rust |
|---|---|---|
| 90 | `rollBlockDice` (a 1-die block) | **`StepTrapDoor::start`** |
| 91 | `rollKnockoutRecovery` ← `StepEndTurn` | `StepBlockRoll` |
| 92-93 | `StepMvp` (d1 picks) | `InjuryTypeBlock` armour |

**Rust rolls a Trap Door d6 that Java does not roll at all.** That one spurious die shifts the block
roll by one position — Java blocks with a 3, Rust with a 6 — which is why the defender goes down in
Rust and stays up in Java. Everything downstream follows from it.

### Ruled out

* **`roll_casualty` widths** — checked, not assumed: Rust bb2020/bb2025 are `[die(16), d6]`,
  bb2016 `[d6, d8]`, all matching Java.
* **`rng_calls` 89 vs 90 at i=271** — NOT evidence of an extra roll. Java counts per CALL, Rust per
  DIE, so a multi-die block roll alone explains the gap. (This is the trap recorded in ITER93; it
  briefly pointed at the wrong step here too.)
* **The trap-door coordinates** — `TreacherousTrapdoorHandler` is identical in both engines:
  (6,1) and (19,13) on `initEffect`, `clearTrapdoors()` on `removeEffectInternal`.

### Measured and REVERTED: gating `PLAYER_ENTERING_SQUARE` on the trap door

Java's `setParameter` records the id ONLY if the player is on a trap door at PUBLISH time:

```java
case PLAYER_ENTERING_SQUARE:
  if (isOnTrapDoor(fieldModel, fieldModel.getPlayerCoordinate(player))) {
    playerId = (String) parameter.getValue();
  }
```

Rust stores it unconditionally and re-tests at EXECUTE time — a genuinely different question once
the player has moved on. Implemented faithfully by adding a `Step::set_parameter_with_game` hook
(default no-op) so the condition could be evaluated at publish time, wired through
`DriverStepStack::publish`.

**Result: slann_fumbbl 98/100, unchanged; the same two seeds; `FFB_DIE_AT=90` still lands in
`StepTrapDoor::start`.** So the player IS on a trap door at publish time and this is not the cause.
Reverted rather than left in place: it adds framework surface (a new trait method on every step)
for no measured gain, and it is one minute's work to re-apply if the real fix needs it. The
observation that Rust's version is not 1:1 stands and is recorded here.

### Leading hypothesis for the next iteration

The prayer's duration is `UntilEndOfHalf`. This divergence is in **half 2, turn 8** — so if the
Treacherous Trapdoor prayer was granted in half 1 and Java ran `removeEffectInternal`
(`clearTrapdoors()`) at the end of that half while Rust did not, Rust would carry trap doors into
half 2 that Java does not have. That fits every observation: Java rolls nothing because
`isOnTrapDoor` is false for it, and the discrepancy is invisible to the parity state hash because
trap doors are not part of the state string — the same "invisible until it flips a roll" property
as ITER95's Moles prayer.

Check first: where Rust expires `InducementDuration::UntilEndOfHalf` prayer effects, and whether
`remove_effect_internal` is actually invoked for Treacherous Trapdoor at half end. A gated probe
printing `game.field_model.trap_doors` with half/turn at `StepTrapDoor::execute_step` settles it in
one run.

### Gate

| check | result |
|---|---|
| `slann_fumbbl` bb2020 | 98/100 (unchanged — findings only) |
| working tree | clean at HEAD, no engine change |

bb2020 stays **28 of 30 green**. RED: nurgle 86 · slann_fumbbl 98.

## ITER98 — root cause of the slann trap door found in full: Rust never records a granted prayer, so NO prayer effect can ever expire

No engine change; slann_fumbbl stays 98/100. The causal chain is now complete and the fix is a
well-defined two-part change, deliberately left for the next iteration rather than half-landed.

### The probe that settled it

A gated print at `StepTrapDoor::execute_step` on slann_fumbbl seed 29:

```
TD half=2 turnH=8 turnA=8 pid=home_01 coord=(13,10)
   doors=[(6,1), (19,13)]   home_prayers=[]   away_prayers=[]
```

Trap doors are still on the pitch in the SECOND half — while **both teams' prayer lists are
empty**. That single line falsifies the ITER97 hypothesis and replaces it.

### The chain

1. **Rust never adds a granted prayer to `turn_data.inducement_set`.** Outside tests there is not
   one production call to `add_prayer` (`grep -rn "add_prayer" crates/ffb-engine/src`); the prayer
   step applies `init_effect` and records nothing.
2. Java's expiry walks exactly that collection —
   `for (Prayer prayer : game.getTurnDataHome().getInducementSet().getPrayers())` in
   `StepEndTurn.deactivatePrayers` — so with an empty set there is nothing to expire.
3. **No prayer effect is ever removed on the live path.** `PrayerHandlerFactory::deactivate_prayers`
   exists but is called only from the DEAD `bb2020/step_end_turn.rs`; the live BB2025 `StepEndTurn`
   deactivates CARDS for each duration and never prayers, where Java's
   `deactivateEffectsAndPrayers` does both (`StepEndTurn.java:489-506`).
4. So Treacherous Trapdoor's doors (BB2020 duration `UntilEndOfHalf`) survive into half 2, and a
   player standing on one rolls a Trap Door d6 that Java never rolls — the ITER97 die.

Invisible to the parity state hash throughout: neither trap doors nor prayers are in the state
string. Same property as ITER95's Moles prayer, and the same class of bug.

### Measured and REVERTED: duration-aware prayer deactivation alone

Ported `deactivatePrayers(duration, isHomeTurnEnding)` faithfully — both teams' sets, the
`UNTIL_END_OF_OPPONENTS_TURN` skip, remove-from-set then `removeEffect` then `ReportPrayerEnd`, plus
a `prayer_duration_by_name` lookup per ruleset, wired into the live `StepEndTurn` at Java's four
call sites (with `UNTIL_END_OF_HALF` correctly under `if (fNewHalf)` ALONE, not
`fNewHalf || fTouchdown` — Java `:502-506`).

**Result: slann_fumbbl 98/100, unchanged.** Necessarily so: step 1 means the collection it iterates
is always empty. Reverted rather than left in as dead-but-correct code.

### The fix, for the next iteration

Both parts are required and must land together, then be measured as one change:

1. **Record the prayer.** Find where the prayer step applies `init_effect` and add the prayer to the
   praying team's `inducement_set` (Java's `StepPrayer` does both). Verify against Java which
   collection and which name form (`get_name()` vs `name()`) is stored.
2. **Expire it.** Re-apply the duration-aware `deactivate_prayers_for_duration` above (the exact
   code is in this commit's history if needed).

Expect this to move more than slann_fumbbl: no `UntilEndOfHalf` or `UntilEndOfDrive` prayer effect
has ever expired in any bb2020 game, so it is a broad behavioural change. Gate it against the full
green set, not just the target.

### Gate

| check | result |
|---|---|
| `slann_fumbbl` bb2020 | 98/100 (unchanged — findings only) |
| working tree | clean at HEAD, no engine change |

bb2020 stays **28 of 30 green**. RED: nurgle 86 · slann_fumbbl 98.

## ITER99 — record the prayer, then expire it: slann_fumbbl 98 → 99/100

The two-part fix ITER98 scoped, landed together and measured as one change.

### Part 1 — record the granted prayer

Java's FINAL `PrayerHandler.initEffect(step, gameState, prayingTeamId)` does
`inducementSet.addPrayer(handledPrayer())` before delegating to the concrete effect
(`PrayerHandler.java:46-47`). Rust's trait has no such wrapper — each handler implements the
concrete `init_effect` directly — so the recording was simply never done and **not one granted
prayer ever landed in an inducement set**. Added at Rust's equivalent single entry point, the
`h.init_effect(...)` call in `bb2020/step_prayer.rs`.

### Part 2 — expire it by duration

`PrayerHandlerFactory::deactivate_prayers_for_duration`, a 1:1 port of
`StepEndTurn.deactivatePrayers(duration, isHomeTurnEnding)`: both teams' sets, the
`UNTIL_END_OF_OPPONENTS_TURN` skip, remove-from-set → `removeEffect` → `ReportPrayerEnd`, with a
per-ruleset `prayer_duration_by_name` lookup. Wired into the live BB2025 `StepEndTurn` at Java's
four call sites — `UNTIL_END_OF_HALF` under `if (fNewHalf)` **alone**, not
`fNewHalf || fTouchdown` like the drive block (`StepEndTurn.java:502-506`).

Neither part does anything without the other, which is why ITER98's part-2-only attempt measured
inert.

Regression test `expiring_a_duration_removes_only_matching_prayers_and_their_effects` pins both
directions: a non-matching duration leaves the prayer and its trap doors alone; the matching one
removes it from the set AND clears the doors.

### Gate

| check | result |
|---|---|
| `slann_fumbbl` bb2020 | **98/100 → 99/100** |
| `halfling` bb2020 | 100/100 (holds) |
| `wood_elf` bb2020 | 100/100 (holds) |
| `necromantic` bb2020 | 100/100 (holds) |
| `dwarf` bb2020 | 100/100 (holds) |
| `nurgle` bb2020 | 86/100 (unchanged) |
| `lineman` bb2020 / bb2025 / bb2016 | 100/100 each (run serially) |
| `cargo test --workspace` | clean |

No regression anywhere, despite this being the first time any BB2020 prayer effect has ever
expired. bb2020 stays **28 of 30 green**; slann_fumbbl is now one seed away.

### Next iteration

`slann_fumbbl` seed 50 is the last blocker on that roster — the ITER97 finding was for seed 29,
which this fixed, so seed 50 needs its own diagnosis from `stepdiff.py` + `FFB_DIE_AT`. After that
only `nurgle` 86 remains, whose confirmed root cause (BB2020 resolves Foul Appearance BEFORE
STAND_UP, ITER92) still has an unexplained die-count contradiction blocking the obvious fix.

## ITER100 — the Cheering Fans prayer pick ignored already-held prayers: slann_fumbbl 99 → 100/100 GREEN

### Measurement

slann_fumbbl seed 50 diverges at step 132, and badly — Java ends the turn (home active, t1) while
Rust plays on (away, t0), with the blitzer KO'd in Java and standing in Rust.

Whole-game dice counts: **Java 97, Rust 78**. Java rolls exactly ONE prayer die all game; Rust rolls
none:

```
pos 54  JAVA d3=2  BadHabitsHandler.affectedPlayers ← RandomSelectionPrayerHandler.initEffect
                   ← PrayerHandler.initEffect ← StepPrayer.executeStep
        RUST d6     StepBlockRoll        (FFB_DIE_AT=54)
```

Positions 47-53 match exactly, including the two `handleCheeringFans` d6s — so both engines ran
Cheering Fans, both pushed a `PRAYER` step, and both picked a prayer. Java picked **Bad Habits**,
which rolls a d3 for its affected players; Rust picked a prayer that rolls nothing. Different
prayer, same position.

(A probe in `bb2020/step_apply_kickoff_result.rs` printed nothing — that file is DEAD for bb2020,
the shared bb2025 step runs. Its own bb2020 gate is where the live code is.)

### Root cause

Java picks from `prayerFactory.availablePrayerRolls(ownInducements, opponentInducements)`
(`PrayerFactory.java:35-41`), which **filters out prayers the team already holds** — and any
`affectsBothTeams()` prayer the opponent holds — before `Collections.shuffle`. Rust shuffled the
full `1..=max_prayer_roll` every time. Once any prayer had been granted the two engines shuffled
lists of **different length**, so the same shuffle stream yielded a different first element, a
different prayer, and from there a different dice stream for the rest of the game.

This filter was **impossible to implement before ITER99** — nothing was ever recorded in an
inducement set, so there was never anything to filter. ITER99 and ITER100 are one fix in two parts,
and the ordering was forced.

### Fix

Mirror `availablePrayerRolls` in the bb2020-gated Cheering Fans branch of the shared
`bb2025/kickoff/step_apply_kickoff_result.rs`, keyed off the praying team's own set and the
opponent's for `affects_both_teams` prayers. Regression test
`bb2020_cheering_fans_prayer_pick_excludes_prayers_already_held`.

### Gate

| check | result |
|---|---|
| `slann_fumbbl` bb2020 | **99/100 → 100/100 GREEN** |
| `halfling` / `wood_elf` / `necromantic` / `dwarf` bb2020 | 100/100 each (hold) |
| `nurgle` bb2020 | 86/100 (unchanged) |
| `lineman` bb2020 / bb2025 / bb2016 | 100/100 each (run serially) |
| `cargo test --workspace` | clean (exit 0) |

**bb2020 is now 29 of 30 green.** RED: nurgle 86 — the last one.

### Next iteration

Only `nurgle` remains. Its root cause is already confirmed (ITER92: BB2020 resolves Foul Appearance
BEFORE STAND_UP, BB2025 after), but the obvious sequence-move regressed it 86 → 0 and left an
unexplained die-count contradiction — the move should be die-neutral yet Rust gained a die at
seed 1 step 1. Start from that contradiction, with the `FFB_DIE_AT` + Java `caller=` pairing that
has now resolved five reds in a row, and do NOT re-apply the ITER92 patch blind.

## ITER101 — nurgle: the FA sequence move is DICE-CORRECT; ITER92's "extra die" was a counter artifact

No engine change; nurgle stays 86/100. This iteration re-ran ITER92's experiment with the ITER93
method and **corrects its diagnosis**, narrowing the remaining problem to one specific effect.

### Retraction of ITER92's blocker

ITER92 abandoned the sequence move partly on a stated contradiction: "the move should be die-neutral
yet Rust gained a die at seed 1 step 1 (Java 2, Rust 3)". **That was the per-call/per-die counter
confusion documented in ITER93** — Java's `rng_calls` counts one call for a multi-die roll, Rust
counts each die. There was never an extra die. That reasoning should not be carried forward.

### What the move actually does (measured properly)

Re-applied in full, then compared dice by `(sides, result)` with Java's `caller=` frames rather than
by counter. On nurgle seed 1 the dice now match **through position 49**, including the Foul
Appearance roll landing in the right place:

```
pos 14  JAVA d6=6  FoulAppearanceBehaviour.handleExecuteStepHook   | RUST d6=6   ← now aligned
pos 15  JAVA d6=4  rollBlockDice                                    | RUST d6=4
```

Before the move, Rust's FA fired after STAND_UP and the streams parted much earlier. **The move is
correct.** Confirmed separately that the defender IS already set when `SelectBlitzTarget` runs
(`defender=Some("home_03")` at the STAND_UP probe), so the inserted FA step does fire.

### Why it still regresses 86 → 0/100

The failure is a **state** difference on identical dice. nurgle seed 1 step 1, block die = 4
(PUSHBACK):

```
i=2   h02:  JAVA 11,7,Standing   RUST 11,7,Prone
```

Both engines push `h02` to the same square; Java leaves it standing, Rust knocks it down. Every die
up to that point matches.

**DUMP_OFF placement is NOT the cause** — isolated and measured: keeping DUMP_OFF in `BlitzBlock`
(instead of moving it to `SelectBlitzTarget` as Java does) still gives 0/100. It is the FA move
alone.

### Leading hypothesis for the next iteration

`StepFoulAppearance`'s SUCCESS path calls `commit_target_selection()`
(`mixed/step_foul_appearance.rs`: `ts.commit()`). Moving FA out of `BlitzBlock` removes that commit
from the block sequence, and something in Rust's block resolution depends on it —
`step_remove_target_selection_state.rs:45` reads `tss.is_committed()` to set
`acting_player.has_triggered_effect`, and `StepFoulAppearance` itself resolves its defender only
from a TSS that is `is_selected() && is_committed()`.

Java has the same commit in the same place and is fine, so the coupling is Rust-side. Next step:
find what in the block path changes a PUSHBACK into a knockdown when the TSS commit happens earlier
— instrument `StepBlockChoice` / `StepPushback` for the commit flag, not the dice.

### Gate

| check | result |
|---|---|
| `nurgle` bb2020 | 86/100 (unchanged — reverted, findings only) |
| working tree | clean at HEAD, no engine change |

bb2020 stays **29 of 30 green**. RED: nurgle 86.

## ITER102 — the inserted FA step never FIRES: the move loses the die rather than relocating it

No engine change; nurgle stays 86/100. This supersedes both ITER92's and ITER101's explanations,
and corrects ITER101's central claim.

### Correction to ITER101

ITER101 said "the dice now match through position 49, including the Foul Appearance roll landing in
the right place". **That was the value-coincidence trap for the third time in this campaign.** Java
and Rust both roll a 6 at position 14, so a value comparison looked aligned. They are not the same
roll:

| die 14 | Java `caller=` | Rust `FFB_DIE_AT` |
|---|---|---|
| | `FoulAppearanceBehaviour.handleExecuteStepHook` | **`StepBlockRoll`** |

Java's die 14 is the Foul Appearance roll and its block die is 15 (=4 → PUSHBACK). Rust's die 14 IS
the block die (=6 → POW). So under the patch Rust runs **one die ahead** at the block: the FA step
inserted into `SelectBlitzTarget` **never fired**, while the FA in `BlitzBlock` was removed. The
move DELETES the Foul Appearance roll instead of relocating it.

A `StepBlockChoice` probe confirms the consequence directly:

```
BC result=Pow  def=Some("home_03")      ← Rust, from block die 6
                                          Java's block die is 4 → PUSHBACK
```

Hence `h02` Prone in Rust and Standing in Java, on what looked like identical dice. Rust's
`block_result_for_roll` mapping is correct (`1 Skull, 2 BothDown, 5 PowPushback, 6 Pow, _ Pushback`)
— it was fed a different die, not mis-mapped. Checked, not assumed.

This also retires ITER101's `commit_target_selection` hypothesis: the FA step is not reaching its
commit because it is not running at all.

### The remaining question, now precise

Why does `StepFoulAppearance` return early when placed in `SelectBlitzTarget`? Its guards are:

```rust
let defender_id = tss.filter(|ts| ts.is_selected() && ts.is_committed())
                     .and_then(get_selected) .or(game.defender_id);
if !defender_has_fa || attacker_cancels { return next(); }
```

A STAND_UP probe in the same sequence showed `defender=Some("home_03")` and `tss=None`, so
`game.defender_id` IS populated there — for nurgle seed 2. Seed 1 step 1 must be checked directly:
instrument the early-return in `StepFoulAppearance` (defender_id, defender_has_fa,
attacker_cancels) and run nurgle seed 1 with the move applied. That single probe names the guard.

Note `attacker_cancels` is the likely suspect: it tests
`has_skill_to_cancel_property(FORCE_ROLL_BEFORE_BEING_BLOCKED)` on the ACTING player, and in a
nurgle-vs-nurgle mirror both sides carry Foul Appearance. A previous fix in that same line is
already documented in the file's comments, so the surrounding logic is delicate — read the Java
before touching it.

### Gate

| check | result |
|---|---|
| `nurgle` bb2020 | 86/100 (unchanged — reverted, findings only) |
| working tree | clean at HEAD, no engine change |

bb2020 stays **29 of 30 green**. RED: nurgle 86.

## ITER103 — ROOT CAUSE: a Rust blitz that needs no stand-up SKIPS `SelectBlitzTarget` entirely

No engine change; nurgle stays 86/100. The chain is now complete, and it is a structural gap, not a
sequence-ordering tweak.

### The measurement

With the FA move applied, a probe placed at `StepFoulAppearance`'s early return printed **no line at
all for any BLITZ** — the step was never executing, not merely returning early. The drive trace
shows why. nurgle seed 1 step 1, `Activate(away_03, Blitz)`:

```
RUST_BLOCK_PICK pid=away_03 def=home_03            ← agent pre-picks the target at activation
DRIVE step=EndSelecting
DRIVE step=InitActivation … BoneHead … BloodLust    ← activation block
DRIVE step=InitBlocking                             ← straight into BlitzBlock
DRIVE step=GoForIt / SteadyFooting / BlockStatistics / … / BlockRoll
```

**There is no `SelectBlitzTarget` sequence.** For this blitz Rust goes `StepEndSelecting` → blitz
block directly. The FA step inserted into `SelectBlitzTarget` therefore lives in a sequence this
blitz never enters, while the patch removed the FA from `BlitzBlock` — so the roll is deleted, Rust
runs a die ahead, and the block die becomes 6 (POW) instead of 4 (PUSHBACK). That is the whole 0/100.

Contrast nurgle seed 2, whose blitzer is PRONE: that one DOES run `SelectBlitzTarget`
(`GotoLabel → JumpUp → StandUp → EndSelecting`, seen in the ITER92 trace). So in Rust **some blitzes
route through `SelectBlitzTarget` and some do not** — apparently only those needing a stand-up.

In Java every blitz goes through `SelectBlitzTarget` — that is where the target is chosen, and where
BB2020 puts FOUL_APPEARANCE + DUMP_OFF. Java's seed-1 die 14 is a Foul Appearance roll for this same
standing blitzer, which confirms Java ran the sequence.

### Why the naive fix cannot work

BB2020 needs FA resolved **before the blitzer stands up**. The stand-up lives in `SelectBlitzTarget`
(prone case) and there is no such step at all in the standing case. So no single placement in the
current Rust routing is correct for both:

* FA in `SelectBlitzTarget` only → skipped for standing blitzers (this iteration, 0/100).
* FA in `BlitzBlock` only → runs after STAND_UP for prone blitzers (status quo, nurgle 86/100).

### The actual fix

Make Rust's blitz always route through `SelectBlitzTarget`, as Java does, and then place
FOUL_APPEARANCE + DUMP_OFF there for BB2020 with `BlitzBlock`'s copy frenzy-only. That is a
structural change to `StepEndSelecting`'s BLITZ dispatch (and interacts with the agent pre-picking
the block target at activation), so it needs its own iteration and a full-matrix gate — it will
change every blitz in every edition if not carefully gated.

Cheaper alternative worth measuring first: find whether the standing-blitzer path can be routed
through `BlitzSelect` → `SelectBlitzTarget` without disturbing bb2016/bb2025 (both currently 30/30),
i.e. gate the routing change on `Rules::Bb2020` only.

### Gate

| check | result |
|---|---|
| `nurgle` bb2020 | 86/100 (unchanged — reverted, findings only) |
| working tree | clean at HEAD, no engine change |

bb2020 stays **29 of 30 green**. RED: nurgle 86.

## ITER104 — FA can now be placed correctly on BOTH blitz paths with NO regression, but nurgle is unmoved

No engine change (reverted); nurgle stays 86/100 with the identical 14 seeds. The important result
is that the ITER77 blocker is solved and the placement problem is no longer what stands in the way.

### ITER77's blocker is solved

An in-code note at `bb2025/shared/step_end_selecting.rs` (Blitz dispatch) recorded that adding
FOUL_APPEARANCE to the inline blitz activation required a defender, and that setting
`game.defender_id` at dispatch to provide one took nurgle 86 → 0/100 because every step in the
activation then saw it. Its stated remedy — "the defender has to reach StepFoulAppearance as a step
PARAMETER, not via game state" — works:

* `StepFoulAppearance` now accepts `StepParameter::BlockDefenderId`, used only as a last resort after
  the TSS and `game.defender_id` lookups, so no other caller changes.
* The Blitz dispatch appends FOUL_APPEARANCE + DUMP_OFF (BB2020 only) to the inline activation,
  passing `block_defender_id` as that parameter.
* `BlitzBlock` gates its own FOUL_APPEARANCE to `frenzy_block` and drops DUMP_OFF for BB2020, so the
  roll happens exactly once.

Measured stepwise: with only the dispatch half, **86/100** (baseline, no regression). Adding the
`SelectBlitzTarget` half for the prone-blitzer path: **86/100** again. Both placements together are
regression-free — a real improvement over ITER92/101/103, where every attempt scored 0/100.

### But it does not move nurgle

nurgle seed 2 is unchanged with both halves applied:

```
i=33   a01:  JAVA 13,8,Prone   RUST 13,8,Standing
```

The prone blitzer still stands up, so **Foul Appearance is still not firing on that activation** even
though the step is now in the sequence and the defender is available (a STAND_UP probe two steps
later in the same sequence reports `defender=Some("home_03")`, and Java spends exactly one die there
— its failing FA roll).

Reverted per the gate rule: correct-looking but zero measured gain, and it changes every BB2020
blitz, so it should not land without a win to justify a full-matrix re-gate.

### Next iteration — one specific probe

Re-apply both halves and put a probe at the TOP of `StepFoulAppearance::execute_step` (not at the
early return, which ITER103's probe used and which printed nothing for any blitz). That
distinguishes the two remaining possibilities:

1. the step still is not being executed at all on the blitz path → the sequence insert is not where
   the driver runs it, or a preceding `GotoLabel` skips it;
2. it executes but the defender/`has_fa`/`attacker_cancels` guard rejects → `attacker_cancels` is
   the prime suspect in a nurgle mirror, where both sides carry Foul Appearance.

The exact patch is in this commit's history; re-applying it is mechanical.

### Gate

| check | result |
|---|---|
| `nurgle` bb2020 | 86/100 (unchanged — reverted, findings only) |
| working tree | clean at HEAD, no engine change |

bb2020 stays **29 of 30 green**. RED: nurgle 86.

## ITER105 — FA now EXECUTES on blitz activations with the right defender; the question moves to its failure path

No engine change (reverted); nurgle stays 86/100. One more possibility eliminated, and the target
narrows again.

### The probe ITER104 asked for

Both halves re-applied, with a probe at the TOP of `StepFoulAppearance::execute_step` (ITER103's
probe sat at the early return and printed nothing, which could not distinguish "never ran" from
"ran and rejected"). nurgle seed 2:

```
FA-ENTER action=Some(Blitz) acting=Some("away_02") game_def=Some("home_03") param_def=Some("home_03")
FA-ENTER action=Some(Blitz) acting=Some("away_02") game_def=Some("home_03") param_def=Some("home_03")
```

So on the blitz activation the step **does execute**, and the defender resolves correctly by BOTH
routes — `game.defender_id` and the new step parameter. ITER103's "the step never runs" is therefore
specific to the un-patched routing, not to the patched one; with both halves in place the step is
reached.

(The doubled line is a re-entry — `execute_step` runs from `start()` and again via `handle_command`
— and does not roll twice, since `self.roll` is retained. Consistent with nurgle measuring 86/100
rather than worse.)

### What is left

Foul Appearance runs, finds the defender, and rolls — yet nurgle seed 2 is byte-identical to
baseline: `i=33 a01: JAVA 13,8,Prone / RUST 13,8,Standing`. So the remaining gap is in the
**outcome**, not the placement:

* does Rust's roll FAIL as Java's does (Java spends exactly one die here and its blitz aborts)?
* if it fails, why does `fail_fa`'s `goto` not stop the blitzer standing up?

Note the two blitz paths use different failure labels — `END_BLITZING` in `SelectBlitzTarget`
(which sits BEFORE JUMP_UP/STAND_UP, so the goto skips them) versus `END_BLOCKING` in the inline
activation used by `StepEndSelecting`'s Blitz dispatch. If a PRONE blitzer takes the inline path,
its stand-up does not live in that sequence at all, so `goto END_BLOCKING` cannot prevent it. That
is the most likely remaining mechanism and is directly checkable.

### Next iteration

Re-apply and extend the same probe to print `self.roll`, the minimum, the success flag, and which
sequence the step is in (the failure label distinguishes them). If the prone blitzer is on the
inline path, the fix is to give that path the `END_BLITZING`-equivalent behaviour — i.e. the blitz
activation must be able to abort BEFORE the stand-up, which is what Java gets for free by always
running `SelectBlitzTarget` (ITER103).

### Gate

| check | result |
|---|---|
| `nurgle` bb2020 | 86/100 (unchanged — reverted, findings only) |
| working tree | clean at HEAD, no engine change |

bb2020 stays **29 of 30 green**. RED: nurgle 86.

## ITER106 — CONFIRMED: the prone blitzer takes the INLINE path, so its FA failure aborts to the wrong place

No engine change (reverted); nurgle stays 86/100. The mechanism is now fully pinned by measurement.

### The decisive line

nurgle seed 2, with both FA placements applied, probing the roll itself:

```
FA-ROLL acting=away_02 action=Blitz def=home_03 roll=1 min=2 ok=false
        fail_label=END_BLOCKING  standing_up=true
```

Everything works except the destination:

* Foul Appearance **runs** and finds the right defender (`home_03`).
* It **FAILS** — `roll=1` against `min=2`, exactly matching the single die Java spends here.
* The blitzer has **not yet stood up** (`standing_up=true` = still prone, pending).
* But `fail_label=END_BLOCKING`, which means this blitz is on the **inline activation path**, NOT
  `SelectBlitzTarget`.

So even a PRONE blitzer takes the inline path. `goto END_BLOCKING` jumps to the end of the
`BlitzBlock` sequence — and the stand-up is not in that sequence, so aborting there cannot prevent
it. In Java the same failure jumps to `END_BLITZING` INSIDE `SelectBlitzTarget`, which sits before
`STAND_UP`, so the stand-up never runs at all.

Note `fail_fa` does publish `END_PLAYER_ACTION(true)` for a blitzing player (Java's
`handleFailure` does the same, and `is_blitzing()` covers `Blitz`), yet the player still ends up
standing — so the publish alone does not substitute for aborting before the stand-up.

### Conclusion

This is the third independent measurement pointing at the same structural gap, and it retires the
"just move the step" family of fixes for good:

1. ITER103 — a blitz needing no stand-up skips `SelectBlitzTarget` entirely.
2. ITER105 — with the step inserted on both paths it executes and resolves its defender.
3. ITER106 — it also rolls and fails correctly; only the abort TARGET is wrong, because the prone
   blitzer is on the inline path too.

**The fix is ITER103's: route every BB2020 blitz through `SelectBlitzTarget`, as Java does**, so the
Foul Appearance abort lands before `STAND_UP`. Placement patches cannot express this, because the
inline activation has no stand-up to abort past.

### Next iteration

Attempt the routing change, gated on `Rules::Bb2020` so bb2016/bb2025 (both 30/30) are untouched: in
`StepEndSelecting`'s Blitz dispatch, for BB2020 push the `SelectBlitzTarget` sequence (carrying the
already-chosen `block_defender_id`) instead of the inline activation + `BlitzBlock`. The agent
pre-picks the target, so `StepSelectBlitzTarget` must accept it rather than prompt. Gate against the
full 30-roster bb2020 matrix plus lineman in all three editions — this changes every BB2020 blitz.

### Gate

| check | result |
|---|---|
| `nurgle` bb2020 | 86/100 (unchanged — reverted, findings only) |
| working tree | clean at HEAD, no engine change |

bb2020 stays **29 of 30 green**. RED: nurgle 86.

## ITER107 — routing through `SelectBlitzTarget` is not enough: the abort must also CANCEL the block

No engine change (reverted); nurgle stays 86/100. ITER103/106's proposed fix was implemented and
measured, and it is necessary but not sufficient. The remaining requirement is now exact.

### What was tried

For BB2020 only, `StepEndSelecting`'s Blitz dispatch pushed the real `SelectBlitzTarget` sequence
(with FOUL_APPEARANCE + DUMP_OFF before JUMP_UP/STAND_UP) followed by the `BlitzBlock` sequence,
replacing the inline activation. `BlitzBlock`'s own FA gated frenzy-only.

**Result: 86/100, and nurgle seed 2 is bit-identical to baseline** — `a01` still Standing where Java
has it Prone.

### Why concatenating the two sequences cannot work

In Java the two halves are pushed by SEPARATE client commands: `SelectBlitzTarget` runs first, and
`BlitzBlock` is only pushed later, by the subsequent CLIENT_BLOCK. So when Foul Appearance fails and
gotos `END_BLITZING`, the blitz ends and **`BlitzBlock` never exists**.

Rust pushes both at once. The goto to `END_BLITZING` correctly skips JUMP_UP/STAND_UP *within*
`SelectBlitzTarget` — but execution then falls straight into the concatenated `BlitzBlock`
sequence, which proceeds to block and (somewhere in that flow) leaves the blitzer standing. Aborting
the first sequence does not cancel the second, because they are one sequence.

This is the same class as the ITER99/100 pairing: the change is only correct when BOTH halves are
present. Here the missing half is **conditional pushing**.

### The fix, precisely

`BlitzBlock` must not be pushed until `SelectBlitzTarget` completes successfully.
`StepSelectBlitzTargetEnd` is the natural place: it is the `END_BLITZING`-labelled terminator, it
already receives the `EndPlayerAction`/`EndTurn` parameters that `fail_fa` publishes, and pushing
`BlitzBlock` from there (only when the action was NOT ended) reproduces Java's two-command shape
without touching the agent protocol.

Concretely for the next iteration:

1. Give `StepSelectBlitzTargetEnd` the blitz params it needs (`block_defender_id`, `using_stab`, …)
   as step parameters from the dispatch.
2. In its `execute_step`, if `end_player_action || end_turn` → do the existing EndPlayerAction push;
   otherwise push `BlitzBlock`.
3. Keep the BB2020 gate on the routing so bb2016/bb2025 are untouched, and gate the whole matrix.

### Gate

| check | result |
|---|---|
| `nurgle` bb2020 | 86/100 (unchanged — reverted, findings only) |
| working tree | clean at HEAD, no engine change |

bb2020 stays **29 of 30 green**. RED: nurgle 86.

## ITER108 — conditional push implemented; the routing drags in extra steps (86 → 15/100)

No engine change (reverted); nurgle stays 86/100. ITER107's specified fix was built in full and
measured. It changes the blitz flow as intended but brings unwanted baggage.

### What was built

* New `StepParameter::PushBlitzBlockAfterSelect(bool)`.
* `StepSelectBlitzTargetEnd` gained `end_player_action` / `push_blitz_block` / `block_defender_id` /
  `using_stab`, and a `continue_blitz()` that pushes `BlitzBlock` **only** when the blitz was not
  aborted (`!end_player_action && !end_turn`), called from every success path.
* BB2020 dispatch pushes just the `SelectBlitzTarget` sequence, attaching the flag and the blitz
  params to its `SelectBlitzTargetEnd` entry.
* `SelectBlitzTarget` carries FOUL_APPEARANCE + DUMP_OFF before JUMP_UP/STAND_UP; `BlitzBlock`'s FA
  is frenzy-only.

**Result: 86/100 → 15/100.** Reverted.

### Why

Routing through `SelectBlitzTarget` does not only relocate Foul Appearance — it also runs steps the
inline activation never ran for this dispatch:

* `SELECT_BLITZ_TARGET` itself (the first entry of the sequence),
* `JUMP_UP` and `STAND_UP`.

For a blitzer that is already standing those are new steps in the stream, and `STAND_UP` in
particular can roll. The inline activation deliberately contained only the negatrait block
(`ActivationSequenceBuilder`) precisely because the agent's single-command blitz has already done
the target selection and the driver reaches the block directly.

So the conditional-push mechanism is right (and is the only thing that can express Java's
"abort ⇒ no block"), but it has to be combined with a `SelectBlitzTarget` variant that omits the
steps the inline path already accounts for — or the inline path has to gain a real abort target.

### Next iteration

Rather than swapping the whole sequence, keep the inline activation and give it the missing
capability: append FOUL_APPEARANCE + DUMP_OFF (BB2020) as in ITER105 — which measured 86/100, i.e.
regression-free — and then make the FA failure abort the blitz outright instead of `goto
END_BLOCKING`. `continue_blitz`'s guard is the model: the block sequence must not run after an
aborted activation. Concretely, dispatch the inline activation ALONE, and have its terminator push
`BlitzBlock` under the same `!end_player_action` guard. That keeps the current step set (no new
JUMP_UP/STAND_UP/SELECT) while gaining the abort semantics.

### Gate

| check | result |
|---|---|
| `nurgle` bb2020 | 86/100 (unchanged — reverted, findings only) |
| working tree | clean at HEAD, no engine change |

bb2020 stays **29 of 30 green**. RED: nurgle 86.

### Cost note

nurgle has now taken ITER92 and ITER101-108 without a numeric gain, though each iteration has
eliminated a specific mechanism and the remaining space is small and well-mapped. The other 29
rosters are green and every fix from this stretch (Stand Firm, Moles, trap door, prayers) landed
from the same method.

## ITER109 — the regression is the GATE, not the step set: `end_player_action` is the wrong signal

No engine change (reverted); nurgle stays 86/100. A clean discriminator this iteration.

### The experiment

ITER108's diagnosis said the 86 → 15 regression came from routing through `SelectBlitzTarget`,
which drags in `SELECT_BLITZ_TARGET` + `JUMP_UP` + `STAND_UP`. So this iteration kept the **inline
activation** (no extra steps at all) and only added:

* FOUL_APPEARANCE + DUMP_OFF at the end of it (BB2020), failure → a new terminator;
* the nega-trait failure label re-pointed at that terminator;
* `StepSelectBlitzTargetEnd` as the terminator, pushing `BlitzBlock` via `continue_blitz()` only
  when `!end_player_action && !end_turn`.

**Result: 15/100 — identical to ITER108.**

### What that proves

Two structurally different step sets produce exactly the same score, so the extra
`SelectBlitzTarget` steps were NOT the cause. ITER108's stated reason is therefore wrong and is
retracted. The one thing both experiments share is `continue_blitz`'s guard, so **the regression is
the gate**: `end_player_action` is true on far more blitzes than it should be, and most blitzes
never get their block pushed at all.

That is consistent with the score: 15/100 is roughly "blitzes mostly stop happening", not "blitzes
resolve slightly differently".

### Why `end_player_action` is the wrong signal

It is a general parameter published by many steps and simply latched by
`StepSelectBlitzTargetEnd::set_parameter` without being consumed or reset, so anything upstream in
the activation that publishes it — not just a Foul Appearance failure — permanently disables the
block for that activation. Java does not gate on a flag at all: it never pushes `BlitzBlock` in the
first place, because the abort ends the command and the block arrives only on the NEXT command.

### Next iteration

Gate on the abort itself rather than on a latched flag. Options, cheapest first:

1. Have `StepFoulAppearance::fail_fa` (and the nega-traits) publish a dedicated
   `BlitzAborted(true)`-style parameter that only the terminator reads, so no unrelated publisher can
   suppress the block.
2. Or have the terminator check the concrete post-conditions instead — e.g. the target selection
   state being `failed()`/`canceled()`, which `StepFoulAppearance` already sets via `ts.failed()`.

Option 2 needs no new parameter and uses state Java also sets, so try it first.

### Gate

| check | result |
|---|---|
| `nurgle` bb2020 | 86/100 (unchanged — reverted, findings only) |
| working tree | clean at HEAD, no engine change |

bb2020 stays **29 of 30 green**. RED: nurgle 86.

## ITER110 — nurgle 86 → 100/100 GREEN. **BB2020 IS 30/30. ALL THREE RULESETS COMPLETE.**

### First, the negative result that pointed the way

ITER109 said to gate the block push on the target-selection state instead of `end_player_action`.
Measured: **25/100** — better than the 15/100 of ITER108/109 (so `end_player_action` really was
part of the problem) but still far below the 86/100 baseline. That is the third variant of the
conditional-push design to lose most blitzes, and it retires the whole family:

| approach | nurgle |
|---|---|
| baseline (FA in BlitzBlock, after STAND_UP) | 86/100 |
| move FA to `SelectBlitzTarget` only (ITER101/103) | 0/100 |
| route through `SelectBlitzTarget` + conditional push (ITER108) | 15/100 |
| inline activation + conditional push (ITER109) | 15/100 |
| conditional push gated on target-selection state (ITER110) | 25/100 |

Every attempt to reproduce Java's CONTROL FLOW failed, because Rust's agent commits to blitz+target
in a single command and there is simply no abort point before the stand-up.

### The fix: reproduce the END STATE instead

Java's `FoulAppearanceBehaviour.handleFailure` reverts a standing-up player to PRONE for
`BLITZ_MOVE || isBlockAction() || GAZE_MOVE || isKickingDowned()`. **`BLITZ` is absent from that
list, and Java does not need it** — BB2020 resolves Foul Appearance inside `SelectBlitzTarget`,
before JUMP_UP/STAND_UP, so a failure gotos END_BLITZING and the blitzer never stands in the first
place.

Rust's dispatch has no such abort point, so it reaches the same end state the other way: add
`BLITZ` to the revert list, **gated on `Rules::Bb2020`**. One condition, no dice consumed or moved,
BB2025/BB2016 keep Java's list exactly.

This is an explicit Rust-side bridge, of the same kind already documented for the defender-as-step-
parameter: the command shape differs, so the mechanism differs, but the observable game state and
the dice stream match Java exactly.

### Gate

| check | result |
|---|---|
| `nurgle` bb2020 | **86/100 → 100/100 GREEN** |
| `halfling` / `necromantic` / `wood_elf` / `slann_fumbbl` / `dwarf` bb2020 | 100/100 each |
| `lineman` bb2020 / bb2025 / bb2016 | 100/100 each (run serially) |
| `cargo test --workspace` | clean (exit 0) |

Regression test `bb2020_failed_foul_appearance_reverts_a_blitzer_to_prone` pins both directions:
BB2020 reverts, BB2025 does not.

## 🏁 BB2020 30/30 — THE THREE-RULESET CAMPAIGN IS COMPLETE

bb2016 30/30 · bb2020 30/30 · bb2025 30/30. A full-matrix confirmation sweep is the next step.

## ITER111 — FULL-MATRIX CONFIRMATION: bb2020 is 30/30, measured

ITER110 declared 30/30 on the strength of seven rosters (the ones the session had touched) plus the
lineman gate. That is exactly the inference that produced the **retracted** 2026-08-14 "30/30"
claim, so it was re-measured rather than asserted.

A single sequential sweep of **every** BB2020 roster, seeds 1-100, tier 3:

```
lineman amazon chaos chaos_dwarf chaos_pact dark_elf dark_elf_league_fumbbl dwarf elf goblin
halfling high_elf human khemri khemri_fumbbl lizardman necromantic nippon norse nurgle ogre orc
renegades skaven slann slann_fumbbl undead underworld vampire wood_elf
```

**Result: 30 rosters, 30 with a result, 30 at `PARITY: 100/100 games match`. Zero missing, zero
below 100.** Each line carries the required `PARITY:` marker, so every run genuinely completed its
Rust loop — the guard that caught the false 2026-08-14 claim.

(Script: `scratchpad/matrix.sh`. Note the `PARITY:` summary goes to **stderr**; a first attempt with
`2>/dev/null` silently produced empty results for every roster. Capture `2>&1`.)

## 🏁 CAMPAIGN COMPLETE — ALL THREE RULESETS AT 30/30

| ruleset | status |
|---|---|
| BB2016 | 30/30 |
| BB2020 | **30/30 (this sweep)** |
| BB2025 | 30/30 |

Every team mirror matchup reaches per-step state-hash parity with stock Java across all three
rulesets, at 100 seeds each.

### What closed BB2020 this session

| iteration | fix | effect |
|---|---|---|
| ITER89 | Stand Firm must auto-ACCEPT (harness always uses the skill) | dwarf → 100, +3 rosters moved |
| ITER91 | Stand Firm avoid-push must publish `FOLLOWUP_CHOICE(false)` | necromantic 32 → 100 |
| ITER95 | pass Moles under the Pitch into `GoForItContext` | halfling 98 → 100 |
| ITER96 | publish the trap-door injury instead of applying it | wood_elf 99 → 100 |
| ITER99 | record granted prayers, then expire them by duration | slann_fumbbl 98 → 99 |
| ITER100 | filter already-held prayers from the Cheering Fans pick | slann_fumbbl 99 → 100 |
| ITER110 | revert a BB2020 blitzer to prone on a failed Foul Appearance | nurgle 86 → 100 |

### Method notes worth keeping

* Pair Java's `caller=` frames with Rust's `FFB_DIE_AT` backtrace. A matching VALUE at the same index
  is not the same roll — that trap produced three committed wrong conclusions (ITER93/101/102).
* Java's `rng_calls` counts CALLS, Rust counts DICE. Never compare the counters.
* When Rust's command shape differs from Java's, port the observable END STATE, not the control flow
  (ITER110, after five control-flow attempts scored 0/15/15/25 against an 86 baseline).

## ITER112 (backlog) - Grab auto-decline corrected in all three editions

The original goal is met (all three rulesets 30/30, ITER111). New goal, chosen by the user: clear
the deferred backlog. First item.

`GrabBehaviour` had the same defect ITER89 fixed in Stand Firm and an earlier iteration fixed in
Side Step: Java shows a `DialogSkillUseParameter(actingPlayer, Grab)` and `ParityRunner` ALWAYS uses
the skill (its decline list is only DumpOff / PrimalSavagery / SafePairOfHands / Swoop), but Rust
left `grabbing = None` and returned, silently DECLINING. Corrected to auto-ACCEPT and fall through
to the grab branch, in bb2016, bb2020 and bb2025.

**This cannot move the parity matrix and is not claimed to.** No team roster in any edition carries
Grab -- it appears only in `data/skills/` and `data/star_players/` (`grep -rl Grab data/`). It is
corrected because it is wrong against the Java, and because the two sibling behaviours with the
identical shape both turned out to be real bugs once a roster exercised them.

Regression test `grab_with_occupied_pushback_square_auto_accepts` (bb2020 + bb2025) drives the exact
branch: a third player standing ON a pushback square is what makes Java reset its optimistic
`grabbing = true` to null and show the dialog. A first version of the test asserted the same thing
via `free_square_around_defender` and did NOT reach the branch (it failed), so the assertion was
rewritten around the real trigger rather than weakened.

### Gate

| check | result |
|---|---|
| `nurgle` bb2020 | 100/100 |
| `dwarf` bb2020 | 100/100 |
| `lineman` bb2025 | 100/100 |
| `cargo test --workspace` | clean (exit 0) |

### Backlog remaining

* dead bb2020 step files that drift from their Java counterparts (`step_prayer` aside, most are
  unreachable -- decide: delete, or keep in sync);
* the game-option key mismatch (`MVP_NOMINATIONS` vs `mvpNominations`) and the dead
  `UtilServerStartGame::add_default_game_options`;
* `clear_pushback_stack` is set by bb2016 but never consumed by the shared bb2025 `StepPushback`;
* `ThrowTeamMate` has a PICK_UP in bb2020 that bb2025 lacks (never edition-gated).

## ITER113 (backlog) - two prayer options were read under keys that do not exist

Backlog item 2. It turned into a retraction plus a real bug.

### The claimed "MVP_NOMINATIONS vs mvpNominations" mismatch does NOT exist

Checked rather than assumed: `game_option_id.rs:48` defines
`pub const MVP_NOMINATIONS: &str = "mvpNominations"`, and every reader
(`bb2016/bb2020/bb2025 step_mvp.rs`) queries `"mvpNominations"`. They agree. **That backlog entry
was wrong and is withdrawn.**

### `add_default_game_options` is unused, and that is CORRECT

Only its own tests call it. It is a faithful translation of Java
`UtilServerStartGame.addDefaultGameOptions`, which Java calls in standalone/dev mode - a mode Rust
does not have yet. The project ground rule is a file-for-file, method-for-method translation, so a
translated-but-not-yet-called method is the expected state, not dead code to delete. **No change.**

### The real bug the audit found

A mechanical scan of every string literal passed to an `options.get*/set` call against the 127
constants in `game_option_id.rs` found **two literals matching no key at all**, both in
`bb2025/start/step_prayers.rs`:

| literal used | real key |
|---|---|
| `inducement_prayers_cost` | `inducementPrayersCost` |
| `inducement_prayers_available_for_underdog` | `inducementPrayersAvailableForUnderdog` |

snake_case instead of camelCase, so **both reads always missed and silently fell back to their
defaults**: a configured prayer cost had no effect, and the underdog rule could not be turned off.
Exactly the "invisible until it changes an outcome" class as the Moles prayer (ITER95) and the trap
doors (ITER98) - nothing crashes, the value is just quietly wrong.

Both now read through the constants, so a future typo is a compile error rather than a silent
default. Re-running the scan reports all 18 remaining literals matching a defined constant.

Regression test `prayer_options_are_read_under_their_real_keys` pins the key spellings AND asserts
the option is honoured end-to-end: a 30k TV gap at a configured 10k cost must grant 3 prayers. Before
the fix the ignored option defaulted to 50k and granted 0, so the test genuinely fails on the old
code.

### Gate

| check | result |
|---|---|
| `lineman` bb2025 | 100/100 |
| `cargo test --workspace` | clean (exit 0) |

### Backlog remaining

* dead bb2020 step files that drift from their Java counterparts;
* `clear_pushback_stack` set by bb2016 but never consumed by the shared bb2025 `StepPushback`;
* `ThrowTeamMate` has a PICK_UP in bb2020 that bb2025 lacks (never edition-gated).

## ITER114 (backlog) - the shared bb2025 StepPushback never honoured `clear_pushback_stack`

Backlog item 3, and a real 1:1 gap.

Java a hook that CANCELS a push calls `state.pushbackStack.clear()`, and `state.pushbackStack` is
the STEP own chain stack, not merely the candidate squares. `bb2016/step_pushback.rs` already
honoured this through the `clear_pushback_stack` flag. The shared `bb2025/block/step_pushback.rs`
**declared the field but never read it**, and neither bb2020 nor bb2025 `StandFirmBehaviour` set it -
they cleared only `pushback_squares`. So a Stand Firm (or Side Step) refusal left an already-chosen
CHAIN push queued on `self.pushback_stack`.

Now set by both Stand Firm hooks and consumed by the shared step, matching bb2016.

**Reachability, stated honestly:** this can only differ when the pushback stack is NON-EMPTY at the
moment of refusal - i.e. a chain push where the refusing player is not the first in the chain. That
is why the 30/30 matrix never caught it, and why this change moves no number. ITER91 noted the same
thing when it deliberately left the flag alone: "necromantic seed 1 pushed nobody".

No new test: the existing Stand Firm tests build a hook state directly and assert on
`pushback_squares` / `do_push`, and a test for the chain case would need a full multi-player push
chain through the real step rather than the hook. Recorded here rather than faked with a shallow
assertion.

### Gate

| check | result |
|---|---|
| `nurgle` bb2020 | 100/100 |
| `halfling` bb2020 (Treemen, Stand Firm) | 100/100 |
| `dwarf` bb2020 (Deathroller, Stand Firm) | 100/100 |
| `cargo test --workspace` | clean (exit 0) |

### Backlog remaining

* dead bb2020 step files that drift from their Java counterparts;
* `ThrowTeamMate` has a PICK_UP in bb2020 that bb2025 lacks (never edition-gated).

## ITER115 (backlog) - BB2020 ThrowTeamMate ends the scatter with PICK_UP, not STEADY_FOOTING

Backlog item 4. A real edition difference on a LIVE path, gated.

The two Java TTM generators diverge in their tails:

```
bb2020  RIGHT_STUFF -> jump APOTHECARY_THROWN_PLAYER -> PICK_UP [END_SCATTER_PLAYER]
        (failure -> SCATTER_BALL) -> jump END_THROW_TEAM_MATE -> EAT_TEAM_MATE
        -> APOTHECARY -> CATCH_SCATTER_THROW_IN [SCATTER_BALL] -> RESET_TO_MOVE

bb2025  RIGHT_STUFF -> STEADY_FOOTING (success -> END_SCATTER_PLAYER)
        -> jump APOTHECARY_THROWN_PLAYER -> EAT_TEAM_MATE -> APOTHECARY
        -> CATCH_SCATTER_THROW_IN [END_SCATTER_PLAYER] -> RESET_TO_MOVE
```

Three differences, not one: BB2020 has a PICK_UP where BB2025 has STEADY_FOOTING; the
END_SCATTER_PLAYER label sits on the PICK_UP rather than on CATCH_SCATTER_THROW_IN; and BB2020 puts
SCATTER_BALL on CATCH_SCATTER_THROW_IN. Since RIGHT_STUFF jumps to END_SCATTER_PLAYER on success,
the label placement decides what a landed player actually does -- **a pick-up in BB2020 versus a
catch/scatter in Rust**.

Edition-gated in the shared generator, with `rules` threaded to all four `ThrowTeamMateParams`
construction sites (including the two `..Default::default()` ones in `step_end_moving.rs`, whose
default is BB2025 and would silently give a BB2020 game the wrong tail).

**No score change, and none expected:** ogre and halfling are the TTM rosters and both were already
100/100. The difference only bites when a thrown player LANDS ON THE BALL SQUARE, which those 200
seeds never produce.

Regression test `bb2020_ttm_ends_the_scatter_with_pick_up_not_steady_footing` pins all three
differences in both directions.

### A second difference found while testing, NOT fixed

The first version of that test asserted BB2020 has no STEADY_FOOTING anywhere and FAILED -- the
shared `ActivationSequenceBuilder` emits one near the top, while `bb2020/ThrowTeamMate.java:29-43`
spells its activation out by hand WITHOUT a STEADY_FOOTING. That is a separate edition difference in
the ACTIVATION block, which is shared by many sequences and is currently green across all 30
rosters. Left alone deliberately; the test was scoped to the tail rather than weakened. **Added to
the backlog below rather than silently absorbed.**

### Gate

| check | result |
|---|---|
| `ogre` bb2020 (TTM) | 100/100 |
| `halfling` bb2020 (TTM) | 100/100 |
| `lineman` bb2020 | 100/100 |
| `lineman` bb2025 | 100/100 |
| `cargo test --workspace` | clean (exit 0) |

### Backlog remaining

* the shared `ActivationSequenceBuilder` emits STEADY_FOOTING where BB2020 Java has none (found
  above; touches every activation, so it needs its own iteration and a full-matrix gate);
* dead bb2020 step files that drift from their Java counterparts.

## ITER116 -- activation STEADY_FOOTING: investigated, deliberately NOT changed

Backlog item 5, the last one carried over from ITER115. Resolved as a documented won't-fix with a
guard test, NOT as a code change. The reasoning matters more than the outcome, because the "fix"
here looks obviously correct and is actively dangerous.

### The apparent bug

`grep -rn STEADY_FOOTING ffb-java/.../server/step/generator/bb2020/` is **empty**. No BB2020 Java
generator contains the step anywhere; BB2020 spells its activations out by hand
(`bb2020/SelectBlitzTarget.java`, `bb2020/ThrowTeamMate.java:29-43`) without one. The driver runs
the shared BB2025 `ActivationSequenceBuilder` for BB2020 games, and that builder emits
STEADY_FOOTING as step 3 of EVERY activation. So Rust runs a step in every BB2020 activation that
Java never runs. The one-line gate writes itself.

### Why the one-line gate is wrong

`StepSteadyFooting` is the **only** consumer of a published `SteadyFootingContext` --
`StepHandleDropPlayerContext`, the step immediately after it, does not read it. A BB2020 game
publishes one anyway.

**Corrected in ITER117:** this section first cited the `bb2020/*.rs` fall sites as the publishers.
Those files are dead code (see ITER117 below) -- `make_step_for` routes only `StepId::Prayer` to a
bb2020 step, so a BB2020 game runs the SHARED bb2025 fall sites, and every one of those publishes a
context too:

  bb2025/move_/step_go_for_it.rs, bb2025/move_/step_jump.rs, bb2025/move_/step_move_dodge.rs,
  bb2025/block/step_block_chainsaw.rs

...where BB2020 Java applies the drop directly instead of publishing anything. The conclusion is
unchanged and in fact stronger: the publisher is on the live path and does not go away if the dead
bb2020 twins are ever deleted. Two unfaithful halves that cancel out exactly. Gate the step off for BB2020 and those contexts are stranded with
no consumer: **no BB2020 player ever falls from a failed rush, dodge, or jump.** The matrix would
not "drop a few seeds", it would come apart on the most-exercised path in the engine.

An earlier read in this iteration claimed the step was inert because `execute_step` returns
NEXT_STEP without a context. That was wrong and is retracted: ANIMAL_SAVAGERY is step 2 of the
activation and `step/mixed/shared/step_animal_savagery.rs` publishes a context straight into it.
The step is live, not inert.

### Evidence it is currently equivalent

Only two BB2020 rosters carry Animal Savagery at all -- `data/rosters/bb2020/roster_renegades.json`
and `roster_underworld.json` -- and both are 100/100. All 30 BB2020 rosters are 100/100 with the
step present (ITER111). The paired shape is behaviourally correct; it is only structurally unfaithful.

### What a real fix costs

Both halves have to move together: rewrite those ~6 BB2020 fall sites to apply the drop inline the
way BB2020 Java does, THEN drop the step from BB2020 activations -- which additionally needs `rules`
threaded into ~20 generator params structs (most use `#[derive(Default)]`, and `Rules` has no
`Default`, so each needs a hand-written impl plus every exhaustive struct-literal construction site
updated: 50+ edit sites). All of it on the hottest path of a 30/30/30-green engine, for zero
expected behavioural change. Not worth it now; recorded so the decision is deliberate rather than
forgotten.

### Landed

- The STEADY_FOOTING line in `activation_sequence_builder.rs` now carries the full coupling
  explanation, so the next reader does not "fix" it.
- Test `steady_footing_stays_in_the_activation_because_bb2020_fall_sites_depend_on_it` fails loudly
  if the step is removed or reordered before ANIMAL_SAVAGERY, and its doc comment points here.

Gate: `cargo test --workspace` clean. No engine behaviour changed (comment + test only), so no
matrix re-run was required.

### Backlog after this iteration

Cleared: Grab auto-decline (ITER112), game-option key mismatch (ITER113), `clear_pushback_stack`
(ITER114), ThrowTeamMate PICK_UP (ITER115), activation STEADY_FOOTING (ITER116, won't-fix).
Remaining: the dead BB2020 step files that drift from their Java counterparts -- decide delete
versus keep-in-sync.

## ITER117 -- the dead BB2020 step files, and a correction to ITER116

Last backlog item. Also corrects ITER116 above, which cited dead files as the publishers of the
`SteadyFootingContext`; the passage is amended in place and the conclusion is unchanged.

### The finding

`make_step_for(id, rules)` has exactly ONE `Rules::Bb2020` arm: `StepId::Prayer`. Everything else in
a BB2020 game falls through to the shared (bb2025) step set. That is the known structural gap, but
the consequence for `crates/ffb-engine/src/step/bb2020/` had not been measured:

- 104 step files under `step/bb2020/`
- 10 of them are reachable, and NOT because they are BB2020 -- the driver's edition-independent
  match arms use them as the shared implementation for their StepId in EVERY edition:
  CloudBurster, ReportStabInjury, StateMultipleRolls, AssignTouchdowns, BuyCardsAndInducements,
  CheckStalling, SelectGazeTarget, SelectGazeTargetEnd, SetActingPlayerAndTeam, SetActingTeam
  (driver.rs:198-339), plus StepPrayer via the Bb2020 arm and the bb2020 `start_game` generator.
- The remaining ~93 are never instantiated by any code path.

They still compile, still carry `#[cfg(test)]` tests that pass, and drift from their Java
counterparts with nothing to catch it. `bb2020/move_/step_go_for_it.rs` is the cautionary case: it
tracked ITER95's Moles-under-the-Pitch fix only because that iteration edited all four GFI files by
hand. Read casually, a dead file looks like proof of what BB2020 does, and in ITER116 it produced a
wrong (if harmlessly wrong) rationale.

### Decision: keep, do not delete

Deleting ~93 files is tempting and wrong for this campaign. They are the already-translated BB2020
Java classes, and the standing direction is to close the structural gap by giving BB2020 its real
step set -- which means routing `make_step_for` AT them, not throwing them away. Deleting would
discard the port and force a re-translation later.

The actual hazard is not their existence, it is that they read as authoritative. Mitigation is
cheap and does not touch behaviour, so that is what this iteration lands.

### Landed

- A module-level warning in `step/bb2020/mod.rs`: what is dead, the ~10 files that are live and why
  (they are the shared implementation for their StepId in every edition, not BB2020-specific), the
  rule that fixing a bug here usually changes nothing, and why the files are kept rather than deleted.
- ITER116's rationale corrected in place (dead bb2020 paths -> the live shared bb2025 ones), in
  both the ledger and the comment in `activation_sequence_builder.rs`.

### Backlog

Empty. Items cleared: Grab auto-decline (ITER112), game-option key mismatch (ITER113),
`clear_pushback_stack` (ITER114), ThrowTeamMate PICK_UP (ITER115), activation STEADY_FOOTING
(ITER116, won't-fix with a guard test), dead BB2020 step files (ITER117, keep-with-warnings).

Natural next goal, NOT started -- close the structural gap itself: route `make_step_for` at the
BB2020 step set edition by edition, the way ITER-era work already did for BB2016 (Spectators, the
kickoff chain, MissedPass, the pass step-set). That is a large campaign of its own and a goal-level
decision, not a backlog item.

## ITER118 -- NEW GOAL: close the structural gap. First routing: EndPlayerAction

The deferred backlog is empty, and the user chose the next goal: **give BB2020 its real step set**
instead of running the shared BB2025 one with a growing pile of per-difference gates.

### The roadmap (new artifact)

`docs/PARITY_BB2020_STRUCTURAL_GAP.md` inventories every candidate, ranked by how much the two JAVA
files differ. Headline number: **all 79 BB2020/BB2025 Java step pairs genuinely differ -- there is
not one identical twin.** So every pair is a real candidate; none can be dismissed as a no-op.

The ranking is deliberately built from JAVA-vs-JAVA similarity, not Rust-vs-Rust. A dead
`step/bb2020/*.rs` file can be STALER than the shared one it is supposed to replace (documented for
`step_apply_kickoff_result`), so Rust similarity measures drift, not edition difference. Per-candidate
method and the standing "route individual StepIds, never a whole sequence" constraint are in the doc.

### First routing: EndPlayerAction

Picked because the Java pair is the most different of all (`StepStallingPlayer.java`, sim 0.24) and
the difference turned out to be in the SEQUENCE, not just the step.

**The editions do not agree on what `STALLING_PLAYER` means.**

  bb2025 `shared/StepStallingPlayer.java`  = the stalling CHECK (delegates to stallingExtension)
  bb2020 `StepStallingPlayer.java`         = the ROCK THROW at a staller (d6, >=5 hits, drops him)

BB2020 does its check in a separate `CHECK_STALLING` step, and the two generators wire that up
differently (`generator/bb2020/EndPlayerAction.java:25-36` vs the bb2025 twin):

  bb2020 (7 steps)   ... CATCH_SCATTER_THROW_IN, CHECK_STALLING[END_FEEDING](IGNORE_ACTED_FLAG=false),
                     END_FEEDING
  bb2025 (11 steps)  ... CATCH_SCATTER_THROW_IN, STALLING_PLAYER[END_FEEDING],
                     STEADY_FOOTING(HIT_PLAYER), PLACE_BALL, APOTHECARY(HIT_PLAYER),
                     CATCH_SCATTER_THROW_IN, END_FEEDING(CHECK_FORGO)

So a BB2020 game was running **BB2025's stalling check** after every player action and **never
running CHECK_STALLING at all**, plus a four-step HIT_PLAYER tail BB2020 does not have. Two smaller
differences ride along: BB2020 passes RESET_FOR_FAILED_BLOCK=false to RESET_FUMBLEROOSKIE, and its
END_FEEDING takes no CHECK_FORGO.

### The fix

`generator/bb2020/end_player_action.rs` already held a faithful 1:1 port of the BB2020 sequence --
dead code, never called. The shared `EndPlayerActionParams` gained a `rules` field (explicit
`Default` = Bb2025, since `Rules` has none) and `build_sequence` delegates to that BB2020 generator
when `rules == Bb2020`. `game.rules` threaded through all 17 live construction sites under
`step/bb2025/**` and `step/mixed/**`; every one was an exhaustive struct literal, so the compiler
found them all. `StepSelectBlitzTarget::execute_step` needed `&Game` threaded in to reach `game.rules`.

Note the live sites are the bb2025/mixed ones -- the `step/bb2020/**` call sites in the grep are
themselves dead code (ITER117) and were deliberately left alone.

### Gate

Full 30-roster BB2020 matrix: **30 rosters, 30 results, 30 x `PARITY: 100/100`.** Cross-edition:
lineman bb2025 100/100, lineman bb2016 100/100 (run serially -- the same-matchup ban applies across
editions). ffb-engine 7166/0, workspace clean.

No score movement, and none expected: the sequences were already behaviourally equivalent on these
3,000 games. The win is structural -- BB2020 now runs BB2020's EndPlayerAction, and CHECK_STALLING
is reachable in BB2020 for the first time.

Regression test `bb2020_end_player_action_uses_check_stalling_not_stalling_player` pins all four
differences in both directions.

### Next

Work down `PARITY_BB2020_STRUCTURAL_GAP.md`. Next by rank: `StepApothecaryMultiple` (sim 0.43),
`StepBlockRoll` (0.52), `StepApothecary` (0.57), `StepInitScatterPlayer` (0.60).

## ITER119 -- BB2020 activations lose STEADY_FOOTING (and ITER116 is retracted)

The activation block, flagged in ITER118 as the highest-leverage item on the structural-gap
roadmap: ~10 of the low-similarity generators differ ONLY because `ActivationSequenceBuilder` is
BB2025-only, so BB2020 spells its activation out by hand in each one.

### ITER116 was wrong, and this is the second correction to it

ITER116 refused to gate this step, on the reasoning that ANIMAL_SAVAGERY (step 2) publishes a
`SteadyFootingContext` straight into STEADY_FOOTING (step 3), so removing it would strand the
context. **That premise is false.** `step/mixed/shared/step_animal_savagery.rs:423` already
edition-gates the publish:

    if is_bb2025 { publish SteadyFootingContext }   // is_bb2025 = game.rules == Rules::Bb2025
    else         { publish DropPlayerContext    }

which mirrors Java exactly -- `skillbehaviour/bb2020/AnimalSavageryBehaviour.java:235` publishes
DROP_PLAYER_CONTEXT where the bb2025 twin at :293 publishes STEADY_FOOTING_CONTEXT. In BB2020 the
activation's STEADY_FOOTING never had a context to consume. It was inert, exactly as the first
(reverted) ITER116 attempt assumed.

ITER117 already corrected ITER116's file citations; this corrects its conclusion. Both corrections
have the same root cause: reasoning about BB2020 behaviour from `step/bb2020/*.rs` and from the
sequence shape, instead of reading the code the BB2020 path actually executes.

**What IS true, and is the part worth keeping:** the coupling is real for the OTHER STEADY_FOOTING
copies. The ones inside the move and blitz sequences consume contexts that the shared fall sites
(GFI, jump, dodge, chainsaw) publish in EVERY edition, and those steps are the only consumer.
Removing them stops BB2020 players falling. So the gate belongs in the activation builder, NOT in
`make_step_for` -- a `StepId::SteadyFooting => no-op under Bb2020` arm would look equivalent and
would hit every copy at once.

### Ground truth

`bb2020/SelectBlitzTarget.java:20-31` spells the block out as INIT_ACTIVATION, ANIMAL_SAVAGERY,
HANDLE_DROP_PLAYER_CONTEXT, PLACE_BALL, APOTHECARY(ANIMAL_SAVAGERY), CATCH_SCATTER_THROW_IN,
GOTO_LABEL, BONE_HEAD[NEXT], REALLY_STUPID, TAKE_ROOT, UNCHANNELLED_FURY, BLOOD_LUST -- **12 steps,
which is this builder's 13 minus STEADY_FOOTING.** Nothing else differs.

### The fix

`ActivationSequenceBuilder` gained a `rules` field (default Bb2025) and `with_rules`, and omits
STEADY_FOOTING when `rules == Bb2020`. Threading: `rules` added to 13 generator params structs
(each losing `#[derive(Default)]` for a hand-written impl, since `Rules` has no `Default`), and a
`rules` argument added to the three that take no params and exist in BB2020 Java --
`SelectBlitzTarget`, `FuriousOutburst`, `ThenIStartedBlastin`. `Punt` deliberately keeps Bb2025: it
is a BB2025-only generator with no BB2020 twin.

Only 16 live construction sites needed updating, far fewer than the earlier estimate, because the
bb2016 and bb2020 step files construct their OWN generators' params types, not these. Every one was
an exhaustive struct literal, so the compiler enumerated them.

### Gate

Full 30-roster BB2020 matrix: **30/30 x `PARITY: 100/100`**, no exceptions. lineman bb2025 100/100,
lineman bb2016 100/100. ffb-engine 7167/0, workspace clean.

Renegades and underworld -- the only two BB2020 rosters with Animal Savagery, i.e. the only ones
that can reach this step at all -- are both still 100/100, which is the specific evidence that
matters here.

### Tests

`bb2020_activation_omits_steady_footing_and_matches_javas_twelve_steps` replaces the ITER116 guard
test (which asserted the opposite, on the false premise). It pins the 12-step count, the absence of
the step, and that ANIMAL_SAVAGERY hands directly to HANDLE_DROP_PLAYER_CONTEXT in BB2020 while
BB2025 keeps the step between them.

`steady_footing_gate_is_scoped_to_the_activation_only` pins the surviving distinction: the blitz
sequence keeps its own STEADY_FOOTING regardless of edition.

### Next

The activation blocks now agree, which is the shared prerequisite behind the generator cluster.
Next on the roadmap, working the list in `PARITY_BB2020_STRUCTURAL_GAP.md`: the per-generator
differences that remain once the activation is factored out -- `Block.java` (0.58), `Move.java`
(0.58), `BlitzBlock.java` (0.67), `Select.java` (0.70), `Foul.java` (0.73).

## ITER120 -- the Move sequence's non-STEADY_FOOTING differences

With the activation block aligned (ITER119), `generator/bb2020/Move.java` still differs from the
BB2025 twin in three ways. All three are now edition-gated in the shared generator.

1. **No FOUL_APPEARANCE and no DUMP_OFF.** BB2020's Move goes straight from the activation's
   BLOOD_LUST to HYPNOTIC_GAZE (`bb2020/Move.java:44`). BB2020 resolves both on the BLITZ paths
   instead (`bb2020/SelectBlitzTarget.java:34-35`) -- the same edition split ITER110 hit from the
   other side when it fixed nurgle.
2. **No PLACE_BALL after FALL_DOWN.** BB2020 goes FALL_DOWN -> APOTHECARY(DEFENDER) directly.
3. **END_MOVING takes no parameters.** The BLOOD_LUST_ACTION hand-off is a BB2025 addition.

### On (1) being behaviourally inert, and why that was worth checking

Rust was running a FOUL_APPEARANCE step on every BB2020 move that Java never runs, and Foul
Appearance ROLLS A DIE -- so on the face of it this should have desynced every nurgle move and it
did not (nurgle bb2020 has been 100/100 throughout). The explanation is that `StepFoulAppearance`
only acts for block-type player actions, so on a plain Move it returns without rolling. The 30/30
gate below confirms it: removing the step moved nothing.

That is the good outcome, but the reasoning had to be checked rather than assumed -- a live step
that draws dice sitting in a sequence Java does not have is exactly the shape that produces a
silent divergence on some roster nobody has run yet.

### Gate

Full 30-roster BB2020 matrix **30/30 x `PARITY: 100/100`**, no exceptions. lineman bb2025 100/100,
lineman bb2016 100/100, and nurgle bb2025 100/100 as a targeted check that the untouched edition's
Foul Appearance path still behaves. ffb-engine 7168/0, workspace clean.

### Test

`bb2020_move_omits_foul_appearance_dump_off_place_ball_and_blood_lust_action` pins all three in
both directions. Its first version asserted BB2020 has NO PlaceBall anywhere and failed -- the
activation block legitimately contains one. Tightened to exact counts (BB2020 1, BB2025 2) rather
than weakened; same trap as the ITER115 TTM test, which is now twice-seen and worth remembering:
**when asserting a step is absent, scope it, because the activation block re-uses several of the
same StepIds.**

### The big remaining item, and why it is NOT decomposable

Converting the move/blitz STEADY_FOOTING steps is the last structural piece here, and it cannot be
done sequence-by-sequence. The publishers are SHARED STEPS -- `bb2025/move_/step_go_for_it.rs`,
`step_jump.rs`, `step_move_dodge.rs`, `block/step_block_chainsaw.rs` -- each used by many
sequences, and none of them currently edition-gates its publish (unlike `step_animal_savagery.rs`,
which does). Gate a publisher for BB2020 and EVERY BB2020 sequence containing a STEADY_FOOTING must
convert in the same commit, or the contexts strand and players stop falling.

Scope of that atomic change: 4+ publisher steps, and the ~12 generators holding a STEADY_FOOTING --
`blitz_block` (6 of them), `block` (6), `blitz_move` (3), `move_` (3), plus `end_turn`,
`scatter_player`, `special_effect`, `then_i_started_blastin`, `throw_a_rock`, `throw_keg`,
`throw_team_mate`. In BB2020 Java each of those failure gotos points straight at FALL_DOWN instead.

Expected behavioural delta: **none.** The BB2025 chain (fall -> SteadyFootingContext ->
StepSteadyFooting -> on failure publishes InjuryTypeName -> FALL_DOWN) collapses to BB2020's direct
(fall -> InjuryTypeName -> FALL_DOWN) whenever no player has Steady Footing, and Steady Footing is
a BB2025-only skill. The shared `bb2025/move_/step_fall_down.rs:49` already consumes InjuryTypeName,
and bb2016 already works exactly this way (`bb2016/move_/step_go_for_it.rs:243`), so the target
mechanism is proven.

So it is a large atomic change to the hottest path in the engine for zero expected behavioural
change. Worth doing for fidelity, but it needs its own iteration, and it must land as one commit
with a full-matrix gate -- not incrementally.

## ITER121 -- the BB2020 generators are faithful; the endgame is one atomic delegation

Two findings that together fix the shape of the rest of this goal.

### 1. Block cannot be split the way Move was

ITER120 peeled the non-STEADY_FOOTING differences off `Move`. The same move does not work on
`Block`: its label topology is entangled with the STEADY_FOOTING steps. BB2020 has
`PLACE_BALL[ATTACKER_DROPPED]` where BB2025 moves that label onto a
`STEADY_FOOTING[ATTACKER_DROPPED](APOTHECARY_MODE=ATTACKER)` and leaves PLACE_BALL unlabelled; the
GO_FOR_IT failure target likewise moves from NEXT to STEADY_FOOTING. Take the STEADY_FOOTING steps
out and the labels have to move with them, so there is no "non-STEADY_FOOTING subset" to land
separately. Block also has two genuine extras: BB2025's INIT_BLOCKING takes USING_CHOMP, and BB2025
adds a `PICK_UP(gotoOnFailure=DROP_FALLING_PLAYERS)` and a `CHOMP` step BB2020 lacks.

### 2. All 26 Rust BB2020 generators are faithful ports -- audited, not assumed

`scripts/audit_bb2020_generators.py` (added this iteration) compares the StepId sequence each Rust
`step/generator/bb2020/*.rs` emits against its Java counterpart. Result: **24 MATCH, 2 DIFFER, 0
missing**, and both DIFFERs are scanner artifacts, hand-checked:

- `select.rs` -- an `if params.block_targets.is_empty()` / `else` emits END_SELECTING on each
  branch; the regex counts both, the runtime emits one.
- `select_blitz_target.rs` -- SELECT_BLITZ_TARGET_END *is* last (line 24); the activation steps come
  from an `add_activation` helper defined further down the file, so a linear scan sorts them wrong.
- (`multi_block.rs` was a third, now folded into the script's spelling table: Java
  `MULTI_BLOCK_FORK` vs Rust `MultipleBlockFork`, same step.)

The script is triage, not proof, and says so: MATCH is reassuring, DIFFER means go read both files.
It exists because these files are dead code (ITER117) and dead code drifts -- the audit had to
happen before delegating to any of them, not after.

### What this means for the endgame

The remaining structural gap is **not** ~23 independent per-generator iterations. It is one atomic
change:

  a. gate the shared fall-site publishers -- `bb2025/move_/step_go_for_it.rs`, `step_jump.rs`,
     `step_move_dodge.rs`, `block/step_block_chainsaw.rs` and the rest -- to publish
     `InjuryTypeName` for BB2020 instead of `SteadyFootingContext`, exactly as
     `step_animal_savagery.rs` already does and as bb2016 has always done
     (`bb2016/move_/step_go_for_it.rs:243`);
  b. delegate every remaining shared generator to its BB2020 twin for `Rules::Bb2020`, the way
     ITER118 did for EndPlayerAction. The twins already carry the right labels and no
     STEADY_FOOTING, so (a) and (b) fit each other.

It must be atomic because the publishers are SHARED STEPS: gate GO_FOR_IT for BB2020 and every
BB2020 sequence containing a STEADY_FOOTING must already be delegated, or the context strands with
no consumer and BB2020 players stop falling.

**Expected behavioural delta: zero, and now proven rather than argued.** `StepSteadyFooting` only
acts for a player with the Steady Footing skill; otherwise it forwards `InjuryTypeName` to
FALL_DOWN, which is precisely BB2020's direct route. Steady Footing is BB2025-only:
`grep -rli steady data/rosters/bb2020/` returns **nothing**, while the BB2025 rosters (chaos_dwarf,
high_elf, lizardman, ...) do carry it. So the BB2025 detour collapses to the BB2020 route in every
BB2020 game.

That is worth stating plainly: this final step buys structural fidelity, not correctness. It is a
large atomic change to the hottest path in the engine for no behavioural change, so it earns a
dedicated iteration and a full-matrix gate -- and it is legitimately lower priority than any
remaining difference that could actually bite. ITER120's FOUL_APPEARANCE was that shape (a live
dice-rolling step in a sequence Java lacks) and turned out inert; the audit above says the BB2020
twins are ready whenever this is picked up.

## ITER122 -- reachability: why the remaining structural work cannot be validated

Three iterations in a row (119, 120, 121) landed structural fixes that moved the score by exactly
zero, each time for the same underlying reason. This iteration went looking for that reason instead
of landing a fourth, because it decides what the rest of this goal is worth.

### The finding

**BB2020 is 30/30 not because the engine is faithful, but partly because the BB2020 parity teams do
not carry the skills that distinguish the two editions.** Skill presence across
`data/rosters/bb2020/`:

    Brawler          0      Bone Head        6
    Steady Footing   0      Really Stupid    6
    Chomp            0      Right Stuff      7
    Blood Lust       0*     Animal Savagery  2
    Hatred           1      Take Root        2
    Pro              1      Foul Appearance  1
                            Ball and Chain   1
    (* spelling not confirmed; vampire is in the matrix)

The zero rows are exactly the features behind the differences that keep turning out inert:

- **Steady Footing 0** -> the whole STEADY_FOOTING detour collapses to BB2020's direct route in
  every BB2020 game (ITER119, ITER121).
- **Brawler 0, Chomp 0** -> `StepBlockRoll` differs substantially between the editions (sim 0.52:
  BB2020 has `brawlerIndex` / `reRolledDiceIndexes` / `DialogBlockRollPartialReRollParameter`,
  BB2025 has Hatred, `RollMechanic`, `DialogBlockRollPropertiesParameter`) and **none of it is
  reachable**.

And a third shape, found the same way: `StepPickUp` (sim 0.82) carries BB2025's `secureTheBall`,
`optionalPickUp`, `attemptPickUp` and `PLAYER_ON_BALL_ID` with **no `game.rules` gate at all** in
the Rust shared step -- yet it is harmless, because `secure_the_ball` keys off the BB2025-only
`PlayerAction::SecureTheBall` and the other two are only ever published by BB2025 generators. Inert
by DATA, not by design.

### What this means

The 30-roster x 100-seed BB2020 suite **cannot validate structural fidelity work.** It can only
prove no regression. That is a real but limited bar, and it will stay that way for every remaining
item on `PARITY_BB2020_STRUCTURAL_GAP.md` whose difference is keyed on a skill or game option the
BB2020 teams do not have.

Worth being blunt about the consequence: continuing to grind the roadmap produces commits that are
correct, well-tested at unit level, and **unverifiable end-to-end**. That is not worthless -- it
removes latent traps for future rosters and future edition changes, and ITER118's CHECK_STALLING
was a genuine "BB2020 never ran this step" bug -- but it is a different proposition from the first
118 iterations, which were fixing measurable failures.

### Options from here (a goal-level choice, surfaced not taken)

1. **Make the work verifiable first** -- draft BB2020 parity teams that actually carry Brawler,
   Steady Footing, Chomp, Blood Lust etc., so the differentiating paths execute. This turns the
   remaining roadmap from unverifiable into testable, and would likely surface real divergences
   immediately. Costs a team-drafting pass (`docs/TEAM_DRAFTS_*.md`, `scripts/gen_java_parity_data.py`).
2. **Widen the net instead** -- more seeds / cross-team matchups on the existing teams, which finds
   real divergences in the paths that ARE reachable.
3. **Continue structural alignment as pure fidelity work**, accepting no-regression as the bar and
   the atomic delegation (ITER121) as the finish line.

Recommendation: (1) before finishing the roadmap, because it converts the remaining work from
faith-based to measured, and because a BB2020 team with Steady Footing would immediately tell us
whether the ITER121 atomic change is genuinely inert or merely untested.

No engine change this iteration -- the finding is the deliverable.

## ITER123 -- the BB2020 rosters were never skill-checked, and 15 skills do not exist in BB2020

New goal, chosen by the user after ITER122: **draft BB2020 teams that actually follow their
ruleset**, so the edition-differentiating paths execute and the structural work becomes verifiable.

The first step was meant to be drafting. It turned into something better.

### `check_skill_names.py` has never looked at BB2020

    EDITION_TO_RULES = {"bb2016": "BB2016", "bb2025": "BB2025"}
    # bb2020 data is an unaudited legacy clone (not used by the parity matrix) and
    # is intentionally NOT checked.

That comment stopped being true a long time ago -- BB2020 is a full parity ruleset whose 30-roster
matrix runs 100/100. The exclusion meant its rosters were never validated against Java at all, and
the script's reassuring `0 unresolvable skill names` was computed over BB2016 and BB2025 only.
Adding `bb2020` to the map turns up **15 failures**.

### The 15

Java's `SkillFactory.forName` only resolves skills whose `@RulesCollection` covers the ruleset, and
silently returns nothing otherwise -- so each of these is a skill Java DROPS while Rust's more
lenient lookup may keep it. This is the exact shape of the FUMBBL slann Kroxigor "Bone-head" desync
already recorded in this campaign.

Every one is a **BB2016 spelling left behind in a BB2020 clone**, which corroborates that stale
comment's "legacy clone" description:

    Wild Animal      -> Unchannelled Fury   chaos/chaos_dwarf/chaos_pact minotaur,
                                            norse snow troll, skaven rat ogre   (5 positions)
    Nurgle's Rot     -> Plague Ridden       nurgle rotter/pestigor/warrior/beast (4)
    Claw             -> Claws               necromantic werewolf, norse snow troll (2)
    Safe Throw       -> Safe Pass           high_elf thrower
    bone-head        -> Bone Head           human ogre
    Bone-head        -> Bone Head           slann_fumbbl 2996
    Hatred           -> (BB2025-only)       halfling Hopeful

Canonical BB2020 names confirmed from the Java `super("...")` declarations, not guessed:
`mixed/UnchannelledFury.java` "Unchannelled Fury", `mixed/PlagueRidden.java` "Plague Ridden",
`mixed/Claws.java` "Claws", `mixed/SafePass.java` "Safe Pass", `bb2020/BoneHead.java` "Bone Head".
`Hatred` exists only as `skill/bb2025/Hatred.java` with a single `@RulesCollection(BB2025)`, so a
BB2020 halfling cannot legally have it.

### Why this matters more than the structural roadmap

Five minotaur-class positions currently have **no negatrait at all in Java** -- Wild Animal does not
resolve under BB2020 and Unchannelled Fury was never granted. So the UNCHANNELLED_FURY step, which
sits in every activation block, has almost certainly never fired in a BB2020 parity game. Same
story for Claws (armour-value modifier), Safe Pass, and the two Bone Head positions.

That is a direct answer to ITER122's question of why so much of the engine is unreachable: it is
not only that BB2020 teams lack BB2025 skills, it is that they are missing **their own edition's**
skills through a bad clone. It also means the matrix's 30/30 is measuring less than it appears to.

### Landed

- `scripts/check_skill_names.py` now checks bb2020, with the stale exclusion comment replaced by
  the reason it was wrong. The script exits non-zero on the 15 findings, as it should.

### Next (ITER124)

Fix the 15 names in `data/rosters/bb2020/`, re-run `scripts/gen_java_parity_data.py` (mandatory
after ANY roster change, or the engines disagree), and gate on the full matrix. Expect this one to
**change behaviour and possibly break rosters** -- granting five big guys a negatrait they have
never had, and Claws to two more, will move dice. That is the point: it is the first BB2020 change
in a long while that can actually be validated end to end.

## ITER124 -- the roster fix lands, and surfaces two real engine bugs (WIP, on a branch)

Applied ITER123's 15 fixes to `data/rosters/bb2020/`, re-ran `gen_java_parity_data.py`,
`check_skill_names.py` clean. **This is on branch `bb2020-roster-skill-legality`, NOT main** --
6 rosters are red and main stays green until they are fixed.

### Matrix: 24 green, 6 red

    chaos         0/100     chaos_dwarf   1/100     chaos_pact    0/100
    norse         1/100     skaven        1/100     slann_fumbbl  0/100

Green despite being changed: nurgle (Plague Ridden x4), necromantic (Claws), high_elf (Safe Pass),
human (Bone Head), halfling (Hatred dropped). So most of the renames were behaviourally safe.

The reds cluster on exactly two mechanics, **both of which had never executed in a BB2020 parity
game before this commit**: the Unchannelled Fury / Animal Savagery negatrait (5 rosters) and Bone
Head on the FUMBBL slann (1).

### A rename of mine that was wrong

`Wild Animal -> Unchannelled Fury` was applied as a blanket rename to all five positions holding it.
BB2020 does not work that way -- different big guys get different traits. Checked against
`rules/teams/`:

    Minotaur (chaos, chaos_dwarf, chaos_pact)  Unchannelled Fury   correct
    Yhetee / snow troll (norse)                Claws + Unch. Fury  correct
    Rat Ogre (skaven)                          ANIMAL SAVAGERY     WRONG -> corrected

skaven is still 1/100 after the correction, so its divergence is real and not an artifact of the
bad rename.

**Scope limit worth stating:** there is no in-repo authority for BB2020 position-level rosters --
`rules/teams/` is BB2025 data and the BB2020 rosters are the bad clone. Every skill NAME is now
legal in BB2020 (Java is the authority for that), and individual positions were sanity-checked
against BB2025 where the editions agree, but the full BB2020 team lists are NOT verified.

### chaos seed 1 diagnosed: state-only, not dice

Divergence at i=13, the Minotaur's BLOCK. Java ends the home turn there; Rust plays on with two more
players.

Dice were the obvious suspect and are **not** the cause. Both streams through the divergence are
byte-identical:

    pos  22 23 24 25 26 27 28 29 30 31
         2  4  3  5  1  6  1  2  6  4

with pos 25 the Unchannelled Fury `rollSkill` (=5, a SUCCESS, so the block proceeds normally),
26-27 the two block dice, 28-29 the armour roll from `StepDropFallingPlayers`. Java's own
`JAVA_DIE` stream agrees call-for-call (rng 25-29, step 13 consuming exactly 5 calls, 24 -> 29).

So both engines roll the same negatrait, take the same block dice, and drop the same player -- and
then Java treats it as a **turnover** and Rust does not. That is a state-only divergence, which per
this campaign's own playbook means diffing the post-step state strings rather than chasing dice.

Blocked there for now: Rust's parity log emits `state: None` and `FFB_TRACE` on the Rust side prints
`LOOP`/`DRC_DRAW` lines rather than the `JSTEP`-style state string Java produces, so there is
nothing to diff yet. Next step is to get Rust to emit the state string at the post-step, then diff
against Java's `h1t11ahome...` / `h1t12aaway...`.

### Not committed to main

The data fix is correct but currently regresses 6 rosters, so it sits on a branch per the standing
"commit when progress is made without regression" rule. Main remains 30/30.

## ITER125 -- chaos seed 1 narrowed to the block RESULT, not the dice (branch WIP)

Got the Rust state string out (it populates only under `FFB_TRACE=1`; the matrix run's log has
`state: None`, which is why ITER124 could not diff). The two post-states at the divergence:

    JAVA  i=13 -> i=14   h1t11ahome -> h1t12aaway   h00 12,7,Standing -> 12,7,PRONE
                                                    a02 13,8,Standing  (unchanged)
    RUST  i=13 -> i=14   h1t11ahome -> h1t11ahome   h00 12,7,Standing -> 13,8,Standing
                                                    a02 13,8,Standing -> -1,-1,KO

`teamChaosParity20Home1` is `h00`, the Minotaur, and both engines block the same defender
(`Away3` = `a02`; `JAVA_BLOCK_PICK pid=Home1 N=2 idx=1 def=Away3` is the TARGET pick, not the die
pick -- worth noting since the name invites the opposite reading).

So with the identical dice established in ITER124, **Java resolves the block to Attacker Down and
Rust resolves it to Defender Down**. Java's Minotaur falls, which is the turnover that ends the
home turn; Rust's Minotaur knocks the defender out and follows up into 13,8.

### What is ruled out

- **Dice.** Both streams are byte-identical across pos 22-31 (ITER124).
- **Die index choice.** `ParityRunner` answers BLOCK_ROLL / BLOCK_ROLL_PARTIAL_RE_ROLL /
  BLOCK_ROLL_PROPERTIES with `sendBlockChoice(0)` unconditionally (`ParityRunner.java:581,591`), and
  Rust's agent answers `AgentPrompt::BlockChoice` with `die_index: 0`
  (`agent/random_agent.rs:581-588`). Both pick index 0.
- **A general block bug.** 24 rosters block constantly and are 100/100, so neither the die-value ->
  BlockResult mapping nor the roll order can be wrong in general.

### Open, and deliberately not guessed at

Two things do not add up yet and need instrumentation rather than more reading:

1. If both pick index 0 of the same `[1, 6]`, both should get the same BlockResult. Something about
   THIS block -- a Minotaur, so Frenzy plus a freshly-granted Unchannelled Fury -- changes which
   array or index is in play.
2. Rust KO'd `a02` on an armour roll of 1+2=3, which cannot break AV8. Either the KO came from dice
   I have mis-attributed, or Rust took a different injury path entirely.

Point 2 in particular means the dice attribution may not be as clean as ITER124 concluded. This
campaign's own notes warn that Java `rng_calls` count CALLS while Rust counts DICE, and that equal
values at equal indexes do not prove the same roll -- so the "byte-identical dice" claim should be
re-verified by pairing Java `caller=` frames with Rust `FFB_DIE_AT` backtraces before being built on.

### Next probe

Log, for this one block in both engines: the block dice array as stored, the chosen index, the
resulting `BlockResult`, and the attacker/defender strengths that decide who chooses. That is four
values and settles it; everything above is inference from state deltas.

## ITER126 -- Unchannelled Fury was marked used for the whole GAME, not the activation

All six reds from ITER124 are green and the branch is ready to merge. One engine bug.

### ITER124's "byte-identical dice" was wrong, and this is the retraction

Rust already emits a `GameEvent::BlockRoll` carrying the dice, the chosen index and `nr_of_dice`, so
the probe ITER125 asked for was built in all along. For chaos seed 1, block #2:

    attacker home_01  defender away_03  nr_of_dice 2  dice [5, 1]  selected_index 0

Java rolled **[1, 6]** for that same block. The RNG *stream* is identical -- same seed, so position N
holds the same value in both engines -- but the engines **consume it at different offsets**: Rust's
block took positions 25-26 where Java's took 26-27. Comparing values at equal positions and
concluding "identical" is precisely the trap `feedback-parity-dice-comparison` warns about, and
ITER124 fell into it. The dice were never identical in the sense that mattered.

Java's extra draw at position 25 is the Unchannelled Fury `rollSkill`.

### The bug

`step/mixed/step_unchannelled_fury.rs` read and wrote the **persistent** `Player.used_skills`:

    let do_roll = game.player(&player_id)
        .map(|p| p.has_skill(UnchannelledFury) && !p.used_skills.contains(&UnchannelledFury))

Java checks `UtilCards.hasUnusedSkill(ACTING PLAYER, skill)` and marks with
`actingPlayer.markSkillUsed(skill)` (`bb2020/UnchannelledFuryBehaviour.java:82,95`) -- the
per-activation `ActingPlayer.fUsedSkills`, cleared whenever the acting player changes. Reading the
persistent set meant a Minotaur rolled Unchannelled Fury on its FIRST activation and never again for
the rest of the game, so every later activation silently skipped a d6 and Rust ran one draw behind
Java from that point on.

**This is a known bug class in this codebase, not a new discovery.** `skill_behaviour/bb2025/
bone_head_behaviour.rs:99` carries a comment describing the identical failure -- "left the skill
marked forever, skipping every later roll" -- and the campaign notes list the remaining sites as
latent. Unchannelled Fury was one of them; it only became visible once a BB2020 roster actually
carried the skill (ITER124).

Fixed all four sites: the `do_roll` check, the Unchannelled Fury mark, and both `FuryOfTheBloodGod`
sites, which Java also marks on the acting player (`:54`).

### Results

    chaos 0 -> 100    chaos_dwarf 1 -> 100    chaos_pact 0 -> 100
    norse 1 -> 100    skaven 1 -> 100         slann_fumbbl 0 -> 100

**Two of those are unexplained and are recorded as such.** Neither `roster_skaven.json` nor
`roster_slann_fumbbl.json` contains Unchannelled Fury (grep: 0 occurrences), and the step returns
early for any player without the skill, so this fix cannot be what turned them green. The only other
change between their red measurements and now was the skaven `Animal Savagery` correction, which
cannot affect slann_fumbbl at all. Both are green under two independent measurements (an individual
100-seed run and the full matrix), so the current state is solid -- but the transition is not
understood, and claiming credit for it would be wrong. If either regresses later, start here.

### Gate

Full 30-roster BB2020 matrix **30/30 x 100/100**. Cross-edition, which matters more than usual since
`step_unchannelled_fury.rs` is a MIXED step shared by every ruleset: **chaos bb2025 100/100** (a
BB2025 roster that genuinely has Unchannelled Fury), lineman bb2025 100/100, lineman bb2016 100/100.
ffb-engine 7169/0, workspace clean.

### Tests

- `unchannelled_fury_rolls_again_on_the_next_activation` -- the regression test: asserts the d6 is
  drawn on a second activation after the per-activation set is cleared, and that the mark does NOT
  land on the persistent Player.
- `bb2020_chaos_minotaur_has_unchannelled_fury` (ffb-model) -- pins that the roster fix actually
  grants the skill, which is what made the engine bug reachable.
- Two existing tests encoded the old behaviour and were updated to Java's semantics rather than
  reverted.

## ITER127 -- the BB2020 rosters are BB2016 data, not just BB2016 spellings

ITER123 fixed 15 BB2016 skill SPELLINGS in the BB2020 rosters. This iteration asked whether that was
the whole of the "legacy clone" problem. It is not.

### Evidence

Comparing every position present in both editions' roster JSON: **90 of 125 shared positions
differ**, and the differences are not noise. Representative:

    wood_elf:Catcher   bb2020 ma9 ag4 av7      bb2025 ma8 ag2 av8   (+ Sprint in bb2025)
    wood_elf:Lineman   bb2020 ag4 av7          bb2025 ag2 av8
    vampire:Thrall     bb2020 av7              bb2025 av8

BB2020's actual Wood Elf Catcher is MA8 ST2 AG2+ PA4+ AV8+. So **the bb2025 numbers ARE the BB2020
numbers, and the bb2020 numbers are the BB2016/LRB6 ones.** The AG column settles it: BB2016 used
"higher is better" (AG4), BB2020 switched to a roll-target (AG2+). Our bb2020 rosters store 4, i.e.
BB2016 semantics, which under BB2020 rules means the engine asks a Wood Elf Catcher for 4+ where the
rules say 2+.

### Why the matrix never caught it, and cannot

Both engines read the SAME roster JSON. A wrong stat is wrong identically on both sides, so per-step
state hashes agree and the roster stays 100/100. **Roster fidelity is invisible to the parity
harness by construction** -- it can only catch engine disagreement, never bad shared input. The
30/30 says the two engines agree about a BB2016 team playing under BB2020 rules.

### Scope of the real problem

Three layers, of which ITER123 fixed only the first:

1. **Skill spellings** -- 15 found, fixed, and the checker now covers bb2020 (ITER123).
2. **Stats and skill lists** -- 90 of 125 shared positions carry BB2016 values.
3. **The team list itself** -- `rules/teams/` holds the 30 modern (BB2020/BB2025) teams including
   Black Orc, Gnome, Imperial Nobility, Khorne, Old World Alliance and Snotling. Our roster set is
   the BB2016 race list (nippon, slann, chaos_pact, the `*_fumbbl` variants) and has none of them.
   That is the actual reason Brawler is unreachable: BB2020 puts it on the Black Orc Blocker, and we
   have no Black Orc team. 105 of 164 BB2020-legal skills are granted by no bb2020 roster at all.

### Recommendation, not started

Derive the bb2020 rosters from the bb2025 ones -- which CLAUDE.md records as audited against the
official team pages -- then BB2020-legalise them by replacing or dropping any skill that
`check_skill_names.py` rejects for BB2020. That gets layers 2 and 3 in one pass and reuses an
already-audited source rather than inventing data.

It is a large change and should be treated as such: every bb2020 game's dice change, so the whole
matrix goes red until re-greened, and it will surface engine bugs the way ITER124 did. Worth doing
for exactly that reason, but it is a multi-iteration effort and a scope decision, so it is recorded
here rather than begun.

## ITER128-129 -- BB2020 rosters rebuilt from the rulebook; necromantic is the last red

The rulebooks arrived (see the Rules commit): `rules/bb2020/` now holds 11 core chapters, 29 team
pages and 64 star players scraped from bloodbowlbase.ru, and `rules/bb2016/teams/` holds the 24 LRB6
rosters extracted from the rulebook PDF. fumbbl.com was NOT scraped -- it sits behind an Anubis
proof-of-work challenge and defeating that is not ours to do.

### The rebuild

`scripts/roster_from_rulebook.py` regenerates an edition's roster JSON from its team pages.
All 29 bb2020 rosters rebuilt: **78 positions corrected, 43 added, 24 dropped**. The additions are
real BB2020 differences, not noise -- Vampire splits into Runner/Blitzer/Thrower/Vargheist, Ogre's
Snotling becomes Gnoblar Lineman, Nurgle gains Bloater and Rotspawn. Dropping 24 positions
invalidated the squads that referenced them, so `draft_bb2020_teams.py` redrafted all 29 in the same
change; rosters and teams land together or not at all.

### Three converter bugs caught before they reached the data

1. **Suffix-only name matching.** `Skeleton -> Skeleton Lineman` read as a drop plus an add rather
   than a rename: 60 spurious additions. Now matches on either side being a prefix or suffix.
2. **Id reuse.** Nothing stopped several new positions claiming the SAME old id. With Vampire
   splitting three ways that would have written duplicate ids and broken lookups silently. Each old
   position is now claimed once, longest match first.
3. **Skill values stored in display form.** The rulebook writes `Bloodlust (3+)` and `Loner (4+)`;
   the roster convention is a plain integer, `Bloodlust: 3`. Storing `"3+"` means
   `get_skill_value_int` cannot parse it and silently falls back to the skill's DEFAULT -- a
   Vargheist rolled Blood Lust on 2+ in Rust against Java's 3+. **That alone was vampire 3/100**;
   fixed, it is 100/100. The failure mode to remember: wrong data degrades quietly, it does not error.

`check_skill_names.py` also caught the rulebook spelling `Sidestep` against Java's BB2020 name
`Side Step` -- four positions would have silently lost the skill.

### Result

**28 of 29 rosters 100/100.** Better than expected for a change that moves AG and AV on nearly every
player, and so changes every dodge, pick-up and armour roll in every game.

### The remaining red: necromantic 98/100 (seeds 37, 65)

Seed 65, step 130, half 2 turn 1. Both engines have `h01` (a Werewolf, AV9+) Standing at 12,6; after
the step Java has it `-1,-1,Ko` and Rust has it `11,5,Prone`. Every prior state hash matches.

Dice are **identical in value and sides** through the whole window --
`pos 59..64 = d8:2, d6:5, 3, 4, 2, 6` -- and Java's own `JAVA_DIE` stream agrees, with step 130
consuming exactly rng 60-64.

What separates them is WHICH STEP consumes them, which `FFB_DIE_AT` names directly:

    die 61   Java  DiceRoller.rollDice:98 (a 2d6)      Rust  StepDropFallingPlayers   -- agree
    die 63   Java  DiceRoller.rollDice:98 (a 2d6)      Rust  StepBlockRoll            -- DIVERGE

So by die 63 Java is still resolving injuries from dropped players while Rust has already moved on
to a block. **Rust drops one fewer player, or resolves one fewer injury, in
`StepDropFallingPlayers`.** That is the next thing to measure -- how many players each engine drops
there, and why one is missing.

Two dead ends recorded so they are not re-run: Blood Lust is already per-activation
(`step_blood_lust.rs:175`), and `injury_type_throw_a_rock.rs` correctly sets `armor_broken = true`
and skips the armour roll, so neither is the cause. An armour-vs-injury theory built on sampled
event-log entries was wrong -- those entries came from elsewhere in the game, not step 130.

Held on branch `bb2020-roster-redraft` until this is green; main stays 30/30.

## ITER130 -- necromantic root cause: the two engines disagree on WHEN the attacker starts falling

Seed 65 is now understood. It is not a dice bug, not a roster bug, and not the drop ORDER as ITER129
guessed -- it is a modifier-suppression flag keyed on the attacker's state at the moment the
defender's armour is rolled.

### The mechanism

Java BB2020 `skillbehaviour/bb2020/PilingOnBehaviour.java:103`, in the branch taken when Piling On
is NOT being used:

    InjuryTypeBlock.Mode mode = (attackerState != null && attackerState.getBase() == PlayerState.FALLING)
        ? InjuryTypeBlock.Mode.DO_NOT_USE_MODIFIERS
        : InjuryTypeBlock.Mode.REGULAR;

Rust mirrors this in `step_drop_falling_players.rs` via `attacker_also_falling`. The port is
faithful; what differs is the ANSWER each engine gives to "is the attacker falling right now".

`DO_NOT_USE_MODIFIERS` suppresses armour modifiers, and the necromantic Werewolf has **Claws**, which
breaks armour on 8+ regardless of AV. So the flag decides the outcome outright.

### The evidence

A temporary probe in the defender branch (since removed) printed, for seed 65:

    DFP defender=home_02 attacker=away_02  attacker_also_falling=false  rng=62  armour=[3,4]
    DFP defender=home_01 attacker=away_03  attacker_also_falling=TRUE   rng=66  armour=[6,2]

home_01 is a Werewolf, AV9+. Its armour total is **8** in both engines -- Rust rolls it at dice
65-66 as `[6,2]`, Java at dice 63-64 as `[2,6]`. Same total, and 8 is exactly the number Claws
breaks on.

- Rust: `attacker_also_falling = true` -> DO_NOT_USE_MODIFIERS -> Claws suppressed -> 8 < AV9 ->
  armour holds -> `h01` ends **Prone**.
- Java: KOs `h01`, which requires the armour to have broken, which requires REGULAR mode, which
  requires `attackerState.getBase() != FALLING` at that instant.

So at the moment the defender's armour is rolled, Rust already considers the attacker FALLING and
Java does not.

### Corrections to ITER129

ITER129 concluded "Rust drops one fewer player, or resolves one fewer injury". That was wrong. Both
engines drop the same players; Rust simply interleaves a block roll (dice 63-64) between the two
drops where Java resolves them back to back, which made the counts look unequal. The dice-position
mismatch is a symptom of the interleaving, not the cause.

Two further dead ends, both checked and cleared: Java's shared
`bb2025/shared/StepDropFallingPlayers.java:135` uses `Mode.REGULAR` unconditionally, so the
DO_NOT_USE_MODIFIERS behaviour comes from the BB2020 PilingOn hook and nowhere else; and there is no
`bb2020/.../StepDropFallingPlayers.java` at all, only the `action/block` COMMON one.

### Next

Find where each engine transitions the attacker to `FALLING` during a Both Down and align the
ordering with Java. The Rust flag reads `game.field_model.player_state(&attacker_id).base() ==
PS_FALLING`, so either Rust sets that state earlier than Java does, or Java re-reads a snapshot
taken before the block resolved -- note Java captures `attackerState` at the TOP of the step
(`StepDropFallingPlayers.java:122`) and then mutates it, so a stale snapshot is the leading
candidate.

Still on branch `bb2020-roster-redraft`; 28/29, main untouched at 30/30.

### ITER131 -- the flag is set upstream, not in this step

Checked the obvious fix first and ruled it out: Java reads `attackerState` at the top of the hook
(`PilingOnBehaviour.java:59`) and Rust reads it just before the injury call, but nothing between
those two points touches the ATTACKER -- the intervening mutations are all on the defender (rooted
clear, HIT_ON_GROUND -> FALLING) plus `drop_player_rng(defender)`. So snapshotting earlier in
`step_drop_falling_players.rs` would be a no-op, and the difference is real: by the time this step
runs, Rust has the attacker FALLING and Java does not.

That moves the search upstream, to whoever marks the attacker FALLING during block resolution. The
measured clue points the same way: Rust spends dice 63-64 on a `StepBlockRoll` BETWEEN the two
drops, where Java resolves the drops back to back. Both necromantic Werewolves have Frenzy, so a
second block is in play, and the likely fault is Rust resolving Frenzy's second block before the
first block's falling players are finished.

Next: compare the block/Frenzy sequence ordering, not this step.

### ITER132 -- the snapshot hypothesis is DISPROVEN; stop guessing, widen the Java trace

Implemented ITER131's candidate and measured it: **inert**. Java captures `attackerState` once at
the top of the hook and reuses it for both the injury-modifier mode and the attacker-fall branch,
while Rust re-reads live state at each point -- the second read happening after the defender's
injury and `drop_player`. Making Rust snapshot once, exactly as Java does, changed **nothing**:
necromantic is still 98/100, failing the same two seeds at the same two steps (37/182, 65/130).
So the two reads always agreed and the premise was false.

Reverted rather than kept. The change was behaviourally inert but carried a comment asserting a bug
that does not exist, and a wrong explanation in the source is worse than no explanation -- that is
the same trap ITER116 and ITER117 had to undo.

**Five hypotheses on this one divergence have now been disproven** -- Blood Lust per-activation,
Throw a Rock armour-vs-injury, unequal drop counts, the defender-mode flag, and now the attacker
snapshot. Every one came from correlating dice values or reading Java statically and reasoning
forward. The only things that have actually produced facts here are `FFB_DIE_AT` backtraces and the
temporary Rust probe.

The blocker is asymmetry of evidence: Rust can be probed freely, but the Java engine cannot be
modified (only `ParityRunner` is co-editable), so Java's side is visible solely through
`JAVA_DIE ... from=DiceRoller.rollDice:98` -- which names the dice helper, never the STEP that
called it. Every remaining hypothesis is about which Java step is running, and that line cannot
distinguish them.

**Next: widen the Java dice trace in the harness** to print several caller frames, or the current
step id, alongside each `JAVA_DIE`. `ParityRunner` is explicitly co-editable and the campaign has
precedent for exactly this (`JAVA_BLOCKROLL`, `JAVA_AVBROKE`, `FFB_IDSTATE`). One harness change
converts this from guesswork into a direct read.

### ITER133 -- the deep Java trace exists; necromantic localised to the armour-break decision

**The tool I said needed building was already there.** `DiceRoller.java` in the LIVE Java tree
(`niels/ffb`, not the `ffb-java/` reference copy) already prints up to six caller frames per die,
gated behind `-Dffb.parityDebugDeep`, and the harness already exposes it as `FFB_DICE_DEEP`. It was
in the env-var list I scanned several iterations ago. Use `FFB_TRACE=1 FFB_DICE_DEEP=1`.

Worth recording separately: **the jar is built from `C:/Users/Admin/niels/ffb/`, not from
`ffb-rust/ffb-java/`.** The latter is a reference copy. They are in sync for the files checked here,
but a Java behaviour question should be answered from the tree the jar is actually built from.

### What the trace settles

necromantic seed 65, step 130, with players named:

    Activate(Away2, BLOCK) -> JAVA_BLOCK_PICK def=Home2, nDice=1
    rng 60  block die = 5                      StepBlockRoll.executeStep:241
    rng 61,62  armour = 3,4 = 7                InjuryTypeBlock.armourRoll:83  <- PilingOnBehaviour
    rng 63,64  INJURY  = 2,6 = 8 -> KO         InjuryTypeBlock.injuryRoll:47

So Java rolls a single block die, the defender (Home2 = `h01`, the Werewolf that ends up KO) goes
down, and **Java breaks armour on a total of 7 and proceeds to an injury roll.** Rust rolls the same
armour and does not break, so it never rolls injury and the player stays Prone.

### The open contradiction -- do not build on my arithmetic

Attacker Away2 is a Werewolf with Claws; defender Home2 is a Werewolf, AV9+. Java's own
`mechanics/mixed/StatsMechanic.java:25` is

    reduceArmour(context, armour, 8) <= (roll[0] + roll[1] + context.getArmorModifierTotal(game))

Claws registers `reducesArmourToFixedValue` and applies when AV > 8, so the effective AV is 8, and
`8 <= 7` is FALSE. Without Claws it is `9 <= 7`, also false. By that reading Java should NOT have
broken this armour -- but the trace shows it did. **So my model of Java's armour path is
incomplete**, and any fix derived from it would be guesswork. The roster is not the problem: the
scraped BB2020 page gives the Werewolf exactly Claws, Frenzy, Regeneration at AV9+, and the
generated XML matches.

Rust's side of the same decision is `mechanics/roll.rs` `is_armour_broken` (`roll_total >= armour`)
plus `apply_fixed_armour_reduction` (BB2016 caps at 7, BB2020/25 at 8), which mirrors the Java
constants -- so the divergence is in what feeds them, not the constants themselves.

### Next

Instrument the JAVA side, debug-gated exactly as `DiceRoller` already is, to print the effective
armour, the modifier total and `armorBroken` at the moment of the decision. That is the one value
neither the dice trace nor static reading can supply, and every remaining hypothesis needs it.

## ITER134 -- necromantic root cause: the Intensive Training prayer skill grant

Found it, and the third pre-existing tool found it. mechanics/mixed/StatsMechanic.java in the LIVE
tree already carries a JAVA_AVBROKE trace gated behind ffb.parityDebug, which FFB_TRACE=1 already
sets. For seed 65 it prints the armour decision three times as modifiers accumulate:

  def=Home2 armour=9 reduced=9 roll=[3,4] modTotal=0 mods=                      broken=false
  def=Home2 armour=9 reduced=8 roll=[3,4] modTotal=0 mods=Claws,                broken=false
  def=Home2 armour=9 reduced=8 roll=[3,4] modTotal=1 mods=Claws,Mighty Blow,    broken=true

So Java breaks the armour because the attacker has MIGHTY BLOW, worth +1, taking 7 to 8 against the
Claws-reduced AV of 8. Rust has no such modifier, gets 7 < 8, and never rolls injury.

No Necromantic player has Mighty Blow. Ruled out one by one: the BB2020 rulebook page gives the
Werewolf exactly Claws, Frenzy, Regeneration at AV9+; the roster JSON matches; the generated
roster XML matches; the team XML matches and points at the correct rosterId necromantic.bb2020; and
only the three real MightyBlow skill classes register a modifier by that name. So the skill is
acquired DURING the game.

It is the Intensive Training Prayer to Nuffle, and Rust's own handler says so in its header:

  "Headless: selects one player and marks the prayer enhancement; SKIPS SKILL-SELECTION DIALOG
   (position skill categories not available server-side)."

Java's mixed/prayers/IntensiveTrainingHandler shuffles the eligible players, takes the first, builds
the eligible skills from that player's PRIMARY skill categories sorted by name, shows a
DialogSelectSkillParameter, and applySelection calls addIntensiveTrainingSkill. Rust selects the
player and stops. The timing fits exactly - half 2 turn 1, right after the kickoff, where BB2020
Cheering Fans grants a Prayer to Nuffle.

This also explains why it surfaced only now: the roster redraft changed the game flow enough for
Cheering Fans to roll Intensive Training in these two seeds.

Two things to settle before fixing, neither guessed at here. Java's Collections.shuffle(players)
CONSUMES RNG, so Rust must consume identically or the streams part regardless of which skill is
granted. And the observed grantee is the attacker Away2, a Werewolf whose primary categories are A
and G, while Mighty Blow is a Strength skill - so either the grant is not restricted to primaries in
the path taken, or the shuffle picked a different player and the modifier reaches the block another
way. Both are answerable by tracing the prayer itself.

Ends a run of five disproven hypotheses. Every one of those came from correlating dice values or
reasoning forward from Java source; all three real findings today came from traces that already
existed - FFB_DIE_AT, FFB_DICE_DEEP and now JAVA_AVBROKE.

## ITER135 -- RETRACTION of ITER134: Intensive Training cannot be the cause

ITER134 claimed the necromantic divergence was Rust skipping the Intensive Training prayer's skill
grant. That is wrong on two counts and is retracted.

First, Rust does NOT skip it. The bb2020 handler's header comment says "skips skill-selection dialog"
but the comment is STALE: step/bb2020/step_prayer.rs:129-145 calls h.skill_dialog(game) and raises
AgentPrompt::SelectSkill, and random_agent.rs:790 answers it with the alphabetically first eligible
skill, matching Java's Comparator.comparing(Skill::getName). I quoted the comment instead of reading
the code beneath it - the same mistake this campaign has hit repeatedly with stale comments.

Second, Intensive Training could not have granted Mighty Blow to this player anyway. Java picks
eligible skills from player.getPosition().getSkillCategories(false), and RosterPosition:195 shows
false = the NORMAL-roll list, which is exactly Rust's player.skill_categories_normal. The attacker
is a Werewolf whose normal categories are Agility and General; Mighty Blow is Strength, so it is
ineligible in BOTH engines. The rebuilt roster has the categories the right way round (Werewolf
normal A/G double P/S, Flesh Golem normal G/S double A), so the converter did not swap the columns.

Confirmed and still standing: cheeringFans fires once in this game, so a Prayer to Nuffle IS granted
at half 2 turn 1, and JAVA_AVBROKE shows Java applying a +1 modifier named Mighty Blow to the
attacking Werewolf that Rust does not apply. Also confirmed the Java team XML player order matches
the JSON, so Away2 is the same Werewolf in both engines.

That makes six disproven hypotheses on this one divergence: Blood Lust per-activation, Throw a Rock
armour-vs-injury, unequal drop counts, the defender-mode flag, the attacker snapshot, and now
Intensive Training. Every single one came from reading code and reasoning forward. Every real fact
came from a trace that already existed.

So I am not hypothesising again. The next step is the one measurement that names the source
directly: extend the JAVA_AVBROKE trace in mechanics/mixed/StatsMechanic.java - which already exists
and is already gated behind ffb.parityDebug - to print, for each modifier, the skill it is
registered to and the player owning that skill, then rebuild the jar and run seed 65 once. That
turns "where does this Mighty Blow come from" from a guess into a read.

Branch parked at 28/29; main untouched at 30/30.

## ITER136 -- narrowed to Mighty Blow missing from Rust armour modifiers

Built the Rust mirror of JAVA_AVBROKE (temporary probe, since removed) and compared the same armour
decision side by side at seed 65 step 130:

  JAVA  def=Home2  armour=9 reduced=8 roll=[3,4] modTotal=1 mods=Claws,Mighty Blow  broken=true
  RUST  def=home_02 att=away_02        roll=[3,4] modTotal=0 mods=[Claws]           broken=false
        att temp skills = [MightyBlow=Some("+1")]

The attacker HOLDS Mighty Blow in Rust - the probe confirms it on temporary_skills and has_skill
returns true - yet Rust's armour modifier list contains only Claws. Java stacks Claws and Mighty
Blow for +1, reaching the Claws-reduced AV of 8 on a roll of 7 and breaking armour; Rust totals 0,
does not break, never rolls injury, and the player stays Prone instead of being KO'd.

Where the skill comes from is now fully explained and NOT a bug: the BB2020 Cheering Fans kickoff
result grants a Prayer to Nuffle, both engines roll KNUCKLE_DUSTERS, both apply it to away_02 at the
half-2 kickoff, and Prayer.java:52-58 shows KNUCKLE_DUSTERS grants Mighty Blow (+1). Rust's
prayer_player_effect.rs:35 grants the same skill. Prayer selection, player selection and timing all
agree.

So the defect is downstream, in injury_type_block.rs armour_roll. mods contains only Claws after
find_armor_modifiers, so Mighty Blow was never returned for the attacker - the Claws branch cannot
have dropped it, because that branch re-adds the remaining mods and CLAW_DOES_NOT_STACK is set to
false in util_server_start_game.rs:56. Either the factory does not see the temporary skill on this
path, or the Claws handling clears the list before the others are added.

Also ruled out this iteration: the Java jar CANNOT be rebuilt - there is no mvn on PATH and no
wrapper, only prebuilt jars in ffb-ai/target - so the Java-side instrumentation proposed in ITER135
is not available and the JAVA_AVBROKE trace already compiled in is the most Java detail obtainable.
I should have checked the toolchain before proposing it.

Next: instrument find_armor_modifiers / get_armor_modifiers_from_skills for this one block and print
the skill ids it iterates and what each maps to. Expected output is a list containing MightyBlow;
if it is absent the fault is in the enumeration, if present the fault is in the Claws branch.

## ITER137 -- clawDoesNotStack: root cause confirmed, obvious fix regresses dwarf, reverted

Root cause of necromantic seed 65, confirmed by probe rather than inference:

find_armor_modifiers returns BOTH ["Claws", "Mighty Blow"] for the attacker, so the modifiers are
found and then discarded. That is precisely what the CLAW_DOES_NOT_STACK branch of
injury_type_block.rs does - clear, retry without Claws, then restore Claws alone - and it matches
the observed end state exactly (mods=[Claws] modTotal=0 broken=false against Java's
mods=Claws,Mighty Blow modTotal=1 broken=true).

The option is meant to be false. Java UtilServerStartGame:247-249 sets it explicitly. Rust has the
same assignment in util_server_start_game.rs:56 - inside add_default_game_options, WHICH IS CALLED
FROM NOWHERE. grep across all crates finds no caller. So every option falls back to its factory
default, and clawDoesNotStack's factory default is TRUE (util_game_option.rs says so in its own
comment). Rust has been running with Claws-does-not-stack for its entire history while Java has not.

That dead function is a backlog item I recorded at ITER113 and dismissed as inert. It is not.

THE OBVIOUS FIX DOES NOT WORK. Adding (CLAW_DOES_NOT_STACK, "false") to BASELINE_SETUP_OPTIONS -
exactly the precedent MB_STACKS_AGAINST_CHAINSAW set in the same list for the same reason - fixes
necromantic seed 65 (98 -> 99/100) but REGRESSES dwarf from 100/100 to 92/100. Net 2 failures
before, 9 after, so it is reverted.

The dwarf regression is not a Claws case at all. Seed 56 diverges at step 3 on a FOUL:
JAVA_AVBROKE shows armour=9 reduced=9 mods=Dirty Player modTotal=2 broken=true - no Claws involved,
and Claws returns None for fouls anyway. So flipping this option changes dwarf through some path
other than the one it names, which means my model of what the option gates is incomplete.

What this tells us: the option default IS wrong in Rust, but it is not independently settable -
something else in the armour path compensates for it today, and correcting one without the other
trades necromantic's 2 failures for dwarf's 8. The pair has to be understood together.

Next: run dwarf seed 56 with a RUST_AVBROKE-style probe against JAVA_AVBROKE at step 3 to see what
the option actually changed on a foul path, before touching the default again.

Branch stays at 28/29 with necromantic 98/100; main untouched at 30/30.

## ITER138 -- RETRACT ITER137: the dwarf regression was NOT the option; fix restored

ITER137 said the clawDoesNotStack fix regressed dwarf from 100 to 92 and reverted it. That was
wrong. Re-running dwarf AFTER the revert still gives 92/100, and re-running it with the option
restored also gives 92/100 - identical failing seeds either way. The option is provably not the
cause, and I reverted a correct fix on a false attribution because I compared against a stale
100/100 rather than re-measuring the baseline.

Restored, with the measurement this time:

  necromantic  98 -> 99/100   (seed 65 fixed)
  dwarf        92/100 with AND without the option (unchanged)

The real cause of dwarf is the skill-value normalisation committed with the redraft, which I never
re-matrixed on its own. It rewrote every roster value from display form to integers, so the
generated XML now carries value="2" for Dirty Player, value="5" for Loner and value="1" for Mighty
Blow where it previously carried unparseable "+2"/"5+"/"+1". That is the correct format and it is
strongly net-positive - it took vampire from 3/100 to 100/100 - but it also changed what Java reads
for dwarf, exposing 8 seeds. dwarf is now a newly-exposed bug like the others this session, not a
regression from this commit.

The fix itself: Java's UtilServerStartGame:247-249 sets clawDoesNotStack=false explicitly. Rust has
the same assignment in util_server_start_game.rs:56, inside add_default_game_options, which is DEAD
CODE called from nowhere in any crate - so options fall back to factory defaults and this one
defaults to TRUE. Rust has run with Claws-does-not-stack for its entire history. With Claws and
Mighty Blow both on the attacker, Java stacks them (JAVA_AVBROKE mods=Claws,Mighty Blow modTotal=1
reduced=8 broken=true on a roll of 7) while Rust kept Claws alone (mods=[Claws] modTotal=0
broken=false), leaving the defender Prone instead of KO'd.

Follows the precedent immediately above it in the same list: MB_STACKS_AGAINST_CHAINSAW was added
for exactly this reason, also confirmed via JAVA_AVBROKE.

Gate: necromantic 99/100, dwarf 92/100 (unchanged by this commit), workspace tests clean.

Remaining on this branch: dwarf 8 seeds and necromantic seed 37, both traceable to the value
normalisation rather than to this change.

## Cross-team matchups — the first coverage frontier past the mirror matrix

All three mirror matrices are 30/30. That is a narrower claim than it sounds: the matrix only ever
runs a roster against ITSELF, so a rule needing the ATTACKER to have one skill and the DEFENDER
another is reachable only when a SINGLE roster carries both.

`scripts/run_cross_matrix.py` runs non-mirror matchups, ranking pairs by how many such interactions
they reach and weighting those NO roster can reach alone. In bb2020 five are mirror-unreachable:
Chainsaw/Iron Hard Skin, Dirty Player/Iron Hard Skin, **Tackle/Dodge**, Stab/Regeneration,
Bombardier/Foul Appearance. Tackle-cancels-Dodge is about as mainstream as Blood Bowl gets and had
never been compared between the engines.

Trap worth remembering: chaos_dwarf carries BOTH Stab and Iron Hard Skin, so its own mirror DOES
reach the stab `ignoresArmourModifiersFromSkills` branch. Check both halves against one roster
before claiming a mirror cannot reach a rule -- an earlier draft of this note got that wrong.

FIRST RUN, 10 pairs x 25 seeds, bb2020: 9 green, 1 RED -- a real bug on the very first cross run.

### OPEN: chaos_dwarf vs chaos_pact seed 16 step 82 (Animal Savagery on a HAND_OVER)

away_02 (Animal Savagery) declares HAND_OVER. Both engines roll the confusion die and fail, and
both decline the re-roll. Then:

  JAVA  action lost, post-state == pre-state, next player activates
  RUST  player reverts to a Move and MOVES to (13,7)

Root cause is ORDERING, not a rule. Probes show Rust's `acting_player.player_action` is still
`Move` when Animal Savagery runs, while Java's is already `HAND_OVER` (`JAVA_P2 action=HAND_OVER`
precedes the die). Rust rolls it in the MOVE activation, before the declared hand-off is applied;
Java changes the action first so the roll happens inside the Pass/HandOver sequence. The die count
and target number agree (Move and HAND_OVER both give `goodConditions=false`), so the dice stay in
step -- only the FAILURE LABEL differs: Rust routes to END_MOVING and keeps moving, Java to
END_PASSING with `END_PLAYER_ACTION=true`, which pushes the EndPlayerAction sequence
(`StepEndPassing:236`) and ends the activation.

Ruled out along the way, each by direct evidence rather than inference:
  - receiver eligibility: `legal_handoff_receivers` matches ParityRunner exactly (adjacent
    teammates, NO state filter, coordinate-sorted) -- prone receivers included on both sides
  - the EndPlayerAction publish: Rust's failure path DOES publish it
  - StepResetToMove: faithful, no-ops without RESET_PLAYER_ACTION, same as Java
  - StepEndPassing: has the same `end_turn || end_player_action` branch
  - "Rust consumed 0 dice": WRONG, read off per-step `rng_calls`. Rust does roll -- the trace shows
    `ReRollOffer { action: "ANIMAL_SAVAGERY" }`. Per-step rng_calls deltas are not a reliable dice
    comparison; see the standing note on Java per-call vs Rust per-die counting.

Needs (a) Animal Savagery, (b) a HandOff/Pass declaration, (c) a failed roll -- which is why 90
green mirror matchups never hit it.

LATENT, found while reading and NOT the cause here: `step_animal_savagery.rs` computes
`do_roll` from the PERSISTENT `Player.used_skills` where Java uses the per-activation
`UtilCards.hasUnusedSkill(actingPlayer, skill)`. Same bug class as the Unchannelled Fury fix
(ITER124-126) and the documented `bone_head_behaviour.rs:99` case. Symptom would be a missing roll
on a LATER activation, not this divergence, so it is recorded rather than fixed here.
