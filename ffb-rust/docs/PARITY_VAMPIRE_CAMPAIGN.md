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

## Event coverage (harvested alone, `MATCHUP=vampire scripts/harvest_coverage.sh <edition> 1.0`)

`docs/EVENT_COVERAGE_vampire_bb2016.md` / `_bb2020.md` / `_bb2025.md`. Per notable skill:

| skill | verdict | evidence |
|---|---|---|
| **Bloodlust** | exercised + evented | `bloodLustRoll` 4,389 (bb2016) / 5,359 (bb2020) / 6,592 (bb2025) over 100 games each |
| **feeding after a failed Bloodlust** | exercised + evented | `biteSpectator` 628 / 1,093 / 1,317 — that event is the FAILED-to-feed branch (`StepInitFeeding` with no ST<=3 adjacent victim); a SUCCESSFUL feed on a thrall has no event of its own and is hash-verified only |
| **Regeneration** | exercised + evented | `regenerationRoll` 41 / 73 / 78 |
| **Juggernaut** (Vampire Blitzer) | exercised + evented | `skillUse` 47 (bb2020) / 36 (bb2025), all `Juggernaut`, both `used=true` and `used=false` |
| **Hypnotic Gaze** | **UNREACHABLE under the parity contract** | bb2016 is the only edition whose eligible list offers `PlayerAction.GAZE`, and `ParityRunner.isHandledActingAction` has no GAZE arm, so BOTH harnesses now discard the declaration (`UNHANDLED_ACTING_ACTION_AT_PICK: GAZE`, no step logged). The declared-action histogram has no Gaze row in any edition. This is a harness-contract gap, not a Rust gap — filed as BACKLOG E17 |
| **Claws / Frenzy / Loner 4** (Vargheist) | exercised, unevented | no emit site (BACKLOG E6, the general "skills are used silently" finding). Frenzy is visible only as extra blocks: 1,368 `blockRoll` for 1,304 `block` in bb2020 |

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

## ITER2 — 2026-09-07: the bb2016 baseline was STALE and the sampler is exonerated

**The recorded frontier was wrong, not unsolved.** ITER1 left bb2016 at "0/100 at baseline, 0/20
after Fix 4" and named the two-level re-pick sampler as the suspect, on the evidence that at
bb2016 seed 8 the candidate lists agreed in order, the weights were bit-identical (`w=3db16873`,
all 2018 entries) and both sides reported `draws=7`, yet Java picked H11/MOVE and Rust H10/MOVE.

That is no longer reproducible. **Seed 8 now passes** (`PARITY: 1/1 games match`), and bb2016
`@1.0` measures **4/8 on seeds 1-8**, not 0. The intervening sweep fixes (skaven's give
declaration, slann's dodge pair, the trap-door chain, and the rest of the 30-race sweep) closed
it; nobody re-measured vampire afterwards. **Re-measure a stale red before designing an
instrument for it** — the whole ITER1 frontier section was describing a fixed bug.

### The sampler is not the fault, and the old probe could not have shown that

Added `FFB_GRP=1` to BOTH agents, printing the level that actually decides — group count, the
temperature, the RAW draw as bits, the pick, and the group weights: `RGRP` in Rust's
`softmax_pick` (`heuristic_agent.rs`), `JGRP` in `Sampler.softmaxPick` (the co-editable parity
harness). On seed 8 the two streams are **239 picks, byte-identical after normalisation** — same
lengths, same `t`, same `r`, same picks, same weights.

**Why `FFB_CANDSUM` could never have settled this.** It aggregates candidates per
`(player, action)` through a `BTreeMap`, so two NON-ADJACENT runs of one declaration merge into a
single count there, while `group_declarations` (contiguous runs) hands the sampler two groups.
The probe the ITER1 frontier relied on is structurally blind to the group boundaries that set
`gi`, so "lists identical, weights identical, draws equal" was never evidence that the group-level
inputs agreed. It happens that they do agree — but that had not been measured.

Both samplers were also read line-for-line and are faithful: same `len <= 1` guard, same
`-Float.MAX_VALUE` / `f32::MIN` max seed, same sequential accumulation, same normalisation, same
`r < c` walk with the `len - 1` fall-through. Rust's real EndTurn option at weight 0.0 and Java's
virtual `allW[endIdx] = 0.0f` slot are equivalent, and `groupDeclarations` matches
`group_declarations` key-for-key.

### The real frontier

`MATCHUP=vampire sh scripts/first_state_divergence.sh bb2016 3`:

```
seed 3    first hash diff idx 9     resolving idx 8    R t2 home Activate(home_01,Blitz)
   i=8    R t2  home Activate(home_06,Move)   | J t2  home Activate(Home6,MOVE)
   i=9    R t2  home Activate(home_01,Blitz)  | J t2  home Activate(Home1,BLITZ_MOVE)
   i=10   R t3  home Activate(home_08,Move)   | J t2  away Activate(Away11,MOVE)   <-- DIFF
```

The declarations AGREE — `Move`/`MOVE` and `Blitz`/`BLITZ_MOVE` are the two engines' log spellings
of the same declaration (the matching `i=7`/`i=8` pairs prove the casing is cosmetic). The hash
diff at `idx 9` is produced by the RESOLUTION of `i=8`, home_06's Move. Seed 2 fails the same way
(step 20, pre-state hash already apart).

Next: diff the board at `i=8`/`i=9` for seed 3 with `FFB_IDSTATE`/`JIDSTATE` — the hash cannot see
players `nr > 11` or a player's ACTIVE bit, so use the id-state dump, not the hash.

Measured: bb2016 `@1.0` seeds 1-8 = 4/8 (seeds 2, 3 and two of 5-8 red). bb2020/bb2025 not
re-measured this iteration; their ITER1 numbers (100/98/100 and 99/95/100) are also suspect for
the same staleness reason.

### ITER2 addendum — the seed-3 divergence is NOT on the player board

`FFB_IDSTATE=1` / `JIDSTATE` full-board diff for seed 3, all 26 players both teams (including the
hash-blind `nr > 11`), coordinate + base state:

| i | players | diffs |
|---|---|---|
| 7 | 26/26 | 0 |
| 8 | 26/26 | 0 |
| 9 | 26/26 | **0** |
| 10 | 26/26 | 1 — `away_02` R=`13,6`/base`1` J=`14,5`/base`3` |

**The hash at `i=9` differs while every player's coordinate and base state agrees.** So the
divergence at the reported first-diff index is in a component the id-state dump does not print.
`state_string` (`ffb-model/src/util/state_hash.rs:18`) hashes half, `turn_home`, `turn_away`,
active side, both scores and the ball coordinate/`in_play` BEFORE it reaches the first 11 players —
and `first_state_divergence.sh` prints only the active side and ONE turn number, so a
`turn_away` or ball-coordinate drift is invisible in its output too. Both instruments were
consistent with "the boards agree", and both were hiding the same class of difference.

By `i=10` the drift has reached the board: `away_02` sits one square away with a different base
state (1 vs 3), and the active side/turn diverge (R `t3 home`, J `t2 away`).

Next instrument: dump the `state_string` COMPONENTS (half, both turn counters, active, scores,
ball) per step index on both sides and diff at `i=8`/`i=9`. Do not diff the player board again —
it is already proven equal at the first-diff index.

## ITER3 — 2026-09-07: bb2016 is 46/100, and 52 of 54 failures carry NO state divergence

Measured on a fresh gate (`FFB_PARITY_ROOT=parity_vloop`, one gate at a time, pinned to 4 of 16
CPUs via `ProcessorAffinity`): bb2016 **@1.0 = 46/100**, **@0 = 33/100**. Against ITER1's recorded
0/100. `scripts/vamp_loop.ps1` drives the nine gates.

### The failures are a game-LENGTH problem, not a rule divergence

Per-seed classification of all 54 bb2016 `@1.0` failures, comparing the per-step `state_hash`
over the common prefix:

| family | seeds |
|---|---|
| **hashes agree over the whole common prefix, Rust log LONGER** | **38** |
| **hashes agree over the whole common prefix, Rust log SHORTER** | **14** |
| turn/active/half differ at the first diff | 1 |
| same turn+active+half, hash differs | 1 |

**52 of 54 failures share a bit-identical prefix and differ only in how many steps each engine
logs.** Only two seeds diverge on state at all, both at index 69-70. Vampire bb2016 is therefore
mostly an end-of-game / step-accounting disagreement, not a rules disagreement — which is
consistent with the Java harness emitting `UNHANDLED_STEP WINNINGS turnMode=END_GAME` and
`UNHANDLED_DIALOG WINNINGS_RE_ROLL turnMode=END_GAME` **10,500 times each** across the gate
(105 per game, every game).

Of the 14 SHORTER seeds, Rust's terminal declaration is `HandOffMove` in **14 of the whole set**
of terminal declarations (the give chain), `Move` in 33, `Blitz` 3, `Block` 2.

Also present, and not yet explained: `UNHANDLED_STEP INIT_BLOCKING turnMode=REGULAR` 53 times, and
`UNHANDLED_ACTING_ACTION_AT_PICK GAZE ... deselecting, no step logged` 4,105 times (the expected
bb2016 GAZE re-pick, mirrored by Fix 4).

### Two analysis mistakes worth not repeating

1. **I classified from `parity/` while the gate wrote `parity_vloop/`.** The stale directory gave a
   confident, entirely wrong split (31 hash-diff / 19 short / 4 turn) and a "1-seed" HandOffMove
   family. `FFB_PARITY_ROOT` moves the output tree — always classify from the root the gate
   actually used. This is [[stale_parity_jsonl]] in a new costume.
2. **A regex looking for `rust=None` inside the failure block** reported "37 of 54 = Rust ends
   early, 69%". It was matching a `rust=None` belonging to a different comparison in the same
   block. Seed 25 supposedly "ended at step 1" while its Rust log had 94 steps. Classify from the
   jsonl, not from the failure prose.

Next: find why the two engines log different step counts on a bit-identical prefix. Start at
END_GAME (the WINNINGS pair above), not in the rules — and check whether the 38 Rust-LONGER seeds
are Java stopping early rather than Rust running on.

## ITER4 — 2026-09-07: RETRACTION of ITER3's table, and the real dominant bug

### Retract ITER3's classification

**ITER3's 38/14/1/1 table is void.** All three bb2016 gates in that loop ran with a single
`FFB_PARITY_ROOT=parity_vloop`, and all three scales of one edition share the
`bb2016/vampire_vs_vampire/` matchup dir — so `@0` and `@1e6` **overwrote** `@1.0`'s jsonl before I
classified it. The gate verdicts in ITER3 are sound (they come from the run logs); only the
per-seed classification was computed against the wrong files.

`scripts/vamp_loop.ps1` now uses a per-GATE root (`parity_vloop_<ed>_<sc>`). It also reads the
verdict from BOTH streams: **`PARITY: 100/100 games match.` goes to STDOUT, while
`PARITY: N/100 passed, M FAILED.` goes to STDERR** — reading only one stream makes a green gate
look like a missing one, which is how bb2020 `@1.0` and `@1e6` first read as "no verdict".

### The true nine-gate baseline

| edition | @1.0 | @0 | @1e6 |
|---|---|---|---|
| bb2016 | 46/100 | 33/100 | 67/100 |
| bb2020 | **100/100** | 98/100 | **100/100** |
| bb2025 | 99/100 | 95/100 | **100/100** (parity green; coverage items MISSING) |

**3 of 9 gates green.** bb2016 carries 154 of the 162 seed failures.

### 37 of 54 bb2016 @1.0 failures are ONE bug: the give ends the game

Re-measured into an isolated root (`parity_one_bb2016_1.0`, 46/100 reproduced exactly) and
classified there:

| family | seeds |
|---|---|
| **prefix identical, Rust log SHORTER — terminal declaration `HandOffMove` in ALL 37** | **37** |
| state diverges (same turn/active) | 15 |
| state diverges (turn/active differ) | 2 |

Seed 10 is the minimal case: Rust logs 9 steps to Java's 105 and emits `game_end` at `i=10` in
**half 1, turn 2, score 0-0**. `FFB_STEPTRACE` names the exact stall:

```
RSTATE step=InitSelecting  prompt=Move  ap=A4 act=Some(HandOverMove)
LOOP   applied=Activate(away_04,HandOffMove)  prompt_after=Some(Move{...})
RSTATE step=InitMoving     prompt=Move  ap=A4 act=Some(HandOverMove)
LOOP   applied=Move->(21,5)                   prompt_after=Some(Move{...})
RSTATE step=InitPassing    prompt=-     ap=A4 act=Some(HandOver)
LOOP   applied=HandOff->away_09               prompt_after=None finished=false
RUST_END
```

The give RESOLVES (the ball reaches `away_09`), then the engine yields **no prompt with
`finished=false`** — the step stack has emptied, the harness has nothing to drive, and the game
ends mid-first-half. Java plays on for another 96 steps.

Verified faithful, so the bug is NOT in them: the bb2016 `Pass` generator tail matches Java's
`bb2016/Pass.java` line-for-line (`HAND_OVER`, `CATCH_SCATTER_THROW_IN`, `END_PASSING`, and
correctly NO `RESET_TO_MOVE` — that is bb2025-only); `StepEndPassing`'s two `EndPlayerAction`
push sites mirror Java's; bb2016 `StepInitPassing` sets `thrower_id` at the same sites Java does.

Remaining suspect, and the next thing to probe: Java pushes `EndPlayerAction` in BOTH branches
whenever the thrower IS the acting player, and its only no-push path is the dump-off `else`
(`game.setDefenderAction(null)`). Rust's `thrower_is_acting_player` requires
`game.thrower_id == game.acting_player.player_id` — and the trace shows the acting player moving
from `ap=A4` to `apa03` across the hand-off. If `thrower_id`/`acting_player` disagree at
`StepEndPassing` on the MOVE variant, Rust takes the dump-off path and pushes nothing. Probe
whether `StepEndPassing` runs at all on this path and which branch it takes — that is one line of
output and it names the fix.

bb2020/bb2025 are green at `@1.0`, so whatever this is, it is bb2016-specific or specific to the
bb2016 twin of the shared step.

---

## ITER2 — bb2016 48/90/59 -> **100/100/100**; the bb2016 blitz-dispatch command chain

Frontier at the start: bb2016 `@1.0` 48/100 (52 unclassified reds), `@0` 90/100, `@1e6` 59/100.
The inherited hypothesis was "argmax-green + scale-red means a residual SAMPLER/draw-count
divergence". **That hypothesis was WRONG**, and it is the ITER1 conclusion this iteration retracts:
every bb2016 red traced to ONE engine mechanism — the bb2016 blitz's *second* client command is
never re-delivered — and argmax was less red only because it declares fewer blitzes whose target is
chosen at the move prompt. No sampler, buffer or draw-count work was needed.

Instruments that named it: `first_state_divergence.sh` (bb2016 seed 10 -> "first hash diff idx 2,
resolving idx 1, `Activate(away_02,Blitz)`, declarations AGREE") and then `FFB_STEPTRACE=1` +
`FFB_MOVEP=1` + `FFB_DICE_TRACE=1` on that one seed. The decisive lines:

```
J  JMOVEP k=2 pid=...Away2 at=13,6 n=6 offered=[] ans=FIRE_TERMINAL
J  DICE_TRACE pos=18 ... rollBlockDice ... StepBlockRoll.executeStep:106
R  RMOVEP k=2 pid=away_02 at=Some((13, 6)) n=6 offered=[...] ans=Block { defender_id: "home_01" }
R  RSTATE step=InitBlocking prompt=BlockTarget ...            <- and then the turn just ended
```

Same declaration, same offered count, same answer — and Java rolled the blitz block die while Rust
rolled nothing.

### Fix 1 (ENGINE) — `StepEndMoving.dispatchPlayerAction` must RE-DELIVER the command

`bb2016/move/StepEndMoving.java:191-199`:

```java
private StepCommandStatus dispatchPlayerAction(PlayerAction pPlayerAction) {
    UtilServerSteps.changePlayerAction(this, actingPlayer.getPlayerId(), pPlayerAction, actingPlayer.isJumping());
    if (pushSequenceForPlayerAction(pPlayerAction)) {
        getResult().setNextAction(StepAction.NEXT_STEP_AND_REPEAT);
    }
    return StepCommandStatus.SKIP_STEP;
}
```

`SKIP_STEP` leaves the received `CLIENT_BLOCK` **unconsumed** and `NEXT_STEP_AND_REPEAT` re-delivers
it to the first step of the sequence just pushed, which is `StepInitBlocking` —
`case CLIENT_BLOCK: fBlockDefenderId = blockCommand.getDefenderId()`. That is the *only* route by
which the defender id reaches the block sequence when the blitzer's target was chosen at the MOVE
prompt instead of folded into the activation. (`bb2016/move/StepInitMoving.dispatchPlayerAction`
already used `GOTO_LABEL_AND_REPEAT`, so the command did reach `StepEndMoving`; the chain broke at
the second hop.)

Rust returned a plain `NextStep`, so `StepInitBlocking` ran with `block_defender_id == None`,
prompted `AgentPrompt::BlockTarget`, and the agent's `BlockTarget => Action::EndPlayerAction` arm
threw the blitz away.

Fix: `StepOutcome::next_and_repeat()` (new, additive, in `step/framework.rs` — the driver already
implemented `StepAction::NextStepAndRepeat` and its `forwarded` command slot) at that one site.
Test `dispatch_player_action_forwards_the_command_to_the_pushed_sequence`, verified to FAIL on
`StepOutcome::next()`.

### Fix 2 (ENGINE) — `bb2016 StepInitPassing` had no `CLIENT_HAND_OVER` arm

Fix 1 made the re-delivery real, and the first thing it re-delivered was a hand-off: bb2016 seed 10
then produced **9 Rust steps against Java's 105**, stalled at `RSTATE step=InitPassing prompt=-`
(a bare `cont()` with no prompt = `waiting_for_command` with nothing to ask). Ported
`bb2016/pass/StepInitPassing.handleCommand`'s `case CLIENT_HAND_OVER` verbatim (catcher id, pass
coordinate from the catcher's square, thrower = acting player, `throwerAction = HAND_OVER`,
`EXECUTE_STEP`). Test `hand_over_command_sets_catcher_thrower_and_pass_coordinate`, verified to
FAIL without the arm. bb2016 seeds 1-20: 12/20 -> 17/20.

### Fix 3 (ENGINE) — a blood-lusting blitzer is BLITZ_MOVE, not MOVE

`bb2016/move/StepEndSelecting.executeStep`:

```java
} else if (actingPlayer.isSufferingBloodLust()) {
    if (fDispatchPlayerAction != null) {
        if (!fDispatchPlayerAction.isMoving()) { fDispatchPlayerAction = PlayerAction.MOVE; }
        dispatchPlayerAction(fDispatchPlayerAction, false);
```

Both the GUI client and `ParityRunner` declare a bb2016 blitz as **BLITZ_MOVE**
(`declared = (action == BLITZ) ? BLITZ_MOVE : action`), which `isMoving()` accepts — so Java does
NOT rewrite it, dispatches the **BlitzMove** sequence, and leaves the acting player on BLITZ_MOVE.
Rust stores that same declaration as `PlayerAction::Blitz` (the codebase already carries this
accommodation in `prepare_standing_up`), and `Blitz.is_moving()` is false, so a blood-lusting
blitzer was silently downgraded to a plain MOVE. The agent then could not fire its block at all,
because its move handler mirrors `MoveReplay.decide`:

```java
case BLITZ:
    dispatchable = ("BlitzMove".equals(f.paNow) || "KickEmBlitz".equals(f.paNow))
        && !f.hasBlocked && f.targetAdjacent;
```

`paNow` was `Blitz`, so `dispatchable` was false and Rust walked away where Java threw the block
(seed 2 i=75: `JMOVEP k=96 ans=FIRE_TERMINAL` vs `RMOVEP k=96 ans=Move{...}`). Fix: recognise
`PlayerAction::Blitz` in that branch, write BLITZ_MOVE onto the acting player via
`change_player_action` (the state Java has had since its activation) and dispatch BlitzMove. Test
`blood_lust_keeps_a_declared_blitz_as_blitz_move` asserts both the action and that the pushed
sequence is step-for-step `BlitzMove::build_sequence`. bb2016 seeds 1-20: 17/20 -> **20/20**.

### GATES (all nine re-measured on the final binary, seeds 1-100, tier 3, `--heur-classes all`)

|              | scale 1.0 | argmax (0) | 1e6 |
|---|---|---|---|
| **bb2016**   | **100/100** (was 48) | **100/100** (was 90) | **100/100** (was 59) |
| **bb2020**   | 100/100 | 98/100 (seeds 8, 76) | 100/100 |
| **bb2025**   | 99/100 (seed 75) | 95/100 (seeds 8, 30, 50, 65, 71) | 100/100 |

TIMING at scale 1.0: bb2016 `rust_total=40.8s`, bb2020 `59.6s`, bb2025 `54.4s`.
Random controls (`FFB_PARITY_ROOT=parity_random`, `--agent random`, seeds 1-100): bb2016 100/100,
bb2020 100/100, bb2025 100/100.
`cargo test -p ffb-engine`: **7,451 passed / 0 failed / 15 ignored** (+3 this iteration).

**Regressions — every closed roster, seeds 1-100, `@1.0`, ALL 100/100.** Fix 1 and Fix 2 are in
bb2016 step files that EVERY bb2016 race runs (not just Blood Lust carriers), so bb2016 was gated
too, which is the measurement that actually mattered:

* `bb2016`: amazon chaos dwarf goblin human khemri lizardman necromantic nippon norse nurgle ogre
  orc renegades skaven slann slann_fumbbl underworld wood_elf — 19/19 at 100/100.
* `bb2025`: the same 18 required rosters — 18/18 at 100/100 (Fix 3 is blood-lust-only, so
  vampire-only; Fix 1/2 are bb2016 files; `StepOutcome::next_and_repeat` is additive).

No agent, `Reach` or candidate-building code was touched this iteration, so the Leap carriers'
nine gates are not implicated by these fixes; their bb2016 `@1.0` gates are in the list above.

### Event coverage (re-harvested x3, each run ALONE)

`docs/EVENT_COVERAGE_vampire_bb20{16,20,25}.md`. bb2016's numbers moved because blitzes now actually
execute there (`blockRoll` 1,085 in bb2016).

| skill | verdict | evidence (bb2016 / bb2020 / bb2025, 100 games each) |
|---|---|---|
| **Bloodlust** | exercised + evented | `bloodLustRoll` 4,399 / 5,359 / 6,592 |
| **feeding after a failed Bloodlust** | exercised + evented | `biteSpectator` 631 / 1,093 / 1,317 (the FAILED-to-feed branch; a successful feed on a thrall has no event and is hash-verified only) |
| **Regeneration** | exercised + evented | `regenerationRoll` 40 / 73 / 78 |
| **Juggernaut** | exercised + evented | `skillUse` — / 47 / 36 (bb2016 emits no `skillUse` at all in this run) |
| **Hypnotic Gaze** | **UNREACHABLE** | still zero `gaze` rows in ALL THREE editions' declared-action histograms (BACKLOG E17 — unchanged; a harness-contract gap, not a Rust gap) |
| **Claws / Frenzy / Loner 4** | exercised, unevented | no emit site (BACKLOG E6) |

### Conclusions of mine that turned out WRONG

* **"argmax-green + scale-red means a sampler/draw-count divergence"** (inherited from ITER1's
  "sharpest verified fact", and the first thing I planned to chase). It was a rule divergence in the
  blitz dispatch chain. What argmax actually changes is WHICH declarations happen, and the broken
  path needed a blitz whose target is picked at the move prompt.
* **`activation_already_started || standing_up`** in `step/bb2025/move_/step_init_moving.rs`,
  proposed for bb2025 seed 75 (a prone vampire stands up, fails Blood Lust, and Java still walks it
  over two more prompts). Measured bb2025 seeds 70-80: **0/11** (from 10/11). Reverted.
* **Deleting that early-out entirely** — which is what a strict 1:1 port says, since Java's
  `StepInitMoving` has no such branch at all and `ParityRunner case INIT_MOVING` answers
  `sendMoveAction` whenever `activation != null`. Also measured **0/11**. Reverted. So the guard is
  load-bearing *for the Rust agent contract*: the Rust agent, when handed that move prompt, does not
  reduce to `MoveReplay`'s `END_PLAYER_ACTION` the way Java's does. Seed 75 is an AGENT gap, not an
  engine gap, and the engine early-out cannot be removed before the agent mirrors MoveReplay.

### Measurement notes

* Everything above was gated from a PRIVATE `CARGO_TARGET_DIR=C:/Users/Admin/niels/target-vamp`
  binary, with at most three concurrent streams and one `FFB_PARITY_ROOT` per stream. No phantom
  reds this iteration.
* `scripts/first_state_divergence.sh` now honours `FFB_PARITY_BIN` (one-line change) so it can be
  pointed at that private binary.
* `check_java_trees.py` reports `DIFFERS: .../heuristic/Sampler.java`. It is **not mine** — no Java
  was edited this iteration — and it is inert: the only difference is an env-gated `FFB_GRP` debug
  probe (`diff` of both files with comments stripped is exactly that 15-line block). The jar (14:53)
  is newer than that edit (13:38), so the gates ran with the probe compiled in and unset. Not
  `--fix`ed, to avoid clobbering the other session's work.

### Frontier / next step

1. **bb2025 `@1.0` seed 75** — root-caused, not fixed. `home_07` is PRONE, stands up (`cm=3`), fails
   Blood Lust with `hasMoved=false`; Java's harness answers the empty-stack `INIT_MOVING` prompt with
   a real path twice (`(20,14)->(20,13)->(25,8)`), Rust's engine early-out ends home's turn. Both
   candidate engine fixes measured 0/11 (above). The next concrete step is on the AGENT: make the
   heuristic's move handler reduce to `MoveReplay`'s verdict for a blood-lust activation, then delete
   the invented early-out and re-gate.
2. **bb2025 `@0` (seeds 8, 30, 50, 65, 71) and bb2020 `@0` (seeds 8, 76)** — still unclassified.
   Both editions' `@1.0`/`@1e6` are green, so these are argmax-specific declaration paths.
3. **BACKLOG E17** — give `sendConcreteAction` a `case GAZE:` so Hypnotic Gaze stops being a dead
   mechanic in every edition.
