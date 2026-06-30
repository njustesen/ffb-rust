/// BB2025 Throw Keg step sequence.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.bb2025.ThrowKeg`.
use ffb_model::enums::ApothecaryMode;
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::sequence::{Sequence, SequenceStep, labels};

#[derive(Debug, Clone, Default)]
pub struct ThrowKegParams {
    pub player_id: Option<String>,
}

pub struct ThrowKeg;

impl ThrowKeg {
    pub fn new() -> Self { Self }

    pub fn build_sequence(params: &ThrowKegParams) -> Vec<SequenceStep> {
        let mut seq = Sequence::new();
        // 1 THROW_KEG
        let mut p = vec![];
        if let Some(ref id) = params.player_id {
            p.push(StepParameter::TargetPlayerId(Some(id.clone())));
        }
        seq.add(StepId::ThrowKeg, p);
        // 2 STEADY_FOOTING (goto END on success)
        seq.add(StepId::SteadyFooting, vec![
            StepParameter::GotoLabelOnSuccess(labels::END.into()),
        ]);
        // 3 HANDLE_DROP_PLAYER_CONTEXT
        seq.add(StepId::HandleDropPlayerContext, vec![]);
        // 4 APOTHECARY (DEFENDER)
        seq.add(StepId::Apothecary, vec![
            StepParameter::ApothecaryMode(ApothecaryMode::Defender),
        ]);
        // 5 CATCH_SCATTER_THROW_IN
        seq.add(StepId::CatchScatterThrowIn, vec![]);
        // 6 END_THROW_KEG [END]
        seq.add_labelled(StepId::EndThrowKeg, labels::END, vec![]);
        seq.build()
    }
}

impl Default for ThrowKeg {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn throw_keg_has_6_steps() {
        let steps = ThrowKeg::build_sequence(&ThrowKegParams::default());
        assert_eq!(steps.len(), 6);
    }

    #[test]
    fn throw_keg_ends_with_end_throw_keg_labelled_end() {
        let steps = ThrowKeg::build_sequence(&ThrowKegParams::default());
        let last = steps.last().unwrap();
        assert_eq!(last.step_id, StepId::EndThrowKeg);
        assert_eq!(last.label.as_deref(), Some(labels::END));
    }
}
