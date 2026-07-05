/// BB2016 scatter-player step sequence.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.bb2016.ScatterPlayer`.
use ffb_model::enums::ApothecaryMode;
use ffb_model::types::FieldCoordinate;
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::sequence::{Sequence, SequenceStep, labels};

/// Parameters for the BB2016 ScatterPlayer sequence.
#[derive(Debug, Clone, Default)]
pub struct ScatterPlayerParams {
    pub thrown_player_id: Option<String>,
    pub thrown_player_state: Option<ffb_model::enums::PlayerState>,
    pub thrown_player_has_ball: bool,
    pub thrown_player_coordinate: Option<FieldCoordinate>,
    pub throw_scatter: bool,
    pub has_swoop: bool,
}

pub struct ScatterPlayer;

impl ScatterPlayer {
    pub fn new() -> Self { Self }

    /// Build the BB2016 scatter-player step sequence (Java `pushSequence`).
    pub fn build_sequence(params: &ScatterPlayerParams) -> Vec<SequenceStep> {
        let mut seq = Sequence::new();

        if params.has_swoop {
            // SWOOP variant
            let mut swoop_params = vec![
                StepParameter::ThrownPlayerHasBall(params.thrown_player_has_ball),
                StepParameter::ThrowScatter(params.throw_scatter),
                StepParameter::GotoLabelOnFallDown(labels::APOTHECARY_HIT_PLAYER.into()),
            ];
            if let Some(ref id) = params.thrown_player_id {
                swoop_params.push(StepParameter::ThrownPlayerId(Some(id.clone())));
            }
            if let Some(state) = params.thrown_player_state {
                swoop_params.push(StepParameter::ThrownPlayerState(state));
            }
            if let Some(coord) = params.thrown_player_coordinate {
                swoop_params.push(StepParameter::ThrownPlayerCoordinate(Some(coord)));
            }
            seq.add(StepId::Swoop, swoop_params);
        } else {
            // INIT_SCATTER_PLAYER variant
            let mut init_params = vec![
                StepParameter::ThrownPlayerHasBall(params.thrown_player_has_ball),
                StepParameter::ThrowScatter(params.throw_scatter),
            ];
            if let Some(ref id) = params.thrown_player_id {
                init_params.push(StepParameter::ThrownPlayerId(Some(id.clone())));
            }
            if let Some(state) = params.thrown_player_state {
                init_params.push(StepParameter::ThrownPlayerState(state));
            }
            if let Some(coord) = params.thrown_player_coordinate {
                init_params.push(StepParameter::ThrownPlayerCoordinate(Some(coord)));
            }
            seq.add(StepId::InitScatterPlayer, init_params);
        }

        // APOTHECARY [APOTHECARY_HIT_PLAYER] (HIT_PLAYER)
        seq.add_labelled(StepId::Apothecary, labels::APOTHECARY_HIT_PLAYER, vec![
            StepParameter::ApothecaryMode(ApothecaryMode::HitPlayer),
        ]);
        // CATCH_SCATTER_THROW_IN
        seq.add(StepId::CatchScatterThrowIn, vec![]);
        // END_SCATTER_PLAYER
        seq.add(StepId::EndScatterPlayer, vec![]);

        seq.build()
    }
}

impl Default for ScatterPlayer {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scatter_player_without_swoop_starts_with_init_scatter_player() {
        let steps = ScatterPlayer::build_sequence(&ScatterPlayerParams::default());
        assert_eq!(steps[0].step_id, StepId::InitScatterPlayer);
    }

    #[test]
    fn scatter_player_with_swoop_starts_with_swoop() {
        let steps = ScatterPlayer::build_sequence(&ScatterPlayerParams { has_swoop: true, ..Default::default() });
        assert_eq!(steps[0].step_id, StepId::Swoop);
    }

    #[test]
    fn scatter_player_ends_with_end_scatter_player() {
        let steps = ScatterPlayer::build_sequence(&ScatterPlayerParams::default());
        assert_eq!(steps.last().unwrap().step_id, StepId::EndScatterPlayer);
    }

    #[test]
    fn scatter_player_apothecary_hit_player_is_labelled() {
        let steps = ScatterPlayer::build_sequence(&ScatterPlayerParams::default());
        let apo = steps.iter().find(|s| s.label.as_deref() == Some(labels::APOTHECARY_HIT_PLAYER)).unwrap();
        assert_eq!(apo.step_id, StepId::Apothecary);
    }

    #[test]
    fn thrown_player_id_passed_to_init_scatter_player() {
        let params = ScatterPlayerParams { thrown_player_id: Some("tpid".into()), ..Default::default() };
        let steps = ScatterPlayer::build_sequence(&params);
        let has = steps[0].params.iter().any(|p| matches!(p, StepParameter::ThrownPlayerId(Some(id)) if id == "tpid"));
        assert!(has);
    }

    #[test]
    fn throw_scatter_param_passed() {
        let params = ScatterPlayerParams { throw_scatter: true, ..Default::default() };
        let steps = ScatterPlayer::build_sequence(&params);
        let has = steps[0].params.iter().any(|p| matches!(p, StepParameter::ThrowScatter(true)));
        assert!(has);
    }
}
