/// BB2025 select (activate player) step sequence — the action dispatch hub.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.bb2025.Select`.
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::sequence::{Sequence, SequenceStep, labels};
use super::activation_sequence_builder::ActivationSequenceBuilder;

/// Parameters for the Select sequence — mirrors Java `Select.SequenceParams`.
#[derive(Debug, Clone)]
pub struct SelectParams {
    pub update_persistence: bool,
    /// True when the current player action is a blitz move (affects ResetFumblerooskie).
    pub is_blitz_move: bool,
    /// Java `blockTargets` — defender IDs threaded to END_SELECTING (here as strings, matching
    /// the bb2020 sibling convention, since `BlockTarget` isn't threaded through this step layer).
    pub block_targets: Vec<String>,
    /// Edition this sequence is built for; only BB2020 differs (no STEADY_FOOTING in the activation).
    pub rules: ffb_model::enums::Rules,
}

impl Default for SelectParams {
    fn default() -> Self {
        Self {
            update_persistence: Default::default(),
            is_blitz_move: Default::default(),
            block_targets: Default::default(),
            rules: ffb_model::enums::Rules::Bb2025,
        }
    }
}

pub struct Select;

impl Select {
    pub fn new() -> Self { Self }

    /// Build the select step sequence (Java `pushSequence`).
    pub fn build_sequence(params: &SelectParams) -> Vec<SequenceStep> {
        let mut seq = Sequence::new();
        let fl = labels::END_SELECTING;

        // 1 INIT_SELECTING
        seq.add(StepId::InitSelecting, vec![
            StepParameter::GotoLabelOnEnd(fl.into()),
            StepParameter::UpdatePersistence(params.update_persistence),
        ]);

        // 2 [ACTIVATION(END_SELECTING)] — no SET_DEFENDER for select
        ActivationSequenceBuilder::new()
            .with_rules(params.rules)
            .with_failure_label(fl)
            .add_to(&mut seq);

        // 2b FOUL_APPEARANCE — BB2020 blitz only, gated at runtime inside the step.
        //
        // `bb2020/SelectBlitzTarget.java:35` rolls Foul Appearance right after BLOOD_LUST, i.e.
        // BEFORE JUMP_UP/STAND_UP and before the blitz's GO_FOR_IT; `bb2025/BlitzBlock.java:37`
        // rolls it inside the block sequence instead. Rust's agent commits blitz+target in one
        // command, so SelectBlitzTarget never runs and the roll used to land in the shared BB2025
        // blitz-block position — three dice late for a BB2020 blitz (nurgle-vs-undead seed 1 i=20:
        // Rust rolled GFI 5 / FA 1, Java rolled FA 5 / GFI 1). This is the Java position; the step
        // no-ops (see `in_select`) for every other edition and action, and `blitz_block.rs` drops
        // its own copy under BB2020 so the roll still happens exactly once.
        seq.add(StepId::FoulAppearance, vec![
            StepParameter::GotoLabelOnFailure(fl.into()),
            StepParameter::InSelect(true),
        ]);

        // 3 GOTO_LABEL → NEXT (alternate → END_SELECTING)
        seq.add(StepId::GotoLabel, vec![
            StepParameter::GotoLabel(labels::NEXT.into()),
            StepParameter::AlternateGotoLabel(fl.into()),
        ]);

        // 4 JUMP_UP [NEXT]
        seq.add_labelled(StepId::JumpUp, labels::NEXT, vec![
            StepParameter::GotoLabelOnFailure(fl.into()),
        ]);

        // 5 STAND_UP
        seq.add(StepId::StandUp, vec![
            StepParameter::GotoLabelOnFailure(fl.into()),
        ]);

        // 6 RESET_FUMBLEROOSKIE [END_SELECTING]
        seq.add_labelled(StepId::ResetFumblerooskie, fl, vec![
            StepParameter::InSelect(true),
            StepParameter::ResetForFailedBlock(params.is_blitz_move),
        ]);

        // 7 END_SELECTING (Java always passes params.getBlockTargets())
        if params.block_targets.is_empty() {
            seq.add(StepId::EndSelecting, vec![]);
        } else {
            seq.add(StepId::EndSelecting, vec![
                StepParameter::BlockTargets(params.block_targets.clone()),
            ]);
        }

        seq.build()
    }
}

impl Default for Select {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn select_sequence_starts_with_init_selecting() {
        let steps = Select::build_sequence(&SelectParams::default());
        assert_eq!(steps[0].step_id, StepId::InitSelecting);
    }

    #[test]
    fn select_sequence_ends_with_end_selecting() {
        let steps = Select::build_sequence(&SelectParams::default());
        let last = steps.last().unwrap();
        assert_eq!(last.step_id, StepId::EndSelecting);
    }

    #[test]
    fn select_sequence_has_reset_fumblerooskie_labelled_end_selecting() {
        let steps = Select::build_sequence(&SelectParams::default());
        let rfr = steps.iter().find(|s| s.step_id == StepId::ResetFumblerooskie).unwrap();
        assert_eq!(rfr.label.as_deref(), Some(labels::END_SELECTING));
    }

    #[test]
    fn update_persistence_param_passed_to_init_selecting() {
        let params = SelectParams { update_persistence: true, ..Default::default() };
        let steps = Select::build_sequence(&params);
        let has = steps[0].params.iter().any(|p| matches!(p, StepParameter::UpdatePersistence(true)));
        assert!(has);
    }

    #[test]
    fn is_blitz_move_sets_reset_for_failed_block() {
        let params = SelectParams { is_blitz_move: true, ..Default::default() };
        let steps = Select::build_sequence(&params);
        let rfr = steps.iter().find(|s| s.step_id == StepId::ResetFumblerooskie).unwrap();
        let has = rfr.params.iter().any(|p| matches!(p, StepParameter::ResetForFailedBlock(true)));
        assert!(has);
    }

    #[test]
    fn block_targets_wired_to_end_selecting_when_present() {
        let params = SelectParams { block_targets: vec!["p1".into()], ..Default::default() };
        let steps = Select::build_sequence(&params);
        let end = steps.iter().find(|s| s.step_id == StepId::EndSelecting).unwrap();
        assert!(end.params.iter().any(|p| matches!(p, StepParameter::BlockTargets(v) if v == &vec!["p1".to_string()])));
    }

    /// `bb2020/SelectBlitzTarget.java:35` rolls FOUL_APPEARANCE after BLOOD_LUST and before
    /// JUMP_UP/STAND_UP. The step itself no-ops outside a BB2020 blitz, but its POSITION here is
    /// what puts the roll ahead of the stand-up and of the blitz's GO_FOR_IT.
    #[test]
    fn foul_appearance_sits_between_the_activation_and_jump_up() {
        let steps = Select::build_sequence(&SelectParams::default());
        let fa = steps.iter().position(|s| s.step_id == StepId::FoulAppearance).unwrap();
        let blood_lust = steps.iter().position(|s| s.step_id == StepId::BloodLust).unwrap();
        let jump_up = steps.iter().position(|s| s.step_id == StepId::JumpUp).unwrap();
        let stand_up = steps.iter().position(|s| s.step_id == StepId::StandUp).unwrap();
        assert!(blood_lust < fa && fa < jump_up && jump_up < stand_up);
        assert!(steps[fa].params.iter().any(|p| matches!(p, StepParameter::InSelect(true))),
            "the select copy must be flagged so it no-ops outside a BB2020 blitz");
    }

    #[test]
    fn block_targets_omitted_when_empty() {
        let steps = Select::build_sequence(&SelectParams::default());
        let end = steps.iter().find(|s| s.step_id == StepId::EndSelecting).unwrap();
        assert!(end.params.is_empty());
    }
}
