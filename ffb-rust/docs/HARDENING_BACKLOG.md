# Hardening backlog (§Q): close the port's missing pieces and validation holes

Source: the port review of 2026-11-27 (three audits; findings re-checked in code). This file is the
ordered work queue for §Q. The loop works it top to bottom, one item per iteration. Each item has a
done-criterion; tick it only when that criterion is met and the change is committed and pushed.

**Status legend:** `[ ]` open · `[~]` in progress (note who/what below it) · `[x]` done (commit hash) ·
`[!]` blocked (reason + what decision is needed).

## RESUME HERE
Next item: **Q1.2** (Q1.1 code done, its mirror 330 run is queued for the night). Last certified build: cert9 (`docs/cert_manifest.json`); last view build: view15.

## Ground rules (every item)
- Java (`ffb-java/ffb/`) is the oracle. Fix Rust only. The harness (`ParityRunner.java`, `ffb-ai`) may be
  co-edited, always mirrored on both sides in the same commit. Never touch `C:\Users\Admin\niels\ffb`.
- Port 1:1: open the Java class first; tests are ported from the Java tests where they exist.
- Dice order is sacred: no change may add, drop or reorder an RNG call on the default path.
- Gate every change: `cargo test --workspace` green, then the item's own gate (below), then the
  standing smoke gate (`ffb-parity --home human --away human --edition bb2025 --tier 3 --agent heuristic
  --seeds 1-20` and the same for bb2020/bb2016 with a mixed squad) steps + reports identical.
- Freeze the exe (`target/release/deps/ffb_parity.exe` → `D:/ffb_star/bin/ffb-parity-qN.exe`) and the jar
  for any suite run. Never two runs of the same edition+matchup at once.
- Heavy suites (mirror 330 / cross 4,012 / star 4,012) only 21:00–07:00, pinned to ≤70 % of the cores
  (`start /AFFINITY 7FF`), run in the foreground in ~8-minute resumable chunks.
- Stage files explicitly (never `git add -u`/`-A`); another session edits ffb-model, ffb-parity
  (`main.rs`, `replay/*`) and ffb-server: leave its hunks unstaged. Push after every green item.
- A new agent choice changes games: after any agent change, re-run the affected suite, not just the gate.

---

## Q1 — Make "identical" mean identical (validation)

- [~] **Q1.1 Per-step verdict.** Code + tests done (`compare_lines`); smoke gates bb2025/bb2020/bb2016
  20/20 under the new verdict. Waiting: mirror 330 re-run (night, with Q1.2). `ffb-parity/src/comparator.rs`: a game fails on the first per-step
  `state_hash`/turn/half/active mismatch AND on a step-count mismatch, not only on the final hash.
  Keep the final-hash check. Done: unit tests for converge-after-diverge and extra-step cases; smoke gate
  green; mirror 330 re-run green (night).
- [ ] **Q1.2 Abort = red.** Parse `END_REASON` on both sides; anything other than `finished` is a red
  unless it is a known Java crash recorded in `docs/JAVA_DEFECTS.md` (then reported separately as
  JAVA-CRASH, not as a pass). `DIALOG_LOOP_CLEARED` is a red. Fix `cert_tally.py` to count them.
  Done: the 7 known aborts and 8 loop-clears from cert9 are each classified (Java defect or Rust/harness
  bug fixed).
- [ ] **Q1.3 Wider state hash.** Add to BOTH `ffb-model/src/util/state_hash.rs` and
  `ParityRunner.java` (same string format): PA; full player state incl. flags (rooted, confused,
  hypnotised, used-Pro, …) and the exact off-pitch state (banned, KO, BH, SI, RIP, missing, …);
  granted/temporary skills and properties; SPP and per-player game-result stats; inducement set and
  apothecaries left; bombs; acting player's declared action + has-blocked/passed/fouled/moved flags;
  TTM/KTM-used. Done: a cross-engine test that hashes the same fixture JSON in both engines; smoke gate
  green; mirror 330 green.
- [ ] **Q1.4 Client view in the certification.** Move the scratchpad view runner into
  `scripts/` (job lists → `FFB_CLIENT_VIEW_DIR` dumps → `client_view_check.py`), add a dump after game end
  (SPP, MVP, winnings, fan factor), compare ordered lists in order where Java's order is meaningful, and
  make `cert_tally.py` require `CLIENT VIEW n steps, n identical`. Done: one suite cell end-to-end.
- [ ] **Q1.5 Enforce coverage.** The coverage checklist runs for every gate (aggregate per suite, not per
  50-game gate); MISSING required items are reds at suite level; the skill ledger counts a skill only when
  it changed the outcome path (fired), not when offered. Done: suite-level coverage report in the cert.
- [ ] **Q1.6 Determinism gate.** Run Rust twice on the same seeds (fresh process each) and diff the
  JSONL; replace hash-ordered iteration on RNG/report paths with ordered maps (`player_at` tie,
  `util_server_cards.rs:245`). Done: 100 seeds × 3 editions identical across two runs.
- [ ] **Q1.7 Re-certify (cert10).** Freeze one exe + jar; mirror 330, cross 4,012, star 4,012, skill
  sweep with the strict verdict, wider hash, client view, aborts as reds. Fix every red 1:1. Done:
  `docs/cert_manifest.json` updated to cert10, all suites green on one build.

## Q2 — Exercise the branches the agents never take (both agents, mirrored)

All behind `--coverage-bias` (default off keeps cert behaviour); Rust `heuristic_agent.rs` /
`random_agent.rs` and `ParityRunner.java` changed together; zero extra draws where possible, else the
same draws on both sides. Each item: a 20-seed gate per edition where the mechanic exists, then a
coverage suite (`--coverage-bias`, mirror 330) at night.

- [ ] **Q2.1 Apothecary accept** (and the apothecary result choice both ways). Gate: apothecary roll
  path fires in all 3 editions.
- [ ] **Q2.2 Regeneration re-roll / Igor / Mortuary Assistant accept.**
- [ ] **Q2.3 Argue the call: decline branch; bribes: use branch.**
- [ ] **Q2.4 Kick-off placements:** Quick Snap, Solid Defence, High Kick, Perfect Defence, Charge pick
  real players. Depends on Q3.4 (Solid Defence selection).
- [ ] **Q2.5 Swarming placement** (remove the engine's hard-coded "place nobody" in
  `bb2025/kickoff/step_swarming.rs`, ask the agent as Java asks the client).
- [ ] **Q2.6 Multiple Block / block dice:** choose by value, use Pro / Consummate / single-die re-rolls.
- [ ] **Q2.7 MVP via Java's random path** (no first-eligible shortcut) and the post-game sequence.
- [ ] **Q2.8 Pregame purchases:** buy inducements, star players, mercenaries, cards, prayers within the
  petty cash / treasury; enable Underdog prayers. Depends on Q3.2–Q3.3.
- [ ] **Q2.9 Remaining fixed answers:** Blood Lust choices, Trickster pick-up, punt to crowd, Kick,
  Diving Catch, player-choice dialogs (High Kick, Arm Bar…), Piling On / Pile Driver decline, select
  skill by value, touchback choice, concession (needs Q3.8). List each dialog kind in the heuristic's
  scorer or document why it stays fixed.
- [ ] **Q2.10 Engine-side inline decisions** ("headless auto-…", 81 sites in 38 files): each one either
  becomes a prompt (as Java asks the client) or is documented as Java's own server-side behaviour.
  Includes Grab always used, bb2016 bribes skip argue, `USE_PREDEFINED_INDUCEMENTS`.

## Q3 — Port what is missing from the engine (1:1 from Java)

- [ ] **Q3.1 Overtime / sudden death / golden goal** (`bb2025/step_end_turn.rs:1399` "overtime check not
  yet ported"; `OVERTIME`, `OVERTIME_GOLDEN_GOAL`). Test with the overtime option on in both engines
  (harness option, Java line `UtilServerStartGame.java:234`). Gate: drawn games enter half 3 in all 3
  editions, steps + reports identical.
- [ ] **Q3.2 In-game inducement use:** route Wizard, Throw a Rock, Weather Mage, Master Chef, Fan Factor
  in `step_init_inducement.rs` (bb2025, bb2020, bb2016 card play). Gate: each inducement fires.
- [ ] **Q3.3 Cards (bb2016/bb2020):** build decks, buy, draw, play, deactivate (`handle_card`,
  `step_buy_cards_and_inducements.rs:249-343`, `PlayCard`). Gate: card play in both editions.
- [ ] **Q3.4 Solid Defence player selection** (`step_apply_kickoff_result.rs:834,888`).
- [ ] **Q3.5 bb2016 dodge/leap hooks:** Break Tackle and Arm Bar on dodge (`step_move_dodge.rs:91`),
  Diving Tackle on Leap (`step_jump.rs:41`).
- [ ] **Q3.6 bb2020 `is_wisdom_available`** (`ffb-mechanics/src/bb2020/game_mechanic.rs:152`) and the
  bb2025 grantable-skill check (`bb2025/game_mechanic.rs:167`).
- [ ] **Q3.7 Approximations:** Riotous Rookies shuffle (`bb2025/game_mechanic.rs:80`), bb2016
  `armour_is_broken` modifier total (`bb2016/stats_mechanic.rs:25`), `isValidMove` path validation
  (`step_init_moving.rs:79`), stalling rock-hit branch (`stalling_extension.rs:32,105`), Swoop hooks and
  blitz acting-player change (`step_swoop.rs:168,347`).
- [ ] **Q3.8 Game options Java reads but Rust ignores:** ASK_FOR_KICK_AFTER_ROLL, ALLOW_KTM_REROLL,
  ALLOW_CONCESSIONS, CHECK_OWNERSHIP, the 8 INDUCEMENT_*_MAX, INDUCEMENT_IGORS_COST.
- [ ] **Q3.9 Skill-behaviour structure:** bb2016 must not reuse bb2025 hooks where Java has its own
  bb2016 behaviour (Stab, Wrestle, Animosity, Shadowing, …: `skill_behaviour/registry.rs:247-310`); port
  the per-edition Java behaviours that differ. Gate: bb2016 skill sweep.
- [ ] **Q3.10 Re-measure dead steps** with the heuristic agent + coverage bias; update
  `DEAD_STEP_INVENTORY.md`; every still-dead step is either driven live or documented as unreachable in
  Java too.

## Q4 — Server and protocol (GUI tier)

- [ ] **Q4.1 Lossy decodes:** `ClientUseApothecaries` keeps `injuryDescriptions`, `ClientUseReRoll`
  keeps the source, `ClientInterceptorChoice` keeps the interceptor (`server_command_handler_factory.rs`
  ~990-1066).
- [ ] **Q4.2 The 33 undecoded ClientCommands** (BuyCard, SelectCardToBuy, PettyCash, Journeymen,
  SelectWeather, WizardSpell, UseBrawler/Chainsaw/Hatred/Igors/Fumblerooskie/TeamMatesWisdom,
  UseProReRollForBlock, UseConsummateReRollForBlock, UseSingleBlockDieReRoll, UseMultiBlockDiceReRoll,
  UseReRollForTarget, SynchronousMultiBlock, Set/UnsetBlockTargetSelection, SkillSelection,
  IllegalProcedure, KickTeamMate, ThrowKeg, PileDriver, FieldCoordinate, …).
- [ ] **Q4.3 Per-edition games on the server** (drop the forced `Bb2025` in `join_approved.rs:415`).
- [ ] **Q4.4 Missing dialogs and ModelChange kinds** (31 never produced: cards, prayers/enhancements,
  hatred, chomp, acting-player dodging/jumping/leaping/strength, blitz state, …; dialogs
  BlockRollPartialReRoll, BuyInducements, Journeymen, PilingOn, TeamSetup, UseIgors,
  UseMortuaryAssistant). Done: whole-game fallbacks 0 on a coverage-bias GUI run.
- [ ] **Q4.5 gui_check hardening:** fail on ignored commands; all three editions; all teams (add setups);
  a scripted "human" client path that answers every dialog kind.
- [ ] **Q4.6 Optional server features** (ask before starting): persistence/resume, turn timer, DB login,
  FUMBBL mode, spectators/replays check.

## Q5 — Rules-level replay parity (§N)

- [ ] **Q5.1** Drive the Rust engine from FUMBBL replays: dice tape + decision tape → Rust engine →
  compare per-sync state with the recorded game (bb2025 first). Plan in `docs/REPLAY_PARITY_PLAN.md`.
- [ ] **Q5.2** Extend to bb2020 replays; count rules-level identical replays.

## Q6 — Hygiene

- [ ] **Q6.1** Regenerate `TRANSLATION_TRACKER.md` from the filesystem (77 ✓ rows name missing files;
  dead bb2020/bb2016 step files marked as such).
- [ ] **Q6.2** Remove stale "not ported" comments found by the review (InducementTypeFactory, CardDeck,
  BlitzTurnState, ArmorModifierFactory, Saboteur, EndGame generators, …).
- [ ] **Q6.3** Dead code: `ffb-client` (old protocol, wrong URL), `ffb-server/src/net/wire.rs` and
  `wire_prompt.rs`; either wire them to the real protocol or delete (ask first).
- [ ] **Q6.4** Port the missing Java unit tests for steps/generators without tests (38/120 generators,
  ~50 steps).

---

## Log
(newest first: date · item · commit · one line)

- 2026-11-27 · Q1.1 · (this commit) · verdict now fails on any per-step hash/post-hash/turn/half/active
  difference, a step-count difference or a different start state; 5 unit tests; smoke 3×20 green.
