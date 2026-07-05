/// BB2025 bomb step sequence.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.bb2025.Bomb`.
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::sequence::{Sequence, SequenceStep, labels};

#[derive(Debug, Clone, Default)]
pub struct BombParams {
    pub catcher_id: Option<String>,
    pub pass_fumble: bool,
    pub dont_drop_fumble: bool,
}

pub struct Bomb;

impl Bomb {
    pub fn new() -> Self { Self }

    pub fn build_sequence(params: &BombParams) -> Vec<SequenceStep> {
        let mut seq = Sequence::new();
        // 1 INIT_BOMB
        let mut init_params = vec![
            StepParameter::GotoLabelOnEnd(labels::END_BOMB.into()),
            StepParameter::PassFumble(params.pass_fumble),
            StepParameter::DontDropFumble(params.dont_drop_fumble),
        ];
        if let Some(ref id) = params.catcher_id {
            init_params.push(StepParameter::CatcherId(Some(id.clone())));
        }
        seq.add(StepId::InitBomb, init_params);
        // 2 CATCH_SCATTER_THROW_IN
        seq.add(StepId::CatchScatterThrowIn, vec![]);
        // 3 RESOLVE_BOMB
        seq.add(StepId::ResolveBomb, vec![]);
        // 4 CATCH_SCATTER_THROW_IN
        seq.add(StepId::CatchScatterThrowIn, vec![]);
        // 5 END_BOMB [END_BOMB]
        seq.add_labelled(StepId::EndBomb, labels::END_BOMB, vec![]);
        seq.build()
    }
}

impl Default for Bomb {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bomb_has_5_steps() {
        let steps = Bomb::build_sequence(&BombParams::default());
        assert_eq!(steps.len(), 5);
    }

    #[test]
    fn bomb_ends_with_end_bomb_labelled() {
        let steps = Bomb::build_sequence(&BombParams::default());
        let last = steps.last().unwrap();
        assert_eq!(last.step_id, StepId::EndBomb);
        assert_eq!(last.label.as_deref(), Some(labels::END_BOMB));
    }

    #[test]
    fn bomb_starts_with_init_bomb() {
        let steps = Bomb::build_sequence(&BombParams::default());
        assert_eq!(steps[0].step_id, StepId::InitBomb);
    }

    #[test]
    fn catcher_id_included_when_some() {
        let params = BombParams { catcher_id: Some("catcher1".into()), ..Default::default() };
        let steps = Bomb::build_sequence(&params);
        let has = steps[0].params.iter().any(|p| matches!(p, StepParameter::CatcherId(Some(id)) if id == "catcher1"));
        assert!(has);
    }

    #[test]
    fn pass_fumble_param_passed() {
        let params = BombParams { pass_fumble: true, ..Default::default() };
        let steps = Bomb::build_sequence(&params);
        let has = steps[0].params.iter().any(|p| matches!(p, StepParameter::PassFumble(true)));
        assert!(has);
    }
}
