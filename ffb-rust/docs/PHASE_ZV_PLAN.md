# Phase ZV Plan — Command-Hierarchy Reconciliation + Last Tracker Gaps

*Written 2026-07-12, immediately after Phases AAA/AAB/AAC (17,275 tests, all 11
originally-deferred `ffb-server` handlers closed, 0 remaining).*

Note: this is unrelated to old `todo!("Phase ZV: ...")` placeholder comments left in early
`ffb-server` handler stubs before the actual ZW→ZZ/AAA-AAC lettering was decided — those were
generic "a future phase" markers, all resolved by Phase ZW.1 onward. This is a fresh, separately
scoped phase reusing the same next-available letter.

## Context: where we were

File-level translation was ~100% complete (2,624/2,628 in-scope files `✓`, only 3 `○` rows left
in the whole ~2,970-row tracker, 342 permanently-skipped Swing rows unchanged). But "every file
exists" isn't the same as "the system behaves like Java's": `ServerCommandHandlerFactory::
handle_command`, the live WebSocket dispatch entry point, decoded every incoming message as
`ffb_protocol::client_commands::ClientCommand` — a hand-rolled ~35-variant wire enum — straight
into an `Action` enum for the step engine. Only `ClientPing` (and partially `Join`) reached a real
`ServerCommandHandler*` struct. The other ~30 handlers were fully translated and unit-tested
against the *genuine* 1:1 Java command mirror (`ffb_protocol::commands::*`, 131 files) but
structurally unreachable from production traffic — a gap the factory's own doc comment already
named ("Known gap (Phase ZV)") and `docs/PHASE_ZX_PLAN.md` flagged as item 3, unscoped.

## Goal of Phase ZV

1. Wire as many of the ~32 dormant `ServerCommandHandler*` structs into live dispatch as
   genuinely possible without fabricating wire payloads that don't exist — extending
   `ClientCommand`/`AnyInternalServerCommand` with real variants mirroring the `commands::*`
   shapes, one handler-family batch at a time (5 batches, sequential since they share one match
   statement), with a factory-level unit test per handler proving the real struct runs.
2. Close the tracker's last 3 `○` rows (2 genuinely translatable, 1 mis-marked and actually
   permanent-Swing).
3. No parity/integration testing (deferred, per instruction). Unit tests only.

## What actually happened

**5 sequential batches**, each a foreground agent, committed directly to `main` (not parallel
worktrees — the shared `server_command_handler_factory.rs` match statement and
`client_commands::ClientCommand` enum would conflict):

- **ZVA** (`d1676c77`) — Talk/CloseSession/TransferControl/RequestVersion/PasswordChallenge
  (`ClientCommand` variants) + DeleteGame (internal command). Surfaced and fixed 2 pre-existing
  `Send`-future bugs (a `MutexGuard` held across `.await`; a blocking `reqwest` client built
  inside an async context) — both only reachable once these handlers actually ran inside the
  `tokio::spawn`'d `dispatch_loop` for the first time. Tests: 17,275 → 17,286.
- **ZVB** (`0eb0cf95`) — the 12-handler sketch/marker family. 10 fully wired; 2
  (`ApplyAutomatedPlayerMarkings`, `CalculateAutomaticPlayerMarkings`) left as documented no-ops
  since their internal-command payload (`AutoMarkingConfig`/`Game`) has no serde decode format on
  the wire — a real, separately-scoped follow-up, not fabricated. Added `LazyReqwestHttpClient`
  (build the blocking client per-call rather than eagerly) to fix the same class of bug as ZVA.
  Tests: 17,286 → 17,306.
- **ZVC** (`43981a32`) — the 4-handler replay family (`JoinReplay`, `Replay`, `ReplayLoaded`,
  `ReplayStatus`), all fully wired, reusing the factory's existing `ReplaySessionManager`/
  `ReplayCache`/`ServerReplayer` instances (the replay-playback engine itself was already wired
  end-to-end in Phase AAB — this batch only made it reachable from live traffic). Tests:
  17,306 → 17,310.
- **ZVD** (`cc2b4e38`) — the 7-handler game-management family. 6 fully wired
  (`FumbblTeamLoaded`, `FumbblGameChecked`, `ScheduleGame`, `CloseGame`, `UploadGame`,
  `UserSettings`); `AddLoadedTeam` left as a documented no-op (its internal command carries no
  `Team` payload on the wire). Added `ServerCommunication::from_parts` so `CloseGame` could get a
  non-circular handle back into dispatch. Tests: 17,310 → 17,318.
- **ZVE** (`c4a1470d`) — finished `Join` (a re-join sent mid-session now really calls
  `join_handler`, matching Java; the very first join is still special-cased in
  `command_socket.rs` before enqueue — a Rust-only optimization, not a Java behavior divergence)
  and wired `SocketClosed` directly from `command_socket.rs`'s disconnect cleanup, replacing a
  bare `remove_session` call that had been silently bypassing the real handler's
  sketch-cleanup/leave-broadcast/replay-handoff side effects. Tests: 17,318 → 17,321.

**Result: 29/32 real `ServerCommandHandler*` structs reachable from live dispatch**, up from ~2 at
the start of this phase. The 3 stragglers (`AddLoadedTeam`, `ApplyAutomatedPlayerMarkings`,
`CalculateAutomaticPlayerMarkings`) are real and independently unit-tested, blocked on one
genuinely narrower, separately-scoped gap: their internal commands carry payloads
(`AutoMarkingConfig`/`Team`) that have no typed wire encoding yet. Not forced, not invented.

**Tracker closeout** (`8e930f99`): translated `LogicPluginFactory.java` (reflection-based
`Scanner` substituted with explicit registration, matching the `ReportMessageType::report_id()`
precedent from Phase ZW.3) and `UtilClientTimeout.java` (unblocked now that `StatusReport` is
headless, also since Phase ZW.3). Reclassified `UserInterface.java` (`extends JFrame`, genuinely
Swing) from a mis-marked `○` to the correct `—`. **This closes the last `○` row in the entire
tracker** — every in-scope file is `✓` except the one intentionally-`~` `UtilServerHttpClient`.
Tests: 17,321 → 17,331.

**Total: 17,275 → 17,331 tests (+56), 0 failures.** No parity/integration testing performed, per
instruction.

## What's left after this phase (none of it "translation" anymore)

1. The 3 remaining unreachable handlers named above — needs typed wire payloads for
   `AutoMarkingConfig`/`Team`, a narrower follow-up.
2. A standing, separate product decision on whether to ever build headless/alt-UI equivalents for
   the 271 permanently-skipped Swing files (~31k LOC).
3. Parity/integration testing against the real Java engine (100-seed races) — the natural next
   phase, explicitly deferred through this one too.
4. Live production infra wiring (real MySQL, real Jetty↔axum wire compatibility) beyond
   compile-time/unit-test level.
