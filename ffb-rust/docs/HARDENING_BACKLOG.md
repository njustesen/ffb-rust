# Hardening backlog (§Q): close the port's missing pieces and validation holes

Source: the port review of 2026-11-27 (three audits; findings re-checked in code). This file is the
ordered work queue for §Q. The loop works it top to bottom, one item per iteration. Each item has a
done-criterion; tick it only when that criterion is met and the change is committed and pushed.

**Status legend:** `[ ]` open · `[~]` in progress (note who/what below it) · `[x]` done (commit hash) ·
`[!]` blocked (reason + what decision is needed).

## RESUME HERE
Next item: **Q2.2** (regeneration / Igor / Mortuary Assistant accept). Q1.7 (cert10) waits for the night and for Q2.x/Q3.x
that change games; run the Q1 night queue first (mirror 330 + view suites on a frozen exe). Parked with notes: Q1.2b (punt → throw-in
stall), Q1.2c. Night queue: mirror 330 with the Q1.1–Q1.3 verdict + hash (freeze a q1 exe + jar first).
Q1.1's mirror 330 run is queued for the night (expect snotling cells red until Q2.5). Last certified build: cert9 (`docs/cert_manifest.json`); last view build: view15.

## Ground rules (every item)
- Java (`ffb-java/ffb/`) is the oracle. Fix Rust only. The harness (`ParityRunner.java`, `ffb-ai`) may be
  co-edited, always mirrored on both sides in the same commit. Never touch `C:\Users\Admin\niels\ffb`.
- Port 1:1: open the Java class first; tests are ported from the Java tests where they exist.
- Dice order is sacred: no change may add, drop or reorder an RNG call on the default path.
- Gate every change: `cargo test --workspace` green, then the item's own gate (below), then the
  standing smoke gate (`ffb-parity --home human --away human --edition bb2025 --tier 3 --agent heuristic
  --heur-classes all --seeds 1-20 --no-abort`, plus bb2020 orc-skaven and bb2016 dwarf-elf at
  `--heur-scale 1.0`) steps + reports identical. **Always pass `--heur-classes all`**: it is what every
  cert suite runs; without it the heuristic runs with no classes (a config no suite certifies).
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
- [x] **Q1.2 Abort = red.** (see Log) `game_end` now carries `end_reason` (+ Java `loops_cleared`);
  any non-`finished` end or a cleared dialog loop fails the game; Java exceptions print as JAVA-CRASH.
  cert9 classification (3 suites): SWARMING loop clears 3,106 (harness has no Swarming answer → Q2.5,
  every snotling/swarming game is now red until then), PUNT_TO_CROWD loop clears 8 (harness gap →
  Q2.9), java_crash 5 (JD-002 `armorRoll` null, stock Java), stuck_step 2 → Q1.2a, Q1.2b.
  **Q2.5 and Q2.9 (punt) must land before Q1.7.** Original text: Parse `END_REASON` on both sides; anything other than `finished` is a red
  unless it is a known Java crash recorded in `docs/JAVA_DEFECTS.md` (then reported separately as
  JAVA-CRASH, not as a pass). `DIALOG_LOOP_CLEARED` is a red. Fix `cert_tally.py` to count them.
  Done: the 7 known aborts and 8 loop-clears from cert9 are each classified (Java defect or Rust/harness
  bug fixed).
- [x] **Q1.2a Stuck step: bb2016 ogre mirror** (seed 73, scale 0): both agents' TTM target rule
  (thrower ±3 squares, clamped) produced a square with no TTM passing distance; StepInitThrowTeamMate
  waits for another command in both engines, the harness re-sent the same square → STUCK_STEP / no_prompt.
  Fix: both agents deselect (CLIENT_ACTING_PLAYER null = Action::Acknowledge) when the target is out of
  range; Rust bb2016 StepInitThrowTeamMate gained Java's missing deselect → END_PLAYER_ACTION branch (+test).
  ogre bb2016/bb2020/bb2025 scale 0 100/100 each, goblin-underworld bb2016 30/30.
- [ ] **Q1.2c classes=none divergences** (found while checking Q1.2a): ogre bb2016 scale 0 *without*
  `--heur-classes` diverges on seeds 7, 18, 30, 82 (Java activates a FOUL whose step changes nothing,
  Rust a Move; reports differ). Same with the frozen cert9 exe+jar, so it is old. Not a certified config,
  but a real Java/Rust agent-contract difference: root-cause and fix.
- [~] **Q1.2b Stuck step: underworld_37844 vs dark_elf bb2025** (cross, seed 3, scale 1.0, classes all;
  reproduces on head). Dark elf Away2 PUNTs: PuntDirection d6 (rng 102) + re-roll offer declined,
  PuntDistance 6 (103) → TRR accepted → 3 (104), ball out of bounds → THROW_IN. **Java** then rolls one
  more `rollThrowInDirection` (105) and sits in CATCH_SCATTER_THROW_IN with no dialog (only path:
  `getReRolledAction() != null` → CONTINUE, line ~389); **Rust** sits in CatchScatterThrowIn at rng 104
  with no prompt. Final hashes equal (both stuck). Next: trace which Java step rolled 105 (throwInBall's
  distance roll is not in the JAVA_DIE trace — check `DiceRoller.rollThrowInDistance`), find where the
  catch step's reRolledAction comes from (a stale ClientCommandUseReRoll?), then decide Java defect
  (JAVA_DEFECTS.md) vs harness/Rust fix.
- [~] **Q1.3 Wider state hash.** Part 1 done (see Log): every player part now ends `,<PlayerState id>,<PA>`
  (all state flags + exact off-pitch state + effective PA), both engines identical; smoke 3×20 green.
  Part 2 done: acting player part ends `,<declared action>,<10 flags>` (blocked, passed, fouled, moved,
  fed, goingForIt, standingUp, bloodLust, animosity, jumping); smoke 3×20 green.
  Part 3 done: player parts end `,td/cmp/cas/int/defl/land/blk/foul/rush/pass/awards` (game result),
  identical incl. non-zero values; smoke 3×20 green.
  Part 4 done: ` o<apothecaries home,away> k<bomb x,y,moving | ->` segments; smoke 3×20 green, goblin
  bb2025 mirror 29/30 (seed 4 = JD-002 JAVA-CRASH, expected).
  Part 5 done: player parts end `,t<N>` (temporarily granted skills); Grombrindal star gates bb2020 +
  bb2025 10/10, 527 steps with t>0 identical. **Code complete**; inducement-set hashing moves to Q2.8
  (no gate buys inducements before it). Waiting: mirror 330 (night). Original: Add to BOTH `ffb-model/src/util/state_hash.rs` and
  `ParityRunner.java` (same string format): PA; full player state incl. flags (rooted, confused,
  hypnotised, used-Pro, …) and the exact off-pitch state (banned, KO, BH, SI, RIP, missing, …);
  granted/temporary skills and properties; SPP and per-player game-result stats; inducement set and
  apothecaries left; bombs; acting player's declared action + has-blocked/passed/fouled/moved flags;
  TTM/KTM-used. Done: a cross-engine test that hashes the same fixture JSON in both engines; smoke gate
  green; mirror 330 green.
- [x] **Q1.4 Client view in the certification.** (see Log) `scripts/view_suite.py` (resumable, chunked,
  `--scale`, `--jar`), post-game view dump in both harnesses, `cert_tally.py` counts
  `D:/ffb_view_<cert>_{mirror0,mirror1.0,mirror1e6,cross,star}/summary.txt`. 3 jobs (bb2025/bb2020/bb2016)
  end-to-end OK. Left for later: order-sensitive comparison of scalar lists (Q1.4b). Original: Move the scratchpad view runner into
  `scripts/` (job lists → `FFB_CLIENT_VIEW_DIR` dumps → `client_view_check.py`), add a dump after game end
  (SPP, MVP, winnings, fan factor), compare ordered lists in order where Java's order is meaningful, and
  make `cert_tally.py` require `CLIENT VIEW n steps, n identical`. Done: one suite cell end-to-end.
- [x] **Q1.5 Enforce coverage.** (see Log) Each gate writes `FFB_COVERAGE_JSON`; `ffb-parity
  --coverage-merge <root>` sums them and applies the tier-3 checklist with no small-run exemption
  (exit 1 on MISSING); view_suite.py writes `<root>/coverage.txt` at DONE and cert_tally.py requires it.
  3-job test: merge works, flags `throw-ins` MISSING over 9 games. Left: skill ledger counting a skill
  only when it fired, not when offered (Q1.5b). Original: The coverage checklist runs for every gate (aggregate per suite, not per
  50-game gate); MISSING required items are reds at suite level; the skill ledger counts a skill only when
  it changed the outcome path (fired), not when offered. Done: suite-level coverage report in the cert.
- [x] **Q1.6 Determinism gate.** (see Log) `scripts/determinism_check.py` (bench mode, no JVM, two
  fresh processes, byte-compare every Rust log): bb2025 human, bb2020 snotling-skaven, bb2016
  goblin-underworld, bb2025 nippon-ogre + stars, 100 seeds each → 4×200 files identical. Left (Q1.6b):
  make the `player_at` arrival-counter tie and `util_server_cards.rs:245` deactivation order explicit
  (`field_model.rs` is being edited by another session — do it when that lands). Original: Run Rust twice on the same seeds (fresh process each) and diff the
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

- [x] **Q2.1 Apothecary accept** (see Log). Under `--coverage-bias` > 0 both agents use the apothecary
  and take the new result at the choice (zero draws). It exposed two Rust bugs, both fixed 1:1: the
  no-choice branch never added Java's ReportApothecaryChoice, and that report serialised playerState as
  null; and the choice dialog's answer went through "use / do not use" instead of Java's
  handleApothecaryChoice(old|new). Gates with `--coverage-bias 1`: bb2025 human, bb2020 orc-skaven,
  bb2016 dwarf-elf 20/20 steps + reports; apothecary rolls present in all three.
- [ ] **Q2.2 Regeneration re-roll / Igor / Mortuary Assistant accept.**
- [ ] **Q2.3 Argue the call: decline branch; bribes: use branch.**
- [ ] **Q2.4 Kick-off placements:** Quick Snap, Solid Defence, High Kick, Perfect Defence, Charge pick
  real players. Depends on Q3.4 (Solid Defence selection).
- [x] **Q2.5a Swarming dialog loop** (see Log): the harness clears DialogSwarmingPlayers at once
  instead of looping 500× on a CONFIRM StepSwarming ignores; same state (zero swarmers), no abort.
  snotling bb2020/bb2025 20/20 each, loops_cleared 0.
- [ ] **Q2.5b Swarming placement** (remove the engine's hard-coded "place nobody" in
  `bb2025/kickoff/step_swarming.rs`, ask the agent as Java asks the client; both harnesses place
  swarmers with seeded draws).
- [x] **Q2.9a PUNT_TO_CROWD loop** — does not reproduce on head: dark_elf bb2025 scales 0 and 1.0
  seeds 1-100 and 1e6 seed 14 all `loops_cleared 0`, 100/100 (fixed between cert9 and head). Original: (dark_elf bb2025, 8 loop clears in cert9): the harness DOES answer
  `sendPuntToCrowd(false)`, yet the dialog re-fires; find why in Java `StepInitPunt` and fix the harness
  answer (or Rust, if Rust differs). Seeds: `mirror_cert9/scratch/sw_dark_elf_bb2025_*.log.err`.
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

- 2026-11-27 · Q2.1 · (next commit) · apothecary accept under coverage bias; fixed missing
  ReportApothecaryChoice, its null playerState, and the choice-dialog answer path; 3×20 biased gates green.
- 2026-11-27 · Q1.6 · (next commit) · determinism_check.py; 4 matchups × 100 seeds byte-identical over
  two processes.
- 2026-11-27 · Q1.5 · (next commit) · suite-level coverage: per-gate JSON, --coverage-merge, coverage.txt
  required by cert_tally; merge unit test.
- 2026-11-27 · Q1.4 · (next commit) · view_suite.py in the repo, post-game view dumps, cert_tally counts
  the view suites; human-orc bb2025, dwarf-elf bb2020, skaven-undead bb2016 all identical incl. end view.
- 2026-11-27 · Q1.3 part 5 · (next commit) · temporary-skill count per player in the hash; Grombrindal
  bb2020/bb2025 10/10 with 527 non-zero steps identical. Q1.3 code complete.
- 2026-11-27 · Q1.3 part 4 · (next commit) · apothecaries and bomb square/moving in the hash.
- 2026-11-27 · Q1.3 part 3 · (next commit) · per-player game-result counters in the hash, both engines;
  smoke 3×20 green, non-zero counters identical on a live game.
- 2026-11-27 · Q1.3 part 2 · (next commit) · acting player's declared action + 10 activation flags in
  the hash, both engines; smoke bb2025/bb2020/bb2016 20/20 (classes all).
- 2026-11-27 · Q1.3 part 1 · (next commit) · state hash player parts + full PlayerState id + PA, Java and
  Rust byte-identical (checked on a live step); smoke bb2025/bb2020/bb2016 20/20 with classes all.
- 2026-11-27 · Q1.2a · (next commit) · out-of-range TTM target → both agents deselect; bb2016
  StepInitThrowTeamMate deselect branch ported; ogre ×3 editions 100/100. Q2.9a closed (no repro on head).
- 2026-11-27 · Q2.5a · (next commit) · harness clears the Swarming dialog at once (was a 500× loop on
  an ignored CONFIRM); snotling bb2020 + bb2025 20/20, loops_cleared 0.
- 2026-11-27 · Q1.2 · (next commit) · end_reason/loops_cleared in both logs; aborts fail; cert9 aborts
  classified; snotling bb2025 seeds 1-3 now fail (4 Swarming loop clears each); smoke 3×20 green.
- 2026-11-27 · Q1.1 · (this commit) · verdict now fails on any per-step hash/post-hash/turn/half/active
  difference, a step-count difference or a different start state; 5 unit tests; smoke 3×20 green.
