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
