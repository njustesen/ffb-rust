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
