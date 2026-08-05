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

## Iter 9 (2026-08-02) — FIX: TTM-landing injury applies Stunty (huge advance step 21 → step 105)
- **Root cause:** `make_injury_type("InjuryTypeTTMLanding")` dispatched to a stale duplicate
  `InjuryTypeTtmLandingImpl` (in injury.rs) that rolled the injury with the player-less
  `do_injury_roll` — so Stunty was never applied. bb2016/bb2020 StepRightStuff construct the proper
  `injuryType::injury_type_ttm_landing::InjuryTypeTTMLanding` directly; only bb2025 dispatches by name.
  So an Ogre-thrown Stunty Snotling landing on an injury total of 7 was Stunned in Rust vs KO in Java
  (7 + Stunty = 8) — ogre seed 1 i=21 (away_06 threw the Snotling away_10).
- **Fix:** route the name dispatch to the full `InjuryTypeTTMLanding` (find_injury_modifiers +
  do_injury_roll_for_player, turnover=true, send-to-box=LandingFail); delete the dead duplicate.
  Test `ttm_landing_by_name_applies_stunty_ko_on_seven`. ffb-engine 7012/0, lineman 100/100.

## FRONTIER (2026-08-02) — ogre seed 1 step 105 (i=106, turn 6→7) — Mighty Blow armour-vs-injury, UNSOLVED
- i=13..104 match; first divergent step i=105 = home_01 (Ogre) BLITZ. pre-hash 8af0fad5 matches, post
  differs. **State-only divergence** (dice match): away_08-MOVE state at i=106 differs ONLY in h00
  (home_01): **Java `11,7,Stunned` vs Rust `-1,-1,Ko`**.
- home_01 blitzes away_02, gets a BothDown (block die 2) and FALLS (both-down attacker). Its landing
  injury: armour [6,2]=8, injury [5,3]=8. home_01 is an Ogre (has Thick Skull → 8 converts KO→Stunned).
  But Rust's injury total = **9** because it adds away_02's **Mighty Blow (+1)**; total 9 is past the
  Thick-Skull 8-only conversion → KO. Java keeps total 8 → Stunned.
- **Root cause (to fix):** Mighty Blow is armour-OR-injury, not both (Java `MightyBlow` injury modifier
  `appliesToContext` excludes itself when an armour modifier registered to
  `affectsEitherArmourOrInjuryOnBlock` was already applied — see MightyBlow.java + InjuryTypeBlock
  Mode.REGULAR armourRoll/injuryRoll). Java applies the opponent's MB to home_01's ARMOUR roll (8+1=9,
  breaks AV) and therefore NOT to the injury (stays 8 → Stunned). Rust applies MB to the INJURY instead
  (→9 → KO). The attacker-fall injury goes through `handle_injury_by_name("InjuryTypeBlock")` →
  InjuryTypeBlock(Regular). NEXT: replicate the armour-first / mutual-exclusion choice for the
  `affectsEitherArmourOrInjuryOnBlock` (Mighty Blow) modifier in the Rust InjuryTypeBlock armour+injury
  path — check `do_armor_roll` / `find_injury_modifiers` and whether the armour application marks MB as
  consumed so the injury excludes it. First exercised only now (Ogres are the first Mighty-Blow players).

## Iter 10 (2026-08-02) — FIX: Mighty Blow armour-OR-injury exclusion (step 105 → 143)
- Java's MightyBlow injury modifier excludes itself when an armour modifier registered to a skill
  with `affectsEitherArmourOrInjuryOnBlock` is already applied (MB is +1 to armour OR injury, never
  both). Rust offered it to both (known unported gap). A both-down Ogre fall got MB on the injury
  (8→9), past Thick Skull's 8-only KO→Stun → KO vs Java's Stunned (ogre seed 1 i=105, home_01). Fix in
  InjuryTypeBlock.injury_roll: skip an affectsEither injury modifier when a same-named armour modifier
  is present. Test mighty_blow_on_armour_is_excluded_from_injury. lineman 100/100, ffb-engine 7013/0.

## DIRECTION CHANGE (2026-08-02 ~22:10) — user switched the loop target to FULL HUMAN vs HUMAN
- Per user decision, the OGRE detour is paused (its Iter5-10 fixes are general engine corrections and
  stay committed). The loop now drives HUMAN vs HUMAN (bb2025, tier3, seeds 1-100) to green — the
  actual stated goal. TTM's throw path won't be exercised there (human roster has no Right Stuff /
  canBeThrown player; its Ogre can't find a throwable teammate → TTM only deselects). Hard rule: the
  loop must STOP at 02:00 local.

## FRONTIER (human) — seed 1 step 39 (i=39, turn 3) — action_rng desync from a no-op move
- i=1..38 match; first divergence i=39 = away_09 MOVE (from (21,5)): Java → (21,4), Rust → (22,4).
  Same 6-square target list, but Java picks idx 2 vs Rust idx 3 → the ACTION_RNG is misaligned by 1.
- Root: at i=37 both activate away_01 (at (14,7)) for MOVE — a no-op in BOTH (post==pre). Java's
  ParityRunner.sendMoveAction computes adjacent-empty squares ITSELF (7 targets), picks (14,6) and
  sends the move (consuming 1 actionRng); the engine rejects/no-ops it. Rust's agent uses the engine's
  Move-prompt `squares`; away_09... i.e. away_01 there produced NO move-target pick (deselect,
  0 actionRng). So Java consumes 1 actionRng where Rust consumes 0 → every later pick shifts.
- NEXT: reconcile the move-target source. Java's agent picks from ALL adjacent-empty squares
  regardless of the engine's move legality (0-MA / pinned still counts, move gets rejected → no-op but
  actionRng consumed). Rust must consume actionRng identically for such a no-op move. Determine why
  Rust's away_01 at i=37 yields no move-pick (engine gives 0 squares? deselect path?) — grep
  RUST_ACT_PICK / RUST_SMA / RUST_PICK per activation, compare vs JAVA_SMA/JAVA_PICK counts (they match
  through pick 31 (away_08), diverge at pick 32: Java=Away1→(14,6), Rust=Away6). Verify lineman stays
  100/100 (its moves must not regress).

## FRONTIER (human) REFINED (2026-08-02 ~22:45) — prone-player Move consumes actionRng in Java only
- Definitive: at i=37 away_01 (a00) is PRONE at (14,7). It activates for MOVE (= stand up). In BOTH
  engines it STAYS PRONE (no-op, post==pre) — neither stands it up. But Java's ParityRunner.sendMoveAction
  still computes adjacent-empty squares from (14,7) [7 targets], picks (14,6), and sends a CLIENT_MOVE
  that the engine ignores (a prone player can't move via CLIENT_MOVE) — consuming 1 actionRng. Rust's
  agent is prompt-driven: the prone stand-up move never reaches an AgentPrompt::Move (no RUST_SMA for
  away_01 at i=37; there is NO RUST_SMA N=0 anywhere — the engine short-circuits), so it consumes 0
  actionRng. Net: Java consumes 1 actionRng more than Rust at i=37 → every later action pick shifts →
  first visible divergence at i=39 (away_09 move: same 6-target list, Java idx 2=(21,4) vs Rust idx
  3=(22,4)).
- NEXT (the fix — AGENT, random_agent.rs): make Rust consume a move-target actionRng for EVERY Move
  activation, matching Java's sendMoveAction (which always computes adjacent-empty + picks, even when the
  engine will ignore the move — prone stand-up, 0-MA, 0-legal-move). Cleanest: when the agent picks
  MOVE at the ActivatePlayer action-choice, compute adjacent-empty on-pitch squares from
  gs.game.field_model (sorted by (x,y), like Java's dx/dy loop + sort), pick_action(len) [1 actionRng],
  remember it; at the follow-up AgentPrompt::Move, return the remembered square WITHOUT a second
  pick_action. Verify lineman 100/100 (normal standing moves must be unchanged — same 1 actionRng, same
  target, same order). Prone player + Move = the exercised case here; confirm the human roster's prone
  players (players knocked down earlier) reproduce it.

## FRONTIER (human) — Iter attempt (2026-08-02 ~23:20): first fix REVERTED, deeper root isolated
- Tried: Rust agent consumes 1 actionRng for a prone player's Move (compute adjacent-empty count,
  pick_action if >=1, set moved_this_activation). REGRESSED step 39 -> step 29 — because it also
  consumed for prone players that DO stand up + move (already aligned via the Move prompt), and the
  moved_this_activation guard then made them stand-but-not-move (state divergence). Reverted.
- DRIVE-trace of away_01's i=37 activation (FFB_TRACE + FFB_DRIVE_TRACE, extract between RUST_STEP i=37
  and i=38): away_01 (PRONE at 14,7) → InitSelecting → ResetFumblerooskie → EndSelecting → EndPlayerAction
  (InitFeeding/StallingPlayer/.../EndFeeding) — it DESELECTS, never reaching InitActivation / StandUp /
  InitMoving / a Move prompt. Contrast away_06 (i=38, STANDING) → InitSelecting → EndSelecting → InitMoving
  (dispatches Move). So the Rust ENGINE deselects away_01's prone Move at StepInitSelecting/StepEndSelecting,
  consuming 0 actionRng; Java's ParityRunner.sendMoveAction still computes 7 adjacent-empty + picks (14,6)
  [1 actionRng], sends CLIENT_MOVE which the engine ignores (stays prone). Net: Java +1 actionRng at i=37.
- So NOT all prone Moves diverge — only the ones the Rust engine DESELECTS (away_01) vs DISPATCHES
  (step-28 player stands up + moves, both pick, aligned). NEXT: instrument StepInitSelecting to see WHY
  away_01's prone Move takes the deselect (goto END_SELECTING + EndPlayerAction) path while another prone
  player's dispatches — is standing_up not set? update_move_squares 0? a no-target/prone deselect branch?
  Then reconcile: EITHER make the Rust engine dispatch (reach the Move prompt so the agent picks, matching
  Java — but must keep the player prone to match state), OR make the agent consume 1 actionRng ONLY for the
  prone-Move-that-will-deselect case. Verify lineman 100/100. 0 game dice consumed at i=37 (no stand-up roll).

## FRONTIER (human) — SOLVED (2026-08-02 ~23:34): the Ogre Bone-head, NOT a "no-op prone move"
- CORRECTION to the prior characterization: away_01 is the human team's **OGRE** (Big Guy — Bone Head,
  Throw Team-Mate; it did THROW_TEAM_MATE at i=9). The prior "prone no-op Move / engine deselects with
  0 dice" reading was WRONG: a game die IS rolled (rng_calls 31->32) and it is the **Bone-head roll**
  (JAVA_DIE d6=1 from DiceRoller.rollSkill), which FAILS — the Ogre stays PRONE+confused (a00:14,7,Prone
  through i=37/38/39). Both engines roll Bone-head and fail (dice match). The ONLY divergence is a
  1-actionRng difference in when/whether the move target is drawn.
- ROOT CAUSE (move-prompt vs Bone-head ORDER). Both engines set standing_up=true for the prone Ogre's
  MOVE (changeActingPlayer: standingUp = was_prone). The Select sequence is InitSelecting -> ACTIVATION
  (InitActivation..**BONE_HEAD**..BloodLust) -> JumpUp -> StandUp -> EndSelecting -> dispatch Move seq
  (InitMoving emits the Move prompt). For a standing_up player Rust does next() INTO the Select ACTIVATION,
  so BONE_HEAD rolls, FAILS, and ends the activation BEFORE StepInitMoving ever emits AgentPrompt::Move ->
  0 actionRng. Java's ParityRunner draws the move target at **phase-2** (sendMoveAction, right after the
  ACTIVATE, BEFORE the Select ACTIVATION) -> JAVA_PICK (14,6) = 1 actionRng, THEN Bone-head fails. Net
  Java +1 actionRng. (Standing players skip the Select ACTIVATION via goto END_SELECTING and pick at
  StepInitMoving, which is BEFORE the Move-sequence Bone-head — so they stay aligned. The ONLY divergent
  case is a standing_up player whose Select-ACTIVATION negatrait FAILS.)
- FIX (AGENT — random_agent.rs, mirrors ParityRunner.sendMoveAction exactly; engine-threading of MoveStack
  was rejected because StepBloodLust inside the Select ACTIVATION also consumes MoveStack and would eat a
  published value before StepEndSelecting): when the agent activates a **prone** player for Move, pre-draw
  the move target at activation via legal_move_targets (coordinate-based -> pre-activation list == post-
  stand-up list, (x,y)-sorted, same pick), 1 actionRng, iff non-empty (Java deselects with 0 rng when no
  adjacent-empty). Stored in RandomAgent.pending_move; the AgentPrompt::Move handler reuses it with NO
  second draw. Restricted to prone players so standing moves are untouched.
- RESULT: human seed 1 GREEN (was step 39). Human seeds 1-3 GREEN; seed 4 now advances step 39 -> **step
  174** (next frontier: i=175 turn1 half2 away Activate(away_02,MOVE), post-hash mismatch). Lineman tier
  **100/100** (no regression), ffb-engine **7014/0** (added prone_move_predraw_is_reused_without_second_
  action_rng_draw). Committed. NEXT: drive human seed 4 step 174.

## FRONTIER (human) — seed 4 step 174: PASS skill-reroll not used (2026-08-02 ~23:58, DIAGNOSED, fix pending)
- Seeds 1-3 GREEN. Seed 4 fails at i=174: home_03 (h02, ball carrier @12,8, HAS the **Pass** skill) throws
  LONG to home_08 @(6,9). Both engines roll the SAME pass die (d6=4, DICE_TRACE pos=62) and it FUMBLES
  (2 tackle zones from away a00@13,7 + a02@13,8, + Long-pass mod → modified roll ≤1 → FUMBLE; same in both).
- DIVERGENCE = the **Pass skill re-roll**. JAVA: pos62 d6=4 (StepPass.start) → **pos63 d6=4 (StepPass.handleCommand
  = a re-roll USED)** → still fumble → ball bounces 3× (pos64/66/68 d8, StepCatchScatterThrowIn.bounceBall)
  with catch attempts (pos65/67 d6, catchBall) → ball ends (12,9). RUST: pos62 d6=4 → emits
  `ReRollOffer{source:"Pass", action:"PASS"}` → agent **NoReRoll (declines)** → fumble → 1 bounce (pos63 d8=8)
  → ball ends (11,7). So Java USES the Pass skill re-roll, Rust DECLINES → 5-die RNG desync + wrong ball.
- WHY Java uses it: the Pass re-roll is offered as a **SKILL_USE** (Java AbstractPassBehaviour registers a
  handleCommandHook on CLIENT_USE_SKILL: setReRolledAction(PASS), setReRollSource(isSkillUsed? PASS : null)),
  and ParityRunner's SKILL_USE case ALWAYS uses the skill ("matches Rust engine which auto-uses Sure
  Hands/Catch"). Rust instead routes the Pass re-roll through ask_for_reroll_if_available → ReRollOffer,
  which the agent's ReRollOffer handler always declines (correct for TEAM re-rolls, wrong for this free
  single-use SKILL re-roll).
- Rust infra ALREADY supports skill re-rolls: find_skill_reroll_source(game,"PASS") returns the Pass source;
  use_reroll(game, skill_source, pid) marks the skill used + returns true (non-TRR branch). But StepPass
  (bb2025/pass/step_pass.rs) HARDCODES `self.re_roll_source = Some("TRR")` in BOTH the FUMBLE (~L330) and
  INACCURATE|WILDLY (~L355) branches, and the agent declines the offer anyway.
- FIX PLAN (engine, most localized, matches Java always-use + Rust's Sure-Hands/Catch auto-use pattern): in
  StepPass, when find_skill_reroll_source(game,"PASS") is Some (a FREE single-use skill re-roll available and
  unused), AUTO-USE it — set re_rolled_action=PASS, re_roll_source=<skill source name>, call use_reroll (marks
  skill used), reset roll=0/pass_result=None to re-roll the pass die, WITHOUT emitting a ReRollOffer prompt.
  Only fall back to the team-reroll ReRollOffer (agent declines) when NO skill re-roll is available. Guard with
  already_rerolled so it re-rolls at most once. VERIFY: lineman 100/100 (linemen lack Pass so unaffected) AND
  human seeds 1-4 (seeds 1-3 must stay green; seed 4 must advance past step 174). Add a Rust test in step_pass.rs
  (fumble + thrower with Pass skill → re-roll die consumed + skill marked used, no prompt). REVERT if regressed.
- Alternative (riskier): agent uses skill-source ReRollOffers but declines TRR — rejected for now because it
  changes ALL skill re-rolls routed through ask_for_reroll_if_available, not just Pass.

## seed 4 step 174 — FIXED (2026-08-03 ~00:12, commit 2244d941)
StepPass now auto-uses a free single-use skill re-roll (find_skill_reroll_source(game,"PASS")) on FUMBLE/
INACCURATE/WILDLY before the team-reroll offer — re-entering execute_step so the top gate consumes the skill
token (use_reroll) and re-rolls the pass die once; already_rerolled blocks a 2nd offer. Only TEAM re-rolls
are still offered to the agent (declined). Fixed the hardcoded re_roll_source="TRR". Result: human seeds 1-6
GREEN; lineman 100/100; ffb-engine 7015/0 (+fumble_auto_uses_free_pass_skill_reroll_without_prompt).
NEXT FRONTIER: human seed 7 step 81 (i=82 turn5 half1) — ACTIVE-TEAM divergence: Java away Activate(away_04,
MOVE) vs Rust home Activate(home_03,Block); pre-step state_hash already differs (java bf89d517090d40b4 vs rust
36b5190e4fcd91ca), so the real divergence is earlier — trace seed 7, find the first differing post_hash/dice.

## FRONTIER (human) — seed 7 step 81: CATCH skill re-roll not auto-used (2026-08-03 ~00:28, DIAGNOSED)
- Seeds 1-6 GREEN. Seed 7 fails at i=81: away_03 (ball carrier @13,10) passes LONG to away_09 @(19,5). Chosen
  actions match through i=80; i=81's RESOLUTION diverges (post_hash differs → i=82 active team flips: Java stays
  away t45, Rust turns over to home). NOTE the Pass-reroll fix WORKS here: both engines now roll pos37 d6=1
  (pass fumble) + **pos38 d6=4 (Pass skill re-roll, auto-used)** → accurate → ball to (19,5) → pos39 d6=1
  (catch attempt, FAIL). They diverge at **pos40**: JAVA d6=5 = a **CATCH skill re-roll** (auto, via
  StepCatchScatterThrowIn.catchBall recursion / state.rerollCatch) → catch SUCCESS, away_09 holds the ball, no
  turnover. RUST d8=3 = a SCATTER (ball bounces, NO catch re-roll) → ball leaves → turnover to home.
- ROOT: Rust's catch auto-re-roll is gated by the CatchBehaviour hook (skill_behaviour/bb2025/catch_behaviour.rs:
  CatchStepModifier.handle_execute_step) which sets state.reroll_catch=true iff game.player(state.catcher_id)
  .has_skill(SkillId::Catch). For this catch the hook returned false (no auto-re-roll), so StepCatchScatterThrowIn
  .catch_ball fell through to ask_for_reroll_if_available("CATCH") (checks the ACTING player = thrower, not the
  catcher; hardcodes TRR) → declined → FailedCatch → scatter. Java's rerollCatch fired (catcher has a
  CATCH-reroll skill), so it auto-re-rolled.
- Both engines build the SAME 11-player team (runner.rs make_team_from_roster sorts positions by (quantity ASC,
  cost DESC) to match gen_java_teams.py), and the human roster's Catcher (qty 4) has <skill>Catch</skill>, so
  away_09 (a Catcher by fill order) SHOULD have Catch in BOTH. So the leading hypothesis is that the
  CatchBehaviour hook's `state.catcher_id` is WRONG/empty for this ACCURATE-pass catch — StepResolvePass has a
  KNOWN CatcherId-propagation gap (step_resolve_pass.rs L99-108: the CatcherId param is consumed by
  StepDispatchPassing before reaching StepResolvePass, so it falls back to player_at(pass_coordinate)); if that
  fallback catcher_id isn't threaded into StepCatchScatterThrowIn's hook state, has_skill(catcher) reads None →
  no Catch → no auto-re-roll. NEXT: (1) confirm away_09 has Catch in the Rust team (log its skills), (2) confirm
  what state.catcher_id the CatchBehaviour hook sees at this catch (instrument or FFB_TRACE the catcher_id +
  CatchScatterThrowInMode), (3) fix whichever is wrong — most likely thread the real catcher_id (the player at
  the caught square) into the hook state so the Catch skill is detected. Verify lineman 100/100 + human seeds
  1-7; add a Rust test; REVERT if regressed. Detail: run `--seeds 7-7` with FFB_TRACE+FFB_DICE_TRACE+FFB_DRIVE_TRACE.

## seed 7 step 81 — FIXED (2026-08-03 ~00:40, commit 0929b84b) + NEW frontier step 217
FIX: StepCatchScatterThrowIn.catch_ball now resets self.roll=0 when a catch re-roll is consumed
(already_rerolled + use_reroll true), so the Catch skill auto-re-roll draws a FRESH die instead of reusing
the failed roll. human seed 7 advanced step 81 -> 217; seeds 1-6 stay green; lineman 100/100; ffb-engine
7016/0 (+consumed_catch_reroll_rolls_a_fresh_die).

## FRONTIER (human) — seed 7 step 217: STANDING no-target BLITZ rolls an extra Bone-head (2026-08-03 ~01:00, DIAGNOSED)
- Real divergence is a 1-die RNG desync originating at i=196 (NOT i=217 where it first changes state). At
  i=196 the STANDING Ogre away_01 declares BLITZ with NO adjacent target. JAVA: ParityRunner picks the block
  target at SELECT_BLITZ_TARGET; blockTarget==null → "BLITZ_TARGET_NONE ... ending turn" → injects
  ClientCommandEndTurn → away turn ENDS, and NO Bone-head is rolled (the block sequence never runs). RUST:
  the agent picks Blitz with block_defender_id=None → StepInitSelecting force_goto dispatch → block/blitz
  sequence whose ACTIVATION rolls **Bone-head (pos=87 d6=4, EXTRA die)** → StepInitBlocking no-defender → ends
  the turn. Both END THE TURN (home_04 next in both), so the ONLY divergence is Rust's extra Bone-head die.
  That +1 die shift stays invisible until i=217, where the Ogre's next Bone-head lands on pos=89 d6=1 (FAIL)
  in Rust vs pos=88 d6=3 (PASS) in Java → the Ogre moves in Java but stays in Rust → post_hash diverges.
- KEY nuance (do NOT break the prone case): Java rolls Bone-head for a no-target blitz ONLY when the blitzer
  is PRONE (the Select-sequence ACTIVATION runs Bone-head during the free stand-up), NOT when STANDING
  (SelectBlitzTarget → EndTurn happens before any ACTIVATION). Rust currently rolls Bone-head for BOTH
  (block-sequence ACTIVATION), so it matches the PRONE case but over-rolls the STANDING case. The existing
  prone-blitz-no-target handling (step_init_selecting.rs L113-120 sets standing_up=false) erases the
  standing/prone flag, so the fix must check the player's ACTUAL PlayerState (is_prone) at dispatch.
- FIX DIRECTION: in StepInitSelecting's dispatch (bb2025/shared/step_init_selecting.rs ~L234-303), when
  dispatch==Blitz AND game.defender_id.is_none() AND the acting player is STANDING (base != PRONE), end the
  activation the way Java's standing no-target blitz does — END THE TURN before the block-sequence ACTIVATION
  (so NO Bone-head), mirroring ParityRunner's ClientCommandEndTurn. Keep the current path (block sequence →
  Bone-head → StepInitBlocking no-defender) for a PRONE no-target blitz (Java rolls Bone-head there). Verify
  lineman 100/100 + human seeds 1-7 (1-6 stay green, seed 7 past step 217); add a Rust test (standing player
  Blitz with no adjacent target → NO Bone-head die, turn ends). REVERT if regressed. Confirm via
  `--seeds 7-7` FFB_TRACE+FFB_DICE_TRACE: Java Bone-head at pos=88 vs Rust pos=89; the desync onset is i=196
  (rng diff 0→1); grep "BLITZ_TARGET_NONE".

## seed 16 step 11 — FIXED (2026-08-03 ~01:30, commit 34ec0ea1) + NEW frontier step 74
FIX: added reset_blocked_and_moving_players() (Java UtilActingPlayer.changeActingPlayer) — resets BLOCKED→
STANDING (always) and MOVING→STANDING (except acting player + thrower) whenever the acting player changes.
Called from change_player_action, StepEndPlayerAction, and StepEndBlocking's end branch (the last so the
restore lands within the block step's resolution → captured in the post-block state hash). Root cause: an Ogre's
BLOCK cancelled by a failed Bone-head left the defender BLOCKED (StepInitBlocking marks it before the
ACTIVATION); Java's changeActingPlayer reset restored it, Rust had none. human seeds 1-15 GREEN; seed 16
step 11 -> 74; lineman 100/100; ffb-engine 7018/0 (+reset_blocked_and_moving_players test).
NEXT FRONTIER: human seed 16 step 74 (i=74 = the Ogre home_01 does a PASS). Rust rolls 4 d8 scatters
(pos 55-58) while Java rolls 0 dice in the i=74 window → big desync surfacing at i=75. Likely the Ogre's pass
should deselect / not execute in Rust (matching Java's 0 dice — maybe no valid pass target, or bone-head/
negatrait path), OR the fumble-scatter count differs. NEXT: diff JAVA vs RUST dice+state at i=74; check the
Ogre's pass target/receiver selection and whether Java even executes the pass (0 dice suggests deselect/fumble
with no roll). Run `--seeds 16-16` FFB_TRACE+FFB_DICE_TRACE.

## FRONTIER (human) — seed 16 step 74: Ogre PASS hits ParityRunner UNHANDLED_STEP INIT_PASSING (2026-08-03 ~01:42, DIAGNOSED — HARNESS gap)
- i=74: the Ogre home_01 (ball carrier @12,6, has Bone-head + Throw Team-Mate, NOT the Pass skill) declares a
  PASS. Both engines choose Activate(home_01,PASS). JAVA: sendPassAction picks coord=(5,10) [teammate h06],
  logs JAVA_PASS, then the engine reaches step INIT_PASSING which ParityRunner has NO case for →
  "UNHANDLED_STEP: INIT_PASSING turnMode=REGULAR" → the default injects ClientCommandEndTurn → away turn ends
  with **0 dice** (rng stays 52). RUST executes the pass: pos53 d6=3 + pos54 d6=5 (bone-head + ?), then a
  fumble scatter pos55-58 (4× d8) = **6 dice**, then NoReRoll → turnover. Both end at away's turn but the
  6-die desync diverges the state at i=75.
- WHY the Ogre reaches INIT_PASSING but a normal pass does not: ParityRunner handles a pass at INIT_SELECTING
  phase-2 via sendConcreteAction→sendPassAction (sends the full CLIENT_PASS); the engine then runs the pass to
  completion. For the Ogre the engine instead pauses at a separate INIT_PASSING step (likely because the Big
  Guy / Bone-head path re-prompts), which ParityRunner (cases: INIT_SELECTING, INIT_MOVING,
  INIT_THROW_TEAM_MATE only) doesn't handle → its default EndTurn. ParityRunner.java is co-editable but a
  change needs a jar rebuild + re-verify lineman 100/100 (commit Rust first) — deferred (too risky near the
  2 AM stop).
- NEXT (needs the jar-rebuild path OR a Rust-only match): (1) determine WHY the Ogre's pass reaches
  INIT_PASSING as a waiting step while a normal pass doesn't (diff the Rust/Java step sequence for the pass —
  FFB_DRIVE_TRACE; is it bone-head, big-guy, or a stall?). (2) EITHER add an INIT_PASSING case to ParityRunner
  that sends the correct pass command (so Java executes the pass, matching Rust — then verify Rust's 6-dice
  resolution also matches), OR if the Ogre's pass genuinely should abort, make the RUST engine abort/EndTurn
  the Ogre's pass with 0 dice to match ParityRunner's EndTurn. Prefer understanding the INIT_PASSING re-prompt
  first. Ground truth = STOCK Java ENGINE; ParityRunner + random_agent co-editable.

## Loop resumed (post-push) — landscape + seed 16 refinement (2026-08-03)
- Survey: human seeds 1-15 GREEN, 16 FAILS (Ogre pass), 17-26 GREEN, next failure = seed 27 step 139
  (i=140 turn1 half2 — active-team divergence: Java away Activate(away_07,MOVE) vs Rust home
  Activate(home_11,Move); pre-step state_hash differs → real divergence earlier; likely a Rust-side
  turn/RNG issue, NOT the Ogre pass). So seed 16's Ogre-pass gap is ISOLATED (doesn't block 17-26).
- seed 16 refinement: confirmed the flow — StepInitSelecting DOES handle CLIENT_PASS (publishes
  TARGET_COORDINATE, changePlayerAction(PASS), dispatch=PASS) at INIT_SELECTING phase-2, and JAVA_PASS
  coord=(5,10) is logged. But StepInitPassing.executeStep only proceeds (NEXT_STEP) when
  passCoordinate!=null && thrower==actingPlayer && findPassingDistance!=null; for the Ogre one of those
  is unset by the time StepInitPassing runs (so it waits → ParityRunner's missing INIT_PASSING case →
  ClientCommandEndTurn → 0 dice). Pinning WHICH (passCoordinate vs thrower vs distance) and WHY (Big-Guy /
  Bone-head dispatch path) needs Java-side instrumentation = a jar rebuild. Fix = add an INIT_PASSING
  handler to ParityRunner (co-editable) so Java executes the pass, then verify Rust's 6-dice resolution
  matches; rebuild jar, re-verify lineman 100/100 (commit Rust first). DEFERRED behind the tractable
  Rust-side seeds (27, …) — a focused jar-rebuild effort.
- STRATEGY: drive the Rust-side frontiers (27 next) to green first; return to seed 16 (harness/jar) last.
  To keep 17-26 verified while skipping 16, run ranges that exclude 16 (e.g. --seeds 17-100).

## seed 27 step 139 — FIXED (2026-08-03, commit ca52794f) + next = seed 36
FIX: StepEndPassing now sets end_turn when the ball's FINAL catcher is on the thrower's opponent team
(Java findOtherTeam(thrower).hasPlayer(catcher) — the term was in the comment but not the code). Root: home_03
hands off to the Ogre, who fumbles the catch; the ball bounces to away_03 (opponent) → Java turnover, Rust
kept home's turn. human seeds 1-15 + 17-35 GREEN (16 deferred). lineman 100/100, ffb-engine 7019/0.
NEXT FRONTIER: human seed 36 step 249 (i=250 turn7/6 half2 — active-team divergence again: Java away t7 vs
Rust home t6; another turnover/turn-count divergence — diff per-step chosen + rng_calls to the first divergent
step). Run `--seeds 36-36`.

## seed 36 step 249 — FIXED (2026-08-03, commit d22e3cd5) + next = seed 85
FIX: dropped the standing-only guard on the no-target-Blitz EndTurn — a PRONE no-target Blitz also ends the
turn with 0 dice (Java's SELECT_BLITZ_TARGET→EndTurn precedes the stand-up ACTIVATION in both cases). Root:
seed 36 i=170 a PRONE away Ogre's no-target blitz rolled a stray Bone-head in Rust (block-sequence ACTIVATION),
+1 RNG desync surfacing at i=250. human seeds 36-84 GREEN. lineman 100/100, ffb-engine 7019/0.
NEXT FRONTIER: human seed 85 step 207 (i=208 turn3 half2, active=home both — Activate(home_01,MOVE); state_hash
already differs so root is earlier; diff chosen+rng_calls). Run `--seeds 85-85`.

## seed 85 step 207 — FIXED (2026-08-03, commit aaa844ba) + next = seed 98
FIX: make_injury_type now routes InjuryTypeDrop{GFI,Dodge,DodgeForSpp,Jump} to the proper injuryType::
injury_type_drop_* impls (do_injury_roll_for_player → Stunty + Thick Skull), not the stale player-less
InjuryTypeDropFall. Root: seed 85 i=207 an Ogre (Thick Skull) failed a dodge, fell, rolled injury 8 → KO in
Rust vs Java's Thick-Skull Stunned. human seeds 1-15 + 17-97 GREEN (96/100; seed 16 deferred). lineman 100/100,
ffb-engine 7020/0. NEXT: human seed 98 step 124 (i=125 turn7 half1 away — Activate(away_08,MOVE), state_hash
already differs → root earlier). Run `--seeds 98-98`.

## FRONTIER (human) — seed 98 step 124: pass re-roll used by Java, declined by Rust (2026-08-03, DIAGNOSED)
96/100 green (1-15, 17-97; seed 16 deferred). Seed 98 i=124: home_03 PASS to (6,9). JAVA: pos59 d6=2 (pass,
StepPass) → pos60 d6=5 (2nd StepPass roll = RE-ROLL used) → pos61 d6=2 + pos62 d6=1 (catch attempts,
StepCatchScatterThrowIn) → pos63 d8=1 (bounce); ball ends (6,8). RUST: pos59 d6=2 (pass fail) → emits
ReRollOffer{source:"TRR"} → agent NoReRoll (declines) → pos60 d8=5 (bounce); ball ends (12,10). So Java uses a
re-roll at the pass; Rust offers a TEAM re-roll (TRR) and declines. KEY QUESTION: does home_03 have the Pass
skill? If yes, Rust's find_skill_reroll_source(game,"PASS") should have returned Some (→ auto-use per commit
2244d941) but returned None (→ TRR offer) — so investigate why (Pass already used? not detected? wrong
player?). If home_03 lacks Pass, Java's pos60 must be a team/Pro re-roll that ParityRunner uses — reconcile the
agent's re-roll policy. NEXT: --seeds 98-98 with FFB_TRACE+FFB_DICE_TRACE; check home_03's skills + the
ReRollOffer source; get the FULL caller of Java pos60 (StepPass.start vs handleCommand → skill vs team).

## seed 98 step 124 — FIXED (2026-08-03, commit pending) — 99/100 human seeds GREEN
ROOT: home_03 (Thrower, Pass skill) passes twice — i=92 turn 5 and i=124 turn 7. At i=92 both engines
auto-use the Pass re-roll (dice match). Rust's use_reroll marks SkillId::Pass in the PERSISTENT
Player.used_skills; the live activation path (change_player_action → set_player clears only
ActingPlayer.used_skills, the Bone-head field) NEVER cleared Player.used_skills. So at i=124
find_skill_reroll_source(game,"PASS") saw Pass still "used" → returned None → the pass fell to a declined
TRR ReRollOffer (Rust 2 dice: pos59 d6=2 fail → decline → pos60 d8=5 bounce, ball→(12,10)) instead of
Java's auto-used Pass re-roll (5 dice: pos59 d6=2 → pos60 d6=5 reroll via StepPass.handleCommand → pos61-62
catch → pos63 d8 bounce, ball→(6,8)). The legacy step/engine.rs:2214 StepInitSelecting did
used_skills.clear() at activation, but that's dead code — driver.rs is live.
FIX (crates/ffb-engine/src/step/util_server_steps.rs change_player_action): after set_player, reset the
activated player's SkillUsageType::Regular skills via reset_used_skills(Regular) — precisely the set NOT
tracked outside the player's own activation (track_outside_activation==false); OncePer{Turn,Half,Drive,Game}
reset at their own boundaries and stay intact. So a Regular skill re-roll (Pass, Sure Hands, Catch, …) is
available again each activation, matching Java. +regression test change_player_action_resets_regular_skill_
reroll_but_keeps_once_per_game. lineman 100/100, human 17-100 84/84, ffb-engine 7021/0.
STATUS: human seeds 1-15 + 17-100 GREEN (99/100). ONLY seed 16 remains (DEFERRED — Ogre PASS reaches
ParityRunner INIT_PASSING with no handler; needs a ParityRunner INIT_PASSING case + jar rebuild, see "seed 16
step 74" above).

## seed 16 step 74 — INIT_PASSING harness gap: attempted ParityRunner fix FAILED, REVERTED (2026-08-03)
STATUS: human parity is 99/100 GREEN (seeds 1-15 + 17-100). Seed 16 is the sole holdout and is a HARNESS
limitation, NOT a Rust engine bug — the Rust engine correctly EXECUTES the Ogre's pass exactly as the stock
engine would (the stock engine reaches INIT_PASSING and waits for the pass command).

RECAP: i=74 home_01 (Ogre: Bone-head + Throw Team-Mate, Thick Skull, Mighty Blow, Loner; NO Pass skill) is the
ball carrier and declares PASS to (5,10). RUST executes: bone-head + pass roll + d8 fumble-scatters (6 dice,
rng 52→58), ball→(6,9), turnover. JAVA (stock engine + stock ParityRunner): sendPassAction picks (5,10) [1
actionRng, aligned with Rust's random_agent pick_action] and injects ClientCommandPass at INIT_SELECTING
phase-2, BUT the engine then WAITS at INIT_PASSING (ParityRunner has no case) → default injects EndTurn →
pass NEVER executes (0 dice, ball stays (12,6)). The two pre-existing injuries (a02,h02) predate the pass and
match; the ONLY divergence is the ball (Java 12,6 vs Rust 6,9).

WHY the Ogre differs from a normal pass (home_03 seeds 4/27/98 all GREEN): a normal pass's phase-2
ClientCommandPass is consumed AT INIT_PASSING within the same MatchRunner cycle, so ParityRunner never
observes INIT_PASSING as a waiting step. The Ogre's pass goes through an extra move-phase auto-advance (the
drive trace shows InitActivation→…→BoneHead→…→Move→…→InitMoving→EndMoving before the pass sequence) that
consumes/discards the phase-2 ClientCommandPass, so INIT_PASSING is then reached as a genuine WAITING step.

ATTEMPTED FIX (REVERTED): added a ParityRunner `case INIT_PASSING` that re-sends the stored pass target
(lastPassPlayerId/lastPassCoord saved in sendPassAction, NO new actionRng). Rebuilt the jar
(/c/Users/Admin/bin/maven/bin/mvn.cmd -q -pl ffb-ai -am install -DskipTests -o). RESULT: INFINITE LOOP —
1,999,762 resends of ClientCommandPass(home_01,(5,10)); the step never advances (ball stays (12,6), rng stays
52). Diagnostics confirmed actingPlayer=home_01, action=PASS, homePlaying=true, turnMode=REGULAR at
INIT_PASSING — so StepInitPassing's CLIENT_PASS guards (checkCommandIsFromCurrentPlayer +
checkCommandWithActingPlayer) LOOK satisfied, yet the injected command does not drive executeStep. So a
ParityRunner-OBSERVED INIT_PASSING step cannot be advanced by re-injecting ClientCommandPass the way a
phase-2 injection is — the inject/command-delivery model differs between the two contexts. Reverted
ParityRunner.java (git checkout) + rebuilt the CLEAN jar; verified seed 16 fails at step 74 as before, seeds
14-15 + 17-19 GREEN, lineman 1-6 GREEN — baseline 99/100 fully intact.

NEXT (for a future focused effort): the re-inject-at-INIT_PASSING approach is a dead end. Instead investigate
(a) WHY the Ogre's phase-2 ClientCommandPass is consumed before INIT_PASSING (instrument the stock engine's
step transitions between phase-2 and INIT_PASSING for the Ogre vs home_03 — likely StepInitMoving/EndMoving in
the PASS_MOVE sub-sequence swallows it), then EITHER prevent that early consumption so the phase-2 command
survives to INIT_PASSING (as it does for a normal pass), OR determine the correct command/sequence to drive a
ParityRunner-observed INIT_PASSING (why does executeStep not fire despite valid guards? — check
super.handleCommand's return and MatchRunner.inject delivery timing within the ParityRunner while-loop). Keep
Rust untouched (it is already correct); this is purely ParityRunner + jar-rebuild work.

## seed 16 step 74 — FIXED (2026-08-03, commit e1f28183) → HUMAN TIER 100/100 COMPLETE
ROOT (a RUST ENGINE bug, NOT the INIT_PASSING harness gap previously suspected): StepInitPassing's
passing-distance gate (crates/ffb-engine/src/step/mixed/pass/step_init_passing.rs) used
ffb_model::util::passing::passing_distance — a pure [dy][dx] table lookup with NO weather gate. Java's
PassMechanic.findPassingDistance nulls out a Long Pass / Long Bomb in a BLIZZARD (only Quick/Short allowed),
so the stock engine refuses the throw → turn ends, ball unmoved, 0 dice. Rust treated it as a valid LongPass
and executed → auto-fumble + scatter (6 dice), ball moved. seed 16 i=74: Ogre home_01 at (12,6) throws to
(5,10) [dx=7,dy=4 = LongPass] in a Blizzard.
FIX: weather-gate the check — `Some(d) => !(weather==Blizzard && matches!(d, LongPass|LongBomb))`. Rust's
existing out-of-range branch then ends the turn (matching Java). +2 regression tests (blizzard long-pass
out-of-range; short-pass still-in-range). NO jar rebuild / NO ParityRunner change needed.
INVESTIGATION NOTE: the earlier "ParityRunner INIT_PASSING handler" approach was a DEAD END (infinite loop —
StepInitPassing.executeStep bails when getThrower()==null and the re-sent CLIENT_PASS couldn't advance a
ParityRunner-observed INIT_PASSING; and even when the thrower was set, findPassingDistance==null blocked
NEXT_STEP). Gated JCMD/JIP traces in GameState.java + mixed/StepInitPassing.java pinned it: JIP showed
`dist=null` for a table-L entry with throwTeamMate=false → could only be the Blizzard gate. All stock-repo
experiments (ParityRunner + the 2 gated-trace files) were REVERTED; the fix is Rust-only.
**RESULT: Human vs Human parity 100/100 GREEN (bb2025 tier-3). lineman 100/100, ffb-engine 7023/0.**

## ROSTER PROGRESSION (2026-08-03): amazon ✅ 100/100, chaos ▶ FRONTIER seed 1 step 100 (DIAGNOSED, unfixed)
User directed: after human 100/100, drive per-roster mirror parity ALPHABETICALLY (amazon first); OGRE
DEFERRED. amazon vs amazon = 100/100 (no fix). CHAOS vs chaos: seed 1 diverges at step 100.
ROOT (block DICE COUNT, not the MB red herring): i=100 away_01 (away Minotaur: Horns, Mighty Blow, Thick
Skull) BLITZes home_01 (home Minotaur). Agent picks Both Down → both fall. **Java rolls 2 block dice
(JAVA_BLOCKROLL nDice=2); Rust rolls 1.** That 1-die shift reassigns every downstream die, so the both-down
attacker away_01 ends Prone in Java vs Stunned in Rust (looked like a Mighty-Blow-on-both-down armour bug —
that was DOWNSTREAM NOISE from the shift; confirmed via instrumented do_armor_roll/recalc/injury + Java
JAV_ARMOURROLL, all since reverted).
WHY 2 vs 1: Horns = +1 ST on a Blitz → att 6 > def 5 → 2 dice (Java). Rust's blitz nr_of_dice = 1 (att 5 =
def 5) — Horns's +1 ST NOT reflected. In Rust, Horns is a StepHorns hook (+1 ST) that runs DURING the block
sequence (drive order: InitBlocking → BlockStatistics → **Horns** → BlockRoll), but the block dice count was
already fixed before it. Instrumentation proved: `find_nr_of_block_dice` is called only 3× the whole game
(NONE at step 100) and step_block_roll's `block_dice_count` path (`if nr_of_dice==0`) was skipped (nr_of_dice
already non-zero) — so the blitz count comes from a PRE-SET value (a DiceDecoration from
ServerUtilBlock::update_dice_decorations, which uses base `strength_with_modifiers()` with NO Horns, OR from
StepSelectBlitzTarget/StepInitBlocking).
NEXT: find exactly where the blitz's nr_of_dice is set (grep StepSelectBlitzTarget / StepInitBlocking /
DiceDecoration reads in the bb2025 block path) and ensure it's (re)computed AFTER StepHorns applies +1 ST — or
include Horns's blitz +1 in the strength used. Check WHY Beastman (also Horns) blitzes matched in steps 1-99
(likely their ST/assist gap gave the right count independent of the exact +1, so only an exactly-equal-ST
blitz like Minotaur-vs-Minotaur exposes it). Verify att/def strengths at the set point. Rust-engine fix only;
NO jar change (Java is correct). Baseline at pause: chaos seed 1 FAIL step 100, lineman/amazon/human all green.

### chaos seed 1 i=100 — FIXED (2026-08-03, commit 25c5292c): Horns +1 ST feeds block-dice count
Rust StepHorns only set a display flag (using_horns) + emitted SkillUse; the +1 ST was never applied to the
block-dice count (apply_add_block_die had no production caller). A Horns blitz got the wrong die count when the
+1 crossed a boundary: Minotaur(ST5,Horns) blitz Minotaur(ST5) → Java 2 dice (6>5), Rust 1 (5==5). The 1-die
shift desynced the whole stream (surfaced downstream as a fake Mighty-Blow-on-both-down armour diff — RED
HERRING). FIX: in StepBlockRoll block-dice calc, mirror Java RollMechanic.getAttackerBaseStrength — +1 to the
attacker BASE strength (pre-assists) when acting player has addStrengthOnBlitz and action is Blitz/BlitzMove;
-1 when the blitzer moved and the defender has weakenOpposingBlitzer. +regression test. **chaos seeds 1-39
GREEN**; lineman/human/amazon 100/100, ffb-engine 7024/0. NEXT: chaos seed 40 i=142.

### chaos seed 40 i=142 — FIXED (2026-08-03, commit abe8636c): Dodgy Snack roll order
Kickoff event "Dodgy Snack" (bb2025): Java handleDodgySnack picks BOTH random players (home then away)
BEFORE rolling either player's snack d6 (the snack rolls happen in the trailing insertSteps calls). Rust
interleaved (pick-home, snack-home, pick-away, snack-away), so the home snack roll consumed the away
random-player's die slot → a player whose snack roll should be 1 was NOT benched to RESERVE → half-2 setup
diverged (home_05 stayed on the pitch in Rust, benched in Java). FIX: pick player_home + player_away first,
then roll+apply each snack (roll 1 → RESERVE+box; else -MA/-AV). +regression test. **chaos seeds 1-50 GREEN**;
lineman/human/amazon 100/100, ffb-engine 7025/0. NEXT: chaos seed 51 i=166 (again a half-2 state-only diverge).

### chaos seed 51 i=166 — FIXED (2026-08-03, commit 7a18ceff): chain pushback → CHAOS 100/100 COMPLETE
Chain pushback (a player shoved into an OCCUPIED square → the occupant is chain-pushed) was unimplemented:
Rust's Action::PushTo carries only a coord and always attributed the push to game.defender_id, so the chain
PushTo re-pushed the ORIGINAL defender instead of the occupant (Java's Pushback carries the player id +
updates state.defender to the occupant). seed 51 i=166: home_01 blitzes away_01 into home_04's square → Java
away_01@(12,8), home_04→(11,9); Rust swapped them (away_01@(11,9), home_04@(12,8)), which later diverged a
dodge. FIX: track chain_pushed_player (occupant of a chosen-but-occupied square) and push it on the follow-up
PushTo; drain the stack LIFO (Java pops last-first). +regression test. **CHAOS vs CHAOS 100/100 GREEN.**
Gates: lineman 100/100, human 100/100, amazon 100/100, ffb-engine 7026/0.

## ✅ CHAOS TIER COMPLETE (2026-08-03) — 100/100. Three fixes: Horns block-dice (25c5292c), Dodgy Snack roll
order (abe8636c), chain pushback (7a18ceff). User said "stop after fixing chaos" → loop stops here.

## underworld seed 1 step 41 — Animal Savagery lash-out must end the foul (option factory default)

**Symptom:** underworld (Underworld Denizens) seed 1 diverged at step 41. Away Animal
Savagery player declared FOUL on a non-adjacent Home player; savagery roll (pos53) failed →
lashed out at an adjacent teammate (InjuryTypeBlock armour[6,3]+injury[5,2], pos54-57). Java
ended the activation there (5 dice, rng 52→57, foul aborted). Rust rolled the foul too
(StepFoul armour pos58-59), arriving at the next activation 2 rng ahead → every later step desynced.

**Root cause:** `UtilGameOption::is_option_enabled` consulted only the explicitly-stored option
value; Java's `isOptionEnabled` uses `getOptionWithDefault(id).isEnabled()`, which materializes
the `GameOptionFactory` default. `animalSavageryLashOutEndsActivation` has factory default `true`.
With it wrongly `false` in Rust, the lash-out branch (StepAnimalSavagery, line ~368) that
publishes END_PLAYER_ACTION + USE_ALTERNATE_LABEL — making the activation's GotoLabel jump to the
failure label END_FOULING and skip FoulChainsaw/StepFoul — was never taken.

**Fix (`crates/ffb-model/src/option/util_game_option.rs`):** `is_option_enabled` now resolves the
id via `GameOptionId::for_name`, builds the option through `get_option_with_default`, and reads its
value — 1:1 with Java. Side effect: other unset true-default booleans now read correctly
(clawDoesNotStack, inducements, pettyCash, forceTreasuryToPettyCash, divingTackleLeavingTzOnly,
inducementPrayersAvailableForUnderdog, pilingOnUsesATeamReroll, …). Six unit-test files that
assumed unset==false now set the option explicitly to keep their intended scenario.

**Verified:** underworld seed 1 GREEN (advances to seed 2 step 1, a pre-existing divergence).
No regression: lineman/human/amazon 1-100 = 100/100; ffb-engine 7026/0, ffb-model 2773/0.
Commit e56fb06b.

## renegades seed 1 step 93 — JSON string skill values must be stored unquoted (Animosity)

**Symptom:** renegades (Chaos Renegades) seed 1 diverged at step 93. Renegade Away8 declared a
PASS to the teammate at (14,6); both engines picked the same target square, but Java refused the
pass via an **Animosity** roll (pos84, no throw, no turnover) while Rust skipped Animosity, threw
(accuracy pos84 → d8 scatter pos85) and turned the ball over → active team flipped a step early.

**Root cause:** `com.fumbbl...SkillMechanic.animosityExists` builds a pattern set of the catcher's
position keywords + the Animosity "allValue" (`all`) and tests whether any of the thrower's
lowercased Animosity values is in it. Renegade Animosity is configured `value="All"`. The Rust
roster loader (`skill_entry_to_skill_with_value`) stored the value via `serde_json::Value::to_string()`,
which re-serializes a JSON string WITH quotes (`All` → `"All"`). The quoted `"all"` never matched
the unquoted `all`, so `animosity_exists` returned false, `do_roll` was false, and Animosity never
fired for ANY Renegade pass.

**Fix (`crates/ffb-model/src/data/loader.rs`):** unwrap `serde_json::Value::String(s)` to its inner
text; numeric/other values keep `to_string()`. +2 loader regression tests.

**Verified:** renegades seed 1 GREEN (advances to seed 2 step 1). No regression:
lineman/human/amazon 1-100 = 100/100; ffb-engine 7026/0, ffb-model 2775/0. Commit 403b8482.

## dark_elf seed 1 step 36 — decline all PlayerChoice modes (Shadowing), no rng draw

**Symptom:** dark_elf seed 1 diverged at step 36. A Dark Elf (Away1) dodged out of a Shadowing
opponent's (Home3) tackle zone; Rust selected the shadower via the SHADOWING PlayerChoice, offered a
reroll, and rolled the bb2025 Shadowing skill die (pos37) — an extra game die. Java rolled nothing
(rng 34→36 vs Rust 34→37), so the shared RNG desynced for the rest of the game.

**Root cause:** the random agent's `AgentPrompt::PlayerChoice` handler picked a player (sorted +
`pick()`, consuming a decision_rng draw) for EVERY mode. Java's ParityRunner PLAYER_CHOICE handler
declines every `PlayerChoiceMode` dialog with an empty `Player[0]` selection and draws no rng — the
only exception is MVP, which reaches the Rust agent through a separate SelectPlayer prompt, not this
one. So selecting a player made the Rust engine roll Shadowing (and, for other rosters,
Tentacles/Diving Tackle/Animal Savagery/Pile Driver/Wisdom) skill dice Java never rolls.

**Fix (`crates/ffb-engine/src/agent/random_agent.rs`):** the handler returns
`SelectPlayer { player_id: "" }` (decline) with no `pick()`. Every consuming step already treats an
empty selection as "don't use the skill", matching ParityRunner exactly.

**Verified:** dark_elf seed 1 → seed 55. No regression — all 9 green rosters still 100/100;
underworld/renegades unchanged at seed 2; ffb-engine 7026/0, ffb-model 2775/0. Commit 58abe2b4.

## dwarf seed 1 step 101 (part 1) — apply successful Dauntless to block-dice strength

**Symptom:** dwarf seed 1 diverged at step 101. A Dwarf blitzer (Away3) with Dauntless blocked a
stronger target; Rust rolled 3 block dice (defender-choice, `nr_of_dice: -3`) where Java rolled 1
(`nDice=1`). Both rolled the Dauntless d6 (pos45), but Rust ignored its success → +2 extra dice →
game-die stream desynced.

**Root cause:** bb2025 StepBlockRoll set `successful_dauntless` (from StepDauntless) but never used it
in the strength calc. Java RollMechanic.getTotalAttackerStrength does
`blockStrengthAttacker = max(base, doubleTargetStrength ? 2*defenderStrength : defenderStrength)` on a
successful Dauntless — applied to the base BEFORE the Horns +1 and before assists.

**Fix (`crates/ffb-engine/src/step/bb2025/block/step_block_roll.rs`):** insert the Dauntless max using
the defender's BASE strength, in Java's order (base → Dauntless → Horns → assists). +1 regression test.

**Verified:** i=101 blitz RNG now aligns (1 die, armour [1,2]). Advances **norse seed 2 → seed 74**
(Dauntless blitz there too). No regression: 9 green rosters 100/100; ffb-engine 7027/0. Commit 2c0621cf.
Residual: dwarf step 101 still has a state-only pushback/chain-push divergence (part 2, next).

## dwarf seed 1 step 101 (part 2) — Stand Firm auto-use + suppress follow-up

**Symptom:** after the Dauntless fix, dwarf seed 1 still diverged at step 101 (state-only). A Dwarf
Deathroller (Stand Firm) was blitzed; Java kept it in place (push avoided), Rust pushed it to (10,7)
and the blitzer followed up. Java's StepPushback hook returned stopProcessing=true (Stand Firm used).

**Root cause (two bugs in bb2025 StandFirmBehaviour):** (1) with no skill-use dialog answer, the hook
auto-DECLINED Stand Firm → defender pushed. Java shows a DialogSkillUseParameter and ParityRunner's
SKILL_USE handler ALWAYS uses the skill → headless must auto-USE. (2) Java also publishes
FOLLOWUP_CHOICE=false when the push is avoided; the Rust hook had no way to publish step params, so the
blitzer still followed up into the (still-occupied) defender square.

**Fix:** default undecided Stand Firm to USE (cancel push); add a `published: Vec<StepParameter>`
channel to StepPushbackHookState that StepPushback drains into its output, and emit FOLLOWUP_CHOICE(false)
from the Stand Firm hook. Updated the unit test.

**Verified:** dwarf seed 1 step 101 → step 166. No regression: 9 green rosters 100/100; ffb-engine
7027/0, ffb-model 2775/0. Commit 4ee41b16.
Residual: dwarf step 166 — half-1→half-2 transition, away player 0 in Reserve (Java) vs on-field (Rust);
suspected missing Secret Weapon end-of-drive send-off (Deathroller has Secret Weapon). Next.

## dwarf seed 1 step 166 — Secret Weapon send-off + argue-the-call (end of drive)

**Symptom:** dwarf seed 1 diverged at the half-1→half-2 transition. Rust STUBBED the Secret Weapon
send-off, so a played Deathroller (Secret Weapon) stayed on the pitch into half 2 where Java sent it off.
Java also rolls TWO `rollArgueTheCall` d6 at the transition (pos53 away argue=fail→away Deathroller banned,
pos54 home argue=6 success→home Deathroller kept) BEFORE the half-2 kickoff dice; Rust rolled neither, so
its kickoff RNG was shifted → half-2 state diverged. Found by instrumenting Java PlayerResult
.setHasUsedSecretWeapon (home1's flag was cleared by argueTheCall, not a ban).

**Fix (`crates/ffb-engine/src/step/bb2025/step_end_turn.rs`):** new `resolve_secret_weapons`, run once at
end of drive (new_half||touchdown, not end_game) before KO recovery/kickoff — report (Stunty 2d6 / penalty-0
auto-ban) → argue-the-call AWAY then HOME (ParityRunner always argues first-eligible, looping; 6=keep,
1=coach banned, else banned; +1 friendsWithTheRef) → remove still-flagged (PS_BANNED + off-pitch,
SendToBoxReason::SecretWeaponBan → state "-1,-1,Reserve"). The 2 argue dice enter the stream away→home
before the kickoff dice, matching Java.

**Verified:** dwarf seed 1 step 166 → **seed 4 step 294** (seeds 1-3 GREEN). No regression: 9 green rosters
100/100; ffb-engine 7028/0, ffb-model 2775/0 (+1 test). Commit 62005506.

## dwarf seed 4 step 294 — no Secret Weapon send-off at end of game

**Symptom:** seed 4 diverged at step 294 (final move, turn 8 half 2): all 294 step lines matched but the
game_end state_hash differed (Java dd91d433 vs Rust c5093c7e = each engine's i=294 post_hash). Rust ran the
Secret Weapon send-off at the END OF HALF 2 (game end), banning a played Deathroller; Java gates argue/remove
by `!fEndGame` and keeps it.

**Fix (`step_end_turn.rs`):** `self.end_game` is never set, so the `!self.end_game` resolve gate never fired.
Compute the end-of-game condition at the gate (game.half not yet incremented): `(new_half && game.half > 1)
|| (touchdown && both teams turn_nr>=8)`. Run `resolve_secret_weapons` only when it's a drive end that is NOT
the game end.

**Verified:** dwarf seed 4 → seed 8 (seeds 4-7 GREEN). No regression: 9 green rosters 100/100; ffb-engine
7028/0. Commit ea5dfcf4.

## dwarf seed 8 step 5 — bb2025 has no NoHands skill (drop it at roster load)

**Symptom:** a hand-off to the Deathroller (No Hands) — Rust scattered the ball (NoHands→preventCatch→
catch_ball SCATTER_BALL) → turnover; Java caught it (no turnover). Proven: `JAVA_CATCHBALL catcher=Home1
preventCatch=FALSE` in bb2025.

**Root cause:** bb2025's SkillFactory has NO NoHands class (only bb2016/bb2020 define NoHands with
preventCatch + preventHoldBall + preventRegular{Pass,HandOver}Action). So bb2025 "No Hands" resolves to
null and is not applied. Rust's SkillId::NoHands is edition-agnostic → a bb2025 No-Hands player wrongly
gained preventCatch. (bb2025's ball-denial skill is NoBall, kept.)

**Fix (`loader.rs` + `runner.rs`):** `position_json_to_roster_position` takes `is_bb2025` and drops the
NoHands skill for bb2025 rosters (matching Java's unresolved skill); threaded through find_roster/
roster_json_to_roster and the parity runner. +1 regression test.

**Verified:** dwarf seed 8 → seed 60 (seeds 8-59 GREEN). No regression: 9 green rosters 100/100 (incl.
chaos_dwarf, also a Deathroller team); ffb-engine 7028/0, ffb-model 2776/0. Commit d941b8f4.

## dwarf seed 60 step 36 — Dirty Player armour-OR-injury mutual exclusion (foul)

**Symptom (state-only, dice matched):** Home1 (Deathroller: Dirty Player) fouls Away2 (dwarf, Thick
Skull). Armour [4,3]=7, injury [6,2]=8. Java: `a01` Prone (Stunned); Rust: `a01` Ko.

**Root cause:** Rust applied Dirty Player's +1 to BOTH the armour AND the injury roll. Java's
`InjuryTypeFoul.armourRoll` rolls with the foul-assist modifiers and checks `isArmourBroken` FIRST,
then `if (!isArmorBroken())` applies the general skill-based armour modifiers (Dirty Player) and
re-checks. Because Dirty Player registers `affectsEitherArmourOrInjuryOnFoul`, its injury modifier's
`appliesToContext` excludes itself when an armour modifier with that property was used. Net: Dirty
Player's +1 goes to ARMOUR when the base roll didn't break AV (else it stays free for INJURY). Here
base 7 < AV8 → DP breaks armour (7+1=8), injury stays 8 → Thick Skull → Stunned. Rust's extra injury
+1 made 9, bypassing Thick Skull → KO.

**False start:** first cut made Dirty Player unconditionally armour-only (never injury). That
REGRESSED seed 8, where the base armour roll already breaks AV so Java DOES spend Dirty Player on the
injury roll. Reverted; implemented the conditional instead.

**Fix (`injury_type_foul.rs`):** restructured `armour_roll` to Java's order — roll + foul assists
first, and add `ARMOR_DIRTY_PLAYER_1` (recomputing `armor_broken`) only `if (!armor_broken)`. In
`injury_roll`, skip the "Dirty Player" injury modifier when `ctx.armor_modifiers` already contains the
Dirty Player armour +1 (mutual exclusion). +3 regression tests.

**Verified:** dwarf 100/100 GREEN (seed 8 and seed 60 both pass). No regression: 9 green rosters
100/100; ffb-engine 7030/0, ffb-model 2776/0. Commit pending.

## elf seed 38 step 265 — Dodgy Snack enhancement not cleared at end of drive

**Symptom (state-only after a block, RNG desync):** home_01 blocks away_03 at i=265. Both roll the same
block dice [3,2,1] and armour [5,1]=6, but Rust consumes 8 dice vs Java's 4 (rng 73→81 vs 73→77): Rust
BREAKS armour and rolls an injury+casualty (d16) → away_03 Injured (off-pitch); Java's armour HOLDS →
away_03 Prone. `FFB_ARMOUR_TRACE` showed away_03 base_av=7 but eff_av=6 (a "Dodgy Snack" -1 AV temp
stat-mod), so armour 6 ≥ 6 breaks in Rust while Java uses AV 7 (6 < 7 holds).

**Root cause:** Dodgy Snack (kickoff event) gives a random player -1 MA / -1 AV **for the drive**. Java
`StepEndTurn` clears it at end of drive: `if (fNewHalf || fTouchdown) { … players with active
DODGY_SNACK enhancement → removeSkillEnhancements(DODGY_SNACK); }` (alongside the UNTIL_END_OF_DRIVE
effect/prayer/reroll cleanup). Rust's bb2025 StepEndTurn never removed the "Dodgy Snack" temporary
stat-mods, so a player snacked in an earlier drive kept -1 AV for the rest of the game. away_03 was
snacked in half 1; Java restored AV 7 at the half transition, Rust kept AV 6 into half 2 turn 6.

**Fix (`step_end_turn.rs`):** in the `if self.new_half || touchdown` drive-end block (next to the
UntilEndOfDrive/UntilEndOfHalf card deactivation) remove `"Dodgy Snack"` temp stat-mods from every
player: `for p in home.iter_mut().chain(away.iter_mut()) { p.remove_temporary_stat_mods("Dodgy Snack") }`.
+1 regression test.

**Verified:** elf 100/100 GREEN. No regression: 10 green rosters (incl. dwarf) 100/100; ffb-engine
7031/0, ffb-model 2776/0. Commit pending. **11 green total.**

## high_elf seed 14 step 138 — Safe Throw is bb2016-only (inert in bb2025)

**Symptom (RNG desync on a PASS):** home_03 (High Elf Thrower: Pass + Safe Throw) passes. Both roll
pickup, pass=2 (modified FUMBLE), then a Pass-skill re-roll = natural 1. Java FUMBLES the re-rolled 1
→ ball bounces (d8 scatter at thrower) → TURNOVER. Rust evaluates the re-rolled 1 as SAVED_FUMBLE (ball
stays with the thrower) → home keeps its turn. Traced with FFB_PASS_TRACE: Rust `result=SAVED_FUMBLE`
because the thrower had the `dontDropFumbles` property.

**Root cause:** `SafeThrow` has only a bb2016 skill class (`@RulesCollection(Rules.BB2016)`,
canCancelInterceptions + dontDropFumbles). bb2020/bb2025's equivalent is the mixed `SafePass`
(`@RulesCollection` BB2020+BB2025, dontDropFumbles only). So a bb2025 roster's "Safe Throw" resolves to
null in Java's SkillFactory and grants NOTHING. Rust's `SkillId::SafeThrow` is edition-agnostic and
always returns `[canCancelInterceptions, dontDropFumbles]`, so it wrongly saved the fumble. Same class
as the dwarf NoHands bug.

**Fix (`loader.rs`):** extend the existing bb2025 bb2016-only-skill drop (was NoHands) to also drop
`SafeThrow`: `filter(|s| s.skill_id != NoHands && s.skill_id != SafeThrow)`. +1 regression test.

**Verified:** high_elf 100/100 GREEN. No regression: 12 green rosters (lineman amazon chaos chaos_dwarf
chaos_pact dwarf elf human lizardman nippon orc + high_elf) 100/100; ffb-engine 7031/0, ffb-model
2777/0. **12 green total.** Commit pending.

## khemri seed 40 step 185 — pass caught by an opponent must be a turnover (StepEndPassing path 7)

**Symptom (state-only, dice match):** home_04 (Khemri Thro-Ra) passes; the accurate pass bounces
(pos75 d8) and is caught by an OPPONENT (away_02) at (13,6). Java turns the ball over (i=186 active→away,
FOUL); Rust lets home keep its turn. Same 3 dice (pass=6, scatter, catch=5), identical ball landing and
player positions — only the active team / turn count diverge (an invisible turnover flag).

**Root cause:** Java `StepEndPassing` has an `else` branch (when `fEndTurn || fEndPlayerAction ||
bloodlust` is false and the thrower IS the acting player) that RECOMPUTES the catcher as the player under
the ball (`field.getPlayer(field.getBallCoordinate())`) and sets `fEndTurn |= checkTouchdown ||
(catcher == null) || findOtherTeam(thrower).hasPlayer(catcher) || (fPassFumble && !dontDropFumble)`. Rust's
path 7 computed `end_turn |= check_touchdown || catcher_id.is_none() || (pass_fumble && !dont_drop)` — it
read the stored `catcher_id` (set to the opponent away_02, so `is_none()` was false) and OMITTED the
`otherTeam.hasPlayer(catcher)` term, so an opponent-caught pass never turned over.

**Fix (`step_end_passing.rs`):** in path 7, recompute the catcher as the player under the ball
(`field_model.player_at(ball_coordinate)`) and add the opponent-catcher term:
`end_turn |= check_touchdown || ball_catcher.is_none() || ball_catcher_is_opponent || (pass_fumble &&
!dont_drop)`. Updated the `quick_pass_accurate_allows_move_continuation` unit test to place the catcher
(a teammate) under the ball (the test previously only set `catcher_id`).

**Verified:** khemri seed 40 → seed 99 (seeds 40-98 GREEN). No regression: 12 green rosters 100/100;
ffb-engine 7031/0, ffb-model 2777/0. Commit pending. Next khemri frontier: seed 99 step 157 (home_08
MOVE, unrelated).

## khemri seed 99 step 157 — Decay's second casualty roll is BB2016-only → khemri 100/100

**Symptom (latent RNG desync surfacing at the half break):** at i=128 home_02 BLITZes and casualties a
Khemri Tomb Guardian (Decay + Regeneration). Rust consumed 11 dice for the blitz (rng 39→50), Java 9
(39→48) — Rust rolled 2 EXTRA. The desync then "parked" through a run of dice-less MOVEs (rng constant
50/48) until the half-1→half-2 kickoff, whose dice landed 2 positions off → state diverged at i=157/158.
Dice trace: after the casualty (d16+d6, pos46-47) Java rolled the **Regeneration** die (d6 pos48); Rust
rolled a **second casualty** (d16 pos48) then a d6 — i.e. Decay's second casualty roll.

**Root cause:** Decay's `requiresSecondCasualtyRoll` is BB2016-only — `bb2016/Decay` registers it, but
`mixed/Decay` (@RulesCollection BB2020+BB2025) registers only cancelsAllowsRaisingLineman. Rust's
`SkillId::Decay.properties()` is edition-agnostic and returned it for all editions, so a bb2025 Decay
player rolled a second casualty Java never rolls. Same edition-property class as NoHands/SafeThrow, but
here Decay is still valid in bb2025 — only the one property is edition-gated, so the fix gates the roll
rather than dropping the skill.

**Fix (`injury.rs`):** gate the second casualty roll to `game.rules == Rules::Bb2016`. +2 tests (bb2016
still rolls it; bb2025 does not). Also fixed the existing Decay test which used Bb2020 (wrong edition).

**Verified:** khemri 100/100 GREEN (combined with the earlier opponent-catch turnover fix c0ce1d75). No
regression: 12 green rosters 100/100; ffb-engine 7032/0, ffb-model 2777/0. **13 green total.** Commit pending.
