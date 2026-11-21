# §N — FUMBBL replay parity (PLANNED 2026-11-21; starts after §M's cert8 closes)

**Goal (user, 2026-11-21):** download real FUMBBL games (start with 10), drive the Java engine
and the Rust engine through each one, and compare every per-step state hash. Human coaches field
wild, varied rosters (advanced players, injuries, inducements, stars) and play nothing like our
bots (long paths, fouls, passes to arbitrary squares, real setups, risky re-roll choices). The
existing suites cannot reach those situations.

Java is still the truth: a red is fixed in Rust 1:1. The Java **engine** is never changed. The
harness (`ffb-ai`, including the new replay code) is ours.

## What a FUMBBL replay is (measured on replay 1832851, BB2020 Dwarf v Dark Elf)

- `FumbblReplayDownloader` (ffb-ai, already in the repo) speaks to `ws://fumbbl.com:22223/command`
  like the real client, so the Anubis filter on the HTTP pages does not block it. It writes
  `<id>.ffbr` (gzip JSON).
- Content: one `serverGameState` (the full `Game`: both teams with their **inline roster**,
  FUMBBL numeric position ids, players with gained skills and lasting injuries, and the
  `gameOptions` as played), then about 3,200 `serverModelSync` commands. Each sync is a list of
  model changes plus a list of reports.
- **No client commands and no RNG seed.** Coach decisions must be *inferred* from what changed
  (`FumbblReplayParser` already infers about 20 decision kinds for ML training). Dice exist
  only as values inside reports: `blockRoll [2,5]`, `injury.armorRoll/injuryRoll/casualtyRoll`,
  `kickoffScatter.rollScatterDirection/Distance`, `scatterBall.rolls`, `goForItRoll.roll`,
  `kickoffResult.kickoffRoll`, `weather.weatherRoll`, `cheeringFans.rollHome/rollAway`, and so on.
  Some dice appear only as an outcome: `coinThrow` has heads/tails, not a die.

## Architecture: Java follows the replay; Rust follows Java's tapes

One component interprets FUMBBL data, and it lives on the Java side. Rust never reads a FUMBBL
replay. It replays exactly what Java consumed, which keeps the comparison the same kind of
comparison as every existing suite.

```
 .ffbr ──► Java ReplayFollower ──► init.json      (the replay's Game at kick-off of the match)
              │  Fortuna delegate   dice.tape      (sides,value per die call, in call order)
              │  follower agent     decisions.tape (prompt kind + answer, in order)
              │                     java.jsonl     (per-step state hash, as today)
              │                     java reports   (ReportSink, as today)
              └─ fidelity check: Java's reports == the replay's reports, sync by sync
                                         │
 Rust ffb-parity --tape <dir> ◄──────────┘   GameRng tape mode + TapeAgent
              └─► rust.jsonl + reports ──► the existing hash + report diff
```

Both engines already funnel every die through one seam: Java `Fortuna.setDelegate(...)`, which
ParityRunner already uses for Xoshiro, and Rust `GameRng::die(sides)`. A tape at that seam needs
no engine change on either side.

## Phases

**N0. Prep (done 2026-11-21, while cert8 runs):**
- `scripts/fetch_fumbbl_replays.py` uses `java -cp` on a private jar copy
  (`D:/ffb_replays/ffb-ai-replay.jar`), never Maven, so a cert jar is never rebuilt or locked.
- It waits ≥ 10 s between every contact with fumbbl.com, one connection at a time (user rule).
- It walks ids downward from the newest. Default batch: 5 BB2020 + 5 BB2025.
- Replays go to `D:/ffb_replays/fumbbl/` plus `manifest.jsonl`, **outside git**, because they hold
  FUMBBL coach and team names.
- `scripts/replay_inventory.py` (offline) lists rulesets, rosters, skills, game options and
  report kinds, and writes one sample per report kind.

**N1. Java start-of-match from a replay (harness only).** Build a headless `GameState` from the
replay's `serverGameState` through `NetCommandFactory`, as `FumbblReplayParser` already does.
Push the edition's start-game sequence. Gate: Java's pregame reports (fan factor, weather, petty
cash, inducements) equal the replay's when fed the replay's dice.

**N2. Java ReplayFollower (harness only; the main work).**
- *Dice:* a Fortuna delegate pops the next value from a queue built from the replay's report
  stream, in report order.
  - A per-report-kind extraction table says which fields hold dice and in what order.
  - The die's sides are checked against the field (d6/d8/d16/block).
  - Outcome-only dice (coin toss, random-player picks) get an inference rule, or the follower
    stops there with a named reason.
- *Decisions:* a follower agent answers each prompt by looking ahead at the replay.
  - Reports cover: `playerAction`, `blockChoice`, `reRoll`, `skillUse`, `apothecaryChoice`,
    `receiveChoice`, …
  - Model changes cover: path squares from `fieldModelSetPlayerCoordinate`, pushback and
    follow-up squares, setup placements, the kick target.
  - The inference rules in `FumbblReplayParser` are reused.
- *Fidelity check:* after each sync, Java's emitted reports must equal the replay's. The
  first mismatch ends the followed prefix and gets classified:
  - follower bug, which we fix;
  - unsupported decision, which goes on the queue;
  - FUMBBL-server vs in-repo-engine version drift, which is reported, never "fixed" in the engine.
- *Output:* the tapes above, written only for the followed prefix.
- Gate: at least 1 of the 10 is followed to the final whistle; every other one has a named
  stopping cause.

**N3. Rust consumer.**
- (a) Port 1:1 the Java JSON `initFrom` for `Team`, `Roster`, `RosterPosition`, `RosterPlayer`,
  `GameOptions`, and whatever inducement and star state the pregame needs. Rust currently has
  no Java-JSON `Game` loader; `ServerCommandGameState` falls back to serde with Rust names.
- (b) `GameRng` tape mode: pop `(sides, value)`. A sides mismatch or an empty tape is a
  divergence with `FFB_DIE_AT`-style context.
- (c) `TapeAgent`: pop the next decision and check that the prompt kind matches, else report a
  divergence; then convert the answer to an `Action`.
- (d) `ffb-parity --tape <dir>` writes `rust.jsonl` plus reports in the format the existing
  diff tools read.
- Gate: `cargo test --workspace` green; a synthetic tape recorded from a bot game, run through
  tape mode on both sides, matches bit for bit before any FUMBBL tape is tried.

**N4. Parity on the 10.**
- Diff state hashes and reports over Java's followed prefix, and fix Rust 1:1 (statediff,
  `FFB_DIE_AT`, probes in both engines).
- Gate: 10/10 identical over the followed prefix.
- Also record coverage gained over the bot suites: new report kinds, skills, actions and
  modifier names.

**N5. Scale.** 100 replays across BB2020 and BB2025 (BB2016 if FUMBBL still has any), then a
`replay` suite in `cert_chain.ps1`.

## Risks, with the mitigation already chosen
- **Engine version drift:** FUMBBL's live server is newer than `ffb-java`, so Java can't follow
  every replay. Prefer the newest replays, measure the drift through the fidelity check, and
  never patch the engine to match.
- **Hidden dice:** an inference rule or a named stop. The stop rate is the N2 metric.
- **Inline FUMBBL rosters and numeric position ids:** the 1:1 JSON loader (N3a) takes the
  roster from the replay instead of `data/rosters`. Every skill name must still resolve in Rust
  (`check_skill_names.py` logic).
- **FUMBBL game options** (e.g. `pilingOnUsesATeamReroll`, `clawDoesNotStack`): Rust must honour
  every option in the JSON. The inventory lists them, and N3 adds a test per non-default option.
- **Inducements, stars and mercenaries bought pregame:** they ride on the decision tape. Start
  N2 with whatever the 10 contain and count what is unsupported.
- **Concessions and timeouts:** the tape ends there, and the parity prefix ends with it.
- **Resources:** all daytime-light. Separate jar copy, separate Rust target dir; nothing touches
  the frozen cert exe or jar or the overnight window.
