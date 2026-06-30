/// BB2025 scatter-player step sequence (for Throw Team-Mate landing).
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.bb2025.ScatterPlayer`.
use ffb_model::enums::ApothecaryMode;
use ffb_model::types::FieldCoordinate;
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::sequence::{Sequence, SequenceStep, labels};

#[derive(Debug, Clone, Default)]
pub struct ScatterPlayerParams {
    pub thrown_player_id: Option<String>,
    pub thrown_player_state: Option<ffb_model::enums::PlayerState>,
    pub thrown_player_has_ball: bool,
    pub thrown_player_coordinate: Option<FieldCoordinate>,
    pub throw_scatter: bool,
    /// Whether to include the SWOOP step (Swoop skill).
    pub has_swoop: bool,
}

pub struct ScatterPlayer;

impl ScatterPlayer {
    pub fn new() -> Self { Self }

    pub fn build_sequence(params: &ScatterPlayerParams) -> Vec<SequenceStep> {
        let mut seq = Sequence::new();

        // Optional leading SWOOP step
        if params.has_swoop {
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
        }

        // INIT_SCATTER_PLAYER
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

        // TRAP_DOOR [SCATTER_BALL]
        seq.add_labelled(StepId::TrapDoor, labels::SCATTER_BALL, vec![]);
        // APOTHECARY (TRAP_DOOR)
        seq.add(StepId::Apothecary, vec![
            StepParameter::ApothecaryMode(ApothecaryMode::TrapDoor),
        ]);
        // STEADY_FOOTING (HIT_PLAYER, goto END on success)
        seq.add(StepId::SteadyFooting, vec![
            StepParameter::ApothecaryMode(ApothecaryMode::HitPlayer),
            StepParameter::GotoLabelOnSuccess(labels::END.into()),
        ]);
        // PLACE_BALL
        seq.add(StepId::PlaceBall, vec![]);
        // APOTHECARY [APOTHECARY_HIT_PLAYER] (HIT_PLAYER)
        seq.add_labelled(StepId::Apothecary, labels::APOTHECARY_HIT_PLAYER, vec![
            StepParameter::ApothecaryMode(ApothecaryMode::HitPlayer),
        ]);
        // CATCH_SCATTER_THROW_IN
        seq.add(StepId::CatchScatterThrowIn, vec![]);
        // END_SCATTER_PLAYER [END]
        seq.add_labelled(StepId::EndScatterPlayer, labels::END, vec![]);
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
    fn scatter_player_without_swoop_has_8_steps() {
        let steps = ScatterPlayer::build_sequence(&ScatterPlayerParams::default());
        assert_eq!(steps.len(), 8);
    }

    #[test]
    fn scatter_player_with_swoop_has_9_steps() {
        let steps = ScatterPlayer::build_sequence(&ScatterPlayerParams { has_swoop: true, ..Default::default() });
        assert_eq!(steps.len(), 9);
    }

    #[test]
    fn scatter_player_ends_with_end_scatter_player_labelled_end() {
        let steps = ScatterPlayer::build_sequence(&ScatterPlayerParams::default());
        let last = steps.last().unwrap();
        assert_eq!(last.step_id, StepId::EndScatterPlayer);
        assert_eq!(last.label.as_deref(), Some(labels::END));
    }

    #[test]
    fn scatter_player_apothecary_hit_player_is_labelled() {
        let steps = ScatterPlayer::build_sequence(&ScatterPlayerParams::default());
        let apo = steps.iter().find(|s| s.label.as_deref() == Some(labels::APOTHECARY_HIT_PLAYER)).unwrap();
        assert_eq!(apo.step_id, StepId::Apothecary);
    }
}
