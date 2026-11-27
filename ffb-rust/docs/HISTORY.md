# Project history

FFB-Rust is a 1:1 Rust port of the FUMBBL "FFB" Blood Bowl engine, written in Java.
Every Java step, generator, mechanic and model class has a Rust twin with the same name and logic.
The port covers all three rule editions: BB2016, BB2020 and BB2025.
Parity suites prove it identical to the stock Java engine, game for game.
They compare per-step state hashes and the full report stream, on one frozen Rust build and one frozen Java jar.
Rust also replays every downloaded FUMBBL replay exactly as Java does.
Finally, the unmodified Java Swing client plays on the Rust server and sees the same client view as on the Java server.

This file replaces about 150 campaign logs, plans and sweep dumps that were kept in `docs/`.
The live work queue stays in [`BACKLOG.md`](BACKLOG.md).
The certified milestones stay in [`MILESTONE_FULL_PARITY.md`](MILESTONE_FULL_PARITY.md) and [`MILESTONE_SKILL_COVERAGE.md`](MILESTONE_SKILL_COVERAGE.md).

## Timeline

Newest first. "Gate" means seeds 1-100 of one matchup in one edition unless stated otherwise.
A "cell" is one roster in one edition; the heuristic matrix runs each cell at three agent scales.

| Date | Milestone | Result | What it took |
|---|---|---|---|
| 2026-11-27 | §P GUI integration: star client view | 4,012/4,012 star gates, client view identical | Four star fixes: bb2020 Wisdom source key, bb2020 Baleful Hex hypnotises, property display names, Old Pro marks Pro used. |
| 2026-11-26 | §P GUI integration: cross client view | 4,012/4,012 cross gates, client view identical | No fix needed. |
| 2026-11-25 | §P GUI integration: skill sweep and mirror client view | Skill squads bb2016 31/31, bb2020 42/42, bb2025 43/43; mirror every race x3 | Skills marked used where Java marks them; bb2016 `Select` routed by edition; bb2020 forced second block asks for the block kind. |
| 2026-11-24 | §O replay processing parity | 2,224/2,224 FUMBBL replays: Java complete, Rust states and actions byte-identical | Java-exact `Game` JSON, `ModelChangeProcessor` port, MDE port, JDK21 `Double.toString`, frozen night run plus manifest. |
| 2026-11-23 | §P GUI integration started | Unmodified Java Swing client logs in and plays on the Rust `ffb-server` | LZString transport, real Java net commands, `serverGameState` and model sync from §O's Java-JSON model. |
| 2026-11-22 | §M skill coverage, cert9 | Mirror 330, cross 4,012, star 4,012, skill squads 121, all green; 411/411 skills fire; about 55,000 games, 0 red | Both agents use every skill; skill-advancement squads; Team Captain ported; report-order bug from an un-Java `apply_to`. |
| 2026-11-17 | §L star tier, cert4 | Cross 4,012 + star 4,012 + mirror 330 x 100, steps and reports identical | One hired star per team, every squad pair; full 4,012 sweep reds fixed one by one. |
| 2026-11-15 | §K coverage reach (folded into §L and §M) | Dead mechanics wired into both agents | Stab, Chainsaw, Projectile Vomit, Breathe Fire, Gaze, Jump Up, Shadowing offered identically in Rust and Java agents. |
| 2026-11-15 | §J report-stream parity | Mirror 33,000/33,000 + cross 12,036/12,036 games report-identical, one build (`d4513b93e`) | Started at 8/33,000; ranked report diff families, fixed every missing or misordered `addReport` in Rust. |
| 2026-11-12 | §I.9 real pregame | Mirror 330/330 + cross 4,012/4,012 on the real edition pregame | Petty cash and inducement purchase run (all declined) so report streams match from report 0. |
| 2026-11-12 | §I cross-matchup campaign | Cross 4,012/4,012 + mirror 330/330, one build | Diving Tackle made live, Grab vs Side Step per edition, roster keywords, bb2016 Tackle/Dodge, Getting Even, whole-roster hash. |
| 2026-09-14 | Matrix on report stream + event bridge | 330/330 | Reports became the coverage record; report-to-event bridge; event kinds 76 to 83. |
| 2026-09-13 | Heuristic kick-off setup | Setup sweep 329/330 (+1 stock Java crash), then 330/330 | Scored setup in both engines exposed latent BB2016 kickoff and chain-injury paths. |
| 2026-09-12 | Re-drafted matrix certified | 330/330 (110 cells x 3 scales) | Squads re-drafted from the rulebook; 16 red gates closed incl. stalling, raise-dead, Cloud Burster, JD-001. |
| 2026-09-10 | Full heuristic matrix | 333/333 gates, 33,300 games (tag `parity-matrix-green-2026-09-10`) | New teams drafted (snotling, gnome, black orc, imperial nobility...); Pogo, Grab, Trickster, Pro fixes. |
| 2026-09-01 to 09-09 | Heuristic per-race campaigns | amazon ... wood elf, every race nine gates 100/100 (3 editions x 3 scales) | Alphabetical sweep; skill-use dialogs sampled not auto-used; mix64 cache stamp; goblin 14 fixes. |
| 2026-08-30 | Heuristic amazon parity | 100/100 x 3 editions x 3 scales | Kickoff-return window, `push_self`, pass-block window port, Safe Pass re-roll order. |
| 2026-08-29 | Heuristic agent parity, lineman | Nine gates 100/100 (ITER70) | Class-ladder decision mechanism, bit-identical `DetMath` in both engines. |
| 2026-08-26 | Heuristic agent ball moves | Hand-off and pass chains work; passing +3.75 SE over 12,800 games | Too-small effects look like mistuned weights; measured at scale before tuning. |
| 2026-08-25 | Dead-step frontier | 167/200 steps reached; all 33 dead classified | Stab injury driven live; remaining blocker was one-square-per-activation agents. |
| 2026-08-25 | §12 blitz/gaze select chain | 90/90 (30/30 per edition), merged to main | About 16 engine fixes; last red was an edition-defaulted shared generator. |
| 2026-08-18 to 08-24 | Mechanic tiers: TTM, KTM, Hit and Run, interception, bomb chain, Multiple Block, Old Pro, star specials | Each gate 30/30/30 | Switching on dead mechanics found bugs; state hash gained the ACTIVE bit; vacuous-green checks via events. |
| 2026-08-16 | BB2020 ruleset and roster redraft | All three editions 30/30, merged and pushed | Dirty Player armour and injury, Frenzy guard; about 93 of 104 `step/bb2020` files found dead. |
| 2026-08-13 | BB2016 team matrix | 30/30 | bb2016 steps made agent-drivable with prompt bridges, approach A (faithful 1:1). |
| 2026-08-08 | Per-roster mirror parity, random agent | All 30 bb2025 mirror matchups 100/100 | Rule-legal hand-drafted teams; FUMBBL roster-id aliasing; necromantic, khemri, lizardman fixes. |
| 2026-08-03 | Full human vs human | 100/100 bb2025 | Bone-head prone pre-draw, pass weather range, catch re-roll fresh die. |
| 2026-08-02 | Tier 1: lineman vs lineman | 100/100 bb2025 against stock Java | First per-step state-hash parity; harness `ParityRunner.java` co-edited, Java engine untouched. |
| 2026-08-01 | Test equalization | Rust 14,255/0, Java `ffb-server` 4,635/0 | Same tests on both engines; 14 bugs and 1 fidelity removal found. |
| 2026-07 | Translation complete | All 2,521 step and generator files translated | Phases ZR-ZW: `engine.rs` deleted, `driver.rs` live; Phases AD-AK audit rounds. |

## Lessons that shaped the method

- Java is the truth: a parity difference is fixed in the Rust engine only, never in the Java engine.
- Dice order is sacred: one extra or missing draw shifts every later roll, so compare call stacks, not totals.
- Java `rng_calls` count calls, not dice, and the first dice diff is meaningless once state has diverged.
- Rust `HashMap` order is nondeterministic; a seed that flips between pass and fail on one build is that.
- Edition-gated shared steps need `rules` at every push site, because `..Default::default()` silently supplies BB2025.
- Reason from the live path: most `step/bb2020/*.rs` files were dead, and a silent probe means a dead file.
- The state hash has blind spots (players above 11, ACTIVE bit, report order); green can be vacuous, so check events.
- Freeze the Rust exe and the Java jar for a certification; a stale exe or cache turned gates silently wrong.
- A half fix can measure worse than none; diff the failing seed sets before reverting a faithful fix.
- Probe both engines with the same probe and diff the outputs, instead of inferring Java behaviour.
- Port a Java mechanism as one change set and gate once; components measured alone mislead.
- A new agent policy reaches engine paths that hundreds of green gates never did, so re-sweep after each one.

## Removed documents

Removed from the tree on 2026-11-27; recover any of them with `git log --all -- ffb-rust/docs/<NAME>` and `git show <commit>:ffb-rust/docs/<NAME>`.

- Race and tier campaign logs: `PARITY_<RACE>_CAMPAIGN.md` for amazon, bb2016, bb2020, chaos, chaos_dwarf, chaos_pact, dark_elf, dark_elf_league_fumbbl, dwarf, elf, goblin, halfling, heuristic, high_elf, human, khemri, khemri_fumbbl, lizardman, necromantic, nippon, norse, nurgle, ogre, orc, renegades, skaven, slann, slann_fumbbl, undead, underworld, vampire, wood_elf; plus `PARITY_BB2020_STRUCTURAL_GAP.md`, `PARITY_TIER1.md`, `PARITY_TTM.md`, `PARITY_ROSTER_MATRIX.md`, `CROSS_MATRIX_BB2020.md`.
- Phase plans: `PHASE_AAD_PLAN.md`, `PHASE_AI_PLAN.md`, `PHASE_AJ_PLAN.md`, `PHASE_AK_PLAN.md`, `PHASE_ZV_PLAN.md`, `PHASE_ZW_PLAN.md`, `PHASE_ZX_PLAN.md`, `PHASE_AAF_SKILL_HOOK_AUDIT.md`.
- Sweep and report dumps: `SWEEP_*.txt` (2026-09-10 to 2026-11-13e), `REPORT_PARITY_*.txt` (2026-11-12 to 2026-11-13e).
- Event coverage and census: `EVENT_COVERAGE.md`, `EVENT_COVERAGE_<race>_<edition>.md`, `EVENT_CENSUS_2026-09-10.md`, `EVENT_CENSUS_2026-09-10_REDRAFT.md`, `EVENT_CENSUS_2026-09-13_SETUP.md`, `COVERAGE_GAPS_2026-09-15.md`.
- Team matrices: `TEAM_MATRIX_BB2016.md`, `TEAM_MATRIX_BB2020.md`, `TEAM_MATRIX_BB2025.md`.
- Test-parity ledgers: `TEST_PARITY.md`, `TEST_PARITY_PORT.md`.
- Generated output and scratch: `CROSS_SWEEP_*.txt`, `sweep_*.json`, `cross_sweep_*.json`, `coverage_gaps_*.txt`, `oracle_check_*.txt`, `report_census*.json`, `census_redraft_report.txt`, `SKILL_LEDGER_2026-11-17_base.txt`, `heuristic_agent.html`, `setup_census_report.html`, `squad_page.html`, `squad_report.json`, two `.patch` notes.
- Outside `docs/`: the old flattened Java copies `ffb-java/ffb-{server,common,client,client-logic,resources,tools}` (the engine lives in `ffb-java/ffb/`), `ffb-rust/SESSION.md`, the early `run_t2*/run_t3*` scripts and `.wip-snapshots/`.
