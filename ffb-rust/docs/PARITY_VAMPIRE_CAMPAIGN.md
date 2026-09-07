# Vampire — heuristic-agent parity campaign

Matchup: `--home vampire --away vampire`, tier 3, `--heur-classes all`, seeds 1-100, three
editions x three `--heur-scale` values (`1.0`, `0`, `1e6`).

Surface: Vampire Blitzer / Thrower / Runner and the Vargheist all carry **Bloodlust** (a negatrait
that fails the declared action) and **Regeneration**; the three Vampire positions also carry
**Hypnotic Gaze**, the Vargheist carries **Claws / Frenzy / Loner 4**. Thralls carry nothing.
Vampire is the ONLY roster in `data/rosters/**` that carries Bloodlust in any edition, which bounds
the blast radius of every engine fix below.

## ITER1 — 0/100 everywhere -> bb2020 100/98/100, bb2025 99/95/100, bb2016 48/90/59

Baseline (before this iteration): **0/100 in all three editions**, diverging at the FIRST or second
activation of every seed. The `--agent random` control was already 100/100 x 3 — the engine was
fine, the heuristic path had simply never been driven for this race.

### Fix 1 (HARNESS, called) — `hasNegatrait` missed the bb2020+ spelling of Bloodlust

`ParityRunner.hasNegatrait` matches on the skill's DISPLAY NAME. Java has two Bloodlust classes with
two different names:

| edition | class | `getName()` |
|---|---|---|
| bb2016 | `skill/bb2016/BloodLust` | `Blood Lust` |
| bb2020 / bb2025 | `skill/mixed/Bloodlust` | `Bloodlust` |

The harness listed only `"Blood Lust"`, so in bb2020/bb2025 **no vampire counted as a negatrait
carrier** on the Java side, while Rust's `has_negatrait` — which matches on the edition-independent
`SkillId::BloodLust` — counted all of them. The consequence is one line of the value model,
`heuristic_agent.rs:2533` `w_player *= 0.55`:

```
bb2025 seed 1, activation k=1, identical candidate lists (n=2169, draws=6):
  thralls   away_08..11  J 0.402000  R 0.402000
  vampires  away_02..07  J 0.402000  R 0.257100      (= 0.402 * 0.55)
  Vargheist away_01      J 0.185000  R 0.137750      (= 0.185 * 0.55)
```

Every vampire candidate was mispriced, so the very first declaration of every seed differed.
Fix: add `"Bloodlust"` to the name list (`ParityRunner.java`, with the edition table in the
javadoc). bb2025 seeds 1-10: **0/10 -> 6/10**. This is a CALLED harness fix; there is no Rust-side
gap — Rust was right.

*Lesson (a repeat of the slann_fumbbl "Bone-head" case): a harness predicate that matches a skill by
name must carry EVERY edition's spelling. Both engines' skill sets agreed; only the harness's string
list did not.*

### Fix 2 (ENGINE) — the Blood-Lust action-change dialog list was approximated, not ported

`bb2025/BloodLustBehaviour` shows `DialogBloodlustActionParameter` on a failed roll only for an
explicit list of declared actions:

```java
if (Arrays.asList(new PlayerAction[]
      {VICIOUS_VINES, BLOCK, PASS, HAND_OVER, THROW_BOMB, THROW_TEAM_MATE, KICK_TEAM_MATE,
       FOUL, STAND_UP, STAND_UP_BLITZ, MULTIPLE_BLOCK, SECURE_THE_BALL, PUNT})
    .contains(actingPlayer.getPlayerAction())) { ... showDialog ... }
else { publishParameter(MOVE_STACK, null); GOTO_LABEL / NEXT_STEP; }
```

`bb2020/BloodLustBehaviour` uses the same list with `SECURE_THE_BALL`/`PUNT` replaced by
`BLITZ_MOVE`/`GAZE_MOVE`; `bb2016/BloodLustBehaviour` has **no such branch at all**.

Rust's live step (`step/bb2025/shared/step_blood_lust.rs` — the one `driver.rs` builds for all three
editions) approximated the membership test as `a != Move && get_alternate_action(a) != a`, which is
true for every non-MOVE action because `get_alternate_action` defaults to `Move`. So a bb2025
vampire who failed Blood Lust on a **BLITZ_MOVE** got a `BloodlustAction` prompt Java never shows,
and the resulting `MOVE_STACK`/`BLOOD_LUST_ACTION` publish diverged the post-hash on an otherwise
identical declaration.

Ported the three lists verbatim into `shows_bloodlust_action_dialog(rules, action)` (plus
`bloodlust_dialog_change_to_move`, the dialog's `changeToMove` flag, kept beside it), and pointed the
dead mirror in `skill_behaviour/bb2025/blood_lust_behaviour.rs` at the same function so the two
copies cannot drift again. Regression tests assert each edition's list member-by-member and were
verified to FAIL against the old predicate (3 of them).

### Fix 3 (ENGINE) — `StepInitMoving`'s invented Blood-Lust early-out ended activations Java carries on

`StepInitMoving.executeStep` in Java has NO Blood-Lust branch. With an empty `MOVE_STACK` it simply
sets no next action, so the step **parks** and the client is asked; `ParityRunner`'s
`case INIT_MOVING` answers with `sendMoveAction` -> `MoveReplay`. Rust instead had

```rust
if game.acting_player.suffering_blood_lust { ...; return goto(label).publish(EndPlayerAction(true)); }
```

which ended the activation unconditionally (with an added carve-out for the pre-block blitz leg).
Measured counter-examples:

| where | Java | Rust (before) |
|---|---|---|
| bb2025 seed 3 step 54, blitz AFTER its block | `JSTATE INIT_MOVING cm=1 -> cm=5` (4 more squares) | activation ended, cm stayed 1 |
| bb2020 seed 7 i=30, plain MOVE, `moved=true` | `INIT_MOVING cm=0 -> cm=4` | activation ended, cm stayed 0 |
| bb2020 seed 7 i=29, plain MOVE, `moved=false` | one INIT_MOVING look, no move | already matched |

The discriminator is whether the activation has **already started**: `hasMoved` (StepInitMoving has
popped the client's first square) or `hasBlocked`. The early-out is now taken only for an activation
that has done nothing yet.

bb2025: `1.0` 96 -> 99, `0` 75 -> **95**, `1e6` 100. bb2020: `1.0` 94 -> **100**, `0` 81 -> **98**,
`1e6` 99 -> **100**.

### Fix 4 (AGENT, Rust only) — the heuristic agent never mirrored `isHandledActingAction`

`ParityRunner`'s heuristic pick loop (~line 699), immediately after `chooseActivation` has done all
of its bookkeeping:

```java
if (!isHandledActingAction(hAction)) {
    System.err.println("UNHANDLED_ACTING_ACTION_AT_PICK: " + hAction + " ... deselecting, no step logged");
    continue;
}
```

`continue` injects nothing: the engine is still parked at INIT_SELECTING phase 1, the sampler draws
are spent, the player is already in `usedThisTurn` and the bucket/action are already recorded, so the
harness just picks again. The RANDOM agent has mirrored this since the goblin campaign
(`random_agent.rs`, `continue 'reselect`); the heuristic agent, which replaced the pick loop
wholesale, inherited none of it.

The action that exposes it is **GAZE**. `isHandledActingAction` has no GAZE case (only
`AUTO_GAZE_ZOAT` and `BLACK_INK`), and bb2016 is the only edition whose eligible list offers it
(`computeEligiblePlayers` adds `PlayerAction.GAZE` for `canGazeDuringMove`, a bb2016-only property).
So EVERY bb2016 vampire seed diverged at its first Hypnotic Gaze declaration — Java logged no step
and re-picked, Rust carried the gaze out and the game fell over immediately (seed 8: `game_end` at
i=2 against Java's 91 steps).

Mirrored in both `handle_activate` and `handle_activate_deep`. **Placement is load-bearing**: Java
records the bucket inside `chooseActivation`, so the DISCARDED pick still consumes the one-shot
`novelty` bonus. With the re-pick placed ahead of `seen_bucket`, every k=2 weight was exactly
**0.08** higher in Rust than in Java (`novelty = 0.08`), and because `EndTurn` keeps weight 0.0 the
uniform shift changed EndTurn's share of the softmax and the re-pick landed on a different player
(bb2016 seed 8: Rust H10, Java H11).

### Fix 5 (AGENT, Rust only) — the re-pick read the DISCARDED pick's option buffer

Fix 4 on its own left bb2016 at 0/20: at the re-pick (k=2) the two `FFB_CAND` dumps were identical
line for line — same 2018 candidates in the same order, bit-identical weights, `draws=7` on both
sides — and Java still declared `Home11/MOVE` where Rust declared `home_10/MOVE`. Everything the
candidate probe could see agreed, which is exactly the ITER2 shape: the divergence was downstream of
the list.

A temporary probe on the two-level draw itself settled it in one line:

```
RPICK k=1 ... i=408  buflen=2224 candlen=2223  buf_i=home_02/HypnoticGaze  cand_i=home_02/HypnoticGaze
RPICK k=2 ... i=1989 buflen=4243 candlen=2018  buf_i=home_10/Move          cand_i=home_11/Move
```

`buflen=4243` for `candlen=2018`: the option buffer still held k=1's 2224 entries with k=2's 2019
appended. `decide` clears `self.buf` before dispatching a prompt to its handler
(`heuristic_agent.rs:3714`); `handle_activate` never clears it itself, so re-entering it directly
left the stale buffer in place. `group_declarations` indexes `cands`, but `gw`, `cw` and `take(i)`
all index `buf` — so the whole draw ran on the DISCARDED pick's weights (`gw[gi]` = 0.2375, the k=1
value, instead of 0.1575) and the winning index named a different player in the two arrays.

One line — `self.buf.clear()` before the recursive call — and bb2016 seed 8 goes green.

*Lesson: "identical candidate lists, identical weights, identical draw counts, different pick" does
not mean the sampler's arithmetic differs. It means the sampler is not reading the array you dumped.
The probe that mattered printed the chosen index resolved through BOTH arrays.*

## Gates after ITER1

Baseline for every cell was **0/100**.

| | scale 1.0 | scale 0 (argmax) | scale 1e6 |
|---|---|---|---|
| bb2016 | **48**/100 | **90**/100 | **59**/100 |
| bb2020 | **100**/100 | **98**/100 | **100**/100 |
| bb2025 | **99**/100 | **95**/100 | **100**/100 |

`--agent random` vampire control: **100/100 in all three editions** (unchanged, re-measured after
every fix). `cargo test -p ffb-engine`: **7,448 / 0**. `mvn -o -pl ffb-ai test`: green. Java trees
agree. TIMING at scale 1.0, 100 seeds, batched JVM: bb2016 `rust_total=55.0s`, bb2020 `60.9s`,
bb2025 `51.3s`.

Closed-roster regression, bb2025 @1.0, seeds 1-100 — all **100/100**: amazon, chaos, dwarf, goblin,
human, khemri, lizardman, necromantic, nippon, norse, nurgle, ogre, orc, renegades, skaven, slann,
slann_fumbbl, underworld, wood_elf. (The Leap carriers' remaining six gates each are PENDING — see
the measurement note below.)

## Frontier / next step

1. **bb2016 @1.0, 52 reds — unclassified.** This is the biggest block left and no family analysis
   has been done on it. Start with `MATCHUP=vampire sh scripts/frontier.sh bb2016` off the gate log.
   Two things are already known: every Hypnotic Gaze declaration is now discarded on both sides
   (grep a gate log for `UNHANDLED_ACTING_ACTION_AT_PICK: GAZE`), and bb2016 is the one edition
   whose Blood-Lust failure CANCELS the declared action with no conversion dialog. Note the shape of
   the three bb2016 numbers — 48 at `1.0`, **90 at argmax**, 59 at `1e6` — an argmax-only-green race
   usually means a residual SAMPLING/draw-count divergence rather than a rule divergence, since
   argmax consumes no draws.
2. **bb2020 `@0` 2 reds, bb2025 `@1.0` 1 red / `@0` 5 reds.** Unclassified.
3. **Re-measure the Leap carriers' other six gates** (slann, slann_fumbbl, wood_elf, nippon,
   goblin at bb2016/bb2020 x three scales). Their bb2025 @1.0 gates are all 100/100 on the final
   binary; the rest were invalidated mid-run (see the measurement note) and have to be re-run
   because Fix 4/5 touch the agent's activation pick.

## Conclusions of mine that turned out WRONG

* "Removing the invented `suffering_blood_lust` early-out from `StepInitMoving` entirely is the
  Java-faithful fix." Measured 18/20 -> **7/20** on bb2020 seeds 1-20. Java's harness declines the
  move for an activation that has not started (`MoveReplay` returns `END_PLAYER_ACTION`), so the
  early-out is right for exactly that case; only the already-started legs were wrong.
* The comment that justified that early-out claimed "the client is never asked for a move" for a
  blood-lusting vampire. That is a property of the RANDOM contract (ParityRunner's `INIT_MOVING` arm
  deselects when `activation == null`), not of Java's engine, and it is false under the heuristic
  contract — which is exactly why the guard survived every random-agent matrix.

## A measurement trap this iteration walked into

Ten concurrent `ffb-parity` processes (three vampire edition streams + three Leap-carrier streams +
two regression streams, each with its own JVM) produced **0/100 on amazon bb2025 @1.0** and on
`slann`/`wood_elf` bb2025 @1.0 — races that had reported 100/100 an hour earlier on effectively the
same binary, and whose OTHER scales in the same batch stayed at 100/100. Several background shells
also reported "completed, exit 0" with a truncated log and no `PARITY:` line at all.

Re-run alone with the final binary, amazon bb2025 @1.0 is **100/100**. The reds were the
measurement, not the engine. The existing rule ("never two runs of the same edition+matchup at
once") is not sufficient — there is a machine-wide concurrency ceiling too, and past it a run can be
killed mid-flight and still look like a red. Ten `ffb-parity` STREAMS is not ten workers — each
spawns its own batched JVM, so ten streams is ~twenty processes on sixteen logical CPUs.

The second cause was worse, and only surfaced when a re-run printed
`./target/release/ffb-parity: No such file or directory`: **another agent working in the same tree
had renamed the binary to `ffb-parity.exe.stale`** as part of its own `cargo build --release`. The
"never rebuild target/release while a gate is running" rule cannot be enforced from inside one
session when the working directory is shared. The fix that works is to gate from a PRIVATE build:

```
CARGO_TARGET_DIR=<repo>/../target-vamp cargo build --release -p ffb-parity
<repo>/../target-vamp/release/ffb-parity.exe ...
```

Practical rules: keep it to two or three streams; build the gating binary into a private
`CARGO_TARGET_DIR`; and treat any 0/100 that contradicts a neighbouring scale of the same race as a
suspect measurement until it has been reproduced alone. Every red in this iteration that was NOT
reproducible alone came from one of these two causes, not from the engine.
