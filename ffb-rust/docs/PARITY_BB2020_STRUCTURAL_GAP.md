# BB2020 structural gap — routing roadmap

Generated inventory for the CURRENT campaign goal: give BB2020 its real step set instead
of running the shared BB2025 one with per-difference gates.

`make_step_for(id, rules)` has exactly ONE `Rules::Bb2020` arm (`StepId::Prayer`), so every
other StepId in a BB2020 game runs the shared BB2025 step. **All 79 BB2020/BB2025 Java step
pairs genuinely differ — there is not a single identical twin**, so every row below is a real
candidate, ordered most-different first.

Columns: `sim` = similarity of the two JAVA files (the ground truth; Rust-vs-Rust similarity
is NOT a proxy, because a dead bb2020 Rust file can be staler than the shared one).
`rust port` = whether a bb2020 Rust file already exists to route at.

## Method per candidate

1. Diff the two **Java** files and name the actual behavioural difference.
2. Verify the Rust `step/bb2020/` file is a faithful port of BB2020 Java — dead files drift,
   so do not assume. Fix it first if stale.
3. Route it (a `make_step_for` arm, or an edition gate in the shared generator when the
   difference is in the SEQUENCE rather than the step).
4. Gate on the full 30-roster BB2020 matrix. Revert on any roster below 100/100.

Hard-won constraint: route **individual StepIds**, never a whole sequence wholesale —
swapping the bb2020 sequence in regressed goblin 96 -> 10/100.

| sim | Java step | Rust bb2020 port |
|-----|-----------|------------------|
| 0.24 | `StepStallingPlayer.java` | `step_stalling_player.rs` |
| 0.43 | `StepApothecaryMultiple.java` | `step_apothecary_multiple.rs` |
| 0.52 | `StepBlockRoll.java` | `step_block_roll.rs` |
| 0.57 | `StepApothecary.java` | `step_apothecary.rs` |
| 0.60 | `StepInitScatterPlayer.java` | `step_init_scatter_player.rs` |
| 0.64 | `StepPrayers.java` | `step_prayers.rs` |
| 0.64 | `StepBlockRollMultiple.java` | `step_block_roll_multiple.rs` |
| 0.70 | `StepApplyKickoffResult.java` | `step_apply_kickoff_result.rs` |
| 0.74 | `StepMoveDodge.java` | `step_move_dodge.rs` |
| 0.80 | `StepJump.java` | `step_jump.rs` |
| 0.81 | `StepKickoffScatterRoll.java` | `step_kickoff_scatter_roll.rs` |
| 0.81 | `StepEndMoving.java` | `step_end_moving.rs` |
| 0.82 | `StepMultipleBlockFork.java` | `step_multiple_block_fork.rs` |
| 0.82 | `StepWisdomOfTheWhiteDwarf.java` | `step_wisdom_of_the_white_dwarf.rs` |
| 0.82 | `StepPickUp.java` | `step_pick_up.rs` |
| 0.82 | `StepEndThrowTeamMate.java` | `step_end_throw_team_mate.rs` |
| 0.84 | `StepInitInducement.java` | `step_init_inducement.rs` |
| 0.85 | `StepInitBomb.java` | `step_init_bomb.rs` |
| 0.85 | `StepRightStuff.java` | `step_right_stuff.rs` |
| 0.87 | `StepBlitzTurn.java` | `step_blitz_turn.rs` |
| 0.87 | `StepDispatchScatterPlayer.java` | `step_dispatch_scatter_player.rs` |
| 0.88 | `StepSetup.java` | `step_setup.rs` |
| 0.88 | `StepThenIStartedBlastin.java` | `step_then_i_started_blastin.rs` |
| 0.89 | `StepEndFeeding.java` | `step_end_feeding.rs` |
| 0.89 | `StepEndInducement.java` | `step_end_inducement.rs` |
| 0.89 | `StepWinnings.java` | `step_winnings.rs` |
| 0.89 | `StepPass.java` | `step_pass.rs` |
| 0.90 | `StepSpecialEffect.java` | `step_special_effect.rs` |
| 0.91 | `StepEndTurn.java` | `step_end_turn.rs` |
| 0.91 | `StepEndFuriousOutburst.java` | `step_end_furious_outburst.rs` |
| 0.91 | `StepEndFouling.java` | `step_end_fouling.rs` |
| 0.91 | `StepInitActivation.java` | `step_init_activation.rs` |
| 0.91 | `StepInitMoving.java` | `step_init_moving.rs` |
| 0.91 | `StepEndScatterPlayer.java` | `step_end_scatter_player.rs` |
| 0.92 | `StepFollowup.java` | `step_followup.rs` |
| 0.92 | `StepResolvePass.java` | `step_resolve_pass.rs` |
| 0.93 | `StepTrickster.java` | `step_trickster.rs` |
| 0.93 | `StepInitSelecting.java` | `step_init_selecting.rs` |
| 0.93 | `StepHypnoticGaze.java` | `step_hypnotic_gaze.rs` |
| 0.94 | `StepPlayerLoss.java` | `step_player_loss.rs` |
| 0.94 | `StepBreatheFire.java` | `step_breathe_fire.rs` |
| 0.94 | `StepGoForIt.java` | `step_go_for_it.rs` |
| 0.94 | `StepEndSelecting.java` | `step_end_selecting.rs` |
| 0.94 | `StepEndBlocking.java` | `step_end_blocking.rs` |
| 0.94 | `StepMissedPass.java` | `step_missed_pass.rs` |
| 0.95 | `StepSelectBlitzTarget.java` | `step_select_blitz_target.rs` |
| 0.95 | `StepInitEndGame.java` | `step_init_end_game.rs` |
| 0.95 | `StepHailMaryPass.java` | `step_hail_mary_pass.rs` |
| 0.95 | `StepBlockChainsaw.java` | `step_block_chainsaw.rs` |
| 0.95 | `StepEndPassing.java` | `step_end_passing.rs` |
| 0.95 | `StepInitFeeding.java` | `step_init_feeding.rs` |
| 0.96 | `StepKickoffResultRoll.java` | `step_kickoff_result_roll.rs` |
| 0.96 | `StepStandUp.java` | `step_stand_up.rs` |
| 0.96 | `StepTreacherous.java` | `step_treacherous.rs` |
| 0.96 | `StepBlackInk.java` | `step_black_ink.rs` |
| 0.96 | `StepPlaceBall.java` | `step_place_ball.rs` |
| 0.97 | `StepCatchOfTheDay.java` | `step_catch_of_the_day.rs` |
| 0.97 | `StepBalefulHex.java` | `step_baleful_hex.rs` |
| 0.97 | `StepInitBlocking.java` | `step_init_blocking.rs` |
| 0.97 | `StepPrayer.java` | `step_prayer.rs` |
| 0.97 | `StepFallDown.java` | `step_fall_down.rs` |
| 0.97 | `StepTakeRoot.java` | `step_take_root.rs` |
| 0.97 | `StepThrowTeamMate.java` | `step_throw_team_mate.rs` |
| 0.97 | `StepBloodLust.java` | `step_blood_lust.rs` |
| 0.97 | `StepShadowing.java` | `step_shadowing.rs` |
| 0.98 | `StepPushback.java` | `step_pushback.rs` |
| 0.98 | `StepIntercept.java` | `step_intercept.rs` |
| 0.98 | `StepMove.java` | `step_move.rs` |
| 0.99 | `StepAlwaysHungry.java` | `step_always_hungry.rs` |
| 0.99 | `StepHitAndRun.java` | `step_hit_and_run.rs` |
| 0.99 | `StepBribes.java` | `step_bribes.rs` |
| 0.99 | `StepLookIntoMyEyes.java` | `step_look_into_my_eyes.rs` |
| 0.99 | `StepInitFouling.java` | `step_init_fouling.rs` |
| 0.99 | `StepCatchScatterThrowIn.java` | `step_catch_scatter_throw_in.rs` |
| 0.99 | `StepMvp.java` | `step_mvp.rs` |
| 0.99 | `StepBlockChoice.java` | `step_block_choice.rs` |
| 0.99 | `StepInitThrowTeamMate.java` | `step_init_throw_team_mate.rs` |
| 0.99 | `StepRaidingParty.java` | `step_raiding_party.rs` |
| 0.99 | `StepHandleDropPlayerContext.java` | `step_handle_drop_player_context.rs` |

79 pairs, all differing. Rust bb2020 ports present: 79.

## Java bb2020-only steps (no BB2025 twin)

These 9 have no shared counterpart and are ALREADY live for every edition via the driver's
edition-independent arms — they are the shared implementation for their StepId, not BB2020
specialisations: StepAssignTouchdowns, StepBuyCardsAndInducements, StepCheckStalling,
StepReportStabInjury, StepSelectGazeTarget, StepSelectGazeTargetEnd,
StepSetActingPlayerAndTeam, StepSetActingTeam, StepStateMultipleRolls.
