/// BB2020 bomb step sequence.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.bb2020.Bomb`.
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
        // 3 END_BOMB [END_BOMB]
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
    fn bomb_has_3_steps() {
        let steps = Bomb::build_sequence(&BombParams::default());
        assert_eq!(steps.len(), 3);
    }

    #[test]
    fn bomb_ends_with_end_bomb_labelled() {
        let steps = Bomb::build_sequence(&BombParams::default());
        let last = steps.last().unwrap();
        assert_eq!(last.step_id, StepId::EndBomb);
        assert_eq!(last.label.as_deref(), Some(labels::END_BOMB));
    }

    #[test]
    fn bomb_has_no_resolve_bomb() {
        let steps = Bomb::build_sequence(&BombParams::default());
        assert!(!steps.iter().any(|s| s.step_id == StepId::ResolveBomb));
    }

    #[test]
    fn bomb_starts_with_init_bomb() {
        let steps = Bomb::build_sequence(&BombParams::default());
        assert_eq!(steps[0].step_id, StepId::InitBomb);
    }
}
