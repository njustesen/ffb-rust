# Renegades — heuristic-agent parity campaign

Mirror matchup `--home renegades --away renegades`, tier 3, seeds 1-100, `--heur-classes all`.
Started 2026-09-06 on `a2744e62b` (the commit that closed orc), the next unstarted race in the
alphabetical sweep. **CLOSED the same day, ITER1**, after three defects.

## Surface — what renegades actually carries

| position | bb2016 | bb2020 | bb2025 |
|---|---|---|---|
| Human Lineman | — | — | Animosity |
| Human Thrower | Animosity, Pass | Animosity, Pass, **Safe Pair of Hands** | Animosity, Pass, Sure Hands |
| Goblin | Animosity, Dodge, Right Stuff, Stunty | same | same |
| Orc / Skaven / Dark Elf | Animosity | Animosity | Animosity |
| Troll | Always Hungry, Loner, Mighty Blow, Really Stupid, Regeneration, Throw Team-mate | + Projectile Vomit | + Projectile Vomit |
| Ogre | Bone Head, Loner, Mighty Blow, Thick Skull, Throw Team-mate | same | same |
| Minotaur | Frenzy, Horns, Loner, Mighty Blow, Thick Skull | + **Unchannelled Fury** | + Unchannelled Fury |
| Rat Ogre | Frenzy, Loner, Mighty Blow, Prehensile Tail | + **Animal Savagery** | + Animal Savagery |

The **bb2020 team alone fields a star**: jersey 11 is **Hakflem Skuttlespike** (Dodge, Extra Arms,
Loner 4, Prehensile Tail, **Treacherous**, Two Heads). That single roster line is the whole reason
bb2020 was the only red edition. The bb2020 team also has no Troll and no apothecary; bb2025 has no
Rat Ogre.

## Baseline (measured first, before any change)

| edition | @1.0 |
|---|---|
| bb2016 | **100/100** |
| bb2020 | **89/100** |
| bb2025 | **100/100** |

The eleven bb2020 reds were seeds 2, 19, 23, 38, 41, 50, 58, 60, 61, 62, 97 — and
`first_state_divergence.sh` showed **every one of them naming jersey 11**: eight resolved an
`Activate(*_11, Treacherous)` and three were declaration differences where Java activated `*11` and
Rust did not. One family, three defects.

**The briefing's historical hint was wrong for this agent.** The old RANDOM matrix recorded the
renegades frontier as "a bb2016 Throw-Team-Mate landing bug". On the current binary bb2016 was
green at baseline in all three scales, and TTM appeared in no divergence anywhere. Re-measuring
first is what kept that hint from costing an iteration.

## Defect 1 — the harness could not declare Treacherous for the heuristic (HARNESS gap)

`ParityRunner.sendStarSpecialDeclaration` is the heuristic path's translation of a star special
into the client's command pair. It had cases for BALEFUL_HEX, LOOK_INTO_MY_EYES, CATCH_OF_THE_DAY,
THEN_I_STARTED_BLASTIN, RAIDING_PARTY, BLACK_INK, AUTO_GAZE_ZOAT, WISDOM_OF_THE_WHITE_DWARF and
THROW_KEG — and **none for TREACHEROUS**. It therefore returned `false`, and the caller declared
`ClientCommandActingPlayer(pid, TREACHEROUS)` BARE. Java's `StepInitSelecting` never saw a
`CLIENT_USE_SKILL`, never set `fDispatchPlayerAction = TREACHEROUS`, and phase 2 deselected the
activation: **Java no-opped it** (seed 97 i=1, `post_hash == state_hash`) while Rust stabbed the
ball carrier and took the ball.

Fixed by adding a `case TREACHEROUS` that sends the pair the RANDOM path already sends
(`ParityRunner:960`): `ActingPlayer(pid, PASS_MOVE)` + `UseSkill(treacherous)`. **PASS_MOVE, not
the generic tail's MOVE** — Treacherous is the skill that ADDS the pass entries to a non-carrier's
menu, and `StepTreacherous.markActionUsed` keys `turnData.passUsed` off that declared action.
Exactly the same family as the BLACK_INK / WISDOM_OF_THE_WHITE_DWARF / THROW_KEG gaps already
recorded in that method.

Said plainly: this is a **harness** fix, not an engine fix. The Rust engine was right; the Java
harness could not express the client's command pair for this one action. The Rust-side gap it
exposed is filed under "Not verified" below.

## Defect 2 — the post-Treacherous PASS_MOVE park was answered by the wrong contract

With the declaration fixed, seed 97 still diverged at i=1: after the stab both engines held the
same board, but **Rust threw a quick pass** (`passRoll away_11` → `catchRoll away_07`) while
**Java walked the star seven squares** (22,6 → 15,2) carrying the stolen ball.

`ParityRunner.sendConcreteAction` splits on exactly this state:

```java
case PASS_MOVE: case HAND_OVER_MOVE:
    if (activation != null) { sendMoveAction(game, gameState, pid); break; }   // heuristic: MOVE
    sendBallAction(game, gameState, pid, pa);                                  // random: THROW
```

Rust's `StepInitSelecting` raises `AgentPrompt::BombRethrow` for that park (a Rust-side bridging
construct — Java's engine shows no dialog at all and simply waits for the client's next command),
and the heuristic agent had **no arm for it**: it fell through to `_ => self.parity.act(gs)`, i.e.
the RANDOM contract, and threw.

Two changes, one unit:

* `crates/ffb-engine/src/agent/heuristic_agent.rs` — a `BombRethrow` arm **gated on
  `acting_player.player_action == Some(PassMove)`** that answers with the agent's own move logic
  (`handle_move` / `handle_move_deep`) over `legal_move_targets` — the Rust engine's answer to the
  same question Java's `sendMoveAction` answers with `freeNeighbours`. The gate keeps the GENUINE
  bomb re-throw window untouched: its two producers
  (`step/mixed/pass/step_init_passing.rs`, `step/bb2016/pass/step_init_passing.rs`) raise the
  prompt only when the acting action `is_bomb()` or the turn mode is `Bomb*`, and there both agents
  do send the throw. The prompt also had to be added to `needs_features` / `needs_heavy`, or the
  arm is unreachable — `act()` routes every non-feature prompt to `act_boardless` first, and that
  is where the first attempt silently died.
* `crates/ffb-engine/src/step/bb2025/shared/step_init_selecting.rs` — an `Action::Move { path }`
  arm, 1:1 with Java's `StepInitSelecting:177` `CLIENT_MOVE`: publish `MOVE_START` + `MOVE_STACK`,
  set `fDispatchPlayerAction = MOVE`, and **do not rewrite the acting player's declared action** (a
  PASS_MOVE stays PASS_MOVE so its throw can still be fired later). There was no such arm at all,
  so an `Action::Move` answered here fell through to `execute_step` and re-raised the same prompt
  for ever. Mirrors the bb2016 twin at `step/bb2016/move_/step_init_selecting.rs:111`. Nothing else
  sends `Action::Move` to this step, so the arm is reachable only through defect 2's new agent arm.

Regression test: `client_move_dispatches_move_and_keeps_the_declared_pass_move`.

bb2020 @1.0 went 89 → **97/100**.

## Defect 3 — BB2020 raised a Safe Pair of Hands offer Java never raises (ENGINE)

The last three reds (38, 50, 58) were all the first activation after a turn flip, with **identical
candidate lists** (`JSUM`/`RSUM` both `n=1596` at seed 38 k=34) but different running draw totals —
Java 84, Rust 86. `FFB_DRAWS` named the two extra draws:
`RDRAW cls=skill total=84 skill=SafePairOfHands pid=home_05`, with no Java counterpart between
`JDRAW cls=RE_ROLL total=82` and the next Java dialog.

home_05 is the Renegade Human Thrower, who carries **Safe Pair of Hands in bb2020**. He failed a
GFI holding the ball. The two Java classes differ on one argument:

```java
bb2020/move/StepFallDown:88  dropPlayer(this, player, ATTACKER);        // 3-arg → false
bb2025/move/StepFallDown:88  dropPlayer(this, player, ATTACKER, true);  // 4-arg → true
```

`make_step_for` routes only bb2016 away from the shared (bb2025) `StepFallDown`, so **every bb2020
game ran the bb2025 rule**: `DROPPED_BALL_CARRIER` was published, `StepPlaceBall` showed the
dialog, and the heuristic's `useSkill` sampler spent two draws Java never spends — splitting the
two agents' RNG streams for the rest of the game while leaving the state hash identical (the offer
is pinned to DECLINE by both contracts, so the board never differed; only the sampler position
did).

Fixed by **edition-gating the one argument inside the shared step**
(`eligible_for_safe_pair_of_hands = game.rules == Rules::Bb2025`), not by routing the step to the
bb2020 twin — that file is staler (`drop_player` instead of `drop_player_rng`: no Ball & Chain
branch, no `PlayerFellDown` event), and routing whole steps to bb2020 twins is the mistake
`docs/PARITY_BB2020_CAMPAIGN.md` already records twice. Audited the rest of the mechanism:
`StepFallDown` is the **only** `dropPlayer` call site where the bb2020 and bb2025 classes disagree
on this argument (`grep dropPlayer( step/bb2020 step/bb2025`).

Regression test: `bb2020_falling_carrier_is_not_eligible_for_safe_pair_of_hands`, alongside the
existing bb2025 `falling_carrier_is_eligible_for_safe_pair_of_hands`.

bb2020 @1.0 went 97 → **100/100**.

## Conclusions of mine that turned out WRONG (recorded deliberately)

* *"Java's `StepTreacherous` found no target, so the difference must be `hasBall`'s
  `ballInPlay && !ballMoving` terms, or the `can_be_blocked()` filter, or
  `hasActedIgnoringNegativeTraits()` vs the stored `has_acted`."* All three ARE real 1:1 gaps in
  Rust's `find_treacherous_target` — and **none of them was the bug**. Java never reached the step
  at all. Left unchanged rather than "fixed" on a theory the evidence had disproved; filed below.
* *"`step/bb2020/step_treacherous.rs` is the live file for bb2020."* It is DEAD — `driver.rs:79`
  imports the **bb2025** `StepTreacherous` and there is no `make_step_for` arm for it. The probe I
  put in it never fired, which is the documented "silent probe = dead file" signal; I had to
  re-probe the bb2025 twin.
* *"My `JTREACH_DECL` probe will print."* It did not — I had put it on the RANDOM declaration path
  (`ParityRunner:960`), and the heuristic declares through `sendStarSpecialDeclaration`. That
  silence is what actually located the missing case, but it cost a jar rebuild.
* The briefing's bb2016 Throw-Team-Mate hint (see Baseline).

## Final gates — seeds 1-100, tier 3, `--agent heuristic --heur-classes all`

| edition | @1.0 | @0 | @1e6 |
|---|---|---|---|
| bb2016 | **100/100** | **100/100** | **100/100** |
| bb2020 | **100/100** | **100/100** | **100/100** |
| bb2025 | **100/100** | **100/100** | **100/100** |

Random controls (`FFB_PARITY_ROOT=parity_random --agent random`, seeds 1-100):
bb2016 **100/100**, bb2020 **100/100**, bb2025 **100/100**.

`cargo test -p ffb-engine`: **7426 passed, 0 failed**, 15 ignored.

Timing (batched JVM, 100 seeds): bb2020 @1.0 `rust_total=30.69s` (java 57.18s);
bb2025 @1e6 `rust_total=26.83s` (java 57.79s).

## Not verified / follow-ups

* No bb2016 or bb2025 Treacherous evidence exists: only the bb2020 team fields Hakflem, so
  `sendStarSpecialDeclaration`'s new case and the heuristic's `BombRethrow` arm are exercised by
  bb2020 alone. The bb2025 `Treacherous` generator and step are still only reachable through the
  star-drafting tier.
* `find_treacherous_target` (in the live `step/bb2025/step_treacherous.rs` and its dead bb2020
  twin) still hand-rolls the target search where Java calls
  `UtilPlayer.findAdjacentBlockablePlayers(game, actingTeam, coord)` filtered by
  `UtilPlayer.hasBall`: it omits `PlayerState::can_be_blocked()`, uses bare ball-coordinate
  equality instead of `ballInPlay && !ballMoving && coords equal`, and reads
  `acting_player.has_acted` where Java derives
  `!hasActedIgnoringNegativeTraits() || justStoodUp()`. No seed in this campaign distinguishes
  them. **Hygiene item — not fixed on a theory.**
* `step/bb2020/step_treacherous.rs` and `step/bb2020/move_/step_fall_down.rs` are both dead files
  that the driver never dispatches. They are now known-stale as well as dead.
