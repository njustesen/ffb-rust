# Event coverage — HeuristicAgent, amazon v amazon, all three rulesets

Harvested 2026-08-30 by `scripts/harvest_coverage.sh <edition> 1.0` (one clean run per edition,
seeds 1-100, `--heur-classes all`, nothing else running). Every run was `PARITY: 100/100`, so every
count below is a count of things BOTH engines did identically. Per-edition detail (the tier-3
checklist as written by the run, the full `GameEvent` catalog, the action tallies):
`EVENT_COVERAGE_bb2016.md`, `EVENT_COVERAGE_bb2020.md`, `EVENT_COVERAGE_bb2025.md`.

## What the skilled roster reached that lineman play never did

| per 100 games | lineman (heuristic, ITER0) | amazon bb2016 | amazon bb2020 | amazon bb2025 |
|---|---|---|---|---|
| touchdowns | 40 | **60** | **46** | **43** |
| GFI rolls | 4,400 | (not evented, see F3) | **4,762** | **4,867** |
| dodge rolls (success / fail) | — | 822 / 91 | 642 / 359 | 686 / 392 |
| pass rolls | 231 passes | **219** | **187** | **195** |
| hand-offs | 132 | 112 | 93 | 96 |
| block rolls | 1,048 | 1,871 | ~1,600 | 1,662 |
| pass-block windows (`passBlock`) | 0 | 0 (no On the Ball) | **157** | **161** |
| Hit and Run (`hitAndRun`) | 0 | 0 (not on roster) | **185** | **216** |
| Baleful Hex declared (Estelle) | 0 | 0 | 0 | **178** |
| `skillUse` events (all = Dodge on a block, see F1) | 0 | 304 | 234 | 251 |
| interceptions | 0 | 0 | 0 | 0 (contract: agents decline) |

Dodge failures rise from 91 (bb2016) to 359/392 (bb2020/25): the bb2020+ rosters run two
On-the-Ball windows a game and the agent dodges into contact for them. Every one of those rolls, its
Dodge re-roll, and its outcome is parity-verified.

## Skill by skill — exercised? evented?

"Exercised" is proven by parity plus the traces in `PARITY_AMAZON_CAMPAIGN.md`; "evented" means a
`GameEvent` in the run's stream names it. The gap between the two columns is this report's finding.

| skill (who) | exercised | evented | evidence |
|---|---|---|---|
| Dodge (everyone, all editions) | yes | **yes, on two events** | dodge rolls 913/1001/1078, of which **322/264/284 carry `rerolled:true`** — the Dodge re-roll is visible on `dodgeRoll`, not as a `skillUse`; the Dodge-negates-Defender-Stumbles use IS evented as `skillUse` (304/234/251 = the `PowPushback` counts exactly). The agent prices Dodge in `Reach` (a tackle-zone exit costs `p_with_reroll`), which is why dodge attempts triple from bb2016 to bb2020/25 |
| Block (bb2016 Blitzer) | yes | no | 297 `BothDown` results in bb2016; the Block save is not evented |
| Catch (bb2016 Catcher) | yes | no | 322 catch rolls; the Catch re-roll is not evented |
| Pass (Thrower, all editions) | yes | no | 219/187/195 pass rolls; the Pass re-roll is a `SkillUse` DIALOG answered by both agents (ITER10, ITER30 `JDRAW skill=Pass`) — no event |
| Safe Pass (Thrower, bb2020/25) | yes | no | bb2025: a `SkillUse` dialog (`handleSafePass`), declined half the time at 1e6 (ITER30 seeds 26/40/89); bb2020: automatic. Neither evented |
| On the Ball — pass block (Thrower, bb2020/25) | yes | **partly** | 157/161 `passBlock` events, but every one carries `player_id: null` — the event does not say whether a window opened or who moved (F2) |
| On the Ball — kickoff return | yes | **no** | the whole ITER19–29 frontier; observable only through `FFB_STEPTRACE`/`FFB_KR` — there is no kickoff-return event type |
| Hit and Run (Blitzer, bb2020/25) | yes | yes | 185 / 216 `hitAndRun` |
| Jump Up (Blitzer, bb2020/25) | unknown | no | no event; fires inside the Select sequence's JUMP_UP step for a prone Blitzer — parity says whatever fires matches Java, nothing says how often |
| Defensive (Jaguar Warrior / Catcher, bb2020/25) | passive | no | an assist modifier; never evented by design |
| Baleful Hex (Estelle, bb2025) | yes | yes (as an action) | 178 `playerAction BalefulHex`; the target choice and the 2+ roll are not evented — the ITER29 Estelle family was found through traces |
| Sidestep (Estelle, bb2025) | yes | no | `JDRAW skill=Sidestep` dialogs in the 1e6 traces; no event |
| Guard / Loner / Disturbing Presence (Estelle) | passive | no | modifiers |

## Findings

**F1 — `skillUse` is emitted by three steps only.** `GameEvent::SkillUse` is raised by the
block-result Dodge path, `step_dump_off.rs` and `step_horns.rs`. Every re-roll skill (Dodge on a
dodge, Pass, Catch), every dialog skill (Safe Pass, Sidestep, Pass re-roll) and every activation
skill (Jump Up, Hit and Run's use bit) is used without an event. The `skill_id` on the 789 events in
this harvest is the single value 127 (Dodge). A coverage report built from events therefore
UNDER-reports skill usage on a skilled roster by an order of magnitude. Not an engine defect —
parity is green — but the instrument this campaign was asked to read is mostly blind to the thing
it was asked to measure. Fix: emit `SkillUse` wherever Java adds `ReportSkillUse`/`ReportReRoll`
(the report list is 1:1 already; mirror it into the event stream).

**F2 — `passBlock` events do not name the window.** All 318 carry `player_id: null`: the step emits
the same event when no blocker is available and when it opens the window. Emit the blockers (or the
mover) so a window opening is countable.

**F3 — the bb2016 move twins emit no movement events.** bb2016 has 13,104 Move actions and 60
touchdowns but ZERO `playerMoved` and ZERO `goForItRoll` events; bb2020/25 have 62k/64k and
4.7k/4.9k. `step/bb2016/move_/*` (the twins the driver dispatches for bb2016) never emit them. The
checklist's `GFI rolls 0` for bb2016 is this gap, not the agent.

**F4 — the tier-3 checklist's `action Pass` / `action HandOver` are counted wrong.** They read
`MISSING` in all three editions while 171–210 `PassMove` and 93–113 `HandOverMove` were declared:
the checklist counts the bare action names and the heuristic declares the MOVE variants (BACKLOG
E5). Its `GFI rolls` / `touchdowns` rows are also still annotated "BLOCKED on the
one-move-per-activation decision" while reporting 4,867 and 43.

**F5 — no re-roll event type.** Team re-rolls, Pro, and skill re-rolls leave no `GameEvent`; the
only "reroll" types in the catalog are the kickoff-event ones. The lineman campaign's "501 re-rolls
per 100 seeds" was measured from the dice trace, not from events.

**Interceptions: 0 by contract.** Both agents decline voluntary interference (`AGENT_CONTRACT`),
so the interception path is genuinely unexercised at this tier — a contract choice, not a bug.

## What this means for the goal's third half

The amazon skills ARE reached: the state-hash parity over 900 games (3 editions × 3 scales) is the
proof, and where the events are blind the traces in the ledger are the record. What is NOT yet true
is that the event stream can show it on its own — F1–F5 are the work that would make
`harvest_coverage.sh` a sufficient instrument. They are queued as BACKLOG §E5–E7.


---

# Event coverage — HeuristicAgent, chaos_pact v chaos_pact (harvested 2026-09-01)

Per-edition detail: `EVENT_COVERAGE_chaos_pact_bb2016.md` / `_bb2020.md` / `_bb2025.md` (each a
clean 100-seed run at scale 1.0; parity 100/100 in all three).

## What chaos_pact reached that chaos/chaos_dwarf never did

- **Throw Team-Mate resolves under the heuristic** for the first time (the ITER1 fold): declared
  571/287/495 (bb2016/20/25), RESOLVED (throwTeamMateRoll) 4/15/4 — the huge declared-to-resolved
  gap is the contract's targetless-declaration deselect (a throw needs an adjacent standing
  Right-Stuff goblin, which the one-square-per-activation agents rarely produce; see BACKLOG's
  one-move decision).
- **Regeneration**: 3 `regenerationRoll` events (bb2025; the event fires from the apothecary
  step's reporting) — and the ITER3 failed-regen TRR offer is exercised by parity (the dialog's
  two draws are the proof; declines are unevented, bucket exercised-unevented).
- **skillUse** events: 264/343/271 — dominated by Horns (the blitzing big guys, id 10) plus the
  Goblin's Dodge (id 127). Safe Pair of Hands (ITER2/5/6) and the Animal-Savagery/negatrait
  family are exercised-unevented: their proof is the nine 100/100 gates + the traced dialogs
  (seed 8/99 SkillUse prompts), not the event stream — consistent with findings F1–F5 above.
- `confusionRoll` 4536/…/2844 — the three per-edition big-guy negatraits (Really Stupid /
  Wild Animal / Animal Savagery) fire constantly; bb2025 also shows 32 touchdowns.

Buckets (chaos_pact's new skills): TTM/Always Hungry/Right Stuff exercised+evented;
Regeneration exercised+evented (bb2025) / exercised-unevented (bb2016/20 — rolls happen, no
event site); SafePairOfHands + DumpOff + Swoop + PrimalSavagery exercised-unevented (prompted,
pinned DECLINE by both contracts — their USE paths are undriveable coach dialogs, genuinely dead
under the harness); Animosity — no event, engine-internal (parity is the proof).


---

# Event coverage — HeuristicAgent, dark_elf v dark_elf (harvested 2026-09-01)

Per-edition detail: `EVENT_COVERAGE_dark_elf_bb2016.md` / `_bb2020.md` / `_bb2025.md` (each a
clean 100-seed run at scale 1.0; parity 100/100 in all three).

## What dark_elf reached that no earlier race did

- **The gaze stars execute** (ITER1): `BlackInk` declared 138× (bb2020, star 54496) and
  `AutoGazeZoat` 188× (bb2025, Zoat 39558) — both were silently deselected by the harness before
  this campaign. The gaze itself is exercised-unevented (retirement bit + hashes are the proof).
- **PUNT is driven end-to-end for the first time in ANY campaign** (ITER3-5): 43 `Punt`
  declarations resolved in bb2025 (direction roll, bounce, punter retirement); the PuntToCrowd
  boundary dialog is pinned DECLINE by contract. Punt had been unreachable since the dead-step
  frontier.
- **MultipleBlock under the heuristic**: 67 declarations (bb2020, Helmut Wulf) including the
  defender-picks-the-die `DialogOpponentBlockSelection` path (ITER7 harness case).
- **block 3 dice: 4** (bb2025) — first non-zero since the checklist began tracking it.
- `hitAndRun` 82 (bb2025), `passBlock` 114 (bb2020), `regenerationRoll` 13 (bb2025),
  `throwAtStallingPlayer` 1 (bb2025); dodge volume is the highest of any race (1019 successes
  bb2016) — an all-Dodge/Agility roster.

Buckets (dark_elf's new skills): Black Ink / Zoat gaze / Punt / Multiple Block exercised (gaze +
punt unevented beyond the declaration; hashes + RPUNTDIR trace are the record); Stand Firm
(ITER2) and the Frenzy-blitz Foul Appearance split (ITER7) exercised-unevented (328
`foulAppearanceRoll` bb2020); Dump-Off / Safe Pair of Hands / Swoop / Primal Savagery pinned
DECLINE by contract (ITER6 — the pins are now HARD: a 0.0 wUse through the 1e6 softmax was a
coin flip); Stab/Shadowing (Assassins) present in rosters, no dedicated event — parity is the
proof.


---

# Event coverage — HeuristicAgent, dwarf v dwarf (harvested 2026-09-02)

Per-edition detail: `EVENT_COVERAGE_dwarf_bb2016.md` / `_bb2020.md` / `_bb2025.md` (each a clean
100-seed run at scale 1.0; parity 100/100 in all three).

## What dwarf reached that no earlier race did

- **Beer Barrel Bash executes under the heuristic**: bb2025 declared `ThrowKeg` 385× with 249
  resolved `kegThrow` rolls (fumbles, re-rolls through Thorsson's Loner, and the ONCE_PER_DRIVE
  reset across touchdown drives all live).
- **Wisdom of the White Dwarf**: 176 declarations (bb2025) — grants resolve through the
  PLAYER_CHOICE + SELECT_SKILL chain; the un-acted-deselect mark revert (ITER5) keeps it
  re-offerable exactly as stock Java does.
- **Hail Mary Pass (bb2016)**: 8 declarations resolve through the previously-dead bb2016 twin +
  MissedPass 3-scatter chain (ITER3/4).
- **Break Tackle** (ST-tiered -3/-2/-1) and **Diving Tackle** are exercised-unevented: the proof
  is the nine 100/100 gates plus the kept RDODGEMIN traces — BT consumed only by the dodge it
  saves and covering the rest of the move (ITER7/9), the DT-threat pre-emptive re-roll offer on
  successful dodges (ITER8).
- **Stand Firm prompts** in all three editions (ITER1) — dwarf's Deathrollers exercise the
  bb2016/bb2025 parks the dark_elf campaign only fixed for bb2020.
- Highest dodge volume yet under Break Tackle (748/707/683 dodgeRolls) on a Dodge-less roster.

Buckets: ThrowKeg/Wisdom/HMP exercised+evented; BreakTackle/DivingTackle/StandFirm/Tackle
exercised-unevented (parity + kept gated probes are the record); Defensive/Sprint/Hatred(troll)
present on the roster with no dedicated event — parity is the proof.
