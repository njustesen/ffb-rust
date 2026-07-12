# Phase AAD Plan — Close the Last Handler Gaps + Fix Documented Behavioral Approximations

*Written 2026-07-12, immediately after Phase ZV (17,331 tests, 29/32 `ServerCommandHandler*`
structs reachable from live dispatch, tracker's last `○` rows closed).*

## Context: where we were

File-level translation was ~100% complete and the command-hierarchy reconciliation effort
(Phases ZVA–ZVE) had wired 29 of 32 real `ServerCommandHandler*` structs into live WebSocket
dispatch. Every phase doc since named parity/integration testing against the real Java engine as
"the natural next phase" — but per instruction, parity testing stays deferred and unit tests stay
the priority. That left two categories of genuine, non-parity work still open:

1. 3 handlers still unreachable (`AddLoadedTeam`, `ApplyAutomatedPlayerMarkings`,
   `CalculateAutomaticPlayerMarkings`) — real, unit-tested structs blocked on their
   `InternalServerCommand*` wrappers carrying opaque `String` payloads instead of typed
   `Team`/`AutoMarkingConfig`/`Game` data.
2. A handful of documented behavioral approximations (hardcoded `false` stand-ins) inside files
   already marked `✓` — the same failure mode Phase ZW.0's audit found at much larger scale
   (644 `ffb-client-logic` rows that were actually thin stubs).

## Goal of Phase AAD

1. Wire the last 3 dormant handlers into live dispatch → 32/32 reachable.
2. Replace hardcoded/approximated behavior in `✓`-marked files with real Java-derived logic.
3. Spot-check the highest-TODO-count `✓` files for stale gaps vs. real bugs.
4. Unit tests only, no parity/integration testing.

## What actually happened

Three steps, run as described in `SESSION.md`'s Phase AAD entry (Steps 1 and 2 ran concurrently
as separate foreground agents in the same working directory, restricted to non-destructive git
usage given this repo's documented past incident with unrestricted parallel git access; Step 3 ran
afterward, solo):

- **Step 1** (`fe03b161`) — wired `AddLoadedTeam`/`ApplyAutomatedPlayerMarkings`/
  `CalculateAutomaticPlayerMarkings` into live dispatch by giving their `InternalServerCommand*`
  wrappers real typed fields and adding real JSON/XML parsing on the request side. **Result:
  32/32 handlers reachable.** Documented (not fabricated) follow-up: no dispatch channel exists
  yet from the 3 `FumbblRequest*` types to the internal-command queue, so their callers still
  discard the newly-parsed typed result — a narrower, separately-scoped gap.
- **Step 2** (`24d81084`) — fixed `ActingPlayer.isMustCompleteAction()` and `SwarmingLogicModule`'s
  bb2025 LINEMAN keyword check, both previously hardcoded `false`. Confirmed the two
  `unimplemented!()` bodies originally suspected as gaps (`pass_block_logic_module.rs`,
  `kickoff_return_logic_module.rs`) are actually correct 1:1 translations of Java's own
  `UnsupportedOperationException` throws — left untouched.
- **Step 3** (`01a8fded`) — audited the 5 highest-TODO-count `✓` files. 4 of 5 were stale
  comments over already-correct logic (tightened wording only). Found and fixed one real bug in
  `step_init_throw_team_mate.rs`: it was missing Java's `UtilRangeRuler` range-gate check, always
  advancing once a target coordinate was set regardless of distance.

**Total: 17,331 → 17,359 tests (+28), 0 failures.** No parity/integration testing performed, per
instruction.

## What's left after this phase (none of it "translation" anymore)

1. The `FumbblRequest*` → internal-command dispatch-tail gap named in Step 1 — a narrower
   follow-up (needs a `ServerRequestProcessor`-style dispatch channel for these 3 request types).
2. Parity/integration testing against the real Java engine (100-seed races) — still explicitly
   deferred, and still the natural next phase after this one.
3. A standing, separate product decision on whether to ever build headless/alt-UI equivalents for
   the 271 permanently-skipped Swing files (~31k LOC).
4. Live production infra wiring (real MySQL, real Jetty↔axum wire compatibility) beyond
   compile-time/unit-test level.
5. The one intentionally-`~` `UtilServerHttpClient.java` row (duplicate-of-`ReqwestHttpClient`
   rationale stands; untouched by this phase).
