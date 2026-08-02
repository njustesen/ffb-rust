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
