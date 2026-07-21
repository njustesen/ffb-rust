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
