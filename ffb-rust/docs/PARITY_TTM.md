# Parity: Throw Team-Mate coverage — OGRE vs OGRE tier

**Goal (user decision, 2026-08-02):** actually exercise the Throw Team-Mate mechanic to parity
(scatter / landing / catch-or-crash / injury), which the lineman AND human tiers never cover — the
human roster fields **no Right Stuff (throwable) player**, so its Ogre can only ever deselect. So we
run a dedicated throwable tier first, then return to the human tier.

**Roster choice: OGRE vs OGRE** (`--home ogre --away ogre`, teams `teamOgreParity*`). Cleanest real
throwable roster — only two positions, no secret weapons:
- **Ogre** ×6 — `Bone-head, Mighty Blow, Thick Skull, Throw Team-Mate` (the thrower)
- **Snotling** ×16 — `Dodge, Right Stuff, Side Step, Stunty, Titchy` (the throwable)

Baseline (2026-08-02): `ogre vs ogre` runs; THROW_TEAM_MATE fires constantly (currently
`UNHANDLED_ACTING_ACTION → deselect` in ParityRunner, and Rust STALLS on the TTM activation —
`prompt_after=None finished=false`). This is effectively a NEW tier: besides TTM it also brings
Bone-head (activation roll like Really Stupid), Snotling Dodge/Stunty/Titchy/Side Step, Mighty Blow,
Thick Skull, and Right Stuff landing — expect those to need aligning too. Log iterations here.

## The engine already has TTM

`Action::ThrowTeamMate { player_id, coord }`, the bb2025 TTM step chain
(`StepInitThrowTeamMate` → `StepThrowTeamMate` → `StepEndThrowTeamMate`), and
`UtilThrowTeamMateSequence` (scatter/land) are all translated. The gaps are the **agent** (doesn't
pick the thrown player + target) and the **dispatch threading** (StepInitSelecting doesn't hand the
thrown player to the TTM step → the stall). The command protocol matches Java exactly (verified):
two `ClientCommandThrowTeamMate` — first the `thrownPlayerId` (pick up → PICKED_UP, step CONTINUEs),
then the `targetCoordinate` (range check → resolve).

## Agent RNG contract (random_agent.rs ⇔ ParityRunner.java — identical, same actionRng)

1. **Action pick** — existing; the activation already spends 1 actionRng choosing the action.
2. **Thrown player** — `legal_throw_team_mate_targets(thrower)` = adjacent teammates whose skill set
   has the `canBeThrown` property (`SkillId::RightStuff`), sorted by `(x,y)`; `pick_action(n)` →
   **+1 actionRng**. Empty list → **DESELECT** (EndPlayerAction, 0 actionRng) — the no-valid-target
   path, same shape as the human Ogre and the Iter44 HandOver deselect.
3. **Target square** — **DETERMINISTIC, 0 actionRng**: `x = clamp(thrower.x + dir*3, 0..=25)`,
   `y = clamp(thrower.y, 0..=14)`, `dir = home_playing ? +1 : -1` (toward the opponent end zone). A
   3-square throw is quick-pass range → in range → the throw resolves. VERIFY the clamp keeps it in
   range at the pitch edges; if an edge case falls out of range the step `cont()`s (would re-stall).

## Implementation pieces

**Rust:**
1. `legal_actions::legal_throw_team_mate_targets(game, thrower_id, side)` → sorted `Vec<PlayerId>` of
   adjacent teammates with `canBeThrown`. (Mirror `legal_handoff_receivers`; a `legal_kick_team_mate_targets`
   too if KTM ever arises — Ogres don't kick, so defer.)
2. `random_agent` ActivatePlayer match — add a `ThrowTeamMate | KickTeamMate` arm: pick thrown player
   → `block_defender_id`. Empty → deselect (mirror the Iter44 HandOff `block_defender_id.is_none()`
   handling routed through the dispatch below).
3. `StepInitSelecting::execute_step` dispatch block — for `PlayerAction::ThrowTeamMate | KickTeamMate`
   with a defender, publish `ThrownPlayerId(defender)` (and `IsKickedPlayer(true)` for KTM); with no
   defender, extend the existing Iter44 Pass/HandOver no-target deselect to include TTM/KTM.
4. New `AgentPrompt::ThrowTeamMateTarget { thrower_id, thrown_player_id }` (ffb-model). Emit it from
   `StepInitThrowTeamMate::execute_step` at the "picked up, no target yet" branch (currently a bare
   `cont()` → the stall) via `cont().with_prompt(...)`.
5. `random_agent` — handle `ThrowTeamMateTarget`: compute the deterministic target →
   `Action::ThrowTeamMate { player_id: thrown, coord: target }` (0 actionRng).

**Java ParityRunner (co-editable; rebuild the jar AFTER, keeping the ffb ENGINE stock):**
6. `sendConcreteAction`: `THROW_TEAM_MATE`/`THROW_TEAM_MATE_MOVE` → `sendThrowTeamMateAction`:
   adjacent `canBeThrown` teammates (coordinate-sorted); empty → `ClientCommandActingPlayer(null,null,false)`
   deselect; else `idx = actionRng % n` → `ClientCommandThrowTeamMate(actingPlayer, thrownPlayerId)`.
7. A waiting-state handler (acting action == THROW_TEAM_MATE, thrownPlayerId set, targetCoordinate
   null) → compute the SAME deterministic target → `ClientCommandThrowTeamMate(actingPlayer, target)`.
8. Rebuild: `cd C:/Users/Admin/niels/ffb/ffb && /c/Users/Admin/bin/maven/bin/mvn -pl ffb-ai -am package -DskipTests`.
   ⚠️ Keep the ffb ENGINE (ffb-common/ffb-server) STOCK — only ParityRunner + the 2 gated-logging
   files change. **After the rebuild, re-verify the LINEMAN tier is STILL 100/100** — the ParityRunner
   change must not perturb non-TTM paths (jar rebuild also risks dropping uncommitted state — commit first).

## Verify
Run `ogre` seeds (individually / small foreground batches). Priority: the TTM throw resolution
itself — scatter direction, landing square, land/catch roll, crash injury — since that's the point.
The other Ogre/Snotling skills (Bone-head, Dodge, Stunty, Titchy, Side Step, Mighty Blow, Thick Skull)
are a normal tier campaign; log each fix here. Return to the human tier ([[parity-tier-human]]) once
ogre is green (TTM will then correctly deselect there — no throwable player).

## Iter 1 (2026-08-02) — TTM machinery implemented (both engines drive the throw)
- **Rust (committed):** `legal_throw_team_mate_targets` helper; `random_agent` picks the thrown player
  on a TTM activation (+1 actionRng) and answers the new `AgentPrompt::ThrowTeamMateTarget` with a
  deterministic 3-square-toward-endzone throw (0 actionRng, mirrored for away); `StepInitSelecting`
  dispatch threads `ThrownPlayerId` (+ no-target deselect for TTM/KTM); `StepInitThrowTeamMate` emits
  the target prompt after pick-up; `uniform_agent` handles it; tests added. Verified: Rust now drives
  a TTM end-to-end (pick up → prompt → `ThrowTeamMate` step) with NO stall/panic.
- **Java (ParityRunner committed in ffb repo; jar rebuilt):** `sendThrowTeamMateAction` (pick thrown
  player, 1 actionRng, empty→deselect) + `INIT_THROW_TEAM_MATE` case → `sendThrowTeamMateTarget` (same
  deterministic target). ffb ENGINE still stock (only DiceRoller + StepGoForIt gated logging).
- **Regression guard PASSED:** lineman tier still green after the jar rebuild (spot-checked 1/7/22/46/57).
- **NEW FRONTIER = ogre seed 1, i=3** (active team differs: Java away / Rust home) — a NON-TTM ogre
  mechanic, reached BEFORE any throw. Almost certainly **Bone-head** (the Ogre's activation roll, like
  Really Stupid) or an early turnover; my TTM changes only touch TTM activations so this is pre-existing
  roster work. NEXT: isolate ogre seed 1 i=3, root-cause the Bone-head/turn divergence, then keep
  advancing — a TTM will eventually be reached and its resolution (scatter/land/catch-or-crash/injury)
  must then be verified to parity (that's still the point; not yet confirmed end-to-end).

## Iter 2 (2026-08-02) — FIRST THROW RESOLVES TO PARITY: fix double-scatter
- **Frontier was ogre seed 1 i=2** (the throw): Java scatters the thrown player once (3 d8) then does
  the Right Stuff landing d6; Rust scattered SIX times. Root cause: Rust's `StepThrowTeamMate`
  (bb2025) pushed a `ScatterPlayer` sequence on a successful throw AND the sequence's separate
  `StepDispatchScatterPlayer` pushed ANOTHER — two throw-scatters. Java's `StepThrowTeamMate` only
  rolls the throw (the "pushes scatterPlayerSequence" comment is stale); `DISPATCH_SCATTER_PLAYER` in
  the generator does the scatter. The extra Rust scatter consumed the correct dice, so the real
  dispatch scattered from fresh (wrong) dice, hit a bystander, and turned the turn over.
- **Fix (`step_throw_team_mate.rs`):** on a successful throw, publish `PassResultParam` + `UsingBullseye`
  and advance — do NOT push a ScatterPlayer sequence (matches the else/unsuccessful branch, which
  already relied on DispatchScatterPlayer). Test `successful_throw_does_not_push_scatter_sequence`.
- **Result:** the first TTM throw now resolves to parity (ogre seed 1: i=3 rng aligned 26/26 both;
  scatter → Right Stuff landing → armour/injury all match). ffb-engine 7009/0.
- **NEW FRONTIER = ogre seed 1 step 6 (i=7):** a SECOND throw (away_06, i=6) after which Java's game
  ENDS (java=None) but Rust continues (away_02 Move) — a turnover Rust lacks, or a game/turn-end
  difference in the second throw's resolution. Root-cause next: compare the away_06 throw dice +
  outcome; likely the thrown player hits an own-team player (turnover) or lands differently.

## Iter 3 (2026-08-02) — fix second-throw infinite loop (Throw Team-Mate is once per turn)
- **Frontier was ogre seed 1 step 6:** Java's ParityRunner LOOPED forever on a SECOND throw (away_06
  after away_05 already threw) — repeated `JAVA_TTM pid=away_06` with alternating idx; the engine
  rejects a 2nd throw (`ttm_used` set) so sendConcreteAction re-fired endlessly and the harness aborted
  (java=None). Rust deselected it (1 die) but the turnover/flow still diverged.
- **Root cause:** Throw/Kick Team-Mate are once-per-team-turn (`ttm_used`/`ktm_used`), but the agents'
  turn-start eligibility snapshot still offered a second one, and `filterStaleActions` didn't drop it
  (only Blitz/Pass/HandOver/Foul were filtered).
- **Fix (both agents):** `random_agent.rs` + ParityRunner `filterStaleActions` now drop `ThrowTeamMate`
  when `ttm_used` and `KickTeamMate` when `ktm_used`. Jar rebuilt; lineman tier still green.
- **NEW FRONTIER = ogre seed 1 step 7 (i=8):** away_02 (an Ogre) BLITZes — identical dice both engines
  (Bone-head pos28-29, block dice 6,4, armour), but Rust TURNS OVER (blitzer a01 + defender h00 both
  fall Prone → "both down") while Java continues (no turnover). A block-result / Bone-head logic diff
  in the Ogre blitz, NOT dice. NEXT: compare the block die[0] result + Bone-head handling (Rust vs Java
  StepBlock/StepInitBlocking + BoneHeadBehaviour); why does the same die give both-down in Rust only.

## Iter 4 (2026-08-02, DIAGNOSIS ONLY — no code change) — Ogre blitz skips Bone-head before the block
- **Frontier: ogre seed 1 step 7 (i=8):** away_02 (Ogre a01) blitzes home_01. Precisely root-caused (deep trace):
  - Both engines share the same dice stream (pos 28-35 = 1,2,6,4,1,2,4,3).
  - JAVA rolls away_02's Bone-head at pos 29 (=2), THEN the block at pos 30,31 = raw [6,4] → [Pow, Pushback];
    picks die[0]=Pow (Defender Down): home_01 pushed to (11,8)+Prone, blitzer stays up, NO turnover.
  - RUST does NOT roll away_02's Bone-head before the block, so the block rolls one die EARLIER at pos 29,30 =
    raw [2,6] → [BothDown, Pow]; picks die[0]=BothDown → blitzer AND defender fall in place → TURNOVER.
  - The `used_skills` double-push guard is INNOCENT (traced `do_roll=true` correctly; the rolling Bone-head for
    away_02 is a LATER turn ~600 lines on). The real issue is the STEP SEQUENCE: the Ogre blitz's block
    sub-sequence `InitBlocking → GoForIt → SteadyFooting → FoulAppearance → DumpOff → BlockStatistics → Dauntless
    → Horns → Trickster → PickUp → CatchScatterThrowIn → Stab → BlockChainsaw → … → BlockRoll` contains **NO
    BoneHead step**. The adjacent blitz dispatches (force_goto, block_defender_id set) EndSelecting → InitBlocking
    directly, skipping the move/negatrait phase where Bone-head lives. Java runs the Bone-head negatrait check
    before the block regardless.
- **NEXT (the fix):** make the Blitz run the Bone-head / negatrait check BEFORE the block even when the blitzer
  is adjacent (no movement). Compare the Rust vs Java bb2025 Blitz/Block sequence: where does Java put the
  negatrait/Bone-head step for a Blitz, and why does a plain Block (away_03) DO roll Bone-head in Rust but the
  adjacent Blitz does not. Likely: the Blitz's `force_goto_on_dispatch`/EndSelecting path skips the negatrait
  sub-sequence that a plain Block goes through; the fix wires the negatrait steps into the adjacent-blitz path.
  Verify: away_02's block then rolls at pos 30,31 = [6,4] → Defender Down, no turnover; seed 1 advances; lineman
  stays green (linemen have no Bone-head so unaffected).

## Iter 5 (2026-08-02) — FIX: Ogre blitz now rolls Bone-head before the block (SelectBlitzTarget activation restored)
- **Root cause (confirmed the Iter4 diagnosis):** Java resolves a blitz in two commands — CLIENT_ACTING_PLAYER(BLITZ_MOVE)
  dispatches BLITZ_SELECT → the `SelectBlitzTarget` sequence, which runs `ActivationSequenceBuilder`
  (the negatrait sub-sequence: Bone Head, Really Stupid, Take Root, Unchannelled Fury, Blood Lust,
  Animal Savagery) BEFORE any move/block; then CLIENT_BLOCK dispatches BLITZ → the `BlitzBlock`
  sequence, which faithfully has NO activation of its own. `PlayerAction.forceDispatch()` is true only
  for FURIOUS_OUTPBURST/FORGO/PUNT — NOT blitz. Rust's random agent picks the blitz + target in one
  Action::ActivatePlayer, so StepInitSelecting `force_goto_on_dispatch` jumps straight to StepEndSelecting
  and SelectBlitzTarget (with its activation) is skipped. For a plain Block the Block sequence re-adds the
  activation, so Bone-head still rolled; for a Blitz nothing did → the blitzer never rolled Bone-head and
  every later die shifted one position.
- **Fix (StepEndSelecting `dispatch_player_action`, PlayerAction::Blitz):** prepend the same plain
  `ActivationSequenceBuilder` (failure → END_BLOCKING) SelectBlitzTarget would have run, ahead of the
  BlitzBlock steps, in one pushed sequence. Bone-head now rolls immediately before the block, matching
  Java's dice order. Test `dispatch_player_action_blitz_prepends_activation_then_blitz_block`.
- **Result:** ogre seed 1 advanced step 7 → step 10 (i=7,8,9,10 hashes now identical). ffb-engine
  7010/0; lineman tier re-verified 100/100 (linemen have no negatraits → activation is dice-neutral).
- **New frontier = ogre seed 1 step 10 (i=11):** away's turn — Java ends it after 4 activations
  (away2 BLITZ, away10, away1, away8 → home), Rust activates a 5th (away_09) then home. away_09 is the
  Right Stuff player THROWN by away_05's TTM at i=2 (landed (18,3), armour held → standing). Java treats
  a thrown player as having used its activation (not re-activatable this turn); Rust re-activates it.
  Engine landing steps don't mark it moved, so this is an AGENT eligibility/snapshot difference
  (ParityRunner.computeEligiblePlayers/filterStaleActions vs random_agent). NEXT: make a thrown player
  ineligible for its own later activation in the same team turn, matching Java.

## Iter 6-8 (2026-08-02) — ogre seed 1 driven step 10 → step 21 (three fixes)
- **Iter6 (random_agent.rs):** a THROWN team-mate lands STANDING but active=false; Java ParityRunner
  tier>=3 rejects ANY picked player with `!PlayerState.isActive()` (not just prone). Rust only skipped
  PRONE inactive players, so the thrown player re-activated. Fixed to `!is_active()`. i=11 fixed.
- **Iter7 (step_throw_team_mate.rs):** a declined TTM reroll re-rolled the pass. Java keeps reRolledAction
  set across the decline (accepts original result); Rust cleared it → fresh roll → 1-scatter FUMBLE became
  a 3-scatter throw. Keep re_rolled_action, clear only the source. i=12 fixed.
- **Iter8 (acting_player.rs + bb2025 bone_head_behaviour.rs):** Bone Head use tracked on persistent
  Player.used_skills, never reset → a player that rolled Bone Head in turn 1 skipped it forever. Java tracks
  it on ActingPlayer.fUsedSkills (cleared by setPlayerId each activation). Added per-activation
  ActingPlayer.used_skills; routed BoneHead through it. i=21 dice divergence moved pos 50 → pos 78.
- **All three: lineman tier re-verified 100/100; ffb-engine 7011/0, ffb-model 2771/0.**

## FRONTIER (2026-08-02) — ogre seed 1 step 21 (i=22, turn 2) — STEP-SEQUENCE divergence, UNSOLVED
- i=13..20 match; first divergent step i=21 = away_06 THROW_TEAM_MATE (pre-hash 44bc8e52 matches, post
  differs). **This is a step-SEQUENCE divergence, NOT a throw-mechanic bug.** Evidence:
  - Rust's 5th TTM pass roll is at dice pos 89; Java's is at pos 77 → Rust does ~12 EXTRA dice rolls before
    its 5th throw in away's turn 2. The two engines' action/step sequences desync somewhere in i=15..21.
  - First dice-TYPE divergence is pos 78: Java rolls a d8 `UtilThrowTeamMateSequence.scatterPlayer` (a
    fumbled throw's single scatter, pass=1 at pos 77), while Rust consumes the same RNG position as a d6
    from a NON-scatter step (verified: no scatter_player/swoop_scatter call at callcount 78; d6 values
    coincidentally align through pos 77, then die-type mismatches at 78).
  - So Java performs a throw (pass 77 + scatter 78) where Rust does other d6 rolls; the state hashes only
    coincidentally matched until i=21.
- **NEXT:** DRIVE_TRACE (FFB_DRIVE_TRACE=1) BOTH engines across away's turn-2 activations (i=15..21) and
  diff the step/activation sequence to find WHERE they diverge (which player activates, which action, or a
  thrown-player/target selection difference). The dice interleave unreliably under buffering — rely on the
  DRIVE step names + per-activation state hashes, not raw dice position. Suspect a TTM target/thrown-player
  selection or an activation-eligibility difference specific to turn 2.
- **Also latent:** bb2020 + bb2016 BoneHeadBehaviour still mark Player.used_skills (same bug Iter8 fixed for
  bb2025) — migrate to ActingPlayer.used_skills when those editions' tiers are worked.
