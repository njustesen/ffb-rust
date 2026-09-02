# Parity campaign — HeuristicAgent on dwarf vs dwarf (all three rulesets)

Started 2026-09-02, immediately after dark_elf closed (nine gates + coverage, pushed `11fb2b600`).
Same goal shape, same three-loop procedure (`.claude/commands/amz-iter.md`, MATCHUP=dwarf).

## Goal

Nine `PARITY: 100/100` — dwarf v dwarf, tier 3, seeds 1-100, heuristic `--heur-classes all`,
bb2016/bb2020/bb2025 × scales 0/1.0/1e6, plus coverage recorded.

## New surface vs dark_elf

- **DEATHROLLER** (all three editions): Break Tackle, Dirty Player(+2 bb2020), Juggernaut,
  Mighty Blow, **No Hands** (bb2016/20) / **No Ball** (bb2025), **Secret Weapon** (ban at drive
  end — the Zzharg family, but on a heuristic that BLOCKS), Stand Firm (now prompts, dark_elf
  ITER2), Loner 5/4 (bb2020/25).
- **Tackle everywhere** (blockers) vs no Dodge on the roster — Tackle's dodge-cancel and
  block-result paths under the heuristic; **Frenzy + Dauntless** (Trollslayers, all editions;
  dauntless roll before every block vs ST4+).
- bb2025: **Defensive** (blockers), **Diving Tackle** (blitzers), **Sprint** (runners),
  **Hatred(troll)** (trollslayer — value is a STRING), special rules "Brawlin' Brutes" +
  "Bribery and Corruption".
- **Stars**: bb2016 Barik Farblast (Hail Mary Pass, Strong Arm, Secret Weapon); bb2025 Thorsson
  Stoutmead (**Drunkard**, **Beer Barrel Bash!**, Loner 4) + Grombrindal (**Wisdom of the White
  Dwarf**, Break Tackle, Dauntless, MB, Stand Firm, Sure Feet).
- Slow team (MV 4-6, Deathroller 4) with high AV — expect long grinding drives, many blocks,
  few passes.

## Baseline (pre-fix probe, seeds 1-20 @1.0)

(to be measured)

Baseline @1.0 (seeds 1-20): bb2016 **7/20**, bb2020 **20/20**, bb2025 **0/20**.

## ITER1 — Stand Firm must PROMPT in bb2016 AND bb2025 (the dark_elf ITER2 fix was bb2020-only)

bb2016 seed 3 (k=6 Trollslayer blitz: RSUM/JSUM same candidates, draws 17 vs 19 — the
JSUM-equal-n/unequal-draws signature) and bb2025 seed 2 (JDRAW SKILL_USE StandFirm@12 vs Rust
nothing): both edition twins still auto-ACCEPTED an undecided Stand Firm inline (written for the
random contract's free always-use), skipping the heuristic's two useSkill sampler draws. Unit:
- bb2016/bb2025 `stand_firm_behaviour.rs`: park `pending_skill_use`, exactly the bb2020 shape.
- `bb2016/step_pushback.rs` had NO park bridge at all (driver.rs:497 dispatches this twin for
  bb2016): added the field, the prompt raise after the hook merge, and the UseSkill arm routing
  StandFirm→standing_firm (mirror of the bb2025 twin).
- Deferred, recorded: Java bb2025's accepted branch also publishes BALL_KNOCKED_LOSE=false +
  CATCH_SCATTER_THROW_IN_MODE=null and reports strip-ball prevention — no StepParameter variants
  exist yet; add when a seed exposes it.
bb2016 seed 3 GREEN after the unit. Tests rewritten from the Java (park, not auto-use).

## ITER2 — Beer Barrel Bash + Wisdom under the heuristic: fold the keg target; the harness declares the PAIRS

bb2025 seed 2 after ITER1: Rust kegThrow `target_id=null, roll=1, fumble` ended the away turn
while Java's keg activation consumed ONE draw, rolled ZERO dice and quietly retired Thorsson.
Two stacked asymmetries:
1. **Rust heuristic never folded the keg target** (only the random agent did): added
   `RandomAgent::fold_keg_target` (ThrowKegLogicModule.isValidTarget mirror, coord-sorted, ONE
   actionRng draw — 1:1 with ParityRunner's kegIdx) and called it from both heuristic answer
   sites (wide + deep), beside the TTM fold.
2. **Java harness deselected every heuristic keg AND wisdom**: `sendStarSpecialDeclaration` had
   no THROW_KEG / WISDOM_OF_THE_WHITE_DWARF cases, so the bare declaration fell to phase 2's
   `default: UNHANDLED_ACTING_ACTION` deselect — the dark_elf ITER1 BLACK_INK shape exactly.
   Added both cases (keg: targets + kegIdx + ActingPlayer(THROW_KEG)+ClientCommandThrowKeg pair,
   empty targets deselect with NO draw; wisdom: ActingPlayer(MOVE)+ClientCommandUseTeamMatesWisdom).
Evidence trail: JAVA_DIE window si=3 zero dice; JSTATE keg window shows NO THROW_KEG step ever ran
in Java; sendConcreteAction's switch has no keg arm; the 872 keg arm belongs to handleStep (random
path). Note bb2025 dwarf specials now EXECUTE under the heuristic on both sides (they were silent
no-ops in every earlier campaign's Java heuristic runs).

## ITER3 — the keg re-roll consumes its offered source; bb2016 HMP goes live end-to-end

After ITER2's probe pass (20-seed: bb2016 19, bb2020 20, bb2025 16, keg-family reds):
- **StepThrowKeg never stored the source it OFFERED**: `ask_for_reroll_if_available` set only
  `re_rolled_action`, so an ACCEPTED keg re-roll (heuristic answered use=true — RREROLLA/JREROLLA
  probes proved both drivers computed the identical wUse=0.4132721 at identical draw totals=132)
  found `re_roll_source=None` and silently no-oped: no TRR spend, no Loner roll, no fresh die.
  Java spent the re-roll, rolled Thorsson's Loner (5≥4, pass) and FUMBLED the re-rolled keg —
  turnover — while Rust played on (bb2025 seed 3 i=48). Same family as chaos_dwarf's declined-HMP
  re-roll fix, on the ACCEPT path. Fix: store the offer's source; the decline arm clears it.
  Test `failed_throw_reroll_offer_stores_the_offered_source`. **bb2025 probe 20/20.**
- **bb2016 HailMaryPass was a dead twin** (driver's shared table dispatched the bb2025 HMP for
  all editions): Java rolls d6 (fumble on 1, else INACCURATE) then MissedPass's 3-d8 scatter
  chain; the bb2025 twin resolves differently. Routed StepId::HailMaryPass → bb2016 twin in
  make_step_for (the TTM-landing precedent). Drive-trace then showed HMP=1 die + MissedPass=3
  dice matching Java — the residue was the TARGET: Java's heuristic phase 2 routes
  HAIL_MARY_PASS to sendPassAction (on-pitch teammates, coord-sorted, ONE actionRng), Rust's
  heuristic declared with no fold. Extracted `fold_pass_receiver` (the random agent's Pass arm)
  and wired it into both heuristic fold sites for HailMaryPass/ThrowBomb/AllYouCanEat (the
  immediate declarations with no move variant).
Instruments kept (env-gated): RREROLLW/RREROLLA (heuristic reroll weight+answer), RLONER,
JREROLLW/JREROLLA (HeuristicDriver, parityDebug-gated).

## ITER4 — the pass-family immediates fold sendPassAction's pick UNCONDITIONALLY

bb2016 seed 20 survived ITER3's twin-routing because the DECLARATION target still differed: the
heuristic's pass plan pre-sets its own receiver, so the target-is-none fold never fired (no
RTTMFOLD in the trace), while Java's phase 2 routes HAIL_MARY_PASS (and THROW_BOMB /
ALL_YOU_CAN_EAT — no move variants) to sendPassAction, which IGNORES the plan and re-picks from
the coord-sorted on-pitch teammates with ONE actionRng draw (Java (12,9) vs Rust's planned
square). Extracted `fold_pass_receiver` from the random agent's Pass arm (one contract, one
copy) and made the override unconditional for those three at both heuristic answer sites.
**20-seed probes: bb2016 20/20, bb2020 20/20, bb2025 20/20.** ffb-engine 7383/0.

## ITER5 — an un-acted deselect REVERTS the granted-skill marks (Wisdom re-offers all game)

@1.0 gates: bb2016 100/100, bb2020 100/100, bb2025 **85/100** — all 15 reds were
first-pick-of-a-new-turn DECLARATION splits, and seed 21's k=16 candidate lists differed by
exactly ONE row: Java still offers away3's WISDOM, Rust had withdrawn it. Probes settled it:
`JAVA_WISDOM_MARK usedAfter=true` at the grant, then `JAVA_WISDOM_OFFER usedProp=false` next
activation — Java's `UtilActingPlayer.changeActingPlayer` runs, in its oldPlayer!=newPlayer
branch, an `if (!actingPlayer.hasActed())` cleanup that (a) strips the bb2025
enhancementsToRemoveAtEndOfTurn set ({Wisdom}) from the old acting player, (b) walks
`skillsGrantedBy`: the GRANTER's mark is `markUnused`-REVERTED, other recorded players lose the
granted enhancement. Grombrindal declares Wisdom (declared action stays MOVE), grants, and if the
activation ends without him acting the ONCE_PER_GAME mark is undone — stock Java re-offers
Wisdom every such activation. Rust had NO deselect cleanup and its `skills_granted_by` ledger
was never cleared. Unit: `cleanup_granted_skills_on_deselect` in util_server_steps (both
transition sites — change_player_action's changed-branch and change_player_action_to_none),
`skills_granted_by.clear()` in set_player + the null-reset, shared WISDOM_ENHANCEMENT_SOURCE
const with the wisdom step. Tests written from the Java (revert on un-acted, keep on acted).

## ITER6 — a touchdown drive-end must reset ONCE_PER_DRIVE skills (the keg re-offers)

After ITER5 the frontier fell 15→4; seed 22's k=50 lists differed by ONE row again — Java
re-offers away2's THROW_KEG. Java `StepEndTurn` (bb2025:394, the fTouchdown branch) calls
`stateMechanic.resetSpecialSkillAtEndOfDrive` — Beer Barrel Bash (ONCE_PER_DRIVE) comes back
after a TD drive. Rust ran that reset only from the HALF-TIME path (its touchdown branch carried
a literal `// Stub: ... resetSpecialSkillAtEndOfDrive → skip`). Fix: the call in the touchdown
branch; test `touchdown_resets_once_per_drive_skills`. ITER5's tests repaired (empty test_team).
Frontier after ITER5+6: seeds 22/60 GREEN, remaining **43** (t3 away_01 Move resolution) and
**61** (t1 h2 home_04 Blitz resolution). Temp stock probe (JAVA_WISDOM_MARK) reverted; stock
diff back to the pre-existing gated-logging set.

## ITER7 — Break Tackle is consumed by the dodge it saves (both bb2020+bb2025 twins)

Seeds 43/61: same dice, second-dodge target R=3 vs J=6. JAVA_DODGEMIN/RDODGEMIN probes (temp
stock probe + permanent FFB_TRACE Rust probe) named it: Java's dodge 1 carried "Break Tackle
ST 5+" (-3, the Deathroller), dodge 2 carried only "1 Tacklezone" — Java `StepMoveDodge`
(bb2025:516-521): a dodge that succeeds only thanks to the use-strength modifier sets
fUsingBreakTackle and `actingPlayer.markSkillUsed(btSkill)` — ONE Break Tackle per activation;
Java also DROPS the modifier when the roll succeeds bare (363-372) or fails regardless
(389-400, WOULD_NOT_HELP), so BT is only consumed when it mattered. Rust applied BT to every
dodge of the activation and never marked it. Unit ported into BOTH edition twins (bb2020's BT is
-2/-1 — its @0 gate had 1 red, likely this). Marks land on Player.used_skills (the factory gate)
AND acting_player.used_skills (acted()). **Seeds 43 + 61 GREEN — the bb2025 @1.0 frontier is
EMPTY.** Probes kept: RDODGEMIN (FFB_TRACE); temp JAVA_DODGEMIN to revert before the gating
rebuild.

## Post-ITER7 nine-gate battery (interim)

bb2016 100/100/96 (4 reds @1e6) · bb2020 **100/100/100** · bb2025 100/99/98 (1 red @0, 2 @1e6);
randoms ×3 100/100. Remaining reds are being re-run with full logs (the battery loop's `tail -2`
clipped the seed numbers — twice now; keep full logs for gates). Inner loop continues on them.

**ITER7 confirmed on gate:** bb2016 @1e6 re-run 100/100 (all four Deathroller-Move reds were the
un-consumed Break Tackle), @1.0 probe 20/20. Board: bb2016 **100/100/100**, bb2020 **100/100/100**,
bb2025 100 @1.0 / 1 red @0 (seed 3) / 2 reds @1e6 (39, 57). Seed 3 @0 inner loop: k=163 lists and
argmax IDENTICAL, zero dice, yet the i=164 pre-state hash differs — RSTATE extended to carry
coordinates to name the moved piece.

## ITER8 — the Diving-Tackle THREAT re-roll on a SUCCESSFUL dodge (bb2025 only)

bb2025 @0 seed 3: the divergence hid behind THREE instrument blind spots in a row — the state
string/hash cover only the first 11 players by nr (`addPlayersFromTeam subList(0,11)`, mirrored
in Rust state_hash.rs — dwarf's nr12/13 blitzers are hash-invisible), RSTATE printed no
coordinates or ball flags (both added, coords + ball/moving/inplay, kept), and the harness state
string lacked ballMoving (",mv" added, kept). The trail: identical k=163/164 lists+argmax,
zero-dice divergence, ball (1,12) vs (8,8) + r2,1 vs r2,2 at i=165. Truth: away_05 (carrier)
dodges (4,10)→(3,11), roll 4 vs min 3 SUCCEEDS on both sides — but a hash-invisible nr12/13
blitzer (Diving Tackle) marks the square: Java bb2025 StepMoveDodge's SUCCESS branch (:439-475)
sees minWithDt = min+2 = 5 > 4, tries a Break-Tackle rescue, then pre-emptively OFFERS a re-roll
("Diving Tackle can make this dodge fail. Reroll the dodge now?") — accepted: TRR spent, fresh
6. Rust's bb2025 twin skipped the whole block (a "client-only" comment). Ported: DT-threat check
(eligible tacklers via find_eligible_diving_tacklers + DIVING_TACKLE_LEAVING_TZ_ONLY), BT rescue
(consume on the spot), the pre-emptive offer gated on dt_reroll_asked, and Java's re-entry
carve-out (declining the DT re-roll keeps the successful roll — must not fail the dodge).
bb2020's twin has no such block (bb2025-only). Instruments kept: RSTATE coords+ball, Java ",mv".

**Instrument lesson (cost one false-red round):** the Java per-step hash is FNV-1a over the
CANONICAL `stateString` — appending the ",mv" debug field there changed every Java hash and
turned all seeds "diverges at the very first step". Debug fields go on the JSTEP print line,
never into the canonical string.

## ITER9 — a committed Break Tackle covers the rest of the move

bb2025 @1e6 seed 57 (the last red): the Deathroller's SECOND dodge of one move — Java keeps the
BT modifier (min 2) because `findModifiers(new DodgeContext(..., fUsingBreakTackle))` includes BT
via the context's `isUseBreakTackle()` disjunct even after markSkillUsed; Rust's DodgeContext was
built with the 4-arg ctor (use_break_tackle always false), so ITER7's mark withdrew BT
mid-activation (min 5 → fall → turnover). Fix: all three edition twins now pass the step's
`using_break_tackle` (Java passes fUsingBreakTackle in every edition). Java's full i=118 flow
reproduced: kept-BT dodge ok → DT-threat re-roll offer (declined) → defender's DIVING_TACKLE
PLAYER_CHOICE → move continues. **Seed 57 GREEN; 20-seed probes bb2016/bb2020/bb2025 all 20/20;
frontier EMPTY.** Full nine-gate battery running.

## 🏁 NINE GATES GREEN (2026-09-02, post-ITER9 battery)

dwarf heuristic bb2016/bb2020/bb2025 × scales 1.0/0/1e6: **ALL 100/100**; random controls ×3
editions 100/100. ffb-engine 7389/0. Standing gates: amazon ×3 + lineman ×3 ALL 100/100. Family regressions: chaos / chaos_dwarf /
chaos_pact / dark_elf ×3 editions each ALL 100/100. Coverage harvest ×3 next, then memory + PUSH.
