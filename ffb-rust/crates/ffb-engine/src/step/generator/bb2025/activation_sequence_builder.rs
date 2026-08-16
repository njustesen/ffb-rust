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
}

impl ActivationSequenceBuilder {
    pub fn new() -> Self {
        ActivationSequenceBuilder {
            failure_label: String::new(),
            old_defender: None,
            eventual_defender: None,
            prevent_null_defender: false,
            target_coordinate: None,
        }
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

        // 3 STEADY_FOOTING
        //
        // DO NOT edition-gate this off for BB2020, even though it looks like you should:
        // `grep -rn STEADY_FOOTING .../server/step/generator/bb2020/` is EMPTY, so no BB2020 Java
        // generator has this step anywhere. It is one half of a COUPLED pair, and removing only
        // this half silently breaks every BB2020 fall.
        //
        // `StepSteadyFooting` is the ONLY consumer of a published `SteadyFootingContext`
        // (`StepHandleDropPlayerContext` below does not read it). A BB2020 game publishes one
        // anyway: `make_step_for` routes only `StepId::Prayer` to a bb2020 step, so BB2020 games
        // run the SHARED bb2025 fall sites — `bb2025/move_/step_go_for_it.rs`, `step_jump.rs`,
        // `step_move_dodge.rs`, `block/step_block_chainsaw.rs` — every one of which publishes a
        // context, where BB2020 Java applies the drop directly instead. (The bb2020/*.rs twins of
        // those files publish one too, but they are dead code and never instantiated; do not be
        // fooled into thinking the publisher disappears with them.) Those publishes and this step
        // cancel out exactly, which is why all 30 BB2020 rosters are 100/100 with it present,
        // including the only two that can even reach it from ANIMAL_SAVAGERY above (renegades,
        // underworld).
        //
        // Making BB2020 truly 1:1 here means changing BOTH halves together: edition-gate those
        // shared fall sites to apply the drop inline as BB2020 Java does, THEN drop this step for
        // BB2020. See docs/PARITY_BB2020_CAMPAIGN.md (ITER116).
        sequence.add(StepId::SteadyFooting, vec![]);
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

    /// Guards the coupling documented at the STEADY_FOOTING line in `add_to`.
    ///
    /// No BB2020 Java generator contains STEADY_FOOTING, so gating this step off for BB2020 looks
    /// like an obvious 1:1 fix. It is not: `StepSteadyFooting` is the only consumer of a published
    /// `SteadyFootingContext`, and Rust's BB2020 fall sites (`bb2020/move_/step_go_for_it.rs:259`
    /// and friends) publish one. Removing just this step strands those contexts and no BB2020
    /// player ever falls. Both halves have to change together, or neither.
    ///
    /// If you are here because this test failed: you removed one half. Read the comment in
    /// `add_to` and `docs/PARITY_BB2020_CAMPAIGN.md` (ITER116) before going further.
    #[test]
    fn steady_footing_stays_in_the_activation_because_bb2020_fall_sites_depend_on_it() {
        let steps = build_with_label("END");
        let sf = steps.iter().position(|s| s.step_id == StepId::SteadyFooting)
            .expect("STEADY_FOOTING must stay: BB2020 fall sites publish a context only it reads");
        let animal_savagery = steps.iter().position(|s| s.step_id == StepId::AnimalSavagery)
            .expect("ANIMAL_SAVAGERY feeds the context consumed below");
        assert!(animal_savagery < sf,
            "ANIMAL_SAVAGERY publishes the SteadyFootingContext that STEADY_FOOTING consumes, \
             so it has to come first");
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
