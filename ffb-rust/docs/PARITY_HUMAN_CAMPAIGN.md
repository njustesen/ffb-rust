# Human — heuristic-agent parity campaign

**Goal**: human v human, HeuristicAgent both sides, per-step state-hash parity 100/100, seeds 1-100,
tier 3, editions bb2016/bb2020/bb2025 × scales 1.0/0/1e6 (nine gates), plus random controls and the
standing regression set. Procedure: `.claude/commands/amz-iter.md` with `MATCHUP=human`.
Started 2026-09-04, after goblin (`fc58b0e1c`), halfling (`e6571447c`) and high_elf (`77d99aacb`).

## Surface

Roster in all three editions, and it is the **Ogre + Halfling Hopeful** mix that matters:

- bb2016 (12): Ogre ×1, Blitzer ×4, Catcher ×4, Thrower ×2, Lineman ×1.
- bb2020 (12): Ogre ×1, Blitzer ×4, Thrower ×2, Catcher(old) ×1, **Halfling Hopeful ×3**, Lineman ×1.
- bb2025 (12): Ogre ×1, Blitzer ×2, Catcher ×2, Thrower ×2, Lineman ×2, **Halfling Hopeful ×3**.

The Ogre brings Bone Head (a Select-sequence negatrait) and Throw Team-Mate; the Halfling Hopefuls
bring Right Stuff/Stunty — i.e. the same TTM/Right Stuff and negatrait machinery that goblin
(ITER19/ITER21) and halfling (ITER5) just hardened, which is very likely why this race starts
almost green.

Prior art: there is an older **tier-1 / full-human** campaign in this repo that took human-vs-human
to 100/100 under the *random* agent and the earlier tiers (`docs/PARITY_TIER1.md`,
`docs/PARITY_TTM.md`, memory `parity_tier_human.md`). This campaign is the *heuristic*-agent
nine-gate version of that, and it inherits the benefit.

## Baseline (2026-09-04, measured on `77d99aacb`, nine gates, seeds 1-100 tier 3)

| edition | @1.0 | @0 | @1e6 |
|---|---|---|---|
| bb2016 | **100** ✅ | **100** ✅ | **100** ✅ |
| bb2020 | **100** ✅ | 99 | **100** ✅ |
| bb2025 | **100** ✅ | **100** ✅ | 96 |

**Only 5 reds across the nine gates — seven gates already green.** The smallest frontier of the
sweep so far (goblin opened at 221-equivalent, high_elf at 221, halfling at 45).

Exact reds:
- **bb2020 @0: seed 45** — a single argmax red. Argmax consumes no sampler draws, so this is a
  resolution/content divergence, not a draw-count split: `first_state_divergence.sh` is the right
  entry point.
- **bb2025 @1e6: seeds 3, 51, 60, 66** — red ONLY at @1e6 (both @1.0 and @0 are 100/100). A fault
  that appears only at the uniform-sampling scale is a draw-count/candidate-count split, so read
  `FFB_CANDSUM` (`RSUM`/`JSUM` `n=` and `draws=`) at the first differing activation before tracing
  dice. Note @1e6 makes the softmax uniform over the SAME option set, so a divergence here with
  @1.0 green points at candidate COUNT/order rather than weights.

## ITER1 — both red families closed; nine gates 100/100

Two independent faults, one per family, each proved with the instruments before any code changed.

### bb2020 @0 seed 45 — the Select sequence ran its negatrait block at DECLARATION

`first_state_divergence.sh` put the split at the resolution of `i=202` (`Activate(home_01, Blitz)`),
which both engines declared identically. The dice streams agreed value-for-value to `pos=189` and
split at `pos=190`: Java rolled two block dice there, Rust rolled a **second Bone Head**. Backtraces
(`FFB_DIE_AT=…`) and a gated `BHPROBE` in `bb2020/bone_head_behaviour.rs` named the extra die as
away_01's, spent during activation **201** (`Activate(away_01, Move)`) — an activation for which
`JSTEP i=201 rng_calls=188` → `i=202 rng_calls=188` shows Java spends **zero** dice.

`FFB_MOVEP` showed both agents answering that activation's move window identically
(`k=201 pid=away_01 at=13,7 n=0 offered=[] ans=EndPlayerAction`) — the away Ogre was prone at (13,7)
with all eight neighbours occupied. The difference is *when* the negatraits run:

* Java `StepInitSelecting.executeStep` sets **no** next action for a plain MOVE declaration — only
  `REMOVE_CONFUSION` / `STAND_UP` / `STAND_UP_BLITZ` get `NEXT_STEP`. The Select sequence's
  activation block (`BONE_HEAD`, `REALLY_STUPID`, `TAKE_ROOT`, `UNCHANNELLED_FURY`, `BLOOD_LUST`,
  `JUMP_UP`, `STAND_UP`) runs only once a `CLIENT_MOVE` arrives (`:185`). A player with nowhere to
  step never sends one — the client answers with `ClientCommandActingPlayer(null, null, false)` →
  `fEndPlayerAction` → `GOTO END_SELECTING`. Java rolls nothing.
* Rust folds declaration and dispatch into one `ActivatePlayer`. For an already-STANDING player that
  still matches Java (`goto(label)` → Move sequence → `StepInitMoving`'s own `end_player_action`
  branch jumps to `END_MOVING`, which precedes `StepId::BoneHead`). The `standing_up` carve-out in
  `bb2025/shared/step_init_selecting.rs` is the one path that runs the block immediately.

**Fix** (`step_init_selecting.rs`): a `standing_up` MOVE dispatch with **no legal move targets**
takes Java's deselect — `goto(goto_label_on_end)` publishing `EndPlayerAction(true)` — instead of
`next()`. Same shape as the Blitz/HandOver/Pass/TTM/Keg no-target deselects already in that block.
Test `prone_move_with_no_move_targets_deselects_instead_of_running_the_activation_block`
(verified failing without the guard).

### bb2025 @1e6 seeds 3, 51, 60, 66 — ONE family: a Tackle-cancelled Dodge re-roll re-offered

`FFB_CANDSUM` matched `n=` and `draws=` through `k=38` and split at `k=39` (Rust still on the home
turn at `draws=106`, Java already away at `draws=104`). The resolving activation is seed 3 `i=32`,
`Activate(home_10, Move)` — home_10 is a **Halfling Hopeful (Dodge)**. Java: dodge `pos=64` = 2,
fail, fall, armour `65/66`, **turnover**. Rust: same failed dodge, then a `ReRollOffer{source:
Dodge}`, an accepted re-roll, and a team-re-roll bank driven from `r0` to **`r-1`** (visible in the
`RUST_STEP` state string; Java stays `r0,2`).

Java `StepMoveDodge.dodge`'s failure branch resolves `uncanceledDodgeRerollSource` — null here,
because an adjacent opponent with **Tackle** cancels `canRerollDodge` — and then calls the **PLAYER**
overload with an explicit `reRollSkill` of `null`:
`askForReRollIfAvailable(gameState, actingPlayer.getPlayer(), DODGE, minimumRoll, false,
modifyingSkill, null)`. `RollMechanic` adds exactly one skill term of its own
(`canRerollSingleDieOncePerPeriod`) and never re-derives an action-keyed source, so with an empty
bank no dialog is shown.

Rust called `ask_for_reroll_if_available` — the ACTING-PLAYER overload — which re-runs
`find_skill_reroll_source(game, "DODGE")` and resurrects exactly the Dodge source the Tackle filter
three lines above had just nulled.

**Fix** (`bb2025/move_/step_move_dodge.rs`): that call becomes
`ask_for_reroll_if_available_for(game, player_id, "DODGE", minimum_roll, false)` (Java's player
overload, `reRollSkill = null`). Test
`tackle_cancelled_dodge_reroll_is_not_re_offered_with_an_empty_bank` (verified failing without it).
All four seeds are this one fault.

**Not touched, and recorded as unverified:** the *Diving Tackle* pre-emptive re-roll ask a few lines
earlier (`min_with_dt`) also uses the acting-player overload where Java passes the *uncanceled*
source, and hard-codes `re_roll_source = TRR`. No human seed reaches it (no Diving Tackle on the
roster) and elf/dark_elf are closed, so it was left alone.

### Gates measured this iteration (all on the post-fix binary)

| edition | @1.0 | @0 | @1e6 |
|---|---|---|---|
| bb2016 | **100** | **100** | **100** |
| bb2020 | **100** | **100** | **100** |
| bb2025 | **100** | **100** | **100** |

Random controls (`FFB_PARITY_ROOT=parity_random`, `--agent random`, human v human, 1-100):
bb2016 **100/100**, bb2020 **100/100**, bb2025 **100/100**.

`cargo test -p ffb-engine` **7418 passed / 0 failed** (was 7416; +2 new tests);
`cargo test -p ffb-model` **2802 passed / 0 failed**.

### Closed-roster regressions (post-fix binary, seeds 1-100 tier 3) — 54 runs, all 100/100

`goblin halfling high_elf amazon lineman dwarf chaos chaos_dwarf chaos_pact dark_elf
dark_elf_league_fumbbl elf` × bb2016/bb2020/bb2025 @1.0 (36 runs) plus the three just-closed
rosters (`goblin`, `halfling`, `high_elf`) × 3 editions × @0 and @1e6 (18 runs) — both fixes touch
SHARED bb2025 step code that BB2020 also runs. Zero failures. The known high_elf bb2025 random
control (seed 72) was NOT re-run this iteration; it is a random-agent carry-over unrelated to this
work.

Coverage harvested ×3: `docs/EVENT_COVERAGE_human_bb2016.md`, `_bb2020.md`, `_bb2025.md`.

**🏁 human CLOSED.** Frontier empty.
