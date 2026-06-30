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
