/// BB2025 blitz-block step sequence.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.bb2025.BlitzBlock`.
/// Like Block but NO activation block, with leading GFI + HORNS, and richer
/// POW/PUSHBACK tail with REMOVE_TARGET_SELECTION_STATE.
use ffb_model::enums::{ApothecaryMode, Rules};
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::sequence::{Sequence, SequenceStep, labels};

/// Parameters for the BlitzBlock sequence — mirrors Java `BlitzBlock.SequenceParams`.
/// Same weapon flags as Block; no activation / no bloodlust.
#[derive(Debug, Clone)]
pub struct BlitzBlockParams {
    pub block_defender_id: Option<String>,
    pub multi_block_defender_id: Option<String>,
    pub using_stab: bool,
    pub using_chainsaw: bool,
    pub using_vomit: bool,
    pub using_breathe_fire: bool,
    pub using_chomp: bool,
    pub ask_for_block_kind: bool,
    pub publish_defender: bool,
    /// Ruleset this sequence is being built for.
    ///
    /// The shared BB2025 step-set runs BB2020 games too, but the two editions' BlitzBlock
    /// generators are NOT identical: `bb2025/BlitzBlock.java:43` adds PICK_UP between TRICKSTER
    /// and CATCH_SCATTER_THROW_IN and `bb2020/BlitzBlock.java` does not (BB2020's only PICK_UP is
    /// the one after SHADOWING, in the pushback branch). Building the whole BB2020 sequence here
    /// regresses badly — the shared steps depend on the BB2025 shape elsewhere — so, following the
    /// precedent set for `StepApplyKickoffResult` in `driver.rs`, edition-gate the one entry that
    /// differs on the measured path.
    pub rules: Rules,
}

impl Default for BlitzBlockParams {
    fn default() -> Self {
        Self {
            block_defender_id: None,
            multi_block_defender_id: None,
            using_stab: false,
            using_chainsaw: false,
            using_vomit: false,
            using_breathe_fire: false,
            using_chomp: false,
            ask_for_block_kind: false,
            publish_defender: false,
            rules: Rules::Bb2025,
        }
    }
}

pub struct BlitzBlock;

impl BlitzBlock {
    pub fn new() -> Self { Self }

    /// Build the blitz-block step sequence (Java `pushSequence`).
    pub fn build_sequence(params: &BlitzBlockParams) -> Vec<SequenceStep> {
        let mut seq = Sequence::new();
        let fl = labels::END_BLOCKING;

        // 1 INIT_BLOCKING
        let mut init_params = vec![
            StepParameter::GotoLabelOnEnd(fl.into()),
            StepParameter::UsingStab(params.using_stab),
            StepParameter::UsingChainsaw(params.using_chainsaw),
            StepParameter::UsingVomit(params.using_vomit),
            StepParameter::UsingBreatheFire(params.using_breathe_fire),
            StepParameter::UsingChomp(params.using_chomp),
            StepParameter::AskForBlockKind(params.ask_for_block_kind),
            StepParameter::PublishDefender(params.publish_defender),
        ];
        if let Some(ref id) = params.block_defender_id {
            init_params.push(StepParameter::BlockDefenderId(id.clone()));
        }
        if let Some(ref id) = params.multi_block_defender_id {
            init_params.push(StepParameter::MultiBlockDefenderId(Some(id.clone())));
        }
        seq.add(StepId::InitBlocking, init_params);

        // 2 GO_FOR_IT (blitz move GFI — goto STEADY_FOOTING on failure)
        seq.add(StepId::GoForIt, vec![
            StepParameter::GotoLabelOnFailure(labels::STEADY_FOOTING.into()),
        ]);
        // 3 STEADY_FOOTING [STEADY_FOOTING] (goto FALL_DOWN on failure)
        seq.add_labelled(StepId::SteadyFooting, labels::STEADY_FOOTING, vec![
            StepParameter::GotoLabelOnFailure(labels::FALL_DOWN.into()),
        ]);
        // 4 FOUL_APPEARANCE — BB2025 only on the primary blitz; see `BlitzBlockParams::rules`.
        //
        // `bb2020/BlitzBlock.java:31-33` adds it here ONLY for a frenzy follow-up block; the
        // primary blitz rolls it a phase earlier, in `bb2020/SelectBlitzTarget.java:35` (mirrored
        // by the SELECT sequence's `in_select` copy). The frenzy re-block sites in
        // `step_end_blocking.rs` build with the default (BB2025) params, so they keep this entry —
        // which is exactly the frenzyBlock branch BB2020 wants.
        if params.rules != Rules::Bb2020 {
            seq.add(StepId::FoulAppearance, vec![
                StepParameter::GotoLabelOnFailure(fl.into()),
            ]);
        }
        // 5 DUMP_OFF
        seq.add(StepId::DumpOff, vec![]);
        // 6 BLOCK_STATISTICS
        seq.add(StepId::BlockStatistics, vec![]);
        // 7 DAUNTLESS
        seq.add(StepId::Dauntless, vec![]);
        // 8 HORNS (bonus block die for blitz)
        seq.add(StepId::Horns, vec![]);
        // 9 TRICKSTER
        seq.add(StepId::Trickster, vec![]);
        // 10 PICK_UP — BB2025 only; see `BlitzBlockParams::rules`.
        if params.rules != Rules::Bb2020 {
            seq.add(StepId::PickUp, vec![
                StepParameter::GotoLabelOnFailure(labels::DROP_FALLING_PLAYERS.into()),
            ]);
        }
        // 11 CATCH_SCATTER_THROW_IN
        seq.add(StepId::CatchScatterThrowIn, vec![]);
        // 12 STAB
        seq.add(StepId::Stab, vec![
            StepParameter::GotoLabelOnSuccess(labels::DEFENDER_DROPPED.into()),
        ]);
        // 13 BLOCK_CHAINSAW
        seq.add(StepId::BlockChainsaw, vec![
            StepParameter::GotoLabelOnSuccess(labels::DEFENDER_DROPPED.into()),
            StepParameter::GotoLabelOnFailure(labels::ATTACKER_DROPPED.into()),
        ]);
        // 14 STEADY_FOOTING (chainsaw attacker — goto END on success)
        seq.add(StepId::SteadyFooting, vec![
            StepParameter::GotoLabelOnSuccess(fl.into()),
        ]);
        // 15 PROJECTILE_VOMIT
        seq.add(StepId::ProjectileVomit, vec![
            StepParameter::GotoLabelOnSuccess(labels::DEFENDER_DROPPED.into()),
            StepParameter::GotoLabelOnFailure(labels::ATTACKER_DROPPED.into()),
        ]);
        // 16 BREATHE_FIRE
        seq.add(StepId::BreatheFire, vec![
            StepParameter::GotoLabelOnSuccess(labels::DEFENDER_DROPPED.into()),
            StepParameter::GotoLabelOnFailure(labels::ATTACKER_DROPPED.into()),
            StepParameter::GotoLabelOnEnd(fl.into()),
        ]);
        // 17 STEADY_FOOTING (attacker — goto END on success)
        seq.add(StepId::SteadyFooting, vec![
            StepParameter::ApothecaryMode(ApothecaryMode::Attacker),
            StepParameter::GotoLabelOnSuccess(fl.into()),
        ]);
        // 18 STEADY_FOOTING (defender — goto END on success)
        seq.add(StepId::SteadyFooting, vec![
            StepParameter::ApothecaryMode(ApothecaryMode::Defender),
            StepParameter::GotoLabelOnSuccess(fl.into()),
        ]);
        // 19 HANDLE_DROP_PLAYER_CONTEXT
        seq.add(StepId::HandleDropPlayerContext, vec![]);
        // 20 CHOMP
        seq.add(StepId::Chomp, vec![
            StepParameter::GotoLabelOnEnd(fl.into()),
        ]);
        // 21 BLOCK_ROLL
        seq.add(StepId::BlockRoll, vec![]);
        // 22 BLOCK_CHOICE
        seq.add(StepId::BlockChoice, vec![
            StepParameter::GotoLabelOnDodge(labels::DODGE_BLOCK.into()),
            StepParameter::GotoLabelOnJuggernaut(labels::JUGGERNAUT.into()),
            StepParameter::GotoLabelOnPushback(labels::PUSHBACK.into()),
        ]);
        // 23 GOTO_LABEL → DROP_FALLING_PLAYERS
        seq.jump(labels::DROP_FALLING_PLAYERS);
        // 24 JUGGERNAUT [JUGGERNAUT]
        seq.add_labelled(StepId::Juggernaut, labels::JUGGERNAUT, vec![
            StepParameter::GotoLabelOnSuccess(labels::PUSHBACK.into()),
        ]);
        // 25 BOTH_DOWN
        seq.add(StepId::BothDown, vec![]);
        // 26 WRESTLE
        seq.add(StepId::Wrestle, vec![]);
        // 27 GOTO_LABEL → DROP_FALLING_PLAYERS
        seq.jump(labels::DROP_FALLING_PLAYERS);
        // 28 BLOCK_DODGE [DODGE_BLOCK]
        seq.add_labelled(StepId::BlockDodge, labels::DODGE_BLOCK, vec![]);
        // 29 PUSHBACK [PUSHBACK]
        seq.add_labelled(StepId::Pushback, labels::PUSHBACK, vec![]);
        // 30 REMOVE_TARGET_SELECTION_STATE (retain)
        seq.add(StepId::RemoveTargetSelectionState, vec![
            StepParameter::Retain(true),
        ]);
        // 31 APOTHECARY (crowd push)
        seq.add(StepId::Apothecary, vec![
            StepParameter::ApothecaryMode(ApothecaryMode::CrowdPush),
        ]);
        // 32 FOLLOWUP
        seq.add(StepId::Followup, vec![]);
        // 33 PICK_UP (second pick-up — for ball dropped during followup)
        seq.add(StepId::PickUp, vec![
            StepParameter::GotoLabelOnFailure(labels::DROP_FALLING_PLAYERS.into()),
        ]);
        // 34 GOTO_LABEL → DROP_FALLING_PLAYERS
        seq.jump(labels::DROP_FALLING_PLAYERS);
        // 35 FALL_DOWN [FALL_DOWN] (reached via GOTO FALL_DOWN when blitz GFI fails)
        seq.add_labelled(StepId::FallDown, labels::FALL_DOWN, vec![]);
        // 36 GOTO_LABEL → ATTACKER_DROPPED
        seq.jump(labels::ATTACKER_DROPPED);
        // 37 DROP_FALLING_PLAYERS [DROP_FALLING_PLAYERS]
        seq.add_labelled(StepId::DropFallingPlayers, labels::DROP_FALLING_PLAYERS, vec![]);
        // 38 STEADY_FOOTING (defender)
        seq.add(StepId::SteadyFooting, vec![
            StepParameter::ApothecaryMode(ApothecaryMode::Defender),
        ]);
        // 39 HANDLE_DROP_PLAYER_CONTEXT
        seq.add(StepId::HandleDropPlayerContext, vec![]);
        // 40 REMOVE_TARGET_SELECTION_STATE (retain)
        seq.add(StepId::RemoveTargetSelectionState, vec![
            StepParameter::Retain(true),
        ]);
        // 41 RESET_FUMBLEROOSKIE (failed_block)
        seq.add(StepId::ResetFumblerooskie, vec![
            StepParameter::ResetForFailedBlock(true),
        ]);
        // 42 PLACE_BALL [DEFENDER_DROPPED]
        seq.add_labelled(StepId::PlaceBall, labels::DEFENDER_DROPPED, vec![]);
        // 43 APOTHECARY (defender)
        seq.add(StepId::Apothecary, vec![
            StepParameter::ApothecaryMode(ApothecaryMode::Defender),
        ]);
        // 43b GO_FOR_IT (ball-and-chain GFI) — BB2020 ONLY.
        //
        // `bb2020/BlitzBlock.java:89-90` places a SECOND `GO_FOR_IT` here, immediately after
        // `APOTHECARY(DEFENDER)` and before the ATTACKER_DROPPED path, carrying
        // `BALL_AND_CHAIN_GFI = true` ("GFI for ball & chain should go here"). BB2025's
        // `BlitzBlock` has no such step. `StepGoForIt.executeStep` computes
        // `runGfi = player.hasSkillProperty(goForItAfterBlock) == ballAndChainGfi`, so for a
        // Ball-and-Chain blitzer ONLY this copy runs — and it is the copy that charges the blitz
        // its movement point (`PlayerAction.BLITZ` → `setBlitzUsed(true)` and
        // `setCurrentMove(getCurrentMove() + 1)`). Without it a BB2020 Ball-and-Chain blitzer
        // resumed its post-block move with `currentMove` one too LOW, so the agent's reach budget
        // (`cap = ma + 2 - currentMove`) was one square too generous.
        if params.rules == Rules::Bb2020 {
            seq.add(StepId::GoForIt, vec![
                StepParameter::BallAndChainGfi(true),
                StepParameter::GotoLabelOnFailure(labels::DROP_FALLING_PLAYERS.into()),
            ]);
        }
        // 44 STEADY_FOOTING [ATTACKER_DROPPED] (attacker)
        seq.add_labelled(StepId::SteadyFooting, labels::ATTACKER_DROPPED, vec![
            StepParameter::ApothecaryMode(ApothecaryMode::Attacker),
        ]);
        // 45 PLACE_BALL [ATTACKER_DROPPED]
        seq.add_labelled(StepId::PlaceBall, labels::ATTACKER_DROPPED, vec![]);
        // 46 APOTHECARY (attacker)
        seq.add(StepId::Apothecary, vec![
            StepParameter::ApothecaryMode(ApothecaryMode::Attacker),
        ]);
        // 47 TRAP_DOOR [SCATTER_BALL]
        seq.add_labelled(StepId::TrapDoor, labels::SCATTER_BALL, vec![]);
        // 48 APOTHECARY (trap door)
        seq.add(StepId::Apothecary, vec![
            StepParameter::ApothecaryMode(ApothecaryMode::TrapDoor),
        ]);
        // 49 CATCH_SCATTER_THROW_IN
        seq.add(StepId::CatchScatterThrowIn, vec![]);
        // 50 REMOVE_TARGET_SELECTION_STATE [END_BLOCKING] (retain)
        seq.add_labelled(StepId::RemoveTargetSelectionState, fl, vec![
            StepParameter::Retain(true),
        ]);
        // 51 END_BLOCKING
        seq.add(StepId::EndBlocking, vec![]);

        seq.build()
    }
}

impl Default for BlitzBlock {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blitz_block_starts_with_init_blocking() {
        let steps = BlitzBlock::build_sequence(&BlitzBlockParams::default());
        assert_eq!(steps[0].step_id, StepId::InitBlocking);
    }

    #[test]
    fn blitz_block_ends_with_end_blocking() {
        let steps = BlitzBlock::build_sequence(&BlitzBlockParams::default());
        assert_eq!(steps.last().unwrap().step_id, StepId::EndBlocking);
    }

    #[test]
    fn blitz_block_remove_target_selection_state_is_labelled_end_blocking() {
        let steps = BlitzBlock::build_sequence(&BlitzBlockParams::default());
        let rts = steps.iter().find(|s| {
            s.step_id == StepId::RemoveTargetSelectionState && s.label.as_deref() == Some(labels::END_BLOCKING)
        });
        assert!(rts.is_some());
    }

    #[test]
    fn blitz_block_has_no_activation_block() {
        let steps = BlitzBlock::build_sequence(&BlitzBlockParams::default());
        assert!(!steps.iter().any(|s| s.step_id == StepId::InitActivation));
    }

    #[test]
    fn blitz_block_has_gfi_before_foul_appearance() {
        let steps = BlitzBlock::build_sequence(&BlitzBlockParams::default());
        let gfi_idx = steps.iter().position(|s| s.step_id == StepId::GoForIt).unwrap();
        let fa_idx = steps.iter().position(|s| s.step_id == StepId::FoulAppearance).unwrap();
        assert!(gfi_idx < fa_idx);
    }

    /// From `bb2020/BlitzBlock.java:89-90` vs `bb2025/BlitzBlock.java`:
    ///
    /// ```java
    /// // GFI for ball & chain should go here.
    /// sequence.add(StepId.GO_FOR_IT, from(StepParameterKey.GOTO_LABEL_ON_FAILURE, IStepLabel.DROP_FALLING_PLAYERS),
    ///     from(StepParameterKey.BALL_AND_CHAIN_GFI, true));
    /// ```
    ///
    /// BB2020 has TWO `GO_FOR_IT` steps in the blitz-block sequence — the leading one and this
    /// ball-and-chain copy, which sits immediately after `APOTHECARY(DEFENDER)` (the step that
    /// follows `PLACE_BALL [DEFENDER_DROPPED]`). BB2025 has exactly ONE.
    #[test]
    fn bb2020_has_ball_and_chain_gfi_after_defender_apothecary_and_bb2025_does_not() {
        let bb2020 = BlitzBlock::build_sequence(&BlitzBlockParams {
            rules: Rules::Bb2020, ..Default::default()
        });
        let bb2025 = BlitzBlock::build_sequence(&BlitzBlockParams {
            rules: Rules::Bb2025, ..Default::default()
        });

        let bc = |steps: &[SequenceStep]| -> usize {
            steps.iter().filter(|s| {
                s.step_id == StepId::GoForIt
                    && s.params.iter().any(|p| matches!(p, StepParameter::BallAndChainGfi(true)))
            }).count()
        };
        assert_eq!(bc(&bb2020), 1, "bb2020 carries the ball-and-chain GFI");
        assert_eq!(bc(&bb2025), 0, "bb2025 has no ball-and-chain GFI");
        assert_eq!(
            bb2020.iter().filter(|s| s.step_id == StepId::GoForIt).count(), 2,
            "bb2020 BlitzBlock has exactly two GO_FOR_IT steps"
        );
        assert_eq!(
            bb2025.iter().filter(|s| s.step_id == StepId::GoForIt).count(), 1,
            "bb2025 BlitzBlock has exactly one GO_FOR_IT step"
        );

        // Placement: directly after the APOTHECARY(DEFENDER) that follows PLACE_BALL
        // [DEFENDER_DROPPED], and before PLACE_BALL [ATTACKER_DROPPED].
        let place_def = bb2020.iter()
            .position(|s| s.step_id == StepId::PlaceBall && s.label.as_deref() == Some(labels::DEFENDER_DROPPED))
            .expect("PLACE_BALL [DEFENDER_DROPPED]");
        let gfi = bb2020.iter().rposition(|s| s.step_id == StepId::GoForIt).unwrap();
        let place_att = bb2020.iter()
            .position(|s| s.step_id == StepId::PlaceBall && s.label.as_deref() == Some(labels::ATTACKER_DROPPED))
            .expect("PLACE_BALL [ATTACKER_DROPPED]");
        assert_eq!(bb2020[place_def + 1].step_id, StepId::Apothecary);
        assert_eq!(gfi, place_def + 2, "the B&C GFI follows APOTHECARY(DEFENDER)");
        assert!(gfi < place_att);

        // GOTO_LABEL_ON_FAILURE = DROP_FALLING_PLAYERS.
        assert!(bb2020[gfi].params.iter().any(|p| matches!(
            p, StepParameter::GotoLabelOnFailure(l) if l == labels::DROP_FALLING_PLAYERS
        )));
    }

    #[test]
    fn blitz_block_has_horns() {
        let steps = BlitzBlock::build_sequence(&BlitzBlockParams::default());
        assert!(steps.iter().any(|s| s.step_id == StepId::Horns));
    }

    #[test]
    fn blitz_block_steady_footing_labelled_steady_footing_before_foul_appearance() {
        let steps = BlitzBlock::build_sequence(&BlitzBlockParams::default());
        let sf = steps.iter().find(|s| {
            s.step_id == StepId::SteadyFooting && s.label.as_deref() == Some(labels::STEADY_FOOTING)
        }).unwrap();
        let fa_idx = steps.iter().position(|s| s.step_id == StepId::FoulAppearance).unwrap();
        let sf_idx = steps.iter().position(|s| std::ptr::eq(s as *const _, sf as *const _)).unwrap();
        assert!(sf_idx < fa_idx);
    }

    #[test]
    fn blitz_block_fall_down_is_labelled_fall_down() {
        let steps = BlitzBlock::build_sequence(&BlitzBlockParams::default());
        let fd = steps.iter().find(|s| s.step_id == StepId::FallDown).unwrap();
        assert_eq!(fd.label.as_deref(), Some(labels::FALL_DOWN));
    }

    #[test]
    fn blitz_block_drop_falling_players_is_labelled() {
        let steps = BlitzBlock::build_sequence(&BlitzBlockParams::default());
        let dfp = steps.iter().find(|s| {
            s.step_id == StepId::DropFallingPlayers && s.label.as_deref() == Some(labels::DROP_FALLING_PLAYERS)
        });
        assert!(dfp.is_some());
    }

    #[test]
    fn blitz_block_has_51_steps() {
        let steps = BlitzBlock::build_sequence(&BlitzBlockParams::default());
        assert_eq!(steps.len(), 51);
    }

    /// `bb2025/BlitzBlock.java:43` runs PICK_UP between TRICKSTER and CATCH_SCATTER_THROW_IN;
    /// `bb2020/BlitzBlock.java` has no PICK_UP there at all (its only one is after SHADOWING, in
    /// the pushback branch). Getting this wrong made a BB2020 goblin Fanatic attempt a No-Hands
    /// pickup off a loose-ball square and BOUNCE the ball — one d8 Java never rolls (goblin
    /// bb2020 seed 85 i=4: the shifted block die turned Java's push into a both-down turnover).
    #[test]
    fn pick_up_before_catch_scatter_is_bb2025_only() {
        let pos = |rules: Rules| {
            let steps = BlitzBlock::build_sequence(&BlitzBlockParams { rules, ..Default::default() });
            let trickster = steps.iter().position(|s| s.step_id == StepId::Trickster).unwrap();
            let catch = steps.iter()
                .position(|s| s.step_id == StepId::CatchScatterThrowIn && s.label.is_none())
                .unwrap();
            steps[trickster..catch].iter().filter(|s| s.step_id == StepId::PickUp).count()
        };
        assert_eq!(pos(Rules::Bb2025), 1, "bb2025 picks up between TRICKSTER and CATCH_SCATTER");
        assert_eq!(pos(Rules::Bb2020), 0, "bb2020 has no PICK_UP between TRICKSTER and CATCH_SCATTER");

        // The pushback-branch PICK_UP after FOLLOWUP — which BB2020 has too — stays in both, and
        // nothing else about the sequence moves.
        for rules in [Rules::Bb2025, Rules::Bb2020] {
            let steps = BlitzBlock::build_sequence(&BlitzBlockParams { rules, ..Default::default() });
            let followup = steps.iter().position(|s| s.step_id == StepId::Followup).unwrap();
            assert_eq!(steps[followup + 1].step_id, StepId::PickUp, "{rules:?}");
        }
        let bb2025 = BlitzBlock::build_sequence(&BlitzBlockParams::default());
        let bb2020 = BlitzBlock::build_sequence(&BlitzBlockParams { rules: Rules::Bb2020, ..Default::default() });
        // Three entries are edition-gated: this PICK_UP and the FOUL_APPEARANCE below are BB2025
        // only, and the ball-and-chain GO_FOR_IT (`bb2020/BlitzBlock.java:89-90`) is BB2020 only.
        assert_eq!(bb2020.len(), bb2025.len() - 2 + 1);
    }

    /// `bb2020/BlitzBlock.java:31-33` adds FOUL_APPEARANCE here only for a frenzy follow-up; the
    /// primary blitz rolls it a phase earlier, in `bb2020/SelectBlitzTarget.java:35`. Building it
    /// in both places would roll twice and desync the whole dice stream.
    #[test]
    fn foul_appearance_here_is_bb2025_only() {
        let has = |rules: Rules| BlitzBlock::build_sequence(&BlitzBlockParams { rules, ..Default::default() })
            .iter().any(|s| s.step_id == StepId::FoulAppearance);
        assert!(has(Rules::Bb2025));
        assert!(!has(Rules::Bb2020));
    }

    #[test]
    fn blitz_block_reset_fumblerooskie_has_failed_block_param() {
        let steps = BlitzBlock::build_sequence(&BlitzBlockParams::default());
        let rf = steps.iter().find(|s| s.step_id == StepId::ResetFumblerooskie).unwrap();
        assert!(rf.params.iter().any(|p| matches!(p, StepParameter::ResetForFailedBlock(true))));
    }
}
