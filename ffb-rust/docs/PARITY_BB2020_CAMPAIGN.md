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
