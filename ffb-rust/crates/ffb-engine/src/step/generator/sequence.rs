/// Step sequence builder — mirrors Java `generator/Sequence.java`.
///
/// Generators call `Sequence::add` / `add_labelled` / `jump` to describe the ordered list of
/// steps for an action, then `build()` returns the `Vec<SequenceStep>`. The stack's
/// `push_sequence` reverses the vec so the first-authored step ends on top and runs first,
/// matching Java's `StepStack.push(List<IStep>)` iterating back-to-front.

use crate::step::framework::{StepId, StepParameter};
pub use crate::step::framework::SequenceStep;

/// Builder that accumulates step entries in authored order.
/// Java API: `sequence.add(stepId, params...)` and `sequence.jump(label)`.
#[derive(Default)]
pub struct Sequence {
    steps: Vec<SequenceStep>,
}

impl Sequence {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a step without a label (Java `sequence.add(stepId, params...)`).
    pub fn add(&mut self, step_id: StepId, params: Vec<StepParameter>) {
        self.steps.push(SequenceStep { step_id, label: None, params });
    }

    /// Add a step with a goto-target label (Java `sequence.add(stepId, label, params...)`).
    pub fn add_labelled(&mut self, step_id: StepId, label: impl Into<String>, params: Vec<StepParameter>) {
        self.steps.push(SequenceStep { step_id, label: Some(label.into()), params });
    }

    /// Push a `GOTO_LABEL` step targeting `target_label` (Java `sequence.jump(label)`).
    pub fn jump(&mut self, target_label: impl Into<String>) {
        let target = target_label.into();
        self.add(StepId::GotoLabel, vec![StepParameter::GotoLabel(target)]);
    }

    /// Consume the builder and return the accumulated steps in authored order.
    pub fn build(self) -> Vec<SequenceStep> {
        self.steps
    }
}

/// Standard step labels — mirrors Java `IStepLabel`.
pub mod labels {
    pub const END_BLOCKING: &str = "END_BLOCKING";
    pub const END_MOVING: &str = "END_MOVING";
    pub const END_FOULING: &str = "END_FOULING";
    pub const END_PASSING: &str = "END_PASSING";
    pub const END_SELECTING: &str = "END_SELECTING";
    pub const END_KICKOFF: &str = "END_KICKOFF";
    pub const END_FEEDING: &str = "END_FEEDING";
    pub const NEXT: &str = "NEXT";
    pub const FALL_DOWN: &str = "FALL_DOWN";
    pub const PUSHBACK: &str = "PUSHBACK";
    pub const BOTH_DOWN: &str = "BOTH_DOWN";
    pub const DODGE_BLOCK: &str = "DODGE_BLOCK";
    pub const RETRY_DODGE: &str = "RETRY_DODGE";
    pub const SHADOWING: &str = "SHADOWING";
    pub const SCATTER_BALL: &str = "SCATTER_BALL";
    pub const DEFENDER_DROPPED: &str = "DEFENDER_DROPPED";
    pub const ATTACKER_DROPPED: &str = "ATTACKER_DROPPED";
    pub const DROP_FALLING_PLAYERS: &str = "DROP_FALLING_PLAYERS";
    pub const JUGGERNAUT: &str = "JUGGERNAUT";
    pub const STEADY_FOOTING: &str = "STEADY_FOOTING";
    pub const HYPNOTIC_GAZE: &str = "HYPNOTIC_GAZE";
    pub const MOVE_START: &str = "MOVE_START";
    pub const BLITZ_TURN: &str = "BLITZ_TURN";
    pub const KICKOFF_ANIMATION: &str = "KICKOFF_ANIMATION";
    pub const PASS: &str = "PASS";
    pub const HAIL_MARY_PASS: &str = "HAIL_MARY_PASS";
    pub const MISSED_PASS: &str = "MISSED_PASS";
    pub const INTERCEPT: &str = "INTERCEPT";
    pub const RESOLVE_PASS: &str = "RESOLVE_PASS";
    pub const HAND_OVER: &str = "HAND_OVER";
    pub const APOTHECARY_ATTACKER: &str = "APOTHECARY_ATTACKER";
    pub const END: &str = "END";
    // Bomb
    pub const END_BOMB: &str = "END_BOMB";
    // EndGame
    pub const END_GAME: &str = "END_GAME";
    // ScatterPlayer / ThrowTeamMate
    pub const APOTHECARY_HIT_PLAYER: &str = "APOTHECARY_HIT_PLAYER";
    pub const END_SCATTER_PLAYER: &str = "END_SCATTER_PLAYER";
    pub const APOTHECARY_THROWN_PLAYER: &str = "APOTHECARY_THROWN_PLAYER";
    pub const END_THROW_TEAM_MATE: &str = "END_THROW_TEAM_MATE";
    pub const END_KICK_TEAM_MATE: &str = "END_KICK_TEAM_MATE";
    pub const EAT_TEAM_MATE: &str = "EAT_TEAM_MATE";
    pub const RIGHT_STUFF: &str = "RIGHT_STUFF";
    // SelectBlitzTarget
    pub const SELECT: &str = "SELECT";
    pub const END_BLITZING: &str = "END_BLITZING";
    // SpecialEffect
    pub const END_SPECIAL_EFFECT: &str = "END_SPECIAL_EFFECT";
    // BB2016-specific
    pub const APOTHECARY_DEFENDER: &str = "APOTHECARY_DEFENDER";
    pub const KICK_TM_DOUBLE_ROLLED: &str = "KICK_TM_DOUBLE_ROLLED";
    pub const APOTHECARY_KICKED_PLAYER: &str = "APOTHECARY_KICKED_PLAYER";
    pub const FUMBLE_TTM_PASS: &str = "FUMBLE_TTM_PASS";
    pub const SKIP_PILE_DRIVER: &str = "SKIP_PILE_DRIVER";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_creates_unlabelled_step() {
        let mut seq = Sequence::new();
        seq.add(StepId::Apothecary, vec![]);
        let steps = seq.build();
        assert_eq!(steps.len(), 1);
        assert_eq!(steps[0].step_id, StepId::Apothecary);
        assert!(steps[0].label.is_none());
        assert!(steps[0].params.is_empty());
    }

    #[test]
    fn add_labelled_creates_step_with_label() {
        let mut seq = Sequence::new();
        seq.add_labelled(StepId::Apothecary, "MY_LABEL", vec![]);
        let steps = seq.build();
        assert_eq!(steps[0].label.as_deref(), Some("MY_LABEL"));
    }

    #[test]
    fn jump_creates_goto_label_step() {
        let mut seq = Sequence::new();
        seq.jump("END");
        let steps = seq.build();
        assert_eq!(steps.len(), 1);
        assert_eq!(steps[0].step_id, StepId::GotoLabel);
        assert!(matches!(steps[0].params[0], StepParameter::GotoLabel(ref s) if s == "END"));
    }

    #[test]
    fn build_returns_steps_in_authored_order() {
        let mut seq = Sequence::new();
        seq.add(StepId::Apothecary, vec![]);
        seq.add(StepId::GotoLabel, vec![]);
        let steps = seq.build();
        assert_eq!(steps[0].step_id, StepId::Apothecary);
        assert_eq!(steps[1].step_id, StepId::GotoLabel);
    }
    #[test]
    fn build_returns_multiple_steps_in_order() {
        let mut seq = Sequence::new();
        seq.add(StepId::Apothecary, vec![]);
        seq.add(StepId::GotoLabel, vec![]);
        let steps = seq.build();
        assert_eq!(steps.len(), 2);
        assert_eq!(steps[0].step_id, StepId::Apothecary);
        assert_eq!(steps[1].step_id, StepId::GotoLabel);
    }
}
