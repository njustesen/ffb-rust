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

## Feasibility analysis (2026-11-21): FEASIBLE, with three design changes

Evidence: 18 real replays (10 finished games in `D:/ffb_replays/fumbbl`, 5 unfinished in `partial/`,
3 from April 2026 in `window/`), a ground-truth dice experiment on 6 of our own Java games
(1,398 die calls), and two read-only code audits (decision inference; Rust gaps).

| Question | Finding | Verdict |
|---|---|---|
| Can we get replays? | WebSocket downloader works; about 10-15 s per replay at the 10 s courtesy delay. FUMBBL today is about 99% BB2025 (BB2020 about 1 per 100 ids, BB2016 not seen). The newest ids are games still running: drop them (fetch script checks for `fumbblResultUpload`). Replays from 2025 and April 2026 still download. | YES |
| Engine version drift | Our oracle is **stock upstream 3.2.0 (2026-03-21)**: 6 engine files compared byte-for-byte all match upstream as of Feb/Mar 2026. Live FUMBBL now runs **3.4/3.5**, about 330 server and 140 common commits ahead. November replays carry a model change our Java lacks (`actingPlayerSetInitialAdjacentPartnerIds`, about 250 per game) plus 3 unknown options, and the Rules changes include e.g. Really Stupid (11-19) and Jump Up (11-21). **April 2026 replays carry no unknown report or model-change id**, but sit after about 27 narrow server commits (Weather Mage duration, confusion on stand-up, B&C bombs, touchback edge cases, timeout option). | Drift does **not** invalidate Java-vs-Rust: Rust replays only what our Java consumed, so the comparison is exact either way. Drift only **shortens the human-followed part** of a game: at the first rule difference our Java leaves the replay's path. From there the follower can stop, or continue on the heuristic agent and seeded dice, and parity stays valid. It also adds triage noise (follower bug vs drift). **Prefer replays from 2026-03-22 to about 04-01** for long human prefixes; current games are usable with shorter ones. |
| Decisions recoverable? | The replay records **every server dialog** (`gameSetDialogParameter`, 26 kinds) and every move square. About 99% of 600-900 decisions per game are mechanically recoverable. Rule-derived mappings are needed for the re-roll source (a report says "Leader" after a failed MASCOT), prayer counts and apothecary type. Wizard fireball centre is only an equivalence class. Timeouts are timer-driven. | YES |
| Dice recoverable? | Ground truth: **98.8%** of die values appear in the reports (unordered upper bound). A naive flat "next report die" stream gets only **69%**: report order is not roll order, and echo fields repeat dice (`blockChoice.blockRoll`). A **source-to-field table** (DiceRoller method + step → report field), learned automatically and cross-validated across matchups, gets **94.8% strict**. The rest is table detail (StepModifier lumps several skills; Team Captain, Argue the Call) plus outcome-only randomness (MVP pick, random-player events, prayer, cheering shuffle), which can be inverted from the reported outcome. | YES, with a hand-written extraction table plus outcome inversion. Never a flat stream. |
| Java shuffles | 11 `Collections.shuffle` sites; FUMBBL's are unseeded. | Needs a permutation record on the tape (inferred from the outcome). |
| Rust consumer | **No Java-JSON loader** (L). **Team holds no Roster**, and the engine re-reads static rosters at about 6 sites, so a FUMBBL inline roster is never found (needs a runtime roster registry, M). **No recorded-decision agent, and no ClientCommand→Action map** (TapeAgent L; record in ParityRunner's prompt vocabulary, which already pairs 1:1 with `AgentPrompt`). **No concession / illegal-procedure handling** (M). bb2025 star/merc/staff purchases unsupported (L, avoided by starting after the pregame). `askForKickAfterRoll` never wired (S). Replay mode must use **factory defaults + the replay's options, not `BASELINE_SETUP_OPTIONS`** (S). Dice tape S; shuffle tape S-M; raw stats (don't re-apply +MA/+AG) S. Skills: bb2020 89/89 resolve in both engines; bb2025 needs a Java-matching name filter (Consummate Professional, Dwarfen Grit, I'll Carry You). | Work, no wall |

**Design changes this forces (supersede the sketch below where they differ):**
1. **Replay window:** pick replays from 2026-03-22 to about 04-01 (`fetch_fumbbl_replays.py --start-id`
   around 1,881,000; ids run about 250-350/day). Before N1, verify the window with the follower's
   fidelity check on 2-3 of them.
2. **Dice:** the follower serves each die from a **per-source queue** (`DiceRoller` method + calling step
   → report kind + field), not a flat report stream, plus an outcome-inversion rule per random pick
   or shuffle. The tape records what was actually served. The experiment scripts are in
   `scripts/replay_feasibility/`.
3. **Start after the pregame** for the first slice: the tape begins at the first kick-off. Pregame
   purchases, stars and mercenaries are a later slice. Skip replays that end in a concession until
   Rust has concession handling.

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
