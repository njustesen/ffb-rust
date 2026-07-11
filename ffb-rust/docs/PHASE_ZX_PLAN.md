# Phase ZX Plan — Close the Last `ffb-server` Handler Gaps

*Written 2026-07-11, immediately after Phase ZW (17,305 tests).*

## Context: where we were

Phase ZW brought `ffb-client-logic` to 100% of in-scope files genuinely translated. What
remained, per a fresh scoping pass done for this phase (not just prior-session memory):

1. **11 `ffb-server` handlers** (`crates/ffb-server/src/handler/server_command_handler_*.rs`)
   each carrying a `todo!("Phase ZV: ...")` naming a missing subsystem.
2. 271 permanently-skipped Swing UI files — confirmed genuinely AWT/Swing-only, no portable
   logic left inside them, correctly out of scope. Not touched this phase.
3. Two parallel protocol command hierarchies (`ffb-protocol/src/commands/` vs
   `client_commands`/`server_commands`) — a known, unresolved wiring/design split. Doesn't add
   new translated Java LOC (the "real" 131-file hierarchy already exists, unit-tested); left as
   documented future work, not this phase.

Of these, item 1 was the only genuine, boundable "translate more Java" gap. Scoping found
that most of the DB/HTTP primitives these 11 handlers need already existed
(`FumbblRequestCheckAuthorization`, `DbPasswordForCoachQuery`, `ServerRequestProcessor`) —
the real net-new work was ~825-925 Java LOC across 5-6 classes: `RosterCache`, `TeamCache`,
`UtilServerReplay`, `UtilServerStartGame`, and a `GameCache` extension.

## Goal of Phase ZX

Translate those 5-6 subsystem classes (unit-tested, no parity/live-infra testing) and wire
as many of the 11 blocked handlers as that genuinely unblocks. No target of "11/11" was
assumed going in — the plan explicitly called for reporting honestly on any that remained
blocked and naming the specific missing subsystem, rather than forcing false completions.

## What actually happened

**Steps 1-2 (leaf classes + `GameCache` extension):** translated cleanly. `RosterCache`,
`TeamCache`, `UtilServerReplay`, `MarkerLoadingService` all landed as real, tested
translations. `GameCache` gained `add_team_to_game`, `close_game`/`remove_game`,
`queue_db_update`, `queue_db_delete`, `query_from_db`, `queue_db_player_markers_update` —
all calling into already-translated `db/query`/`db/update`/`db/delete` functions, following
the established explicit-dependency-injection convention (no `FantasyFootballServer`
god-object handle added to `GameCache`). Discovered along the way: this crate's `Player`/
`Team` model had already collapsed Java's `ZappedPlayer`/`RosterPlayer` polymorphism into one
`Player` struct, so `addTeamToGame`'s `instanceof` reconciliation block had nothing left to
port. Also discovered a second pocket of fake-✓ stub duplicates — `ffb-engine`'s
`roster_cache.rs`, `team_cache.rs`, `util/util_server_replay.rs`,
`util/marker_loading_service.rs` were 10-line `todo!()` placeholders from the Phase ZT infra
sweep, now superseded by the real `ffb-server` translations. Left in place (not deleted) to
avoid scope creep — flagged as a cleanup candidate.

**Step 3 (`UtilServerStartGame`):** translated `joinGameAsPlayerAndCheckIfReadyToStart`,
`sendServerJoin`, `sendUserSettings`, `startGame` (its `addDefaultGameOptions` was already
translated in a prior phase, in `ffb-engine`).

**Step 4 (wire the 11 handlers):** **3 of 11 closed** —
`ServerCommandHandlerCloseGame`, `ServerCommandHandlerAddLoadedTeam`,
`ServerCommandHandlerFumbblTeamLoaded` are now fully wired, `todo!`-free, with real tests.

**8 of 11 remain blocked**, each narrowed to a more specific `todo!` than before:
`ServerCommandHandlerJoin`, `ServerCommandHandlerJoinApproved`,
`ServerCommandHandlerJoinReplay`, `ServerCommandHandlerReplay`,
`ServerCommandHandlerReplayLoaded`, `ServerCommandHandlerScheduleGame`,
`ServerCommandHandlerUploadGame`, `ServerCommandHandlerFumbblGameChecked`. The common root
causes, none of which this phase's scope covered:

- **No XML→`Team` roster deserializer.** `RosterCache`/`TeamCache` (this phase) return raw
  XML text rather than parsed `Roster`/`Team` objects — there's no SAX-equivalent
  `XmlHandler` in this crate. The client side has a separate, already-translated JSON-based
  roster data pipeline (`ffb-model/src/data/loader.rs`), but that's a different pipeline from
  the standalone-mode disk-XML one these handlers need.
- **No server-side command redispatch sink.** `ServerCommandHandlerFactory`/
  `ServerCommunication`'s dispatch loop that would let a handler enqueue a follow-up command
  isn't built yet.
- **No replay/command-log playback engine.** `Replayer`/`ServerReplay` registration is a
  documented no-op in `UtilServerReplay::start_server_replay` (this phase); the handlers that
  need to actually *replay* a game have nothing to call into.
- **No step-stack + `EndGame` sequence dispatch** for `ServerCommandHandlerUploadGame`'s
  game-conclusion path.

## Estimated progress after Phase ZX

- **In-scope Java LOC:** effectively unchanged from Phase ZW's ~100% — this phase's ~900 LOC
  of net-new translation is a rounding error against the ~236k in-scope total, and 8 handler
  bodies remain partial (already counted as in-scope, not fully done).
- **`ffb-server` handler gaps:** 11 → 8 (down from 11 at the start of this phase, 35 before
  Phase ZW.1).
- **Tests:** 17,305 → 17,357 (+52).
- **What's left to reach "0 known handler gaps":** build one or more of the four
  infrastructure pieces named above. Each is a bounded, well-named future phase in its own
  right (e.g. "Phase ZY: XML roster deserializer" or "Phase ZY: replay playback engine") —
  not attempted here to avoid scope creep beyond what this phase's plan committed to.
- **Everything else** (271 permanently-skipped Swing files, the two protocol hierarchies,
  parity testing, live DB/WebSocket/HTTP wiring) is unchanged from Phase ZW's accounting —
  see `PHASE_ZW_PLAN.md` for that reasoning.
