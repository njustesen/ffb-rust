# Replay processing parity (BACKLOG §O) — progress report, 2026-11-23

**Goal (user, 2026-11-22):** "check all the downloaded replays and make sure we can process all of
them; get all states and actions. Make sure Rust engine does the same as Java in that respect."
Done = `python -I scripts/replay_parity_check.py` prints DONE (2,224 unique FUMBBL replays, Java
complete, Rust per-sync states + action records byte-identical, both from manifest-stamped tools).

## Where it stands

| Phase | Status | Evidence |
|---|---|---|
| O1 Java processes every replay completely | ✅ 2,224/2,224 | run5, frozen `ffb-ai-replay-o1.jar`: 0 exceptions, 0 skipped syncs, 0 record errors, end check OK |
| O2 Rust per-sync states == Java | ✅ 2,224/2,224 | whole-corpus run `rust_states_all2` + `fieldModelAddWisdom` (`0896b494b`) |
| O3 Rust action records == Java | ✅ on 63/63 checked replays | fumbbl 10/10, partial+window 9/9, 44-replay sample 44/44 (`0d6966c27`) |
| O4 proof: frozen jar + exe over the whole corpus, manifest | ⏳ scheduled | night chain 2026-11-23 21:00 → 07:00 |

The done-check prints NOT DONE today only because `docs/replay_manifest.json` does not exist yet;
it is written only after the night run passes.

## What was built (all committed and pushed)

- **Java harness (`FumbblReplayParser`, ffb-ai; the engine itself untouched):** `--per-replay`
  outputs (`<id>.states`, `.actions.jsonl`, `.status.json`, `run_info.json`), canonical state form
  (keys sorted, hash-container arrays sorted), `ReplayStartDump` / `ReplayRoundTripCensus` oracles.
- **Rust `ffb-replay` bin:** `.ffbr` reader, canonical JSON writer with Java-exact number printing
  (`Double.toString`, minimal-json), and gates `canon-check`, `start-check`, `dialog-check`,
  `states`, `actions`, `double-check`.
- **Rust Java-JSON model, 1:1:** `Game.initFrom/toJsonValue` and the whole graph — GameOptions,
  ActingPlayer, TurnData, InducementSet, Team/Roster/RosterPosition/RosterPlayer,
  GameResult/TeamResult/PlayerResult, FieldModel and its elements, all 52 corpus dialog parameters;
  per-ruleset Java `SkillFactory` and exact `forName` lookups.
- **Rust replay pipeline, 1:1:** `startOfGame` (= `ClientReplayer.createGame`), `ModelChangeId`/
  `ModelChange`/`ModelChangeProcessor`, the enhancement family, `serverAddPlayer`, Zap/Unzap,
  `fieldModelAddWisdom`.
- **Rust action records, 1:1:** `GameStateSerializer`, report view, the `parseFile` record machine,
  and the MoveDecisionEngine (`findAllPaths` with a 1:1 `java.util.PriorityQueue`, `selectPlayer`,
  `selectMoveTarget`).

## Defects found on the way

| Where | Defect |
|---|---|
| harness | `serverAddPlayer` never applied (546 replays) and Zap/Unzap skipped — the replay client applies every replayable command |
| harness | replay-mode marker filter, coin-choice inducement reset, swallowed record errors |
| harness | `Game.toJsonValue()` arrays from HashMaps change order between JVM runs (canonical form fixed) |
| harness | move-target candidates in no fixed order (now sorted by x,y) |
| harness | newer-server features (Dwarfen Wisdom, Blessing of Nuffle, DRIVE_BONUS/LEADER re-roll properties) crashed or leaked; now dropped and reported on both sides |
| Rust model | special-rules representation (Brawlin' Brutes SPP) |
| engine (noted, not fixed) | a skill usage-type mismatch, recorded in BACKLOG §O iteration 13 |

Model changes the in-repo engine does not know (a newer FUMBBL server) are dropped on both sides
and counted: `actingPlayerSetInitialAdjacentPartnerIds` (~539k), `turnDataSetCheeringFansBlockAssist`
(~2.5k), and the value-level drops above.

## Tonight (2026-11-23 21:00 → 2026-11-24 07:00)

`scripts/replay_night_chain.ps1`, CPU mask 0x7FF: **run6** = frozen `ffb-ai-replay-o3.jar` over the
corpus, then **rust6** = frozen `ffb-replay-o3b.exe actions` against run6. If both are green the
loop writes `docs/replay_manifest.json`, runs the done-check, commits and pushes. Checklist: BACKLOG
§O "RESUME HERE". Chain log `D:/ffb_replay_parity/run6_java.chain.log`.
