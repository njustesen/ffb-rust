# Parity campaign — HeuristicAgent on chaos_pact vs chaos_pact (all three rulesets)

Started 2026-09-01, immediately after chaos_dwarf closed (nine gates 100/100). Same goal shape,
same three-loop procedure (`.claude/commands/chaos-iter.md` applies with MATCHUP=chaos_pact).

## Goal

Nine `PARITY: 100/100` — chaos_pact v chaos_pact, tier 3, seeds 1-100, heuristic
`--heur-classes all`, bb2016/bb2020/bb2025 × scales 0/1.0/1e6, plus coverage recorded.

New skill surface vs chaos/chaos_dwarf: the three Big Guy negatraits together (Really Stupid /
Wild Animal / Animal Savagery per edition), **Throw Team-Mate live under the heuristic for the
first time** (Troll/Ogre + goblin-sized Marauders? — no: thrown = the Goblin renegade), Always
Hungry, Regeneration (Troll), Safe Pair of Hands + Dump-Off (Renegade Thrower, bb2020/25),
Animosity (renegades), plain-valueless **Loner** on the big guys (bb2025).

## ITER1 (recovered session, 2026-09-01, pre-/clear) — six units, baseline 0/100 → 20/20|19/20|20/20

The session that started the campaign was cleared mid-iteration; its work was recovered from the
uncommitted tree and re-measured. Units (all uncommitted at recovery, committed together here):

1. **TTM/KTM candidate arm removed** (`heuristic_agent.rs`): Java's ActivationChoice has NO arm
   for THROW_TEAM_MATE — it lands in the `default:` (ONE candidate, no target). The bespoke Rust
   arm built one candidate PER TARGET at 0.35, so every draw after the first activation read a
   different stream — **baseline 0/100 in ALL THREE editions** (bb2025 seed 2 k=1: Java n=2000,
   Rust n=1998).
2. **TTM target fold** (`heuristic_agent.rs` + `random_agent.rs::fold_ttm_target`): Java declares
   TTM then picks the thrown player at phase 2 via `sendThrowTeamMateAction` (coord-sorted, ONE
   actionRng). Rust's targetless declaration deselected instantly — no TTM ever resolved. The
   identical pick is folded into the declaration from the same action_rng stream.
3. **Valueless Loner defaults to 4** (`player.rs::get_skill_int_value` +
   `skill_id.rs::default_skill_value`): Java falls back to `Skill.getDefaultSkillValue` (mixed/
   Loner = 4, MightyBlow/DirtyPlayer bb2020 = 1); a bare 0 auto-passed every Loner check
   (bb2025 seed 2 i=10, the Troll's dodge re-roll).
4. **Animal Savagery accepted re-roll re-executes** (`step_animal_savagery.rs`): the `_ =>
   next()` wildcard swallowed `UseReRoll{true}` — no spend, no Loner check, no fresh die, the
   failed roll forgotten (bb2020 seed 18 i=9).
5. **Safe Pair of Hands offer prompts** (`bb2020+bb2025/shared/step_place_ball.rs`): auto-decline
   spent ZERO draws where Java's driver spends two on the dialog (bb2020 seed 8 i=47). Plus the
   agents pin DumpOff/PrimalSavagery/SafePairOfHands/Swoop to DECLINE (wUse 0.0, both agents —
   their USE paths are undriveable dialogs).
6. **Scripts take MATCHUP=<race>** (first_state_divergence/frontier/harvest_coverage), and the
   declaration normalizer strips the harness's `*_MOVE` suffixes (FOUL_MOVE, HAND_OVER_MOVE…).

## ITER2 — PLACE_BALL takes its playerId from DROPPED_BALL_CARRIER (bb2020 seed 8)

The recovered SPoH unit was necessary but not sufficient: seed 8 stayed red because the LIVE
place-ball twin (bb2025 — `make_step` imports `bb2025::shared::*` for ALL editions; the edited
bb2020 twin is dead code) never had a playerId. Java's `setParameter` maps **DROPPED_BALL_CARRIER
→ playerId** (published by `UtilServerInjury.dropPlayer` for an eligible carrier — there is NO
PLAYER_ID arm); Rust consumed the parameter and threw it away, so `player_id` stayed `None` and
the dialog never fired. Also ported: `ballCarrierTeamTurn` — a non-active-team carrier's dialog
flips `homePlaying` for the dialog's lifetime and `leave()` flips it back + publishes
DROPPED_BALL_CARRIER = null. Trace: `RPLACEBALL` absent in a FFB_TRACE run + `JSTATE i=48
step=PLACE_BALL dialog=SKILL_USE home=true` on matched boards, rng 80 = 80 (engine dice equal;
only the AGENT streams parted — the two useSkill draws).

Tests: `set_parameter_dropped_ball_carrier_sets_player_id`,
`opponent_carrier_dialog_flips_home_playing_and_decline_restores` (bb2025 twin; bb2020 twin's
stale PlayerId test rewritten from the Java).

**Measured after ITER1+2**: @1.0 bb2016 **100/100**, bb2020 **100/100**, bb2025 95/100
(reds 43 50 56 83 97). ffb-engine 7377/0, ffb-model 2799/0.

**Frontier (bb2025)**: seed 43 idx8 — resolving `Activate(home_03,Blitz)` (the TROLL, skull,
attacker down, casualty, **failed Regeneration d6=3**): Java raises RE_ROLL_PROPERTIES at
step=APOTHECARY (bb2025 `StepApothecary.executeStep` pre-regeneration:
`UtilServerReRoll.askForReRollIfAvailable(player, REGENERATION, 4, false)` → bb2025
`RollMechanic.askForReRollIfAvailable` shows DialogReRollProperties iff TRR available) — the
driver spends 2 draws; Rust never offers, streams split, Java's away turn 2 ends with zero
activations (chooseActivation null) where Rust plays it. bb2025-ONLY: bb2020/StepApothecary has
handleRegeneration but NO askForReRoll. NEXT: port the regen re-roll offer into
`bb2025/shared/step_apothecary.rs` (edition-gated `rules == Bb2025`), re-entry-safe (Rust's
folded flow rolls regen AFTER the apo dialog where Java rolls it before — the 2 offer draws land
at the same stream positions either way because the apo handlers consume 0 draws on both sides).

## ITER3 — a failed Regeneration gets a team re-roll offer (bb2025 only; seeds 43/56/83)

Java bb2025 `StepApothecary.executeStep` (pre-regeneration): casualty + Regeneration + failed roll
→ Igor inducement first (absent from parity teams), then
`UtilServerReRoll.askForReRollIfAvailable(player, REGENERATION, 4, false)` — the bb2025
RollMechanic's `DialogReRollPropertiesParameter`, shown iff a team re-roll is available. The
accepted answer spends via `useReRoll` (Loner rolled inside — the Troll's valueless Loner=4 from
ITER1) and rolls a FRESH regen die; success clears the casualty + seriousInjury and sets
RESULT_CHOICE. **bb2020's StepApothecary has NO such ask — edition-gated.** Rust's folded flow
rolls regen AFTER the apo dialog (Java before it); the orders consume identical engine dice and
agent draws (both apo handlers spend zero), recorded as a deliberate fold. The step's tail
(Getting Even + side effects) was split into `finish_tail` so the re-roll answer resumes without
re-running the status switches. **Wrong first cut, caught by the seed set moving EARLIER (i=8→i=6)
not smaller**: the helper returns `(false, None)` for a casualty WITHOUT Regeneration, and
offering there invented a dialog Java never shows — the ask is gated on the skill.
Tests: `accepted_regen_reroll_spends_trr_and_rolls_fresh`, `declined_regen_reroll_spends_nothing`.

## ITER4 — only an ACCURATE bb2025 TTM throw skips the re-roll offer (seeds 50/97)

Java bb2025 `ThrowTeamMateBehaviour`: `passResult == ACCURATE → handlePassResult; else … ask` —
an INACCURATE throw still scatters AND is offered a re-roll first. bb2020's behaviour asks only
when `!successful` (ACCURATE and INACCURATE both skip). Rust's shared step used the bb2020 shape
for every edition, so the d6=4 INACCURATE goblin throw got no offer — Java's driver spent two
draws on the dialog and the streams split ~25 draws before the next matched dialog (the two-pointer
class-matched JDRAW/RDRAW comparison found it; the plain zip kept slipping on classes only one
side prints). Fix: `offer_eligible = (bb2025 ? result != Complete : !successful)`, offer hoisted
above the successful-branch. Test: `inaccurate_throw_offer_is_bb2025_only`.

**Gate**: chaos_pact @1.0 bb2016 **100** / bb2020 **100** / bb2025 **100** 🏁; ffb-engine 7380/0,
ffb-model 2799/0. rust_total 45.6s/45.4s (bb2020/bb2025, 100 seeds). Scales + standing gates next.

## ITER5 — has_ball at the injury drop-site is Java's hasBall, not coordinate equality (bb2020@1e6 seed 99)

`UtilServerInjury.dropPlayer`'s ball handling uses `UtilPlayer.hasBall` = `ballInPlay &&
!ballMoving && coords equal`. Rust's local check was bare coordinate equality, so a defender
PUSHED ONTO A LOOSE BALL counted as its carrier: `DROPPED_BALL_CARRIER` was published and
StepPlaceBall raised a phantom Safe-Pair-of-Hands dialog Java never shows (seed 99 i=59: away_03
blocks home_04 into the moving ball at (11,7); Java followup@165 → next pick@167, Rust burned two
extra draws on the dialog; found via RSUM/JSUM — candidate sets IDENTICAL at n=1764, only the
draws differed). Fix: call the existing 1:1 `UtilPlayer::has_ball`. The same variable feeds the
acting-team turnover check below it — also Java's shape.

**Full re-verification on the new binary (the fix is in shared injury code): chaos_pact
🏁 ALL NINE gates 100/100** (bb2016/bb2020/bb2025 × 0/1.0/1e6); amazon ×3 + lineman ×3 100/100;
chaos ×3 + chaos_dwarf ×3 100/100; ffb-engine 7382/0, ffb-model 2799/0.

**Random-control frontier**: chaos_pact random bb2016/bb2025 100/100; bb2020 93/100 — six
NO_PROGRESS aborts, all spinning on the NEW SkillUse(SafePairOfHands) prompt: ParityRunner's
random contract DECLINES SafePairOfHands (same list as DumpOff/PrimalSavagery/Swoop) but
RandomAgent had no arm — the generic always-use answer carries the Block placeholder, which
StepPlaceBall's property gate ignores, so the prompt refired forever. → ITER6.

## ITER6 — RandomAgent declines Safe Pair of Hands (the parity contract's decline list)

`ParityRunner`'s SKILL_USE random contract declines exactly four skills — DumpOff,
PrimalSavagery, SafePairOfHands, Swoop (each opens a dialog the harness cannot drive).
RandomAgent had arms for three; SafePairOfHands fell to the generic always-use arm, whose
`SkillId::Block` placeholder fails StepPlaceBall's property gate → the prompt refired until
NO_PROGRESS aborted the game (bb2020 random seeds 9/23/25/55/83/94 — the prompt is new since
ITER2; before it the step auto-declined silently). Fix: the decline arm, mirroring the Java list.

**🏁 CAMPAIGN PARITY COMPLETE**: chaos_pact heuristic ×3 editions ×3 scales ALL 100/100;
random controls chaos_pact ×3 **100/100** (+ amazon ×3, lineman ×3 random 100/100); standing
amazon ×3 + lineman ×3 and chaos ×3 + chaos_dwarf ×3 heuristic 100/100; ffb-engine 7382/0,
ffb-model 2799/0. Remaining: coverage harvest ×3 + docs + push.

## 🏁 CAMPAIGN COMPLETE 2026-09-01

All three halves: (1) nine parity gates 100/100 + chaos_pact random ×3 100/100; (2) the agent
plays the race — TTM resolves via the ITER1 fold (4/15/4 resolved throws per edition), the four
undriveable-dialog skills pinned DECLINE in BOTH agents; (3) coverage harvested →
`EVENT_COVERAGE_chaos_pact_*.md` + the summary section in `EVENT_COVERAGE.md`. Standing:
amazon ×3, lineman ×3, chaos ×3, chaos_dwarf ×3 all 100/100; ffb-engine 7382/0, ffb-model 2799/0.
