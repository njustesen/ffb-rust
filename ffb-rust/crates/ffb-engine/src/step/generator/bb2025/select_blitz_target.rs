/// BB2025 select-blitz-target step sequence.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.bb2025.SelectBlitzTarget`.
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::sequence::{Sequence, SequenceStep, labels};
use super::activation_sequence_builder::ActivationSequenceBuilder;

pub struct SelectBlitzTarget;

impl SelectBlitzTarget {
    pub fn new() -> Self { Self }

    pub fn build_sequence(rules: ffb_model::enums::Rules) -> Vec<SequenceStep> {
        let mut seq = Sequence::new();
        // 1 SELECT_BLITZ_TARGET [SELECT]
        seq.add_labelled(StepId::SelectBlitzTarget, labels::SELECT, vec![
            StepParameter::GotoLabelOnEnd(labels::END_BLITZING.into()),
        ]);

        // [ACTIVATION(END_BLITZING)]
        ActivationSequenceBuilder::new()
            .with_rules(rules)
            .with_failure_label(labels::END_BLITZING)
            .add_to(&mut seq);

        // BB2020 only: `bb2020/SelectBlitzTarget.java:35-36` rolls FOUL_APPEARANCE here, a phase
        // EARLIER than BB2025 does (BB2025 leaves it to the block sequence), and follows it with
        // DUMP_OFF. `bb2025/SelectBlitzTarget.java` has neither.
        //
        // Both editions must have it exactly once. Until the chain existed the BB2020 blitz got
        // its Foul Appearance from the block sequence instead - the shared BlitzBlock generator
        // was being built with the default (BB2025) rules, which kept that entry. Fixing that
        // default without adding this one would delete the roll outright (measured: nurgle
        // bb2020 2/100, dark_elf 15/100, necromantic 70/100 - all Foul Appearance matchups).
        if rules == ffb_model::enums::Rules::Bb2020 {
            seq.add(StepId::FoulAppearance, vec![
                StepParameter::GotoLabelOnFailure(labels::END_BLITZING.into()),
            ]);
            seq.add(StepId::DumpOff, vec![]);
        }
        // 2 JUMP_UP
        seq.add(StepId::JumpUp, vec![
            StepParameter::GotoLabelOnFailure(labels::END_BLITZING.into()),
        ]);
        // 3 STAND_UP
        seq.add(StepId::StandUp, vec![
            StepParameter::GotoLabelOnFailure(labels::END_BLITZING.into()),
        ]);
        // 4 SELECT_BLITZ_TARGET_END [END_BLITZING]
        seq.add_labelled(StepId::SelectBlitzTargetEnd, labels::END_BLITZING, vec![]);
        seq.build()
    }
}

impl Default for SelectBlitzTarget {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn select_blitz_target_has_17_steps_with_activation() {
        // Java pushSequence: SELECT_BLITZ_TARGET (1) + ActivationSequenceBuilder.create()...addTo(sequence)
        // (13) + 3 own steps = 17.
        let steps = SelectBlitzTarget::build_sequence(ffb_model::enums::Rules::Bb2025);
        assert_eq!(steps.len(), 17);
        assert_eq!(steps[1].step_id, StepId::InitActivation);
    }

    #[test]
    fn select_blitz_target_is_labelled_select() {
        let steps = SelectBlitzTarget::build_sequence(ffb_model::enums::Rules::Bb2025);
        let s = steps.iter().find(|s| s.step_id == StepId::SelectBlitzTarget).unwrap();
        assert_eq!(s.label.as_deref(), Some(labels::SELECT));
    }

    #[test]
    fn select_blitz_target_end_is_labelled_end_blitzing() {
        let steps = SelectBlitzTarget::build_sequence(ffb_model::enums::Rules::Bb2025);
        let s = steps.iter().find(|s| s.step_id == StepId::SelectBlitzTargetEnd).unwrap();
        assert_eq!(s.label.as_deref(), Some(labels::END_BLITZING));
    }

    #[test]
    fn select_blitz_target_has_jump_up_and_stand_up() {
        let steps = SelectBlitzTarget::build_sequence(ffb_model::enums::Rules::Bb2025);
        assert!(steps.iter().any(|s| s.step_id == StepId::JumpUp));
        assert!(steps.iter().any(|s| s.step_id == StepId::StandUp));
    }
    /// §12: BB2020 rolls the blitzer's Foul Appearance in the SELECT phase
    /// (`bb2020/SelectBlitzTarget.java:35-36`, followed by DUMP_OFF); BB2025 has neither here and
    /// leaves Foul Appearance to the block sequence. Exactly one of the two placements must be
    /// live per edition - Rust briefly had NEITHER for BB2020 (nurgle 2/100, dark_elf 15/100,
    /// necromantic 70/100) once the BlitzBlock generator stopped being built with BB2025 rules.
    #[test]
    fn bb2020_select_rolls_foul_appearance_and_dump_off_but_bb2025_does_not() {
        let bb2020 = SelectBlitzTarget::build_sequence(ffb_model::enums::Rules::Bb2020);
        let fa = bb2020.iter().position(|s| s.step_id == StepId::FoulAppearance)
            .expect("BB2020 select must roll FOUL_APPEARANCE");
        let dump_off = bb2020.iter().position(|s| s.step_id == StepId::DumpOff)
            .expect("BB2020 select must contain DUMP_OFF");
        let jump_up = bb2020.iter().position(|s| s.step_id == StepId::JumpUp).unwrap();
        // Java order: ... BLOOD_LUST, FOUL_APPEARANCE, DUMP_OFF, JUMP_UP, STAND_UP, END.
        assert!(fa < dump_off && dump_off < jump_up,
            "Java order is FOUL_APPEARANCE -> DUMP_OFF -> JUMP_UP, got {fa}/{dump_off}/{jump_up}");

        let bb2025 = SelectBlitzTarget::build_sequence(ffb_model::enums::Rules::Bb2025);
        assert!(!bb2025.iter().any(|s| s.step_id == StepId::FoulAppearance),
            "BB2025 select must NOT roll Foul Appearance - its block sequence does");
        assert!(!bb2025.iter().any(|s| s.step_id == StepId::DumpOff),
            "BB2025 select must not contain DUMP_OFF");
    }

    #[test]
    fn build_sequence_returns_vec() {
        let seq = SelectBlitzTarget::build_sequence(ffb_model::enums::Rules::Bb2025);
        assert!(!seq.is_empty(), "sequence should not be empty");
    }

}
