# Event census of the 2026-09-10 full-matrix sweep

**What this is.** A re-tally of every `GameEvent` the Rust engine emitted across the 333 gates of the
full-matrix sweep (`docs/SWEEP_2026-09-10.txt`, milestone `docs/MILESTONE_FULL_PARITY.md`) —
2.0 GB of `parity_sw_*/**/seed_*_rust_events.jsonl`, **33,300 games, 27,858,969 events**.

Parity is green on all 333 gates. This document asks the *other* question: **which rules were on the
pitch when it went green.** It is the coverage half of the milestone, and it is not green.

Reproduce with `scripts/sweep_census/` (`agg.py` → `pass2.py`/`pass3.py` → `report.py`/`skills.py` →
`build_page.py`). `agg.py` mirrors `coverage_report.rs::tally()` line for line, and the
reconstruction **reproduces the sweep's 26 short gates exactly, item for item** — that agreement is
what licenses everything else here.

## Headline numbers

| | |
|---|---|
| gates / games / events | 333 / 33,300 / 27,858,969 |
| `GameEvent` variants emitted | **77 of 128** (24 have an engine emit site never reached, 27 have no producer at all) |
| `PlayerAction` variants declared | **26 of 58** |
| skills fielded by the 111 squads | **117 of 200** — 45 provably exercised, 11 dead, 61 with no naming event |
| blocks / block re-rolls | 660,522 / **0** |
| injuries / KO / casualties / deaths | 767,674 / 59,018 / 49,530 / 6,239 (12.6% of cas, vs 12.5% d16 expectation) |
| touchdowns / hand-offs / passes | 25,856 / 32,913 / 47,697 |
| fouls / argue-the-call / ejections | 56,214 / 11,560 / 10,934 |
| kick-offs | 89,345 — all 11 results of each edition's own table |
| weather | all 5 results, all 3 editions |
| positional coverage | every drafted player number acted at least once in **every** one of the 111 cells |

## Findings, worst first

### 1. A block die is never re-rolled — in 660,522 blocks

`blockRoll` fires 660,522 times with `rerolled=true` **zero** times, in every edition and roster.
Dodge (15.9%), rush (6.4%), pickup (11.0%), catch (8.9%) and pass (7.9%) re-rolls all happen.

`StepBlockRoll` emits `AgentPrompt::BlockChoice`, which carries only the dice — no re-roll option.
Its `ask_for_reroll_if_available(game, "BLOCK", 0, false)` branch sits behind `do_roll == false`,
which a first pass cannot reach (`do_roll` starts `true` and is only cleared inside the
`re_rolled_action == Some("BLOCK")` arm). The heuristic agent has no `"BLOCK"` case in its re-roll
policy either and would accept at weight 0.45 if offered.

Collateral, all at zero: team re-rolls on a Skull / Both Down; **Loner** (fielded in 73 of 111 cells,
`lonerRoll` = 0 — and `GameEvent::LonerRoll` had no producer either, so the roll was reported and
never evented); **Pro** / **Old Pro** (3 cells, `proRoll` = 0); **Brawler** (7 cells) and **Hatred**
(3 cells), reachable only via the `re_roll_source` nothing sets; `teamCaptainRoll` and `leader` = 0.
Brilliant Coaching and its bb2016 twin grant 16,423 extra re-rolls that can never be spent on a block.

> **Fixed 2026-09-10, same day** — BACKLOG §H.14 phase 1. Both engines now ask the team-re-roll
> question on the block dialog. Measured after the fix: amazon bb2016 338 block re-rolls / bb2020 312,
> ogre bb2020 193, imperial_nobility bb2025 215 **with 40 Loner rolls**, every gate 100/100, and the
> `--agent random` control still 0 re-rolls and 100/100 in all three editions.

### 2. The whole bb2016 column emits no movement at all

| edition | games | `playerMoved` | `goForItRoll` |
|---|---:|---:|---:|
| bb2016 | 8,700 | **0** | **0** |
| bb2020 | 12,300 | 7,588,679 | 443,582 |
| bb2025 | 12,300 | 7,559,379 | 436,003 |

bb2016 players do move — 77,331 dodges, 20,514 pick-ups, 6,506 touchdowns, 1,159,804 `Move`
declarations. But `GameEvent::PlayerMoved` and `GoForItRoll` are emitted only from
`step/bb2025/move_/*`, and `driver.rs:425` routes bb2016 to `step/bb2016/move_/*`, which emits
neither. A third of the matrix has no movement or rush telemetry; bb2016 rushing is asserted by
nothing but the state hash.

### 3. Two checklist items carry stale BLOCKED notes that hide finding 2

`t3_checklist.rs` still marks `touchdowns` and `GFI rolls` "BLOCKED on the one-move-per-activation
decision". The matrix scores **25,856 touchdowns** and rolls **879,585 rushes**, so the note is
false everywhere. Because both items are `blocked`, neither can fail a gate — which is why the
bb2016 hole went unseen. `GFI rolls` is zero in exactly the 87 bb2016 gates and nowhere else:
a per-edition instrumentation gap wearing a note about agent behaviour.

### 4. The apothecary is offered 62,279 times and never used

All 62,279 `apothecaryRoll` events carry `roll:null, new_state:null, new_serious_injury:null`.
Against 49,530 casualties and 6,239 deaths, nothing is ever healed. `ApothecaryChoice` (3 sites in
`step_apothecary.rs`) is never emitted, so the decision itself is unobservable.

### 5. Six declared actions never produce the roll that defines them

| action | declarations | its roll event | move-only activations |
|---|---:|---|---:|
| `BalefulHex` | 535 | `balefulHexRoll` = 0 | 64% |
| `AutoGazeZoat` | 540 | `hypnoticGazeRoll` = 0 | 80% |
| `LookIntoMyEyes` | 13 | `lookIntoMyEyesRoll` = 0 | 92% |
| `ProjectileVomit` | 0 (16 cells field it) | `projectileVomitRoll` = 0 | — |
| `BreatheFire` | 0 (2 cells field it) | `breatheFireRoll` = 0 | — |
| — | — | `jumpUpRoll` = 0 (Jump Up in 21 cells) | — |

Traced one Baleful Hex declaration through a log: the action is selected, the caster walks, no hex.
`BalefulHexRoll`, `HypnoticGazeRoll`, `LookIntoMyEyesRoll` and `BreatheFireRoll` have **no
construction site anywhere in `ffb-engine`**; `ProjectileVomitRoll` and `JumpUpRoll` have one and
never reach it.

Also declared-then-degenerate, with no event to judge them by: `WisdomOfTheWhiteDwarf` (511, 75%
move-only), `RaidingParty` (894, 67%), `KickTeamMate` (5,168, 92%), `ThrowTeamMate` (94,959 declared,
14,542 thrown — 83% move-only), `BlackInk` (425, 29%), `CatchOfTheDay` (147, 47%).

### 6. 61 of the 117 fielded skills are invisible to the event stream

Fielded in this many cells, with no event that names them: Mighty Blow 78, Thick Skull 69, Block 61,
Pass 52, Stunty 44, Frenzy 43, Sure Hands 29, Catch 23, Stand Firm 22, Prehensile Tail 21, Side Step
19, Claw 14, Disturbing Presence 14, Sprint 12, Stab 12, Secret Weapon 11, Shadowing 10, **Diving
Tackle 9**, Diving Catch 8, Dirty Player 7, Guard 4, Tentacles 4 …

All are implemented and referenced in production code, and the state hash covers their *effects* —
but no coverage assertion is possible either way. Diving Tackle is the case that prompted this
census: fielded in 9 cells, and the sweep can prove neither a use nor a decline.
`GameEvent::SkillUse` has only six live skills behind it; `GameEvent::ReRoll` has no producer at all.

### 7. "Declined" is almost entirely uninstrumented — and Dump Off is 100% declined

| skill | used | declined |
|---|---:|---:|
| Dodge | 15,496 | 205 |
| Horns | 16,021 | 0 |
| Wrestle | 1,736 | 453 |
| Juggernaut | 630 | 182 |
| Tackle | 205 | 0 |
| **Dump Off** | **0** | **3,074** |

Dump Off is offered 3,074 times and taken zero times — a reactive pass that has never executed.
For every skill outside this table, absence of a decline is absence of instrumentation.

### 8. Prayers to Nuffle fire in bb2020 and never in bb2025

bb2020: 4,305 `prayerRoll`s, all 16 catalog entries covered. bb2025: **0 of 16** across 11,100 games.
Same mirror matchups, same equal TV, both editions carrying a 16-entry table and a live `StepPrayers`.
One of the two is wrong about the underdog rule, and `PrayerAmount` — the event that would show the
TV gap each engine computed — has no producer, so the input cannot be read from the logs.

### 9. The entire pre-game economy never runs

Zero across 33,300 games: `buyInducement`, `inducement`, `pettyCash`, `cardsAndInducementsBought`,
`playCard`, `cardEffectRoll`, `cardDeactivated`, `wizardUse`, `masterChefRoll`, `bribesRoll`,
`coinThrow`, `receiveChoice`, `gameOptions`, `secretWeaponBan`, `coachBanned`, `heatExhaustion`.
"Get the Ref" hands out a free bribe 2,460 times and no bribe is ever rolled. Post-game is healthy:
66,500 winnings rolls and 66,500 MVP rolls — exactly two per game.

### 10. The 26 short gates are all agent behaviour

`action HandOver` ×20, `action Pass` ×6, `pass rolls` ×5 — three ball-handling actions the heuristic
never chose with that roster at that scale, concentrated at `@1e6` (near-deterministic) and `@0`
(near-uniform). Khemri and `khemri_fumbbl` never pass at `@0` in bb2020 or bb2025.

### 11. 83 of 200 skills and 32 of 58 actions are outside the matrix entirely

Never fielded by any drafted squad: **Kick**, **Leader**, **Team Captain**, **Pass Block**, **Piling
On**, **Pile Driver**, **Sneaky Git**, **Kick-Off Return**, Bounding Leap, Halfling Luck, Brutal
Block, Whirling Dervish, and 71 more. Kick and Leader are ordinary picks on real teams, and Kick
gates a core kick-off rule no gate touches. This is a drafting gap, not an engine gap — the cheapest
class here to close.

Never declared once: `Stab`, `Chainsaw`, `BreatheFire`, `ProjectileVomit`, `Gaze`, `DumpOff`,
`Swoop`, `MaximumCarnage`, `PutridRegurgitation{Move,Blitz,Block}`, `KickEm{Block,Blitz}`,
`TheFlashingBlade`, `ViciousVines`, `Chomp`, `Incorporeal`, `Forgo`, `SecureTheBall`, and 15 more.

## What the census does prove

Positional coverage is complete. The kick-off, weather, serious-injury and block-result tables are
fully covered in every edition, with tails at expected frequencies (death 12.6% of casualties against
a 12.5% d16 expectation; armour held on 68% of injury events). 45 fielded skills are provably
exercised, including the whole big-guy negatrait family (435,074 Bone Head / Really Stupid / Take
Root rolls), 48,713 Blood Lust rolls, 39,892 Animal Savagery / Unchannelled Fury rolls, 15,082 Foul
Appearance rolls and 14,542 team-mate throws.

## Suggested order of work

1. **Reach the block re-roll** (finding 1). One prompt shape; unblocks Loner, Pro, Old Pro, Brawler,
   Hatred and every team-re-roll grant at once. Mirror it in `ParityRunner.java` — both harnesses
   currently agree on never asking, which is why parity never noticed.
2. **Emit `PlayerMoved` / `GoForItRoll` from the bb2016 move steps** (finding 2), then drop the two
   stale BLOCKED notes (finding 3) so the checklist can fail on them again.
3. **Decide the apothecary path** (finding 4) — the offer without the choice is worse than neither.
4. **Give the silent skills an emit site**, starting with the ones fielded in the most cells
   (finding 6). Without it, "covered" for those 61 skills means only "the narrow state hash matched".
5. **Settle bb2020 vs bb2025 prayers against the rulebook** (finding 8) and give `PrayerAmount` a
   producer so the TV input is inspectable.
6. **Draft Kick, Leader, Team Captain, Pass Block and Piling On into some squad** (finding 11).
