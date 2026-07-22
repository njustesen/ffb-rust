# Test-Count Equalization Campaign (started 2026-07-21)

User goal: SAME tests on both engines — same test METHODS, 1:1 names (camelCase↔snake_case),
same assertions. Strategy chosen: **hybrid** — prune remaining low-signal Rust tests first,
then port everything surviving to Java. Counting convention: test methods (Java
@ParameterizedTest kept parameterized; surefire run-counts will exceed method counts — that is
accepted).

Starting point: Rust 16,294 `#[test]` fns; Java 1,626 test methods (2,540 surefire runs).
Documented exception: `ffb-parity`'s 39 harness self-tests (Rust-only comparison tooling, no
Java counterpart) + any `// PARITY-EXEMPT` modules batch A3 tags (Rust-only infra).

## Wave plan / status

| Batch | Scope | Status |
|---|---|---|
| A1 | Prune ffb-engine step boilerplate (per-file id_is_* → one exhaustive make_step test; trivial construction; Rust-plumbing) | RUNNING |
| A2 | Prune ffb-model {dialog,factory,inducement,model,util,types,injury,data,events,prompts} low-signal | RUNNING |
| A3 | Prune ffb-client + ffb-server low-signal; tag PARITY-EXEMPT Rust-only infra | RUNNING |
| B0 | Java headless GameState fixture (ffb-server/src/test/.../fixture/) + StepInitStartGame POC | RUNNING |
| B1 | Port ffb-protocol (~882) → ffb-common/src/test/.../net/commands/ | RUNNING |
| B2 | Port ffb-model model/dialog/factory/inducement/types/injury/util survivors → ffb-common tests | pending (after A2) |
| B3 | Port ffb-model report/ (391) → Java report JSON tests | pending |
| B4 | Port ffb-mechanics unmirrored (~700: modifiers/, mechanics extras, skills table tests) → Java | pending |
| B5 | Port ffb-client survivors (~1,900) → ffb-client-logic tests | pending (after A3) |
| B6 | Port ffb-server survivors (~1,200) → Java server tests | pending (after A3) |
| B7 | Port ffb-engine steps (~4,900 survivors) → Java step tests using B0 fixture — split ~15-20 sub-batches by edition/dir | pending (after A1+B0) |
| B8 | Port ffb-engine skill_behaviour (403), injury (555), util/mechanic/marking remainder, model/factory | pending (after B0) |
| C | Final: counts equal (± documented exemptions), full cargo + mvn green, T3 gate, docs, commit | pending |

## Rules for every batch
- No git-mutating commands in agents; disjoint file ownership per batch.
- Java src/main is READ-ONLY (reference engine). New Java tests go in the top-level modules
  (ffb-common/ffb-server/ffb-client-logic), never the nested ffb-java/ffb/ clone.
- A Rust test is prunable only if: pure read-back, Rust plumbing (Deref/Clone/Debug/Default),
  restates the same source line, or Rust-only infra that is ALSO low-signal. Behavioral,
  serialization, lookup, and validation tests must survive and be ported.
- A ported Java test must be verified against the actual Java class (Rust test supplies the
  expected values; Java source is ground truth). Genuine cross-engine discrepancies discovered
  while porting: keep the Java-correct expectation, report prominently, fix Rust production if
  clearly a translation bug (with its own scoped verification).
- Every batch runs its scoped `cargo test` / `mvn test` before reporting.

## Progress log
- Baseline: Rust 16,294 / Java 2,540 surefire runs.
- Wave 1 (2881831c): pruned ~1,570 Rust → 14,725; Java fixture built.
- Wave 2 partial (8cba5a40): B1 protocol port; Java ~3,648.
- Wave 2b (1a6ed1e1): R1 report (336), R2 model, R3 step pilot (78). Java ~4,376
  (ffb-common 1,892 / client-logic 17 / server 2,467). All modules green.
- Step-port intel (from R3): generator tests ~100-150/agent-run; step-logic ~4-8 files/run,
  needs a seeded-dice delegate + GeneratorTestSupport promoted to fixture; ~4,700 step tests
  remain → budget ~40-50 agent-runs. THE dominant remaining cost.
- Wave 3 (a2b7f006/9a3edf3e/f4ce3c30): R2 model finish (ffb-common 1,939); P-gen pruned 134
  Rust generator param tests (engine 7,108); B5r client partial (client-logic 101); B6r server
  partial (server 2,505). Java now ~4,545 distinct. Rust ~14,540.
- Recurring: session-limit interruptions kill agents mid-wave ~every wave; recovery = remove
  broken partials (compile errors / initFrom-NPE-on-default / package-private access), restore
  each module green, commit. Fixed ReportMessageTestBase.Run fields → public for subpackage tests.
- Wave 4 (d2dc289d/003d1328): FX fixture enhancement (ScriptedFortuna scripted dice +
  GeneratorTestSupport promoted, ffb-server 2,511); B5r2 client-logic 101→1,208 (301 classes).
- **BLOCKED (2026-07-21): hit the 200-subagent session cap.** Raising
  CLAUDE_CODE_MAX_SUBAGENTS_PER_SESSION only takes effect after a Claude Code session RESTART.
  Set it high (e.g. 600) and restart to resume the parallel port.

## RESUME STATE — BY-HAND EXECUTION (2026-07-23, single-threaded, NO subagents)
User decision: continue **single-threaded by hand** (no subagents / no Workflow); scope = "1:1 as
much as it makes sense — use common sense" (port survivors, prune plumbing/tautology, SKIP-with-
comment the fixture-inexpressible ones and tally them). Java src/main NEVER edited.

**Fresh ground-truth method counts (grep of `#[test]` / `@Test|@ParameterizedTest`):**
- Rust 14,477 (`ffb-parity` 39 exempt). Java 5,098 (ffb-common 1,789 · ffb-server 2,101 ·
  ffb-client-logic 1,208 at baseline). Gap ≈ 9,379, dominated by ffb-engine step (4,637).
- Per-crate Rust: model 2,769 · mechanics 1,146 · engine 7,149 · protocol 882 · client 1,694.
  Mirror map: model+protocol→ffb-common; mechanics+engine→ffb-server; client→ffb-client-logic.

**Toolchain (verified working):** mvn at `/c/Users/Admin/bin/maven/bin/mvn`; run from
`ffb-java/` as `mvn -pl <module> [-am] test` (~11s for client-logic; add
`-Dsurefire.failIfNoSpecifiedTests=false` when using `-Dtest=X` with `-am`). cargo from
`ffb-rust/ffb-rust` as `cargo test -p <crate> [filter]`.

**Step 1 (client) progress:** started. Full client gap analysis in scratchpad
(rust_client.txt/java_client.txt); 33 Rust-only modules (~273 real; `action_keys`→
`UtilClientActionKeys`, `chat`→`UtilClientChat` are just naming, already mirrored) + 77 count
mismatches. Done so far:
- **client_state_factory 64→63 (commit d4d1752c):** ported getStateForGame switch as Java
  `ClientStateFactoryTest` (63). PROVEN PATTERN for factory/state tests whose real concrete impl
  lives in the AWT ffb-client module: build a **test-local concrete `ClientStateFactory` + a stub
  `ClientState`/`LogicModule` per `ClientStateId.values()`**, assert `getStateForGame().getId()`.
  Deep-stub gotcha: `@Mock(RETURNS_DEEP_STUBS)` returns a NON-NULL mock for Date-returning
  `getFinished()` → force `when(game.getFinished()).thenReturn(null)` in @BeforeEach. Pruned 1
  Rust plumbing test (register/get_state_for_id no-op).
- **foul family DONE (commits 298066d6 mixed 7/7, cb34b176 bb2025 5/5).**

**PROVEN LOGIC-MODULE RECIPE (use for all remaining client logic modules):**
- Setup mirrors `MoveLogicModuleTest`: `@Mock(RETURNS_DEEP_STUBS) client` + plain `@Mock` game,
  actingPlayer, fieldModel, teamAway, communication, and raw `@Mock Player actor/defender`.
  MoveLogicModule (and its subclasses) ctor eagerly resolves the MOVE plugin, so ALWAYS stub
  `game.<LogicPluginFactory>getFactory(LOGIC_PLUGIN)` → `forType(MOVE)` → a MoveLogicPlugin mock
  before `new XxxLogicModule(client)`. `@MockitoSettings(strictness = LENIENT)`.
- Rust free fns like `is_foulable(&game, &def)` mirror Java statics (`UtilPlayer.isFoulable(game,
  def)`) — call the static directly; drive it by stubbing the exact fieldModel/team lookups it
  makes (getPlayerState → `new PlayerState(PlayerState.PRONE|STANDING)` [pkg com.fumbbl.ffb; PRONE=3
  STANDING=1 STUNNED=4], getPlayerCoordinate → `new FieldCoordinate(x,y)` w/ isAdjacent, teamAway
  .hasPlayer). Populated-Game construction NOT needed — targeted stubs suffice.
- Private helpers (e.g. `bloodlustActionContext`, `foulActionContext`) aren't callable — exercise
  them through the public method that reaches them (`playerInteraction` on the acting player when
  suffering blood lust; `playerSelected` with the fouling-alternative skill). Assert
  `InteractionResult.getKind()` (Kind.SELECT_ACTION/IGNORE/PERFORM/HANDLED) +
  `getActionContext().getActions()`.
- PRUNE from Rust (no faithful Java mirror): (a) `*_without_game` no-game `client.game()?`
  short-circuits; (b) private-helper "empty/negative" branches unreachable in Java; leave a
  `// NOTE (test equalization): ... pruned` breadcrumb. PRUNE from Java: trivial `getIdReturns*`
  getter tautologies with no Rust twin. Build both scoped gates green, commit per family.
- Gotcha: `-Dtest="pkg.ClassTest"` with `-am` needs `-Dsurefire.failIfNoSpecifiedTests=false`.

- Next client targets (biggest Rust-extra first): hand_over (bb2025 8 + mixed 9), pass +15,
  synchronous_multi_block +14, block +13, gaze +12, select_blitz_target +11, bomb +10,
  throw_keg +9, logic_module +43 (mostly free-fns on &Game/&Player — expect many prunes/SKIPs per
  LogicModuleTest precedent), plus the 33 Rust-only logic modules. Then the -N mismatches (Java
  has MORE: setup, wait_for_opponent, quick_snap, solid_defence, spectate, start_game,
  wait_for_setup) — port Java→Rust or prune Java extras.

## RESUME STATE (prior — parallel/subagent era, superseded by by-hand above)
Committed & green at 003d1328 (+ d2dc289d fixture). Counts:
- Java: ffb-common 1,939 · ffb-client-logic 1,208 · ffb-server 2,511  = ~5,658 distinct.
- Rust: 14,591.  Gap ≈ 8,900, dominated by the step port.

Remaining work-list (each a batch; all use the recover-and-commit discipline, keep module green,
no git in agents, IGNORE nested ffb-java/ffb/):
1. **Steps (~4,600 — the bulk).** Use fixture/ (GameFixture, GeneratorTestSupport, ScriptedFortuna;
   README has the recipe). Order: (a) generator families bb2020+bb2016 → step/generator/{bb2020,bb2016}
   (mechanical, ~100-150 tests/run); (b) step-logic families step/{bb2016,bb2020,bb2025,mixed,action}
   using installScriptedDice for exact outcomes (~4-8 files/run). Skip FUMBBL-mode + missing-roster-
   position tests (README limits). One family per agent; many agents.
2. **Client state/logic (25 modules, ~150 tests).** Recipe:
   scratchpad/CLEANUP_STATE_LOGIC_INSTRUCTIONS.md (LOGIC_PLUGIN factory-cast wiring; green
   reference MoveLogicModuleTest). Modules listed in the wave-4b commit body.
3. **Server remainder (~850).** db/net/admin survivors → ffb-server (not step/, not fixture/).
4. **Model/report tails** if any survivor lacks a Java mirror (mostly done).

## Audit backlog (found during porting, not yet actioned)
- SkillFactory (Java) vs SKILL_TABLE (Rust) membership divergence: union 203 vs 200, BB2016 86 vs
  58, General 15 vs 29, Yoink absent in Java BB2025. Needs a dedicated editions+category audit.
- 6 wire discrepancies from B1 (ServerCommandReplay commandNr, entropy byte-vs-u8, ZapPlayer arg
  order, null-vs-omitted keys, FieldCoordinate array-vs-object, typed-vs-string fields).

## Wave 1 results (done, commit 2881831c)
- A1 pruned 199 step tests (146 id_is_* → 1 exhaustive make_step test); step 4969→4771.
- A2 pruned 887 model tests (dialog 356→66, injury 197→1 constant read-backs, model 585→317,
  inducement 110→36, util/types/data trimmed); factory/events/prompts untouched.
- A3 pruned 541 client/server tests; tagged 3 PARITY-EXEMPT Rust-only infra modules
  (net/wire.rs, net/wire_prompt.rs, connection/mod.rs) + ffb-parity's 39 harness tests.
- B0 built the Java headless GameState fixture (no Mockito) + StepInitStartGame POC (11/11).
- Rust total 16,294 → 14,725. Workspace green.

Exemption set (Rust-only, excluded from 1:1 requirement): ffb-server net/wire.rs,
net/wire_prompt.rs; ffb-client connection/mod.rs; all of ffb-parity (39 harness self-tests).

## Direct (single-threaded) porting progress — bb2025 generators
Subagent cap (200) is exhausted; continuing by hand. Done this pass (all committed, green):
EndTurn, EndPlayerAction, Punt, ThrowKeg, ThrowARock, LookIntoMyEyes, FuriousOutburst,
BalefulHex, BlackInk, CatchOfTheDay, Treacherous, ThenIStartedBlastin, AutoGazeZoat, Bomb,
MultiBlock, RaidingParty, SelectBlitzTarget (17 files, ~110 methods) + the 10 pre-existing.
**2 real Rust bugs found & fixed:** furious_outburst missing the activation sub-sequence
(15→28 steps); bomb missing RECHECK_EXPLODE_SKILL (5→6 steps). Both fixed in Rust + tests.
**bb2025 generators: COMPLETE** (all 31 incl. the formerly-deferred special_effect,
throw_team_mate, activation_sequence_builder, ScatterPlayer). ffb-server ~2,611+.

**ALL generators COMPLETE** (bb2016 15 + bb2020 25 + bb2025 31). Added edition-aware
GameFixture overload `createGameState(playersPerTeam, Rules)`. ffb-server 2,831.
**NEXT: step-LOGIC port.** Accurate count: **4,181** genuine behavioral `#[test]` in
crates/ffb-engine/src/step/**/*.rs (excluding generators, which are done). These are NOT
tautologies (only ~7 pure-tautology tests existed in all of ffb-engine — now pruned). Each is a
real behavioral test (injury/KO/regen, block roll, end-moving, apothecary, dodge, …) needing
per-step GameState setup + scripted dice via the fixture. Labor-intensive per test; this is the
dominant remaining cost. Then client state/logic (25 modules) + server db/net remainder.
Highest-value approachable targets (utilities/steps with many behavioral tests, make_game-style
setup): step/util_server_injury.rs (42), mixed/shared/step_animal_savagery.rs (42),
bb2025/move_/step_end_moving.rs (35), bb2025/shared/step_apothecary.rs (27),
bb2025/block/step_block_roll.rs (27), bb2025/shared/step_end_selecting.rs (24).
Java totals now ~5,992 (ffb-common 1,939 / client-logic 1,208 / server ~2,852); Rust ~14,530.

**Step-logic port PATTERN PROVEN** (StepEndSelecting bb2025, 21 tests, committed): in the Java
test, `IStep step = GameFixture.createStep(gs, StepId.X); step.setParameter(StepParameter.from(
StepParameterKey.KEY, value)); GameFixture.startStep(step);` then read pushed sequences via
`GeneratorTestSupport.sequence(gs)` and step fields via `readField`. Notes: Java often reads more
state at push time than Rust (e.g. block/blitz dispatch needs USING_STAB set; move dispatch needs
an on-pitch acting player via placePlayer+setActingPlayer) — set that state, it's not a bug.
`PlayerAction.FURIOUS_OUTPBURST` is misspelled in Java. Done: bb2025/shared/step_end_selecting.
Next tractable step-logic (transition/param-driven): other shared dispatch steps, then the
state+dice-heavy ones (block_roll, end_moving, apothecary) using installScriptedDice.
Reusable: nested-field reads via readField(readField(step,"state"),"goToLabelOnFailure") for
StepBloodLust; CloudBurster hook field is fGotoLabelOnFailure; SpecialEffect generator/enum
name clash → qualify the enum as com.fumbbl.ffb.SpecialEffect.

Port recipe (proven): read Rust `#[cfg(test)]` in
crates/ffb-engine/src/step/generator/<ed>/<f>.rs → read Java pushSequence + SequenceParams
ctor in ffb-server/.../step/generator/<ed>/<Name>.java (+ base class for the ctor) → write
Java test in ffb-server/src/test/.../step/generator/<ed>/<Name>FixtureTest.java using
GameFixture.createGameState(3) + `new <Name>().pushSequence(new <Name>.SequenceParams(...))`
+ GeneratorTestSupport.{sequence,find,findLabelled,contains,count,indexOf,booleanField,readField}.
Param-content assertions: read the target Step's private field name and use readField/booleanField.
SKIP (with comment): StepBloodLust goToLabelOnFailure (nested `state` field, not observable);
ThenIStartedBlastin GOTO_LABEL_ON_END (Java init doesn't consume it). Watch for real Rust bugs
(2 found & fixed so far: furious_outburst + bomb missing steps vs Java) — assert Java-true value
and fix the Rust generator + its Rust test.

## Wire discrepancies found during porting (review later; NOT test-count blockers)
Byte-parity is already knowingly broken; these are logged for a future wire-parity pass.
- B1: `ServerCommandReplay.initFrom` doesn't restore `commandNr` (Java read/write asymmetry).
- B1: anti-replay `entropy` is signed `byte` in Java vs `u8` in Rust — values ≥128 diverge.
- B1: `ServerCommandZapPlayer`/`UnzapPlayer` ctor arg order reversed Java vs Rust (test-only).
- B1: null-vs-omitted JSON keys; `FieldCoordinate` as `[x,y]` array in commands vs `{x,y}` object;
  typed factory objects (Skill/Card/…) in Java vs name strings in Rust.
- **B4 (INVESTIGATE — potential real bug):** Java `SkillFactory` skill membership disagrees
  with Rust `SKILL_TABLE`: union 203 vs 200, BB2016 86 vs 58, General category 15 vs 29, and
  Yoink absent from Java BB2025. The earlier "mirrored" skill work pinned the Rust numbers; the
  Java side does NOT match. Either Rust's per-skill `editions`/`category` columns are wrong for
  many skills, or the two count differently (e.g. stat-increase/positional skills). Needs a
  dedicated audit of SKILL_TABLE editions+category vs Java SkillFactory registrations. The
  ported Java SkillFactoryTableTest was removed (had Rust-specific pins) pending this audit.

## Wave 2 (medium ports)
| B1 | ffb-protocol ~882 → ffb-common net/commands (137 classes, 815 methods) | DONE — ffb-common 1,044 green |
| B2 | ffb-model survivors (dialog/model/util/types/inducement/injury/factory) → ffb-common | RUNNING |
| B3 | ffb-model report/ 391 → ffb-common report JSON tests | RUNNING |
| B4 | ffb-mechanics unmirrored (~700) → ffb-server tests | RUNNING |
| B5 | ffb-client survivors ~1,698 → ffb-client-logic tests | RUNNING |
| B6 | ffb-server survivors ~896 → ffb-server tests | RUNNING |
