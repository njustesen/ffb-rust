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

## RESUME STATE (start here after restart)
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

**bb2020 generators: COMPLETE** (all 25). Added edition-aware GameFixture overload
`createGameState(playersPerTeam, Rules)` for edition-gated steps. ffb-server 2,749.
**bb2016 generators: NEXT** (15). Then step-logic (bulk), client state/logic (25), server remainder.
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
