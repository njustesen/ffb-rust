# Phase ZW Plan — True Client-Logic Translation + Server Closeout

*Written 2026-07-10, after Phase ZV (14,794 tests).*

## Context: where we actually are

The tracker's headline (2,898/2,933 = 98.8%) materially overstates progress. An audit found
that **all 644 `ffb-client-logic` rows are marked `✓` but their Rust files are 10-line
placeholder stubs** (e.g. `crates/ffb-client/src/client/ActionKeyBindings.rs` is an empty
struct while `ActionKeyBindings.java` is 191 lines of key-binding logic — not Swing). Only
~5 files in the `ffb-client` crate (~3k LOC, `network_encoder` etc.) contain real code, and
the crate has 31 tests against 649 files.

Honest accounting, by Java LOC (modules: common 80.0k, server 128.5k, client-logic 60.5k,
Swing app 9.4k, tools 0.7k = 279.0k total):

| Denominator | Translated today | % |
|---|---|---|
| Everything (279.0k LOC) | ~207k (common + server, minus ~87 `todo!` gaps) | **~74%** |
| In-scope¹ (~237.8k LOC) | ~207k | **~87%** |
| Tracker file-count (claimed) | 2,898/2,933 | 98.8% — **inflated by 644 stub-✓ rows** |

¹ In-scope = everything except genuinely-Swing/AWT rendering (client-logic packages
`dialog`, `ui`, `animation`, `layer`, `overlay`, `sound` ≈ 31.1k LOC; the AWT app ≈ 9.4k)
and build tooling (`ffb-tools`, 0.7k). Rationale below.

## Goal of Phase ZW

Bring the project to **100% of in-scope files genuinely translated 1:1, with unit tests**,
and make the tracker tell the truth. No parity work this phase (explicitly deferred).

**After Phase ZW: ~100% of in-scope LOC (~238k/238k), ~85% of all Java LOC (238k/279k),
~17,500 tests.** The remaining 15% (Swing rendering + tools) stays permanently skipped
with documented reasons — or becomes a later, separate decision.

## What counts as a "really good reason" to skip

Only: (a) AWT/Swing **rendering** — paint/layout/widget code with no headless equivalent;
(b) build tooling. Everything else in `ffb-client-logic` is client *logic* (state machines,
command handlers, report-to-text rendering, key bindings, model/util) and **is in scope**:
it is required for any future headless client, bot client, or new UI, and the Java dialog
*data* classes are already translated (`ffb-model/dialog`, 72 files) — their client-side
Swing rendering is what we skip.

Package triage of `ffb-client-logic` (644 files / 60,453 LOC):

| Package | Files | LOC | Verdict |
|---|--:|--:|---|
| `client/state` | 85 | 10,868 | **Translate** — client state machine |
| `client/report` | 211 | 9,187 | **Translate** — report→text rendering; styles become plain data |
| `client/` (root) | 31 | 5,622 | **Translate** (per-file triage; a few root files are Swing-bound) |
| `client/handler` | 27 | 1,659 | **Translate** — net command handlers |
| `client/net` | 3 | 794 | **Translate** |
| `client/util` | 11 | 858 | **Translate** (skip the JTable/Swing helpers found in triage) |
| `client/model`, `client/factory` | 5 | 352 | **Translate** |
| `client/dialog` | 170 | 17,993 | **Skip (—)** — Swing JDialog rendering of already-translated dialog data |
| `client/ui` | 69 | 9,316 | **Skip (—)** — Swing widgets/panels |
| `client/layer` | 13 | 1,971 | **Skip (—)** — pixel-layer painting |
| `client/animation`, `overlay`, `sound` | 19 | 1,833 | **Skip (—)** — AWT animation/audio playback |

Translate ≈ **373 files / ~29.3k LOC**. Skip ≈ 271 files / ~31.1k LOC (marked `—` with
reason, not `✓`).

## Sub-steps, in order

### ZW.0 — Tracker truth reset (script, ~½ session)
- `scripts/audit_client_stubs.py`: flag every tracker row whose Rust target is ≤12 lines
  but marked `✓`; rewrite status per the triage table (`○` for the 373 translate rows, `—`
  with reason for the 271 skip rows); regenerate the Progress Summary.
- Fix stale docs while at it: CLAUDE.md crate table (missing `ffb-server` crate),
  `docs/step_port/TESTING.md` (still references deleted `engine.rs`), SESSION.md header.
- Delete the misleading stub files for rows reclassified `○` (they hide gaps from grep) —
  or leave them and let translation overwrite; decide once the script lists them.

### ZW.1 — Server closeout: 35 `~` files → ✓, 87 `todo!` → 0 (~½–1 session)
Finish `ffb-server`/`ffb-engine` to true 100%:
- The 4 missing lower-level APIs: `GameState` step-stack reset, `SoundId` enumeration,
  `GameOptionId` enumeration, `SeriousInjuryFactory.forAttribute`.
- The 25 `ServerCommandHandler*`, 4 `TalkHandler*`, 5 `net/*Task`/servlet, 1 db-insert
  rows: translate the remaining Java bodies 1:1 against the existing seams
  (`mysql_async` behind the db registry, the mockable `HttpClient` trait, session
  managers). **Live** DB/HTTP connection config stays behind those seams — translated
  code + mock-based unit tests now; real-infra integration is a later, separate concern.
- Resolve or explicitly document the two-parallel-`ffb_protocol`-command-hierarchies
  split blocking full `ServerCommandHandlerFactory` delegation (unify if mechanical;
  otherwise a design note + tracking row — do not fork logic silently).
- Unit tests: mock-backed handler tests (command in → session/db/http calls + replies
  out), same style as the Phase ZV handler tests.

### ZW.2 — Client core translation (~1–1.5 sessions, the heart of the phase)
Translate in dependency order: `client/model` + `client/util` + `client/factory` →
`client/net` → `client/handler` → `client/` root (incl. `ActionKeyBindings`,
`ClientData`, `FantasyFootballClient` headless-izable core) → `client/state` (85 files,
BB2016/2020/2025 state machines).

**Status (2026-07-11): Batch A (model/util, 7 files), sub-phase ZW.2c, Batch C (`client/`
root), and Batch D (`client/state`, all 85 files) are done — `client/state/` is now 100%
complete.** `client/net` (3 files) and `client/handler` (27 files) are genuinely translated —
Batch B's blocker (no dispatch/serialization layer over the real `commands::` structs) was
closed by building `AnyClientCommand`/`AnyServerCommand` + a real
`NetCommandFactory::for_json_value` (see `TRANSLATION_TRACKER.md`'s Progress Summary, "Phase
ZW.2c"). `client/` root promoted `FantasyFootballClient` to a real hybrid struct; `client/state`
(85 files across 5 batches: interaction/value-types, plugin, logic root, logic editions,
then this final root batch — `ClientState`/`ClientStateFactory`/`IPlayerPopupMenuKeys`) is
fully translated and wired, with `ClientStateFactory::get_state_for_game` now the faithful,
tested ground-truth dispatcher (see `crate::state_dispatch` for the pre-existing, deliberately
coarser TurnMode-only helper it supersedes as ground truth — kept, not merged). **Recommended
next: `client/report/` (211 files, ZW.3)** — the only remaining major work before ZW.4 docs
closeout.
- Same ground rules as everywhere: one Java file → one snake_case Rust file (replacing
  the PascalCase stub), every method in order, no untraceable logic.
- Swing types that leak into logic signatures (e.g. `KeyEvent` constants, style enums)
  become plain data (ints/enums) — the same trick used for report styles; each such
  substitution gets a `// java: <type>` comment.
- **Unit tests (the priority):** per `ClientState*` file, a characterization test per
  public transition: given a game snapshot + incoming command/click-equivalent, assert
  resulting client state + emitted `ClientCommand`s (dice-free, so cheap to pin). Per
  handler: command in → model mutation + UI-event out. Target ≥3 tests/file average.

### ZW.3 — Client report renderers (~½–1 session) — DONE, 2026-07-11
211 `client/report` files translated (55 root + 32 bb2016 + 26 bb2020 + 39 bb2025 + 57
mixed), each a `ReportMessage` trait impl built 1:1 with styles as plain data. Unit tests
construct a representative report → call `render()` → assert exact emitted text runs +
style tags, per file (pinned per logical branch, mirroring the Phase ZU round-trip
convention). Required un-skipping `TextStyle`/`ParagraphStyle` (miscategorized as Swing
in ZW.0 — both are plain string-keyed enums) and giving `StatusReport` a real translation
first, replacing its one Swing sink with a headless `Vec<RenderedRun>` capture. See
`TRANSLATION_TRACKER.md`'s Progress Summary for full detail, including the small
`ffb-model` gaps filled along the way (`PlayerGender`/`PlayerAction` missing getters) and
the per-file `// java:` gap comments where the report data model doesn't retain enough
(e.g. `RollModifier` magnitude).

### ZW.4 — Docs & progress update (closeout) — DONE, 2026-07-11
- Tracker: 216 rows (211 `client/report/*` + 5 prerequisites) flipped to `✓`; Progress
  Summary and Session History updated with the real test-count delta (+893).
- SESSION.md phase entry added; `T3_COVERAGE.md` caveat re-confirmed (parity stays
  paused this phase, as planned).
- Progress claims below updated with the actual final numbers.

## Test budget

14,794 (post-ZV) → 15,647 (post-ZW.2c, actual) → **~17,500** projected after full ZW:
server closeout +117, ZW.2c (protocol JSON + net + handler, ~153 files) +707 (actual),
remaining client core (`client/` root ~30 files + `client/state` 85 files) +~700–900,
report +~800–1,000 (211 files × ~4), plus incidental model/protocol tests.
`cargo test --workspace` green is the gate for every commit; **no parity runs**.

## Progress estimates (the headline numbers)

| Milestone | Of in-scope LOC (~235.2k) | Of all Java LOC (279k) | Tests |
|---|---|---|---|
| Post-ZV | ~87% | ~74% | 14,794 |
| Post-ZW.2c (actual, 2026-07-10) | ~89% | ~75% | 15,647 |
| Post-`client/state/` complete (actual, 2026-07-11) | ~93% | ~79% | 16,412 |
| After full Phase ZW (actual, 2026-07-11) | **~100%** | ~85% | 17,305 |

What remains after ZW, permanently or pending a separate decision: Swing dialog/ui/layer
rendering (~31k), the AWT app (~9.4k), `ffb-tools` (0.7k) — plus the two big non-translation
workstreams this phase deliberately excludes: **re-establishing Layer-3 parity against
`driver.rs`** (harness + both stale `progress.html` dashboards predate the `engine.rs`
deletion) and live DB/WebSocket/HTTP integration wiring.
