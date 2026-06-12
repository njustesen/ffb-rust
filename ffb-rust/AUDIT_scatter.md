# 1:1 audit — ball scatter / bounce / throw-in / catch (Java vs Rust)

Method: side-by-side behavioral review of the Rust scatter subsystem against the Java
owners, scoped by the divergence-cluster data (this subsystem owns the first divergence
of **43 of 96** failing tier-3 lineman seeds). BB2025, skill-less linemen. Java = ground
truth. This is the per-subsystem 1:1 review that replaces seed-by-seed grinding.

Java owners: `StepCatchScatterThrowIn.java` (bb2025/shared), `UtilServerCatchScatterThrowIn.java`,
`DiceRoller.java`, `DiceInterpreter`/`Direction` mapping, `bb2025/CatchModifierCollection.java`,
`bb2025/ThrowInMechanic.java`.
Rust: `crates/ffb-engine/src/engine/mod.rs` (`resolve_ball_landing`, `resolve_throw_in`,
`scatter_ball_if_carried`, inline knockdown copies), `crates/ffb-mechanics/src/mechanics/{scatter,throw_in}.rs`.

## Confirmed MATCHING (do not touch)
- d8→direction mapping (`Direction::for_roll` == Java `forRoll`), `scatter_coordinate`.
- Dice types (scatter d8, distance d6, throw-in dir d6 / corner d3, throw-in dist 2d6, catch d6).
- `is_skill_roll_successful` (6 success, 1 fail). Pitch bounds.
- Failed-pickup bounce: one d8 + resolve (matches). Kickoff scatter lastValid + touchback test.

## DISCREPANCIES (fix in this order)

1. **Knocked-down ball carrier: bounce d8 rolled but ball never moved/resolved.** HIGH.
   `scatter_ball_if_carried` (engine/mod.rs:12034) + inline copies (~:5550, ~:6307) roll the
   d8 and emit BallScattered but never set `ball_coordinate` or call resolve_ball_landing/
   resolve_throw_in. Java `UtilServerInjury.dropPlayer` → SCATTER_BALL → `bounceBall` moves
   one square and resolves. FIX: route all three sites through one helper that bounces AND
   resolves (mirror the failed-pickup path at ~:7848). (StepPlaceBall is a no-op for linemen.)

2. **Corner throw-in uses d6 + sideline table instead of d3 + corner table.** HIGH.
   `resolve_throw_in` (:10977) always rolls d6 + `throw_in_direction_for_roll`. Java
   `throwInBall` uses `isCornerThrowIn` → d3 + corner table. Helpers already exist & are
   tested in `mechanics/throw_in.rs` (`is_corner_square`, `corner_direction`,
   `corner_throw_in_direction_for_roll`) — just never called.

3. **Throw-in distance off-by-one.** HIGH. Java moves `distance-1` squares from the in-bounds
   start (`for i in 0..distance: findScatterCoordinate(start,dir,i)`); Rust moves full
   `distance` (:11003). Walk square-by-square tracking lastValid; re-throw if it exits again.

4. **Throw-in start uses off-pitch coords for the edge/corner test.** MEDIUM. Derive the
   in-bounds (clamped lastValid) start FIRST and feed it to both the corner test and the
   direction function (Java uses lastValid). Same root cause as #2.

5. **resolve_ball_landing missing BB2025 CATCH_SCATTER +1.** MEDIUM. The ball arriving here
   is always via scatter/bounce/throw-in = CATCH_SCATTER → Java adds +1 to the catch target
   (CatchModifierCollection). Rust catch_min (:10767) has no +1, so AG3 lineman catches at 3+
   not 4+. Add `+1` for `rules == Bb2025`. (Kickoff path at :770 already has it — scope the
   fix to resolve_ball_landing only.)

6. **Failed bounce/throw-in catch never offers a team reroll.** MEDIUM. Java `catchBall`
   calls askForReRollIfAvailable on failure; Rust resolve_ball_landing only auto-uses the
   Catch *skill* (linemen lack it) and bounces on. Needs resolve_ball_landing made resumable
   (PendingReroll) — most invasive; do last.

Reachability note: tier-2 lineman logs contain zero ThrowIn/BallScattered — all latent until
tier-3. Priority: 1 and 5 first (most reachable), then 3, then 2/4, then 6.
