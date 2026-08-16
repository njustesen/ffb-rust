/// Builds the activation sub-sequence injected after INIT in Move/Block/Foul/Pass/Select.
/// Resolves negatraits (Animal Savagery, Bone Head, Really Stupid, Take Root, Unchannelled
/// Fury, Blood Lust). For a skill-less lineman every step is a pass-through but still pushed.
///
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.bb2025.ActivationSequenceBuilder`.
use ffb_model::enums::ApothecaryMode;
use ffb_model::types::FieldCoordinate;
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::sequence::{Sequence, labels};

/// Builder for the activation sub-sequence.
///
/// Usage:
/// ```rust,ignore
/// ActivationSequenceBuilder::new()
///     .with_failure_label(labels::END_BLOCKING)
///     .with_old_defender(Some("p1".into()))
///     .with_eventual_defender(Some("p1".into()))
///     .add_to(&mut sequence);
/// ```
pub struct ActivationSequenceBuilder {
    failure_label: String,
    old_defender: Option<String>,
    eventual_defender: Option<String>,
    prevent_null_defender: bool,
    target_coordinate: Option<FieldCoordinate>,
    /// Edition of the activation being built. The driver runs this BB2025 builder for BB2020
    /// games, but BB2020 has no STEADY_FOOTING step -- see `add_to`.
    rules: ffb_model::enums::Rules,
}

impl ActivationSequenceBuilder {
    pub fn new() -> Self {
        ActivationSequenceBuilder {
            failure_label: String::new(),
            old_defender: None,
            eventual_defender: None,
            prevent_null_defender: false,
            target_coordinate: None,
            rules: ffb_model::enums::Rules::Bb2025,
        }
    }

    /// Edition of the activation being built; only BB2020 differs (it has no STEADY_FOOTING).
    pub fn with_rules(mut self, rules: ffb_model::enums::Rules) -> Self {
        self.rules = rules;
        self
    }

    /// Label to jump to when a negatrait fails (usually the sequence's END label).
    pub fn with_failure_label(mut self, label: impl Into<String>) -> Self {
        self.failure_label = label.into();
        self
    }

    /// ID of the previously selected player (e.g. block defender).
    pub fn with_old_defender(mut self, id: Option<String>) -> Self {
        self.old_defender = id;
        self
    }

    /// ID to restore as defender after all negatrait checks (Animal Savagery may clear it).
    pub fn with_eventual_defender(mut self, id: Option<String>) -> Self {
        self.eventual_defender = id;
        self
    }

    /// Prevent StepSetDefender from propagating a null id (used by move actions).
    pub fn prevent_null_defender(mut self) -> Self {
        self.prevent_null_defender = true;
        self
    }

    /// Target coordinate for the action (needed by Animal Savagery for pass actions).
    pub fn with_target_coordinate(mut self, coord: Option<FieldCoordinate>) -> Self {
        self.target_coordinate = coord;
        self
    }

    /// Append the activation steps to `sequence` and return the sequence (Java `addTo`).
    pub fn add_to(self, sequence: &mut Sequence) {
        let fl = self.failure_label.clone();

        // 1 INIT_ACTIVATION
        sequence.add(StepId::InitActivation, vec![]);

        // 2 ANIMAL_SAVAGERY (goto failure on fail; passes old defender / target coord)
        let mut as_params = vec![StepParameter::GotoLabelOnFailure(fl.clone())];
        if let Some(ref id) = self.old_defender {
            as_params.push(StepParameter::BlockDefenderId(id.clone()));
        }
        if let Some(coord) = self.target_coordinate {
            as_params.push(StepParameter::TargetCoordinate(coord));
        }
        sequence.add(StepId::AnimalSavagery, as_params);

        // 3 STEADY_FOOTING -- BB2025 ONLY.
        //
        // `grep -rn STEADY_FOOTING .../server/step/generator/bb2020/` is empty: BB2020 has no such
        // step anywhere, and its hand-written activation block
        // (`bb2020/SelectBlitzTarget.java:20-31`) runs INIT_ACTIVATION, ANIMAL_SAVAGERY,
        // HANDLE_DROP_PLAYER_CONTEXT, ... -- 12 steps to this builder's 13.
        //
        // Safe to omit because the only thing that can feed it here is ANIMAL_SAVAGERY two steps
        // up, and `step/mixed/shared/step_animal_savagery.rs` ALREADY edition-gates its publish:
        // BB2025 publishes a SteadyFootingContext, BB2020 publishes a plain DropPlayerContext for
        // HANDLE_DROP_PLAYER_CONTEXT below (mirroring `skillbehaviour/bb2020/
        // AnimalSavageryBehaviour.java:235` vs the bb2025 twin's :293). So in BB2020 this step
        // never had a context to consume.
        //
        // This is ONLY true of the activation copy. The STEADY_FOOTING steps inside the move and
        // blitz sequences are NOT removable the same way: the shared fall sites (GFI, jump,
        // dodge, chainsaw) publish a SteadyFootingContext in every edition, and those steps are
        // its only consumer -- drop them and BB2020 players stop falling. Do not "simplify" this
        // by making StepSteadyFooting a no-op under BB2020 in `make_step_for`; that would hit the
        // move-sequence copies too. See docs/PARITY_BB2020_CAMPAIGN.md (ITER116, ITER119).
        if self.rules != ffb_model::enums::Rules::Bb2020 {
            sequence.add(StepId::SteadyFooting, vec![]);
        }
        // 4 HANDLE_DROP_PLAYER_CONTEXT
        sequence.add(StepId::HandleDropPlayerContext, vec![]);
        // 5 PLACE_BALL
        sequence.add(StepId::PlaceBall, vec![]);
        // 6 APOTHECARY (Animal Savagery mode)
        sequence.add(StepId::Apothecary, vec![
            StepParameter::ApothecaryMode(ApothecaryMode::AnimalSavagery),
        ]);
        // 7 CATCH_SCATTER_THROW_IN
        sequence.add(StepId::CatchScatterThrowIn, vec![]);

        // 8 SET_DEFENDER — only if an eventual defender is specified
        if let Some(ref eventual) = self.eventual_defender {
            sequence.add(StepId::SetDefender, vec![
                StepParameter::BlockDefenderId(eventual.clone()),
                StepParameter::IgnoreNullValue(self.prevent_null_defender),
            ]);
        }

        // 9 GOTO_LABEL → NEXT (with alternate goto to failure label if USE_ALTERNATE_LABEL)
        sequence.add(StepId::GotoLabel, vec![
            StepParameter::GotoLabel(labels::NEXT.into()),
            StepParameter::AlternateGotoLabel(fl.clone()),
        ]);

        // 10 BONE_HEAD [NEXT] (goto failure on fail)
        sequence.add_labelled(StepId::BoneHead, labels::NEXT, vec![
            StepParameter::GotoLabelOnFailure(fl.clone()),
        ]);
        // 11 REALLY_STUPID
        sequence.add(StepId::ReallyStupid, vec![
            StepParameter::GotoLabelOnFailure(fl.clone()),
        ]);
        // 12 TAKE_ROOT
        sequence.add(StepId::TakeRoot, vec![]);
        // 13 UNCHANNELLED_FURY
        sequence.add(StepId::UnchannelledFury, vec![
            StepParameter::GotoLabelOnFailure(fl.clone()),
        ]);
        // 14 BLOOD_LUST
        sequence.add(StepId::BloodLust, vec![
            StepParameter::GotoLabelOnFailure(fl),
        ]);
    }
}

impl Default for ActivationSequenceBuilder {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_with_label(label: &str) -> Vec<crate::step::framework::SequenceStep> {
        let mut seq = Sequence::new();
        ActivationSequenceBuilder::new()
            .with_failure_label(label)
            .add_to(&mut seq);
        seq.build()
    }

    #[test]
    fn add_to_emits_thirteen_steps_without_defender() {
        let steps = build_with_label("END");
        assert_eq!(steps.len(), 13);
    }

    #[test]
    fn first_step_is_init_activation() {
        let steps = build_with_label("END");
        assert_eq!(steps[0].step_id, StepId::InitActivation);
    }

    #[test]
    fn bone_head_has_next_label_at_index_8() {
        let steps = build_with_label("END");
        // Without eventual_defender: GotoLabel at 7, BoneHead [NEXT] at 8
        assert_eq!(steps[8].step_id, StepId::BoneHead);
        assert_eq!(steps[8].label.as_deref(), Some(labels::NEXT));
    }

    /// The BB2020 activation block is `bb2020/SelectBlitzTarget.java:20-31` exactly: the BB2025
    /// block minus STEADY_FOOTING, 12 steps to 13.
    ///
    /// Supersedes the ITER116 guard test, which asserted the opposite. That test was built on a
    /// wrong premise -- that ANIMAL_SAVAGERY feeds this step in BB2020 too -- when in fact
    /// `step/mixed/shared/step_animal_savagery.rs` already edition-gates its publish and hands
    /// BB2020 a plain DropPlayerContext for HANDLE_DROP_PLAYER_CONTEXT instead. See ITER119.
    #[test]
    fn bb2020_activation_omits_steady_footing_and_matches_javas_twelve_steps() {
        use ffb_model::enums::Rules;
        let mut seq = Sequence::new();
        ActivationSequenceBuilder::new()
            .with_rules(Rules::Bb2020)
            .with_failure_label("END")
            .add_to(&mut seq);
        let bb2020 = seq.build();

        assert!(!bb2020.iter().any(|s| s.step_id == StepId::SteadyFooting),
            "BB2020 has no STEADY_FOOTING step in any generator");
        assert_eq!(bb2020.len(), 12, "Java bb2020 builds exactly 12 activation steps");

        // ANIMAL_SAVAGERY must still hand straight to HANDLE_DROP_PLAYER_CONTEXT, which is what
        // consumes the DropPlayerContext BB2020 publishes in place of a SteadyFootingContext.
        let as_i = bb2020.iter().position(|s| s.step_id == StepId::AnimalSavagery).unwrap();
        let hdpc = bb2020.iter().position(|s| s.step_id == StepId::HandleDropPlayerContext).unwrap();
        assert_eq!(hdpc, as_i + 1, "nothing may sit between them in BB2020");

        // BB2025 keeps the step, and keeps it between those two.
        let bb2025 = build_with_label("END");
        assert_eq!(bb2025.len(), 13);
        let sf = bb2025.iter().position(|s| s.step_id == StepId::SteadyFooting).unwrap();
        let as25 = bb2025.iter().position(|s| s.step_id == StepId::AnimalSavagery).unwrap();
        assert_eq!(sf, as25 + 1);
    }

    /// The move/blitz-sequence STEADY_FOOTING steps are NOT removable for BB2020 the way the
    /// activation copy is: the shared fall sites (GFI, jump, dodge, chainsaw) publish a
    /// `SteadyFootingContext` in every edition and those steps are its only consumer, so dropping
    /// them stops BB2020 players falling. This pins the distinction so nobody generalises the
    /// ITER119 gate into `make_step_for`, which would hit every copy at once.
    #[test]
    fn steady_footing_gate_is_scoped_to_the_activation_only() {
        use crate::step::generator::bb2025::blitz_block::BlitzBlock;
        let blitz = BlitzBlock::build_sequence(&Default::default());
        assert!(blitz.iter().any(|s| s.step_id == StepId::SteadyFooting),
            "the blitz sequence keeps its own STEADY_FOOTING regardless of edition");
    }

    #[test]
    fn eventual_defender_adds_set_defender_and_emits_fourteen_steps() {
        let mut seq = Sequence::new();
        ActivationSequenceBuilder::new()
            .with_failure_label("END")
            .with_eventual_defender(Some("player1".into()))
            .add_to(&mut seq);
        let steps = seq.build();
        assert_eq!(steps.len(), 14);
        assert_eq!(steps[7].step_id, StepId::SetDefender);
    }
}
